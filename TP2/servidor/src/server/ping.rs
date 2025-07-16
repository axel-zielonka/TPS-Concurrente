//! Este módulo contiene la lógica del sitema de 'Ping' entre las instancias del servidor.

// Imports de crates externas.
use actix::prelude::*;
use std::sync::Arc;
use tokio::time::Duration;

// Imports de funciones/estructuras propias.
use crate::coordinador::mensajes_coordinador::ConectarNuevoServidor;
use crate::eleccion::eleccion::EleccionCoordinador;
use crate::eleccion::mensajes_eleccion::{MensajePing, ObtenerDireccionLider, PingCoordinador};
use crate::server::server::Server;
use crate::utils::constantes::INTERVALO_PING;
use crate::utils::logs::log_eleccion;
use common::mensajes::QuienEsCoordinador;
use common::tcp_enviador::MensajeTCP;

// Mensaje que pregunta quien es el coordinador actual del sistema.
#[derive(Message)]
#[rtype(result = "()")]
pub struct WhoIsCoordinator;

// Handler para manejar el mensaje de 'Ping'.
impl Handler<MensajePing> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, message: MensajePing, _: &mut Self::Context) -> Self::Result {
        let tcp_enviador_clone = self.tcp_enviador.clone();
        let coord_clone = self.coordinador.clone();
        Box::pin(
            async move {
                tcp_enviador_clone
                    .try_send(MensajeTCP("Ack".to_string()))
                    .expect("Error al enviar la respuesta.");

                // Conectarse al socket si no está conectado.
                if let Err(e) = coord_clone.try_send(ConectarNuevoServidor {
                    nuevo_servidor: message.id_enviador,
                }) {
                    log_eleccion(format!("Error al enviar ConectarNuevoServidor: {:?}", e));
                }
            }
            .into_actor(self),
        )
    }
}

// Handler para manejar el mensaje que pregunta quien es el coordinador.
impl Handler<WhoIsCoordinator> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, _: WhoIsCoordinator, _: &mut Self::Context) -> Self::Result {
        let coord_elecc_clone = self.eleccion_coordinador.clone();
        let tcp_enviador_clone = self.tcp_enviador.clone();
        Box::pin(
            async move {
                let coord_addr = coord_elecc_clone
                    .send(ObtenerDireccionLider {})
                    .await
                    .expect("Error al obtener la direccion del coordinador");

                if coord_addr.is_none() {
                    tcp_enviador_clone
                        .try_send(MensajeTCP("Ack".to_string()))
                        .expect("Error al enviar la respuesta.");
                    return;
                }

                let who_is_coord_msg = QuienEsCoordinador {
                    direccion_coordinador: coord_addr.unwrap(),
                };

                tcp_enviador_clone
                    .try_send(MensajeTCP(
                        serde_json::to_string(&who_is_coord_msg).unwrap(),
                    ))
                    .expect("Error al enviar la respuesta.");
            }
            .into_actor(self),
        )
    }
}

// Método que spawnea la tarea de 'Ping' de forma indefinida.
pub fn spawn_ping_task(coordinator_election: Arc<Addr<EleccionCoordinador>>) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(INTERVALO_PING)).await;
            if let Err(e) = coordinator_election.try_send(PingCoordinador) {
                log_eleccion(format!("Error al enviar ping: {:?}", e));
            }
        }
    });
}
