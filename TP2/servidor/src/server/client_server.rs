//! Este módulo contiene la lógica de la comunicación 'Clientes --> Servidor'.

// Imports de crates externas.
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

// Imports de funciones/estructuras propias.
use crate::almacenamiento::mensajes_almacenamiento::{
    HayRecursosDisponibles, InsertarComensal, InsertarRepartidor, InsertarRestaurante,
    ObtenerComensal,
};
use crate::coordinador::mensajes_coordinador::{
    Accion, ActualizarComensales, ActualizarRepartidores, ActualizarRestaurantes, HandlePedido,
};
use crate::eleccion::mensajes_eleccion::SoyCoordinador;
use crate::server::server::Server;
use crate::utils::acciones_gateway::{generar_mensaje_chequeo_pago, obtener_respuesta_pago};
use crate::utils::logs::log_funcionamiento;
use common::mensajes::{
    Autorizacion, MensajeIdentidad, PedidoAlRestaurante, Posicion, RechazarViaje, SolicitarPedido,
    SolicitarRepartidor,
};
use common::tcp_enviador::MensajeTCP;
use common::utils::obtener_tupla_random;

// Mensaje que encapsula el estado del repartidor.
#[derive(Debug, Clone, Serialize, Deserialize, Message, PartialEq)]
#[rtype(result = "()")]
pub enum EstadoRepartidor {
    Active,
    Waiting,
    OnTrip,
}

// Mensaje que indica que un viaje finalizo.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct ViajeFinalizado {
    pub id_comensal_tf: SocketAddr,
    pub id_repartidor_tf: SocketAddr,
}

// Handler para manejar la solicitud de un repartidor.
impl Handler<SolicitarRepartidor> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: SolicitarRepartidor, _ctx: &mut Self::Context) -> Self::Result {
        let actor_almacenamiento = self.almacenamiento_addr.clone();
        let client_addr = self.client_addr;
        let cord_eleccion_clone = self.eleccion_coordinador.clone();
        let cord_clone = self.coordinador.clone();
        Box::pin(
            async move {
                if !msg.pedido_aceptado {
                    if let Ok(Some(pedido_rechazado)) = actor_almacenamiento
                        .send(ObtenerComensal {
                            id: msg.direccion_comensal,
                        })
                        .await
                    {
                        let mensaje_tcp = match serde_json::to_string(&RechazarViaje {
                            respuesta: "Viaje rechazado por restaurante".to_string().to_owned(),
                        }) {
                            Ok(json_string) => MensajeTCP(json_string),
                            Err(err) => {
                                eprintln!(
                                    "{} {}",
                                    crate::utils::acciones_gateway::ERROR_AL_SERIALIZAR,
                                    err
                                );
                                MensajeTCP(crate::utils::acciones_gateway::ERROR_STR.to_string())
                            }
                        };

                        if let Some(sender) = pedido_rechazado.enviador_comensal.as_ref() {
                            sender
                                .send(mensaje_tcp)
                                .await
                                .expect("Error al enviar mensaje 'RechazarViaje'.");
                        }
                    }
                    return;
                } else {
                    log_funcionamiento(
                        "| ----- Solicitud DE RESTAURANTE A UN REPARTIDOR ----- | ".to_string(),
                    );
                    if cord_eleccion_clone
                        .send(SoyCoordinador)
                        .await
                        .unwrap_or(false)
                    {
                        if let Err(err) = cord_clone
                            .send(HandlePedido {
                                id_comensal_ht: client_addr,
                            })
                            .await
                        {
                            log_funcionamiento(format!("Failed to handle request: {:?}", err));
                        }
                    }
                }
            }
            .into_actor(self),
        )
    }
}

// Handler para manejar la indicacion de una posicion.
impl Handler<Posicion> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: Posicion, _ctx: &mut Self::Context) -> Self::Result {
        let cord_election_clone = self.eleccion_coordinador.clone();
        let cord_clone = self.coordinador.clone();

        let tcp_enviador = self.tcp_enviador.clone();
        let client_addr = self.client_addr;
        let actor_almacenamiento = self.almacenamiento_addr.clone();

        Box::pin(
            async move {
                if cord_election_clone
                    .send(SoyCoordinador)
                    .await
                    .unwrap_or(false)
                {
                    actor_almacenamiento
                        .send(InsertarRepartidor {
                            id: client_addr,
                            id_comensal_actual: None,
                            posicion_restaurante: msg.posicion,
                            enviador_repartidor: Some(tcp_enviador),
                            estado: EstadoRepartidor::Active,
                            time_stamp: std::time::Instant::now(),
                        })
                        .await
                        .unwrap();

                    if let Err(e) = cord_clone
                        .send(ActualizarRepartidores {
                            repartidor: client_addr,
                            posicion: msg.posicion,
                            accion: Accion::Insertar,
                            id_comensal_actual: None,
                            status: EstadoRepartidor::Active,
                        })
                        .await
                    {
                        log_funcionamiento(format!("Fallo actualizar repartidores: {:?}", e));
                    }
                }
            }
            .into_actor(self),
        )
    }
}

// Handler para manejar la solicitud de un pedido.
impl Handler<SolicitarPedido> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: SolicitarPedido, ctx: &mut Self::Context) -> Self::Result {
        log_funcionamiento("| ----- Solicitud de Pedido DESDE COMENSAL ----- | ".to_string());

        let tcp_enviador = self.tcp_enviador.clone();
        let client_addr = self.client_addr;
        let almacenamiento_actor = self.almacenamiento_addr.clone();

        let cord_eleccion_clone = self.eleccion_coordinador.clone();
        let cord_clone = self.coordinador.clone();
        let direccion = ctx.address();

        Box::pin(
            async move {
                if cord_eleccion_clone
                    .send(SoyCoordinador)
                    .await
                    .unwrap_or(false)
                {
                    almacenamiento_actor
                        .send(InsertarComensal {
                            id: client_addr,
                            posicion_restaurante_del_pedido: obtener_tupla_random(),
                            ubicacion_comensal: msg.destino,
                            enviador_comensal: Some(tcp_enviador),
                        })
                        .await
                        .unwrap();

                    if let Err(e) = cord_clone
                        .send(ActualizarComensales {
                            comensal: client_addr,
                            origen: obtener_tupla_random(),
                            destino: msg.destino,
                            accion: Accion::Insertar,
                        })
                        .await
                    {
                        log_funcionamiento(format!("Error al actualizar comensales: {:?}", e));
                    }

                    let recursos_disponibles = almacenamiento_actor
                        .send(HayRecursosDisponibles)
                        .await
                        .unwrap_or(false);

                    if !recursos_disponibles {
                         println!("SERVIDOR - No hay instancias suficientes de repartidor o restaurante para hacer la transacción.");
                        direccion
                            .try_send(Autorizacion {
                                direccion_comensal: client_addr,
                                esta_autorizado: recursos_disponibles,
                            })
                            .expect("Error al enviar falta de instancias.");
                    } else {
                        let passenger_id = format!("{:?}", client_addr.clone());
                        let auth =
                            obtener_respuesta_pago(generar_mensaje_chequeo_pago(passenger_id)).await;
                        log_funcionamiento(format!("El pago fue autorizado: {:?}", auth));
                        direccion
                            .try_send(Autorizacion {
                                direccion_comensal: client_addr,
                                esta_autorizado: auth,
                            })
                            .expect("Error al enviar confirmación de pago.");
                        if auth {
                            direccion
                                .send(PedidoAlRestaurante {
                                    comida: msg.comida,
                                    direccion_comensal_r: client_addr,
                                    esta_aceptado: true,
                                    posicion_comensal: msg.destino,
                                })
                                .await
                                .unwrap();
                        }
                    }
                }
            }
            .into_actor(self),
        )
    }
}

// Handler para manejar el mensaje que indica la identidad.
impl Handler<MensajeIdentidad> for Server {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: MensajeIdentidad, _ctx: &mut Self::Context) -> Self::Result {
        log_funcionamiento("| ----- Nuevo Restaurante Conectado----- |".to_string());

        let cord_election_clone = self.eleccion_coordinador.clone();
        let cord_clone = self.coordinador.clone();

        let tcp_enviador = self.tcp_enviador.clone();
        let client_addr = self.client_addr;
        let actor_almacenamiento = self.almacenamiento_addr.clone();

        Box::pin(
            async move {
                if cord_election_clone
                    .send(SoyCoordinador)
                    .await
                    .unwrap_or(false)
                {
                    actor_almacenamiento
                        .send(InsertarRestaurante {
                            id: client_addr,
                            enviador_repartidor: Some(tcp_enviador),
                            posicion_repartidor: msg.ubicacion,
                            estado: EstadoRepartidor::Active,
                            time_stamp: std::time::Instant::now(),
                        })
                        .await
                        .unwrap();

                    if let Err(e) = cord_clone
                        .send(ActualizarRestaurantes {
                            restaurante: client_addr,
                            posicion: msg.ubicacion,
                            accion: Accion::Insertar,
                            id_comensal_actual: None,
                            status: EstadoRepartidor::Active,
                        })
                        .await
                    {
                        log_funcionamiento(format!("Error al actualizar restaurantes: {:?}", e));
                    }
                }
            }
            .into_actor(self),
        )
    }
}
