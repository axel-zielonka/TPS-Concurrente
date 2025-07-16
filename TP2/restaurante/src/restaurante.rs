//! Este módulo contiene la lógica del actor 'Restaurante'.

// Imports de crates externas.
use rand::Rng;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};

// Imports de funciones/estructuras propias.
use common::mensajes::{
    MensajeIdentidad, Posicion, QuienEsCoordinador, RecibirPedido, RespuestaOfertaViaje,
    SolicitarRepartidor,
};
use common::tcp_enviador::MensajeTCP;
use common::utils::obtener_tupla_random;

// Constantes.
const PROBABILIDAD_ACEPTACION: f64 = 0.9;
const TIMEOUT_COORDINADOR: u64 = 3;
const ERROR_CONEXION_SERVIDORES: &str = "No se pudo conectar a ningún servidor de coordinación.";
const SERVER_CERRO_CONEXION: &str = "RESTAURANTE - El servidor cerró la conexión.";
const ERROR_LEYENDO_SERVIDOR: &str = "RESTAURANTE - Error leyendo del servidor: ";
const MSJ_ESTADO_ACEPTACION_VIAJE: &str = "RESTAURANTE - ¿Se acepto el pedido?:";
const CASO_POSITIVO: &str = "Sí";
const CASO_NEGATIVO: &str = "No";
const ERROR_ENVIAR_RESPUESTA: &str = "RESTAURANTE - Error al enviar respuesta:";
const POSICION_ENVIADA: &str = "RESTAURANTE - Posición enviada:";
const ERROR_ENVIAR_POSICION: &str = "RESTAURANTE - Error al enviar posición:";
const ACK_DE_COORDINADOR: &str = "Ack";
const BUSCANDO_COORDINADOR: &str = "RESTAURANTE - Buscando coordinador...";
const ERROR_ESCRIBIENDO_CONSULTA_LIDER: &str =
    "RESTAURANTE - Error escribiendo consulta para averiguar el líder:";
const ERROR_LEER_LINEA: &str = "RESTAURANTE - Error al leer línea:";
const TIMEOUT_LEER_LINEA: &str = "RESTAURANTE - Timeout al leer línea.";
const RESTAURANTE_NO_CONECTA_COORDINADOR: &str =
    "RESTAURANTE - No se pudo conectar al coordinador:";
const RESTAURANTE_ENCONTRO_COORDINADOR: &str = "RESTAURANTE - Encontró al coordinador:";
const RESTAURANTE_CONECTADO_COORDINADOR: &str = "RESTAURANTE - Conectado al coordinador.";
const ERROR_SOLICITAR_VIAJE: &str = "RESTAURANTE - Error al solicitar viaje:";
const PEDIDO_LISTO: &str = "RESTAURANTE - El pedido ya está listo para ser retirado por el repartidor. La direccion del comensal es: ";

// Este actor implementa al 'Restaurante' que prepara los pedidos que hace el 'Comensal'.
pub struct Restaurante {
    lector: Lines<BufReader<OwnedReadHalf>>,
    escritor: OwnedWriteHalf,
    ubicacion_fija: Posicion, // Posición fija del restaurante.,
    clientes_atendidos: Vec<SocketAddr>,
    ultima_cocina: HashMap<String, Instant>,
}

// Implementación de los métodos de construcción, inicialización y handlers del actor 'Restaurante'.
impl Restaurante {
    // Constructor.
    pub async fn new(servidores: Vec<SocketAddr>) -> Self {
        // Separamos el stream en dos mitades: Lectura y escritura.
        let tcp_stream: Option<TcpStream> = conectar_con_coordinador(servidores.clone()).await;
        let posicion = Posicion {
            posicion: obtener_tupla_random(),
        };
        if let Some(stream) = tcp_stream {
            let (rx, wx) = stream.into_split();
            Self {
                lector: BufReader::new(rx).lines(),
                escritor: wx,
                ubicacion_fija: posicion,
                clientes_atendidos: Vec::new(),
                ultima_cocina: HashMap::new(),
            }
        } else {
            panic!("{}", ERROR_CONEXION_SERVIDORES);
        }
    }

    // Inicio del actor 'Restaurante'.
    pub async fn run(&mut self) {
        // Informa su posición inicial al servidor.
        self.enviar_posicion().await;
        loop {
            tokio::select! {
                resultado = self.lector.next_line() => {
                    match resultado {
                        Ok(Some(mensaje)) => {
                            self.handle_mensaje_servidor(mensaje).await;
                        }
                        Ok(None) => {
                            println!("{}", SERVER_CERRO_CONEXION);
                            break;
                        }
                        Err(e) => {
                            eprintln!("{} {}", ERROR_LEYENDO_SERVIDOR, e);
                            break;
                        }
                    }
                }
            }
        }
    }

    // Handler de los mensajes recibidos del servidor coordinador.
    async fn handle_mensaje_servidor(&mut self, mensaje: String) {
        if let Ok(pedido) = serde_json::from_str::<RecibirPedido>(&mensaje) {
            self.handle_recibir_pedido(pedido).await;
        }
    }

    async fn handle_recibir_pedido(&mut self, msj: RecibirPedido) {
        let ahora = Instant::now();
        let comida = msj.comida.clone();
        let intervalo = Duration::from_secs(5);
        match self.ultima_cocina.get(comida.as_str()) {
            Some(&momento) if ahora.duration_since(momento) < intervalo => {
                // Una comida tiene que esperar un poco de tiempo para ser cocinada de nuevo.
                return;
            }
            _ => {
                self.ultima_cocina.insert(comida.clone(), ahora);
            }
        }

        if self.clientes_atendidos.contains(&msj.direccion_comensal_o) {
        } else {
            self.clientes_atendidos.push(msj.direccion_comensal_o);
            println!(
                "{} {}",
                "RESTAURANTE - Recibido pedido de comida:",
                msj.comida
            );
            let mut rng = rand::thread_rng();
            let esta_aceptado = rng.gen_bool(PROBABILIDAD_ACEPTACION);
            println!(
                "{} {}",
                MSJ_ESTADO_ACEPTACION_VIAJE,
                if esta_aceptado {
                    CASO_POSITIVO
                } else {
                    CASO_NEGATIVO
                }
            );

            if esta_aceptado {
                println!("RESTAURANTE - Estoy cocinando: {}", msj.comida);
                sleep(Duration::from_secs(2)).await;
                println!(
                    "{}({},{})",
                    PEDIDO_LISTO, msj.ubicacion_comensal.0, msj.ubicacion_comensal.1
                );
            }
            let respuesta = RespuestaOfertaViaje {
                direccion_comensal_r: msj.direccion_comensal_o,
                esta_aceptado,
            };

            if let Ok(serializado) = serde_json::to_string(&respuesta) {
                if let Err(err) = self
                    .escritor
                    .write_all(format!("{}\n", serializado).as_bytes())
                    .await
                {
                    eprintln!("{} {}", ERROR_ENVIAR_RESPUESTA, err);
                }
            }

            // Si el pedido es aceptado, se solicita un repartidor.
            let solicitar_viaje = SolicitarRepartidor {
                comida: msj.comida,
                origen: self.ubicacion_fija.posicion,
                destino: msj.ubicacion_comensal,
                pedido_aceptado: esta_aceptado,
                direccion_comensal: msj.direccion_comensal_o,
            };

            if let Ok(serializado) = serde_json::to_string(&solicitar_viaje) {
                if let Err(e) = self
                    .escritor
                    .write_all(format!("{}\n", serializado).as_bytes())
                    .await
                {
                    eprintln!("{} {}", ERROR_SOLICITAR_VIAJE, e);
                }
            }
        }
    }

    // Envía la posición actual del RESTAURANTE al servidor coordinador.
    async fn enviar_posicion(&mut self) {
        let position = self.ubicacion_fija.clone();
        let mensaje_identidad = MensajeIdentidad {
            ubicacion: position.posicion,
            soy_repartidor: false,
        };

        println!(
            "{} ({}, {})",
            POSICION_ENVIADA, position.posicion.0, position.posicion.1
        );

        if let Ok(serializado) = serde_json::to_string(&mensaje_identidad) {
            if let Err(err) = self
                .escritor
                .write_all(format!("{}\n", serializado).as_bytes())
                .await
            {
                eprintln!("{} {}", ERROR_ENVIAR_POSICION, err);
            }
        }
    }
}

// Función que establece la conexión con el servidor coordinador, con proceso de identificación previo.
async fn conectar_con_coordinador(servidores: Vec<SocketAddr>) -> Option<TcpStream> {
    let mut direccion_coordinador: Option<SocketAddr> = None;
    println!("{}", BUSCANDO_COORDINADOR);
    for servidor in servidores {
        match TcpStream::connect(servidor).await {
            Ok(stream) => {
                let (lector, mut escritor) = split(stream);
                let msj = MensajeTCP("WhoIsCoordinator\n".to_string());
                if let Err(e) = escritor.write_all(msj.0.as_bytes()).await {
                    eprintln!("{} {}", ERROR_ESCRIBIENDO_CONSULTA_LIDER, e);
                }

                let mut lector = BufReader::new(lector);
                let mut linea = String::new();
                match timeout(
                    Duration::from_secs(TIMEOUT_COORDINADOR),
                    lector.read_line(&mut linea),
                )
                .await
                {
                    Ok(Ok(_)) => {
                        if let Ok(quien_es_coordinador_msj) =
                            serde_json::from_str::<QuienEsCoordinador>(&linea)
                        {
                            direccion_coordinador =
                                Some(quien_es_coordinador_msj.direccion_coordinador);
                            break;
                        } else if linea.trim() == ACK_DE_COORDINADOR {
                            break;
                        }
                    }
                    Ok(Err(e)) => {
                        println!("{} {:?}", ERROR_LEER_LINEA, e);
                    }
                    Err(_) => {
                        println!("{}", TIMEOUT_LEER_LINEA);
                    }
                }
                break;
            }
            Err(_) => {
                println!("{} {}", RESTAURANTE_NO_CONECTA_COORDINADOR, servidor);
            }
        }

        if direccion_coordinador.is_some() {
            break;
        }
    }

    if let Some(ip_coordinador) = direccion_coordinador {
        println!("{} {}", RESTAURANTE_ENCONTRO_COORDINADOR, ip_coordinador);
        let stream = match TcpStream::connect(ip_coordinador).await {
            Ok(stream) => {
                println!("{}", RESTAURANTE_CONECTADO_COORDINADOR);
                Some(stream)
            }

            Err(_) => None,
        };

        return stream;
    }

    None
}
