//! Este módulo contiene la lógica del actor 'Gateway de Pagos'.

// Imports de crates externas.
use actix::prelude::*;
use rand::Rng;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{split, AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::LinesStream;

// Imports de funciones/estructuras propias.
use common::mensajes_gateway::{RequerirPago, RespuestaAutorizacion, RespuestaPago};
use common::tcp_enviador::{MensajeTCP, TcpEnviador};

// Constantes.
const PORCENTAJE_ERROR: u8 = 3; // Treinta porciento de probabilidad de error en la autorización de pago.
const MSJ_CONEXION_RECIBIDA: &str = "Conexión recibida desde";
const ERROR_ACEPTAR_CONEXION: &str = "Error al aceptar la conexión:";
const LOG_TRANSACCION_1: &str = "GATEWAY - Pago";
const LOG_TRANSACCION_2: &str = "por comensal";
const MSJ_AUTORIZADO: &str = "AUTORIZADO";
const MSJ_RECHAZADO: &str = "RECHAZADO";
const ERROR_MENSAJE_AUTORIZACION: &str = "Error al enviar mensaje 'RespuestaAutorizacion' a";
const AVISO_MENSAJE_ENVIADO: &str = "Mensaje 'RespuestaAutorizacion' enviado a";
const ERROR_SERIALIZAR_MENSAJE_CHEQUEO: &str = "Error para serializar 'RespuestaAutorizacion'";
const ERROR_AVISO_PAGO_REALIZADO: &str = "Error al enviar 'PagoHecho' a";
const AVISO_PAGO_ENVIADO: &str = "Enviado 'PagoHecho' a";
const ERROR_SERIALIZAR_MENSAJE_PAGO: &str = "Error al serializar 'PagoHecho'";
const ERROR_DESERIALIZAR_MENSAJE: &str = "Error al deserializar el mensaje recibido.";
const ERROR_LEYENDO_LINEA: &str = "Error leyendo la línea:";

// Este actor implementa al 'Gateway' que efectua o rechaza los pagos.
pub struct GatewayActor {
    tcp_enviador: Arc<Addr<TcpEnviador>>,
    pub direccion: SocketAddr,
}

// Implementa el trait 'Actor'.
impl Actor for GatewayActor {
    type Context = Context<Self>;
}

// Implementación de los métodos de construcción e inicialización del actor `GatewayActor`.
impl GatewayActor {
    // Constructor.
    pub fn new(stream: TcpStream, direccion: SocketAddr) -> Addr<Self> {
        // Separamos el stream en dos mitades: Lectura y escritura.
        GatewayActor::create(|contexto| {
            let (mitad_lectura, mitad_escritura) = split(stream);
            GatewayActor::add_stream(
                LinesStream::new(BufReader::new(mitad_lectura).lines()),
                contexto,
            );
            let escribir = Some(mitad_escritura);
            let enviador_actor = TcpEnviador { escribir }.start();
            let tcp_enviador = Arc::new(enviador_actor);
            GatewayActor {
                tcp_enviador,
                direccion,
            }
        })
    }

    // Inicio del actor `GatewayActor`.
    pub async fn start(direccion: SocketAddr) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(direccion)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        loop {
            match listener.accept().await {
                Ok((stream, direccion_cliente)) => {
                    println!(
                        "[{}] {} {:?}",
                        direccion, MSJ_CONEXION_RECIBIDA, direccion_cliente
                    );
                    GatewayActor::new(stream, direccion_cliente);
                }

                Err(e) => {
                    println!("[{}] {} {:?}", direccion, ERROR_ACEPTAR_CONEXION, e);
                }
            }
        }
    }
}

// Implementación de los handlers del actor 'GatewayActor'.
impl StreamHandler<Result<String, tokio::io::Error>> for GatewayActor {
    // Método que maneja las interacciones del gateway con el servidor coordinador.
    fn handle(&mut self, linea: Result<String, tokio::io::Error>, contexto: &mut Context<Self>) {
        let tcp_enviador = self.tcp_enviador.clone();
        let direccion = self.direccion;
        if let Ok(data) = linea {
            if let Ok(mensaje) = serde_json::from_str::<RequerirPago>(&data) {
                match mensaje {
                    // Handler para el chequeo inicial de autorización de pago (Modela la suficiencia de saldo).
                    RequerirPago::ValidarAutorizacionPago(mensaje_autorizacion) => {
                        let mut rng = rand::thread_rng();

                        // Se randomiza la autorización de pago.
                        let autorizado = rng.gen_range(1..=10) >= PORCENTAJE_ERROR;
                        let respuesta = RespuestaAutorizacion {
                            id_comensal: mensaje_autorizacion.id_comensal.clone(),
                            autorizado,
                        };

                        println!(
                            "{} [{}] {} [{}]",
                            LOG_TRANSACCION_1,
                            if autorizado {
                                MSJ_AUTORIZADO
                            } else {
                                MSJ_RECHAZADO
                            },
                            LOG_TRANSACCION_2,
                            mensaje_autorizacion.id_comensal
                        );

                        if let Ok(mensaje_serializado) = serde_json::to_string(&respuesta) {
                            tcp_enviador
                                .send(MensajeTCP(mensaje_serializado))
                                .into_actor(self)
                                .then(move |result, _, _| {
                                    if let Err(err) = result {
                                        eprintln!(
                                            "[{:?}] {} {}: {:?}",
                                            direccion,
                                            ERROR_MENSAJE_AUTORIZACION,
                                            mensaje_autorizacion.id_comensal,
                                            err
                                        );
                                    } else {
                                        println!(
                                            "[{:?}] {} {}",
                                            direccion,
                                            AVISO_MENSAJE_ENVIADO,
                                            mensaje_autorizacion.id_comensal
                                        );
                                    }
                                    fut::ready(())
                                })
                                .wait(contexto);
                        } else {
                            eprintln!("[{:?}] {}", direccion, ERROR_SERIALIZAR_MENSAJE_CHEQUEO);
                        }
                    }

                    // Handler para el pago efectivo (Modela la ejecución del pago).
                    RequerirPago::EfectivizarPago(mensaje_pago) => {
                        let respuesta = RespuestaPago::PagoHecho;
                        if let Ok(mensaje_serializado) = serde_json::to_string(&respuesta) {
                            tcp_enviador
                                .send(MensajeTCP(mensaje_serializado))
                                .into_actor(self)
                                .then(move |result, _, _| {
                                    if let Err(err) = result {
                                        eprintln!(
                                            "[{:?}] {} {}: {:?}",
                                            direccion,
                                            ERROR_AVISO_PAGO_REALIZADO,
                                            mensaje_pago.id_comensal,
                                            err
                                        );
                                    } else {
                                        println!(
                                            "[{:?}] {} {}",
                                            direccion, AVISO_PAGO_ENVIADO, mensaje_pago.id_comensal
                                        );
                                    }

                                    fut::ready(())
                                })
                                .wait(contexto);
                        } else {
                            eprintln!("[{:?}] {}", direccion, ERROR_SERIALIZAR_MENSAJE_PAGO);
                        }
                    }
                }
            } else {
                eprintln!("[{:?}] {}", direccion, ERROR_DESERIALIZAR_MENSAJE);
            }
        } else if let Err(err) = linea {
            eprintln!("[{:?}] {} {}", direccion, ERROR_LEYENDO_LINEA, err);
        }
    }
}
