//! Este módulo contiene la lógica de la comunicación 'Servidor --> Coordinador'.

// Imports de crates externas.
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

// Imports de funciones/estructuras propias.
use crate::{
    almacenamiento::{
        almacenamiento::Almacenamiento,
        mensajes_almacenamiento::{ActualizarRepartidor, ObtenerComensal, ObtenerRepartidor},
    },
    coordinador::mensajes_coordinador::HandlePedido,
    server::{client_server::EstadoRepartidor, server::Server},
    utils::logs::log_funcionamiento,
};
use common::mensajes::{OfertarViaje, RechazarViaje};
use common::tcp_enviador::MensajeTCP;

// Mensaje que modela el viaje del pedido desde el 'Restaurante' hasta el 'Cliente'.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct HacerPedido {
    pub id_comensal_mt: SocketAddr,
    pub id_repartidor_mt: SocketAddr,
}

// Handler para manejar el mensaje que modela el 'Pedido'.
impl Handler<HacerPedido> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: HacerPedido, _ctx: &mut Self::Context) -> Self::Result {
        log_funcionamiento("| ----- Hacer Pedido ----- |".to_string());
        let comensal = msg.id_comensal_mt;
        let coord_clone = self.coordinador.clone();
        let almacenamiento_actor = self.almacenamiento_addr.clone();
        Box::pin(
            async move {
                if let Ok(Some(repartidor)) = almacenamiento_actor
                    .send(ObtenerRepartidor {
                        id: msg.id_repartidor_mt,
                    })
                    .await
                {
                    if matches!(repartidor.estado, EstadoRepartidor::OnTrip) {
                        log_funcionamiento("El repartidor esta en camino.".to_string());
                        coord_clone
                            .send(HandlePedido {
                                id_comensal_ht: comensal,
                            })
                            .await
                            .expect("Error al enviar 'HandlePedido'.");
                        return;
                    }

                    if matches!(repartidor.estado, EstadoRepartidor::Waiting) {
                        log_funcionamiento("El repartidor ya esta esperando.".to_string());
                        coord_clone
                            .send(HandlePedido {
                                id_comensal_ht: comensal,
                            })
                            .await
                            .expect("Error al enviar 'HandlePedido'.");
                        return;
                    }

                    if let Some(sender) = &repartidor.restaurante_sender {
                        let can_accept_trip = OfertarViaje {
                            direccion_comensal_o: comensal,
                        };

                        match serde_json::to_string(&can_accept_trip) {
                            Ok(json_string) => {
                                sender.try_send(MensajeTCP(json_string)).expect(
                                    "Error al enviar la solicitud al repartidor mas cercano.",
                                );
                            }
                            Err(err) => {
                                eprintln!("Error al serializar el mensaje de 'StartTrip': {}", err);
                            }
                        }

                        almacenamiento_actor
                            .send(ActualizarRepartidor {
                                id_repartidor: msg.id_repartidor_mt,
                                estado: EstadoRepartidor::Waiting,
                                id_comensal: Some(comensal),
                                time_stamp: std::time::Instant::now(),
                            })
                            .await
                            .expect("Error al actualizar el estado del repartidor.");
                    } else {
                        Self::rechazar_pasajero(
                            &msg,
                            almacenamiento_actor,
                            "Enviador de repartidor no disponible.".to_string(),
                        )
                        .await;
                    }
                } else {
                    // No se encuentran 'Repartidores', por lo que se rechaza el pasajero.
                    Self::rechazar_pasajero(
                        &msg,
                        almacenamiento_actor,
                        "Ningun repartidor esta disponible ahora.".to_string(),
                    )
                    .await;
                }
            }
            .into_actor(self),
        )
    }
}

// Handler para rechazar el pedido del cliente.
impl Server {
    async fn rechazar_pasajero(
        msg: &HacerPedido,
        actor_almacenamiento: Arc<Addr<Almacenamiento>>,
        mensaje_rechazo: String,
    ) {
        if let Ok(Some(comensal_rechazado)) = actor_almacenamiento
            .send(ObtenerComensal {
                id: msg.id_comensal_mt,
            })
            .await
        {
            let tcp_message = MensajeTCP(
                serde_json::to_string(&RechazarViaje {
                    respuesta: mensaje_rechazo,
                })
                .unwrap(),
            );

            if let Some(sender) = comensal_rechazado.enviador_comensal.as_ref() {
                sender
                    .send(tcp_message)
                    .await
                    .expect("Error al enviar 'RechazarViaje'.");
            }
        }

        log_funcionamiento("No se encontro ningun repartidor para el comensal.".to_string());
    }
}
