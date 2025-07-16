//! Este módulo contiene la lógica del actor 'Comensal'.

// Imports de crates externas.
use rand::seq::IndexedRandom;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::time::timeout;

// Imports de funciones/estructuras propias.
use common::mensajes::{
    FinalizarViaje, IniciarViajeDelivery, QuienEsCoordinador, RechazarViaje, SolicitarPedido,
};
use common::tcp_enviador::MensajeTCP;
use common::utils::obtener_tupla_random;

// Constantes.
const TIMEOUT_ACK_SEGS: u64 = 2;
const TIMEOUT_COORDINADOR: u64 = 3;
const ERROR_CONECTARSE_SERVIDOR: &str = "COMENSAL - No se pudo conectar con ningún servidor.";
const MENSAJE_SERVER_CERRADO: &str = "COMENSAL - El servidor ha cerrado la conexión.";
const ERROR_LEER_DEL_SERVIDOR: &str = "COMENSAL - Error al leer del servidor:";
const ERROR_SOLICITAR_VIAJE: &str = "COMENSAL - Error al enviar 'SolicitarViaje':";
const PEDIDO_RECHAZADO_POR_RESTAURANTE: &str =
    "COMENSAL - La comida que elegiste no puede ser preparada por el restaurante.";
const MENSAJE_DESCONOCIDO: &str = "COMENSAL - Mensaje desconocido recibido del servidor:";
const ESPERAR_COMIDA: &str = "COMENSAL - Esperando la comida.";
const VIAJE_FINALIZADO: &str = "COMENSAL - Llego la comida.";
const ERROR_ENVIAR_FINAL_VIAJE: &str =
    "COMENSAL - Error al enviar 'FinalizarViaje' al servidor. Intentando reconectar...";
const ERROR_AL_RECONECTAR: &str = "COMENSAL - Error al reconectar con el servidor.";
const ACK_MENSAJE: &str = "ACK";
const ACK_DE_COORDINADOR: &str = "Ack";
const RECIBIDIO_ACK: &str = "COMENSAL - Recibido ACK del servidor.";
const ERROR_TIMEOUT_ACK_SEGS: &str =
    "COMENSAL - Timeout al esperar ACK del servidor, intentando reconectar...";
const SE_RECONECTA_ENVIA_FINAL: &str =
    "COMENSAL - Reconectado y enviado 'FinalizarViaje' al servidor.";
const BUSCANDO_COORDINADOR: &str = "COMENSAL - Buscando coordinador...";
const ERROR_ESCRIBIENDO_CONSULTA_LIDER: &str =
    "COMENSAL - Error escribiendo consulta para averiguar el líder:";
const ERROR_LEER_LINEA: &str = "COMENSAL - Error al leer línea:";
const TIMEOUT_LEER_LINEA: &str = "COMENSAL - Timeout al leer línea.";
const COMENSAL_NO_CONECTA_COORDINADOR: &str = "COMENSAL - No se pudo conectar al coordinador:";
const COMENSAL_ENCONTRO_COORDINADOR: &str = "COMENSAL - Encontró al coordinador:";
const COMENSAL_CONECTADO_COORDINADOR: &str = "COMENSAL - Conectado al coordinador.";
const PEDIDO_ELEGIDO_MSJ: &str = "COMENSAL - Pedido elegido:";
const COMIDAS: [&str; 50] = [
    "Pizza",
    "Sushi",
    "Hamburguesa",
    "Ensalada",
    "Tacos",
    "Milanesa",
    "Empanadas",
    "Lasagna",
    "Pollo Frito",
    "Choripán",
    "Panqueques",
    "Guiso de Lentejas",
    "Ravioles",
    "Arepas",
    "Locro",
    "Pastel de Papa",
    "Bife con Puré",
    "Ñoquis",
    "Canelones",
    "Wok de Verduras",
    "Papas Fritas",
    "Fideos con Tuco",
    "Matambre a la Pizza",
    "Arroz con Pollo",
    "Tarta de Verdura",
    "Tortilla de Papas",
    "Shawarma",
    "Falafel",
    "Sopa de Calabaza",
    "Hot Dog",
    "Chow Mein",
    "Paella",
    "Cazuela de Mariscos",
    "Tamales",
    "Curry de Garbanzos",
    "Lomo Saltado",
    "Feijoada",
    "Burrito",
    "Chili con Carne",
    "Arequipe",
    "Croquetas",
    "Ceviche",
    "Carbonada",
    "Costillas BBQ",
    "Risotto",
    "Polenta",
    "Humita en Chala",
    "Pizza Fugazzeta",
    "Fainá",
    "Pure de la nona",
];
const PEDIDOS_DISPONIBLES: [&str; 50] = COMIDAS;
const COMIDA: &str = "Comida";
const SE_COCINO_EL_PEDIDO: &str =
    "COMENSAL - El pedido ya esta listo para ser retirado por el repartidor.";
const REPARTIDOR_EN_CAMINO: &str = "COMENSAL - Repartidor en camino a casa.";
const TRANSACCION_INVALIDA: &str =
    "COMENSAL - El sistema rechazo la transacción, Intente de nuevo mas tarde.";

// Este actor implementa al 'Comensal' que realiza los pedidos por la aplicación.
pub struct Comensal {
    servidores: Vec<SocketAddr>,
    lector: Lines<BufReader<OwnedReadHalf>>,
    escritor: OwnedWriteHalf,
    mi_ubicación: (f32, f32),
}

// Implementación de los métodos de construcción, inicialización y handlers del actor `Comensal`.
impl Comensal {
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
                mi_ubicación: obtener_tupla_random(),
            }
        } else {
            panic!("{}", ERROR_CONECTARSE_SERVIDOR);
        }
    }

    // Inicio del actor 'Comensal'.
    pub async fn run(&mut self) {
        self.solicitar_un_delivery().await;
        loop {
            tokio::select! {
                resultado = self.lector.next_line() => {
                    match resultado {
                        Ok(Some(mensaje)) => {
                         self.handle_mensaje_servidor(mensaje).await;
                     }
                        Ok(None) => {
                            println!("{}", MENSAJE_SERVER_CERRADO);
                            break;
                        }

                        Err(e) => {
                            eprintln!("{} {}", ERROR_LEER_DEL_SERVIDOR, e);
                            break;
                        }
                    }
                }
            }
        }
    }

    // Método para solicitar un viaje.
    async fn solicitar_un_delivery(&mut self) {
        let pedidos_disponibles = PEDIDOS_DISPONIBLES;
        let mut rng = rand::rng();
        let comida_elegida = pedidos_disponibles.choose(&mut rng).unwrap_or(&COMIDA);
        println!("{} {}", PEDIDO_ELEGIDO_MSJ, comida_elegida);
        let solicitar_viaje = SolicitarPedido {
            comida: comida_elegida.to_string(),
            destino: self.mi_ubicación,
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

    // Manejador de los mensajes recibidos del servidor.
    async fn handle_mensaje_servidor(&mut self, mensaje: String) {
        if let Ok(iniciar_viaje) = serde_json::from_str::<IniciarViajeDelivery>(&mensaje) {
            println!("{}", SE_COCINO_EL_PEDIDO);
            println!("{}", REPARTIDOR_EN_CAMINO);
            self.esperar_comida(iniciar_viaje).await;
        } else if let Ok(_) = serde_json::from_str::<RechazarViaje>(&mensaje) {
            if mensaje == r#"{"respuesta":"Viaje rechazado por restaurante"}"# {
                println!("{}", PEDIDO_RECHAZADO_POR_RESTAURANTE);
            } else {
                println!("{}", TRANSACCION_INVALIDA);
            }

            std::process::exit(0);
        } else if mensaje == "Viaje rechazado por restaurante".to_string() {
            println!("COMENSAL - El restaurante rechazo el pedido.");
            std::process::exit(0);
        }
    }

    // Maneja el inicio del viaje, calcula la distancia y simula el tiempo de viaje.
    async fn esperar_comida(&mut self, msg: IniciarViajeDelivery) {
        let position = msg.origen_i;
        let destination = msg.destino_i;

        println!("{}", ESPERAR_COMIDA,);

        // Simula el tiempo de viaje basado en la distancia entre la posición actual y el destino.
        let distance =
            ((position.0 - destination.0).powi(2) + (position.1 - destination.1).powi(2)).sqrt();
        tokio::time::sleep(tokio::time::Duration::from_secs_f32(distance)).await;
        println!("{}", VIAJE_FINALIZADO);
        let viaje_finalizado = FinalizarViaje {
            direccion_comensal_f: msg.direccion_comensal_i,
            direccion_conductor_f: msg.direccion_conductor_i,
            pos_destino: (destination.0, destination.1),
        };

        if let Ok(viaje_finalizado_serializado) = serde_json::to_string(&viaje_finalizado) {
            if (self
                .escritor
                .write_all(format!("{}\n", viaje_finalizado_serializado).as_bytes())
                .await)
                .is_err()
            {
                println!("{}", ERROR_ENVIAR_FINAL_VIAJE);
                if !self.intentar_reconectar(viaje_finalizado_serializado).await {
                    eprintln!("{}", ERROR_AL_RECONECTAR);
                }

                return;
            }

            let ack_future = self.lector.next_line();
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(TIMEOUT_ACK_SEGS),
                ack_future,
            )
            .await
            {
                Ok(Ok(Some(ack_mensaje))) => {
                    if ack_mensaje.trim() == ACK_MENSAJE {
                        println!("{}", RECIBIDIO_ACK);
                    } else {
                        println!("{} {}", MENSAJE_DESCONOCIDO, ack_mensaje);
                    }
                }

                Ok(Ok(None)) | Ok(Err(_)) | Err(_) => {
                    println!("{}", ERROR_TIMEOUT_ACK_SEGS);
                    if !self.intentar_reconectar(viaje_finalizado_serializado).await {
                        eprintln!("{}", ERROR_AL_RECONECTAR);
                    }
                }
            }
        }
        std::process::exit(0);
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
                eprintln!("{} {}", ERROR_ENVIAR_FINAL_VIAJE, e);
            } else {
                println!("{}", SE_RECONECTA_ENVIA_FINAL);
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
                println!("{} {}", COMENSAL_NO_CONECTA_COORDINADOR, servidor);
            }
        }

        if direccion_coordinador.is_some() {
            break;
        }
    }

    if let Some(ip_coordinador) = direccion_coordinador {
        println!("{} {}", COMENSAL_ENCONTRO_COORDINADOR, ip_coordinador);
        let stream = match TcpStream::connect(ip_coordinador).await {
            Ok(stream) => {
                println!("{}", COMENSAL_CONECTADO_COORDINADOR);
                Some(stream)
            }

            Err(_) => None,
        };

        return stream;
    }

    None
}
