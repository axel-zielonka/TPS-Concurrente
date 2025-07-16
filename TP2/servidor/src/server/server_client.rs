//! Este módulo contiene la lógica de la comunicación 'Servidor --> Clientes'.

// Imports de crates externas.
use actix::prelude::*;
use std::sync::Arc;
use std::time::Instant;

// Imports de funciones/estructuras propias.
use crate::almacenamiento::almacenamiento::Almacenamiento;
use crate::almacenamiento::mensajes_almacenamiento::{
    ActualizarComensalUbicacionRestaurante, ActualizarRepartidor, Finalizado, ObtenerComensal,
    ObtenerRepartidor, QuieroPedidoSoyComensal,
};
use crate::coordinador::mensajes_coordinador::{
    Accion, ActualizarComensales, ActualizarRepartidores, HandlePedido,
};
use crate::server::{client_server::EstadoRepartidor, server::Server};
use crate::utils::acciones_gateway::{generar_mensaje_pago_hecho, obtener_respuesta_pago};
use crate::utils::logs::log_funcionamiento;

use common::mensajes::{PedidoAlRestaurante, RecibirPedido};
use common::utils::obtener_tupla_random;
use common::{
    mensajes::{FinalizarViaje, IniciarViajeDelivery, RespuestaOfertaViaje},
    tcp_enviador::MensajeTCP,
};

// Handler para manejar el mensaje de finalizacion de viaje.
impl Handler<FinalizarViaje> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: FinalizarViaje, _ctx: &mut Self::Context) -> Self::Result {
        log_funcionamiento(format!(
            "| ----- Viaje Finalizado {:?} ----- |",
            self.client_addr
        ));

        let almacenamiento_actor = self.almacenamiento_addr.clone();
        let id_repartidor = msg.direccion_conductor_f;
        let id_comensal = msg.direccion_comensal_f;
        let posicion_destino = msg.pos_destino;

        let coord_clone = self.coordinador.clone();

        Box::pin(
            async move {
                // Si el repartidor termino con el envio, vuelve
                if almacenamiento_actor
                    .send(Finalizado { id_comensal })
                    .await
                    .unwrap()
                {
                    return;
                }

                let string_id_comensal = format!("{:?}", id_comensal.clone());
                obtener_respuesta_pago(generar_mensaje_pago_hecho(string_id_comensal)).await;

                avisar_al_comensal(almacenamiento_actor.clone(), msg.clone()).await;
                avisar_al_repartidor(almacenamiento_actor.clone(), msg).await;

                almacenamiento_actor
                    .send(FinalizarViaje {
                        direccion_comensal_f: id_comensal,
                        direccion_conductor_f: id_repartidor,
                        pos_destino: posicion_destino,
                    })
                    .await
                    .unwrap();

                coord_clone
                    .try_send(ActualizarRepartidores {
                        repartidor: id_repartidor,
                        posicion: (posicion_destino.0, posicion_destino.1),
                        accion: Accion::Actualizar,
                        id_comensal_actual: None,
                        status: EstadoRepartidor::Active,
                    })
                    .expect("Error al enviar ActualizarRepartidor");

                coord_clone
                    .try_send(ActualizarComensales {
                        comensal: id_comensal,
                        origen: (0.0, 0.0),
                        destino: (20.0, 20.0),
                        accion: Accion::Eliminar,
                    })
                    .expect("Error al enviar ActualizarComensales");
            }
            .into_actor(self),
        )
    }
}

// Método para avisarle a un comensal que se recibio el fin del viaje.
async fn avisar_al_comensal(storage: Arc<Addr<Almacenamiento>>, msg: FinalizarViaje) {
    let comensal = storage
        .send(ObtenerComensal {
            id: msg.direccion_comensal_f,
        })
        .await
        .unwrap();
    if let Some(entidad_comensal) = comensal {
        if let Some(sender_comensal) = entidad_comensal.enviador_comensal.as_ref() {
            sender_comensal
                .try_send(MensajeTCP("ACK".to_string()))
                .expect("Error al enviar FinalizarViaje");
        }
    }
}

// Método para avisarle a un repartidor que se recibio el fin del viaje.
async fn avisar_al_repartidor(storage: Arc<Addr<Almacenamiento>>, msg: FinalizarViaje) {
    let repartidor = storage
        .send(ObtenerRepartidor {
            id: msg.direccion_conductor_f,
        })
        .await
        .unwrap();
    if let Some(entidad_repartidor) = repartidor {
        if let Some(sender_comensal) = entidad_repartidor.restaurante_sender.as_ref() {
            sender_comensal
                .try_send(MensajeTCP("ACK".to_string()))
                .expect("Error al enviar FinalizarViaje");
        }
    }
}

// Handler para manejar la respuesta que brinda el repartidor sobre la aceptación del viaje.
impl Handler<RespuestaOfertaViaje> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: RespuestaOfertaViaje, _ctx: &mut Self::Context) -> Self::Result {
        log_funcionamiento("| ----- Respuesta Puede Aceptar Viaje ----- |".to_string());

        let coord_clone: Arc<Addr<crate::coordinador::coordinador::Coordinador>> =
            self.coordinador.clone();
        let almacenamiento_actor = self.almacenamiento_addr.clone();
        let enviador_repartidor = self.tcp_enviador.clone();
        let id_repartidor = self.client_addr;

        Box::pin(
            async move {
                if msg.esta_aceptado {
                    log_funcionamiento(format!(
                        "| ----- Comenzar Viaje {:?} ----- |",
                        msg.direccion_comensal_r
                    ));

                    let entidad_comensal_opt = almacenamiento_actor
                        .send(ObtenerComensal {
                            id: msg.direccion_comensal_r,
                        })
                        .await
                        .unwrap();

                    if let Some(entidad_comensal) = entidad_comensal_opt {
                        let mensaje_comienzo = IniciarViajeDelivery {
                            direccion_comensal_i: msg.direccion_comensal_r,
                            direccion_conductor_i: id_repartidor,
                            origen_i: entidad_comensal.posicion_comensal,
                            destino_i: entidad_comensal.destino_comensal,
                        };
                        let enviador_comensal = entidad_comensal.enviador_comensal.as_ref();
                        if let Some(enviador_comensal) = enviador_comensal {
                            match serde_json::to_string(&mensaje_comienzo) {
                                Ok(json_string) => {
                                    enviador_comensal
                                        .try_send(MensajeTCP(json_string.clone()))
                                        .expect("Error al enviar IniciarViaje");
                                    enviador_repartidor
                                        .try_send(MensajeTCP(json_string))
                                        .expect("Error al enviar IniciarViaje");
                                }
                                Err(err) => {
                                    eprintln!(
                                        "Error al serializar el mensaje IniciarViaje: {}",
                                        err
                                    );
                                }
                            }

                            almacenamiento_actor
                                .send(ActualizarRepartidor {
                                    id_repartidor,
                                    id_comensal: Some(msg.direccion_comensal_r),
                                    time_stamp: Instant::now(),
                                    estado: EstadoRepartidor::OnTrip,
                                })
                                .await
                                .unwrap();

                            coord_clone
                                .try_send(ActualizarRepartidores {
                                    repartidor: id_repartidor,
                                    posicion: (0.0, 0.1),
                                    accion: Accion::Actualizar,
                                    id_comensal_actual: Some(msg.direccion_comensal_r),
                                    status: EstadoRepartidor::OnTrip,
                                })
                                .expect("Error al enviar la actualizacion de los repartidores.");
                        } else {
                        }
                    }
                } else {
                    almacenamiento_actor
                        .send(ActualizarRepartidor {
                            id_repartidor,
                            id_comensal: Some(msg.direccion_comensal_r),
                            time_stamp: Instant::now(),
                            estado: EstadoRepartidor::Active,
                        })
                        .await
                        .unwrap();

                    coord_clone
                        .try_send(ActualizarRepartidores {
                            repartidor: id_repartidor,
                            posicion: (0.0, 0.1),
                            accion: Accion::Actualizar,
                            id_comensal_actual: Some(msg.direccion_comensal_r),
                            status: EstadoRepartidor::Active,
                        })
                        .expect("Error al enviar la actualizacion de los repartidores.");

                    // Buscar otro repartidor en caso de ser necesario.
                    coord_clone
                        .try_send(HandlePedido {
                            id_comensal_ht: msg.direccion_comensal_r,
                        })
                        .expect("Error al enviar el manejo del pedido.");
                }
            }
            .into_actor(self),
        )
    }
}

// Handler para manejar el envio del pedido al restaurante.
impl Handler<PedidoAlRestaurante> for Server {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: PedidoAlRestaurante, _ctx: &mut Self::Context) -> Self::Result {
        log_funcionamiento(format!(
            "| ----- Pedido al restaurante de: {:?} ----- |",
            self.client_addr
        ));
        let almacenamiento_actor = self.almacenamiento_addr.clone();
        let enviador_repartidor = self.tcp_enviador.clone();
        let comida = msg.comida.clone();

        Box::pin(
            async move {
                if msg.esta_aceptado {
                    let entidad_restaurante_opt = almacenamiento_actor
                        .send(QuieroPedidoSoyComensal {
                            ubicacion_comensal: obtener_tupla_random(),
                        })
                        .await
                        .unwrap();

                    if let Some(restaurante) = entidad_restaurante_opt {
                        let mensaje_inicio = RecibirPedido {
                            direccion_comensal_o: msg.direccion_comensal_r,
                            comida: comida,
                            ubicacion_comensal: msg.posicion_comensal,
                        };
                        almacenamiento_actor
                            .send(ActualizarComensalUbicacionRestaurante {
                                nueva_ubicacion: restaurante.posicion_restaurante,
                                id_comensal: msg.direccion_comensal_r,
                            })
                            .await
                            .unwrap();
                        let enviador_comensal = restaurante.repartidor_sender.as_ref();
                        if let Some(enviador_comensal) = enviador_comensal {
                            match serde_json::to_string(&mensaje_inicio) {
                                Ok(json_string) => {
                                    enviador_comensal
                                        .try_send(MensajeTCP(json_string.clone()))
                                        .expect("Error al enviar IniciarViaje");
                                    enviador_repartidor
                                        .try_send(MensajeTCP(json_string))
                                        .expect("Error al enviar IniciarViaje");
                                }
                                Err(err) => {
                                    eprintln!(
                                        "Error al serializar el mensaje IniciarViaje: {}",
                                        err
                                    );
                                }
                            }
                        }
                    }
                }
            }
            .into_actor(self),
        )
    }
}
