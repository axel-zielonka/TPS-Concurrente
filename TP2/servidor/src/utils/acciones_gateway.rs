//! Este módulo contiene la lógica del manejo de los mensajes del 'Servidor' con el 'Gateway'.

// Imports de crates externas.
use actix::{ActorFutureExt, Handler, ResponseActFuture, WrapFuture};
use rand::random;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;

// Imports de funciones/estructuras propias.
use super::logs::log_funcionamiento;
use crate::almacenamiento::mensajes_almacenamiento::ObtenerComensal;
use crate::server::server::Server;
use common::mensajes::{Autorizacion, EnviarMensajePago, RechazarViaje};
use common::mensajes_gateway::{
    EfectivizarPago, MensajeGateway, RequerirPago, RespuestaAutorizacion, RespuestaPago,
    ValidarAutorizacionPago,
};
use common::tcp_enviador::MensajeTCP;

// Constantes.
const TIMEOUT_SEGUNDOS: u64 = 5;
const MIN_BYTES_LEIDOS: usize = 0;
const MAX_VALOR_PAGO: f32 = 100.0;
const COMENSAL_STR: &str = "Comensal";
const FUE_AUTORIZADO_MSJ: &str = "fue autorizado.";
const NO_FUE_AUTORIZADO_MSJ: &str = "no fue autorizado.";
const VIAJE_RECHAZADO_SALDO_INSUFICIENTE: &str = "Viaje rechazado por saldo insuficiente.";
pub(crate) const ERROR_STR: &str = "Error";
pub(crate) const ERROR_AL_SERIALIZAR: &str = "Error al serializar el mensaje 'RechazarViaje':";
const ERROR_ENVIAR_MENSAJE_PAGO: &str = "Error al enviar mensaje de pago.";
const PAGO_EXITOSO: &str = "Pago exitoso para ";
const ERROR_RECIBIR_RESPUESTA_GATEWAY: &str = "Error al recibir respuesta válida del Gateway.";
const ERROR_CONECTAR_GATEWAY: &str = "Error al conectar con el Gateway.";

// Implementación de los handlers para el actor 'Servidor' que maneja los mensajes del 'Gateway'.
impl Handler<Autorizacion> for Server {
    // Implementación del handler para el mensaje 'Autorizacion'.
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msj: Autorizacion, _contexto: &mut Self::Context) -> Self::Result {
        let id_comensal = msj.direccion_comensal;
        let direccion = self.addr;
        let actor_almacenamiento = self.almacenamiento_addr.clone();
        Box::pin(
            async move {
                if msj.esta_autorizado {
                    log_funcionamiento(format!(
                        "[{:?}] {} {} {}",
                        COMENSAL_STR, direccion, id_comensal, FUE_AUTORIZADO_MSJ
                    ));
                } else {
                    log_funcionamiento(format!(
                        "[{:?}] {} {} {}",
                        direccion, COMENSAL_STR, id_comensal, NO_FUE_AUTORIZADO_MSJ
                    ));

                    if let Ok(Some(comensal)) = actor_almacenamiento
                        .send(ObtenerComensal { id: id_comensal })
                        .await
                    {
                        let mensaje_tcp = match serde_json::to_string(&RechazarViaje {
                            respuesta: VIAJE_RECHAZADO_SALDO_INSUFICIENTE.to_owned(),
                        }) {
                            Ok(json_string) => MensajeTCP(json_string),
                            Err(err) => {
                                eprintln!("{} {}", ERROR_AL_SERIALIZAR, err);
                                MensajeTCP(ERROR_STR.to_string())
                            }
                        };

                        if let Some(sender) = comensal.enviador_comensal.as_ref() {
                            sender
                                .send(mensaje_tcp)
                                .await
                                .expect("Error al enviar mensaje 'RechazarViaje'.");
                        }
                    }
                }
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        )
    }
}

// Método que maneja la obtención de la respuesta del pago desde el 'Gateway'.
pub async fn obtener_respuesta_pago(msj: EnviarMensajePago) -> bool {
    let direccion_gateway = format!(
        "{}:{}",
        crate::utils::constantes::IP_GATEWAY,
        crate::utils::constantes::PUERTO_GATEWAY
    );

    let tcp_stream = TcpStream::connect(&direccion_gateway).await;
    match tcp_stream {
        Ok(s) => {
            let (lector, mut escritor) = s.into_split();
            let mensaje = match msj.tipo_mensaje {
                MensajeGateway::Validar => {
                    RequerirPago::ValidarAutorizacionPago(ValidarAutorizacionPago {
                        id_comensal: msj.id_comensal.clone(),
                        valor: msj.valor,
                    })
                }

                MensajeGateway::Pagar => RequerirPago::EfectivizarPago(EfectivizarPago {
                    id_comensal: msj.id_comensal.clone(),
                    valor: msj.valor,
                }),
            };

            if let Ok(serializado) = serde_json::to_string(&mensaje) {
                let serializado = format!("{}\n", serializado);
                if escritor.write_all(serializado.as_bytes()).await.is_err() {
                    log_funcionamiento(ERROR_ENVIAR_MENSAJE_PAGO.to_string());
                    return false;
                }
            }

            let mut lector = BufReader::new(lector);
            let mut linea = String::new();
            if let Ok(Ok(bytes_leidos)) = timeout(
                Duration::from_secs(TIMEOUT_SEGUNDOS),
                lector.read_line(&mut linea),
            )
            .await
            {
                if bytes_leidos > MIN_BYTES_LEIDOS {
                    if let Ok(respuesta_autorizacion) =
                        serde_json::from_str::<RespuestaAutorizacion>(&linea)
                    {
                        return respuesta_autorizacion.autorizado;
                    }

                    if let Ok(_respuesta_pago) = serde_json::from_str::<RespuestaPago>(&linea) {
                        log_funcionamiento(format!("{} {}", PAGO_EXITOSO, msj.id_comensal));
                        return true;
                    }
                }
            }

            log_funcionamiento(ERROR_RECIBIR_RESPUESTA_GATEWAY.to_string());
            false
        }

        Err(_) => {
            log_funcionamiento(ERROR_CONECTAR_GATEWAY.to_string());
            false
        }
    }
}

// Genera un mensaje para validar un pago.
pub fn generar_mensaje_chequeo_pago(id_comensal: String) -> EnviarMensajePago {
    EnviarMensajePago {
        id_comensal,
        valor: random::<f32>() * MAX_VALOR_PAGO,
        tipo_mensaje: MensajeGateway::Validar,
    }
}

// Genera un mensaje para realizar un pago.
pub fn generar_mensaje_pago_hecho(id_comensal: String) -> EnviarMensajePago {
    EnviarMensajePago {
        id_comensal,
        valor: random::<f32>() * MAX_VALOR_PAGO,
        tipo_mensaje: MensajeGateway::Pagar,
    }
}
