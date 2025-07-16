//! Este módulo contiene la lógica de la comunicación 'Servidor --> Almacenamiento'.

// Imports de crates externas.
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;

// Imports de funciones/estructuras propias.
use crate::almacenamiento::mensajes_almacenamiento::{
    ActualizarRepartidor, BorrarComensal, BorrarRepartidor, InsertarComensal, InsertarRepartidor,
    InsertarRestaurante,
};
use crate::coordinador::mensajes_coordinador::{
    Accion, ActualizarComensales, ActualizarRepartidores, UpdateRepartidores,
};
use crate::server::client_server::EstadoRepartidor;
use crate::server::server::Server;
use crate::utils::logs::log_funcionamiento;

/// Mensaje para actualizar los repartidores en el Almacenamiento.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct HacerActualizacionRepartidores {
    pub upt_msg: ActualizarRepartidores,
}

/// Mensaje para actualizar los comensales en el Almacenamiento.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct HacerActualizacionComensal {
    pub upt_msg: ActualizarComensales,
}

// Mensaje para hacer actualizaciones en un repartidor.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct HacerActualizacionRepartidor {
    pub upt_msg: UpdateRepartidores,
}

// Handler para manejar el pedido de actualizacion de los repartidores.
impl Handler<HacerActualizacionRepartidores> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(
        &mut self,
        driver_update: HacerActualizacionRepartidores,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let d_addr = driver_update.upt_msg.repartidor;
        let posicion = driver_update.upt_msg.posicion;
        let accion = driver_update.upt_msg.accion;
        let comensal = driver_update.upt_msg.id_comensal_actual;
        let estado_comensal = driver_update.upt_msg.status;
        let tcp_enviador_clone = self.tcp_enviador.clone();
        let almacenamiento_actor = self.almacenamiento_addr.clone();

        Box::pin(
            async move {
                if accion == Accion::Insertar {
                    almacenamiento_actor
                        .send(InsertarRepartidor {
                            id: d_addr,
                            posicion_restaurante: posicion,
                            id_comensal_actual: None,
                            enviador_repartidor: Some(tcp_enviador_clone),
                            estado: EstadoRepartidor::Active,
                            time_stamp: std::time::Instant::now(),
                        })
                        .await
                        .expect("Error al enviar AddDriver al almacenamiento");
                } else if accion == Accion::Eliminar {
                    almacenamiento_actor
                        .send(BorrarRepartidor { id: d_addr })
                        .await
                        .expect("Error al enviar BorrarRepartidor al almacenamiento");
                } else if accion == Accion::Actualizar {
                    almacenamiento_actor
                        .send(ActualizarRepartidor {
                            id_repartidor: d_addr,
                            id_comensal: comensal,
                            estado: estado_comensal,
                            time_stamp: Instant::now(),
                        })
                        .await
                        .expect("Error al enviar BorrarRepartidor al almacenamiento");

                    log_funcionamiento(format!("Comensal actualizado con id: {:?}", d_addr));
                }
            }
            .into_actor(self),
        )
    }
}

// Handler para manejar las actualizaciones en los comensales.
impl Handler<HacerActualizacionComensal> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(
        &mut self,
        passenger_update: HacerActualizacionComensal,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let p_addr = passenger_update.upt_msg.comensal;
        let accion = passenger_update.upt_msg.accion;
        let actor_almacenamiento = self.almacenamiento_addr.clone();
        Box::pin(
            async move {
                if accion == Accion::Insertar {
                    actor_almacenamiento
                        .send(InsertarComensal {
                            id: p_addr,
                            posicion_restaurante_del_pedido: passenger_update.upt_msg.origen,
                            ubicacion_comensal: passenger_update.upt_msg.destino,
                            enviador_comensal: None,
                        })
                        .await
                        .expect("Error al enviar AddPassenger al almacenamiento");

                    log_funcionamiento(format!("Comensal agregado {:?}", p_addr));
                } else if accion == Accion::Eliminar {
                    actor_almacenamiento
                        .send(BorrarComensal { id: p_addr })
                        .await
                        .expect("Error al enviar GetPassenger al almacenamiento");

                    log_funcionamiento(format!("Comensal eliminado {:?}", p_addr));
                }
            }
            .into_actor(self),
        )
    }
}

// Handler para hacer las actualizaciones en los repartidores.
impl Handler<HacerActualizacionRepartidor> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(
        &mut self,
        repartidor_update: HacerActualizacionRepartidor,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let r_addr = repartidor_update.upt_msg.repartidor;
        let accion = repartidor_update.upt_msg.accion;
        let actor_almacenamiento = self.almacenamiento_addr.clone();
        Box::pin(
            async move {
                if accion == Accion::Insertar {
                    actor_almacenamiento
                        .send(InsertarRestaurante {
                            id: r_addr,
                            posicion_repartidor: repartidor_update.upt_msg.posicion,
                            enviador_repartidor: None,
                            estado: EstadoRepartidor::Active,
                            time_stamp: Instant::now(),
                        })
                        .await
                        .expect("Error al enviar AddRepartidor al almacenamiento");

                    log_funcionamiento(format!("Repartidor agregado {:?}", r_addr));
                } else if accion == Accion::Eliminar {
                    actor_almacenamiento
                        .send(BorrarRepartidor { id: r_addr })
                        .await
                        .expect("Error al enviar RemoveRepartidor al almacenamiento");

                    log_funcionamiento(format!("Repartidor eliminado {:?}", r_addr));
                }
            }
            .into_actor(self),
        )
    }
}
