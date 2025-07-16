//! Este módulo contiene la lógica de la comunicación 'Coordinador --> Servidor'.

// Imports de crates externas.
use actix::prelude::*;
use actix::Message;
use std::net::SocketAddr;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

// Imports de funciones/estructuras propias.
use crate::almacenamiento::mensajes_almacenamiento::ObtenerRepartidorCercano;
use crate::coordinador::mensajes_coordinador::HandlePedido;
use crate::eleccion::mensajes_eleccion::ObtenerDireccionLider;
use crate::server::server::Server;
use crate::server::server_coordinador::HacerPedido;
use crate::utils::constantes::MAX_REINTENTOS;
use crate::utils::logs::log_funcionamiento;
use common::tcp_enviador::MensajeTCP;

// Constantes.
const TIMEOUT_SEGUNDOS: u64 = 3;
const SLEEP_DELAY: u64 = 1;

// Mensaje que modela la busqueda de un 'Repartidor' cercano al 'Restaurante' para llevarle el 'Pedido'.
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct EncontrarRepartidorCercano {
    pub direccion_comensal: SocketAddr,
}

// Handler para manejar los 'Pedidos' que recibe el 'Server'.
impl Handler<HandlePedido> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: HandlePedido, ctx: &mut Self::Context) -> Self::Result {
        log_funcionamiento("| ----- Manejar Pedido ----- |".to_string());
        let actor_addr = ctx.address();
        Box::pin(
            async move {
                let encontrar_msj = EncontrarRepartidorCercano {
                    direccion_comensal: msg.id_comensal_ht,
                };

                actor_addr
                    .try_send(encontrar_msj)
                    .expect("Error al enviar 'EncontrarRepartidorCercano'.");
            }
            .into_actor(self),
        )
    }
}

// Handler que controla los mensajes recibidos de los 'RepartidoresCercanos'.
impl Handler<EncontrarRepartidorCercano> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(
        &mut self,
        _msg: EncontrarRepartidorCercano,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let coord_elecc_clone = self.eleccion_coordinador.clone();
        let comensal_actual = _msg.direccion_comensal;
        let almacenamiento_actor = self.almacenamiento_addr.clone();
        let self_addr = ctx.address();
        Box::pin(
            async move {
                log_funcionamiento("| ----- Buscando Repartidores ----- |".to_string());
                let mut hacer_viaje = HacerPedido {
                    id_comensal_mt: comensal_actual,
                    id_repartidor_mt: SocketAddr::new([0, 0, 0, 0].into(), 0),
                };

                if let Some(repartidor_cercano) = almacenamiento_actor
                    .send(ObtenerRepartidorCercano {})
                    .await
                    .expect("Error al obtener repartidores cercanos.")
                {
                    log_funcionamiento(format!(
                        "Encontró un repartidor!: {:?}",
                        repartidor_cercano
                    ));
                    hacer_viaje = HacerPedido {
                        id_comensal_mt: comensal_actual,
                        id_repartidor_mt: repartidor_cercano,
                    };
                }

                // Enviar al coordinador.
                let coord_addr = coord_elecc_clone
                    .send(ObtenerDireccionLider {})
                    .await
                    .expect("Error al obtener dirección del 'Coordinador'.");

                if let Some(coord_addr) = coord_addr {
                    enviar_viaje_a_coordinador(hacer_viaje, coord_addr, self_addr).await;
                } else {
                    log_funcionamiento("Dirección del 'Coordinador' no encontrada.".to_string());
                }
            }
            .into_actor(self),
        )
    }
}

// Método para enviarle el viaje al 'Coordinador' para que lo gestione.
async fn enviar_viaje_a_coordinador(
    hacer_viaje: HacerPedido,
    coordinator_addr: SocketAddr,
    admin_addr: Addr<Server>,
) {
    log_funcionamiento(format!(
        "| ----- Enviar viaje al 'Coordinador' [{:?}] ----- |",
        coordinator_addr
    ));

    let mut got_res = false;
    let mut got_ack = false;
    for _ in 0..MAX_REINTENTOS {
        let con = TcpStream::connect(coordinator_addr).await;
        match con {
            Ok(s) => {
                let stream = Some(s);
                let (lector, mut escritor) = match stream {
                    Some(stream) => split(stream),
                    None => panic!("Stream is NULL"),
                };

                if let Ok(serializado) = serde_json::to_string(&hacer_viaje) {
                    let serializado = format!("{}\n", serializado);
                    let msg = MensajeTCP(serializado);
                    match escritor.write_all(msg.0.as_bytes()).await {
                        Ok(_) => (),
                        Err(e) => println!("Error al escribir: {}", e),
                    }
                } else {
                    eprintln!("Error al serializar.");
                }

                let mut lector = BufReader::new(lector);
                let mut line = String::new();
                match timeout(
                    Duration::from_secs(TIMEOUT_SEGUNDOS),
                    lector.read_line(&mut line),
                )
                .await
                {
                    Ok(Ok(_)) => {
                        if line.trim() == "Ack" {
                            got_ack = true;
                        }
                        got_res = true;
                    }

                    Ok(Err(_)) => {
                        println!("Error al leer la línea.");
                    }

                    Err(_) => {
                        log_funcionamiento("Timeout.".to_string());
                        got_res = false;
                    }
                }
                break;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(SLEEP_DELAY)).await;
                continue;
            }
        }
    }

    if !got_res {
        log_funcionamiento("Error al enviar el viaje al 'Coordinador'.".to_string());
        return;
    }

    if got_ack {
        log_funcionamiento("Se envió el viaje al 'Coordinador'".to_string());
    } else {
        admin_addr
            .try_send(HandlePedido {
                id_comensal_ht: hacer_viaje.id_comensal_mt,
            })
            .expect("Error en el proceso de gestionar el pedido para enviarlo al 'Coordinador'.");
    }
}
