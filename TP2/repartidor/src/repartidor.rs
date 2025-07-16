//! Este módulo contiene la lógica del actor 'Repartidor'.

// Imports de crates externas.
use rand::Rng;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};

// Imports de funciones/estructuras propias.
use common::mensajes::{
    FinalizarViaje, IniciarViajeDelivery, OfertarViaje, Posicion, QuienEsCoordinador,
    RespuestaOfertaViaje,
};
use common::tcp_enviador::MensajeTCP;
use common::utils::obtener_tupla_random;

// Constantes.
const PROBABILIDAD_ACEPTACION: f64 = 0.8;
const MIN_DELAY: f32 = 1.0;
const MAX_DELAY: f32 = 2.0;
const TIMEOUT_ACK_SEGS: u64 = 2;
const TIMEOUT_COORDINADOR: u64 = 3;
const ERROR_CONEXION_SERVIDORES: &str = "No se pudo conectar a ningún servidor de coordinación.";
const SERVER_CERRO_CONEXION: &str = "REPARTIDOR - El servidor cerró la conexión.";
const ERROR_LEYENDO_SERVIDOR: &str = "REPARTIDOR - Error leyendo del servidor: ";
const MENSAJE_DESCONOCIDO: &str = "REPARTIDOR - Mensaje desconocido:";
const ERROR_ENVIAR_RESPUESTA: &str = "REPARTIDOR - Error al enviar respuesta:";
const ERROR_ENVIAR_POSICION: &str = "REPARTIDOR - Error al enviar posición:";
const VIAJE_TERMINADO: &str = "REPARTIDOR - Viaje terminado.";
const ERROR_AVISAR_FIN_VIAJE: &str = "REPARTIDOR - Error al enviar 'FinalizarViaje':";
const ACK_MENSAJE: &str = "ACK";
const ACK_DE_COORDINADOR: &str = "Ack";
const SE_RECIBE_EL_ACK: &str =
    "REPARTIDOR - Se recibe el ACK del servidor, viaje finalizado con éxito.";
const MENSAJE_INESPERADO_SERVIDOR: &str = "REPARTIDOR - Mensaje inesperado del servidor:";
const ERROR_TIMEOUT_ACK_SEGS: &str =
    "REPARTIDOR - Timeout al esperar ACK del servidor, intentando reconectar...";
const ERROR_AL_RECONECTAR: &str = "REPARTIDOR - Error al reconectar con el servidor.";
const RECONEXION_EXITOSA: &str = "REPARTIDOR - Reconexión exitosa con el servidor.";
const BUSCANDO_COORDINADOR: &str = "REPARTIDOR - Buscando coordinador...";
const ERROR_ESCRIBIENDO_CONSULTA_LIDER: &str =
    "REPARTIDOR - Error escribiendo consulta para averiguar el líder:";
const ERROR_LEER_LINEA: &str = "REPARTIDOR - Error al leer línea:";
const TIMEOUT_LEER_LINEA: &str = "REPARTIDOR - Timeout al leer línea.";
const REPARTIDOR_NO_CONECTA_COORDINADOR: &str = "REPARTIDOR - No se pudo conectar al coordinador:";
const REPARTIDOR_ENCONTRO_COORDINADOR: &str = "REPARTIDOR - Encontró al coordinador:";
const REPARTIDOR_CONECTADO_COORDINADOR: &str = "REPARTIDOR - Conectado al coordinador.";
const RECIBI_PEDIDO_DEL_SERVIDOR: &str = "REPARTIDOR - Recibí un pedido del servidor.";
const INICIANDO_DIRECCION_RESTAURANTE: &str =
    "REPARTIDOR - Iniciando viaje desde un restaurante con direccion: ";
const INICIANDO_DIRECCION_COMENSAL: &str = "hacia un comensal con direccion: ";

// Este actor implementa al 'Repartidor' que lleva los pedidos del 'Restaurante' al 'Comensal'.
pub struct Repartidor {
    servidores: Vec<SocketAddr>,
    lector: Lines<BufReader<OwnedReadHalf>>,
    escritor: OwnedWriteHalf,
    ubicacion: Posicion,
}

// Implementación de los métodos de construcción, inicialización y handlers del actor `Repartidor`.
impl Repartidor {
    // Constructor.
    pub async fn new(servidores: Vec<SocketAddr>) -> Self {
        // Separamos el stream en dos mitades: Lectura y escritura.
        let tcp_stream: Option<TcpStream> = conectar_con_coordinador(servidores.clone()).await;
        if let Some(stream) = tcp_stream {
            let (rx, wx) = stream.into_split();
            Self {
                servidores,
                lector: BufReader::new(rx).lines(),
                escritor: wx,
                ubicacion: Posicion {
                    posicion: obtener_tupla_random(),
                },
            }
        } else {
            panic!("{}", ERROR_CONEXION_SERVIDORES);
        }
    }

    // Inicio del actor 'Repartidor'.
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
        if let Ok(pedido) = serde_json::from_str::<OfertarViaje>(&mensaje) {
            self.handle_puede_aceptar_viaje(pedido).await;
        } else if let Ok(iniciar_viaje) = serde_json::from_str::<IniciarViajeDelivery>(&mensaje) {
            self.handle_iniciar_viaje(iniciar_viaje).await;
        } else {
            eprintln!("{} {}", MENSAJE_DESCONOCIDO, mensaje);
        }
    }

    // Handler para aceptar o rechazar un viaje.
    async fn handle_puede_aceptar_viaje(&mut self, msj: OfertarViaje) {
        // Simula la decisión de aceptar o rechazar el viaje de forma aleatoria.
        let mut rng = rand::thread_rng();
        let esta_aceptado = rng.gen_bool(PROBABILIDAD_ACEPTACION); // 80% de probabilidad de aceptar.
        println!("{}", RECIBI_PEDIDO_DEL_SERVIDOR);
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
    }

    // Envía la posición actual del repartidor al servidor coordinador.
    async fn enviar_posicion(&mut self) {
        let posicion = self.ubicacion.clone();
        if let Ok(serializado) = serde_json::to_string(&posicion) {
            if let Err(err) = self
                .escritor
                .write_all(format!("{}\n", serializado).as_bytes())
                .await
            {
                eprintln!("{} {}", ERROR_ENVIAR_POSICION, err);
            }
        }
    }

    // Maneja el inicio del viaje, calcula la distancia y simula el tiempo de viaje.
    async fn handle_iniciar_viaje(&mut self, msj: IniciarViajeDelivery) {
        let posicion = msj.origen_i;
        let destino = msj.destino_i;
        println!(
            "{}({},{}) {}({},{})",
            INICIANDO_DIRECCION_RESTAURANTE,
            posicion.0,
            posicion.1,
            INICIANDO_DIRECCION_COMENSAL,
            destino.0,
            destino.1
        );
        // Simula el tiempo de viaje basado en la distancia entre la posición actual, el destino y un delay random para modelar casos mas reales.
        let distance = ((posicion.0 - destino.0).powi(2) + (posicion.1 - destino.1).powi(2)).sqrt();
        let delay = rand::thread_rng().gen_range(MIN_DELAY..=MAX_DELAY);
        sleep(Duration::from_secs_f32(distance + delay)).await;

        println!("{}", VIAJE_TERMINADO);
        let viaje_finalizado = FinalizarViaje {
            direccion_comensal_f: msj.direccion_comensal_i,
            direccion_conductor_f: msj.direccion_conductor_i,
            pos_destino: (destino.0, destino.1),
        };

        if let Ok(viaje_terminado_serializado) = serde_json::to_string(&viaje_finalizado) {
            if let Err(err) = self
                .escritor
                .write_all(format!("{}\n", viaje_terminado_serializado).as_bytes())
                .await
            {
                eprintln!("{} {}", ERROR_AVISAR_FIN_VIAJE, err);
            }

            let ack_future = self.lector.next_line();
            match tokio::time::timeout(Duration::from_secs(TIMEOUT_ACK_SEGS), ack_future).await {
                Ok(Ok(Some(ack_mensaje))) => {
                    if ack_mensaje.trim() == ACK_MENSAJE {
                        println!("{}", SE_RECIBE_EL_ACK);
                    } else {
                        println!("{} {}", MENSAJE_INESPERADO_SERVIDOR, ack_mensaje);
                    }
                }

                Ok(Ok(None)) | Ok(Err(_)) | Err(_) => {
                    println!("{}", ERROR_TIMEOUT_ACK_SEGS);
                    if !self.intentar_reconectar(viaje_terminado_serializado).await {
                        eprintln!("{}", ERROR_AL_RECONECTAR);
                    }
                }
            }
        }
    }

    // Método que intenta reconectar con el servidor coordinador y enviar el mensaje de finalización del viaje.
    async fn intentar_reconectar(&mut self, mensaje: String) -> bool {
        let coordinador = conectar_con_coordinador(self.servidores.clone()).await;
        if let Some(stream) = coordinador {
            let (lector, escritor) = stream.into_split();
            self.lector = BufReader::new(lector).lines();
            self.escritor = escritor;
            if let Err(e) = self
                .escritor
                .write_all(format!("{}\n", mensaje).as_bytes())
                .await
            {
                eprintln!("{} {}", ERROR_AVISAR_FIN_VIAJE, e);
            } else {
                println!("{}", RECONEXION_EXITOSA);
                return true;
            }
        }

        false
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
                println!("{} {}", REPARTIDOR_NO_CONECTA_COORDINADOR, servidor);
            }
        }

        if direccion_coordinador.is_some() {
            break;
        }
    }

    if let Some(ip_coordinador) = direccion_coordinador {
        println!("{} {}", REPARTIDOR_ENCONTRO_COORDINADOR, ip_coordinador);
        let stream = match TcpStream::connect(ip_coordinador).await {
            Ok(stream) => {
                println!("{}", REPARTIDOR_CONECTADO_COORDINADOR);
                Some(stream)
            }

            Err(_) => None,
        };

        return stream;
    }

    None
}
