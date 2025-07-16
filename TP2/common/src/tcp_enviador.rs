//! Actor responsable de enviar mensajes por una conexión TCP saliente. Recibe mensajes de tipo 'MensajeTCP' y los escribe en el socket.

// Imports de crates externas.
use actix::prelude::*;
use actix_async_handler::async_handler;
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;

// Constantes.
const ERROR_ENVIO: &str = "Error al enviar mensaje TCP:";

// Estructura que representa al actor 'TcpEnviador' que enviará mensajes TCP.
pub struct TcpEnviador {
    pub escribir: Option<WriteHalf<TcpStream>>,
}

// Estructura que representa un mensaje TCP a enviar.
#[derive(Message)]
#[rtype(result = "()")]
pub struct MensajeTCP(pub String);

// Implementa el trait 'Actor'.
impl Actor for TcpEnviador {
    type Context = Context<Self>;
}

// Implementación de los handlers del actor 'TcpEnviador'.
#[allow(clippy::unused_unit)]
#[async_handler]
impl Handler<MensajeTCP> for TcpEnviador {
    type Result = ();
    // Método que maneja el envío de un mensaje TCP.
    async fn handle(&mut self, msj: MensajeTCP, _contexto: &mut Self::Context) -> Self::Result {
        let msj_enviar = format!("{}\n", msj.0);
        if let Some(mut escribir) = self.escribir.take() {
            let resultado_escritura = async move {
                if let Err(e) = escribir.write_all(msj_enviar.as_bytes()).await {
                    if e.kind() == std::io::ErrorKind::BrokenPipe {
                        return None;
                    } else {
                        panic!("{} {:?}", ERROR_ENVIO, e);
                    }
                }

                Some(escribir)
            }
            .await;
            self.escribir = resultado_escritura;
        }
    }
}
