//! Este módulo contiene la definición de los mensajes funcionales entre los actores del sistema.

// Imports de crates externas.
use actix::Message;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

// Imports de funciones/estructuras propias.
use crate::mensajes_gateway::MensajeGateway;

// Mensaje para solicitar la autorización de un pago al gateway.
#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "(bool)")]
pub struct EnviarMensajePago {
    pub id_comensal: String,
    pub valor: f32,
    pub tipo_mensaje: MensajeGateway,
}

// Mensaje con el resultado de la autorización del pago.
#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct Autorizacion {
    pub direccion_comensal: SocketAddr,
    pub esta_autorizado: bool,
}

// Mensaje que responde quien es el coordinador actual del sistema.
#[derive(Debug, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct QuienEsCoordinador {
    pub direccion_coordinador: SocketAddr,
}

// Mensaje que envia un comensal para solicitar un viaje.
#[derive(Message, serde::Serialize, serde::Deserialize, Debug)]
#[rtype(result = "()")]
pub struct SolicitarRepartidor {
    pub comida: String,
    pub origen: (f32, f32),
    pub destino: (f32, f32),
    pub pedido_aceptado: bool,
    pub direccion_comensal: SocketAddr,
}

// Mensaje que envia el coordinador a un conductor para ofrecerle un viaje.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct OfertarViaje {
    pub direccion_comensal_o: SocketAddr,
}

// Respuesta del conductor al coordinador indicando si acepta o no el viaje.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct RespuestaOfertaViaje {
    pub direccion_comensal_r: SocketAddr,
    pub esta_aceptado: bool,
}

// Mensaje que le indica el inicio del viaje a un pasajero y un conductor.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct IniciarViajeDelivery {
    pub direccion_comensal_i: SocketAddr,
    pub direccion_conductor_i: SocketAddr,
    pub origen_i: (f32, f32),
    pub destino_i: (f32, f32),
}

// Mensaje que le indica al comensal que el viaje no ha sido aceptado por el conductor.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct RechazarViaje {
    pub respuesta: String,
}

// Mensaje que indica que el viaje terminó.
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct FinalizarViaje {
    pub direccion_comensal_f: SocketAddr,
    pub direccion_conductor_f: SocketAddr,
    pub pos_destino: (f32, f32),
}

// Mensaje que le indica al coordinador la posicion de un conductor.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct Posicion {
    pub posicion: (f32, f32),
}

// Mensaje que manda el repartidor
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct BuscandoTrabajoRepartidor {
    pub buscando_trabajo: bool,
    pub posicion: (f32, f32),
}

// Mensaje para identificar quien es, o no, un repartidor.
#[derive(Message, serde::Serialize, serde::Deserialize, Debug)]
#[rtype(result = "()")]
pub struct MensajeIdentidad {
    pub ubicacion: (f32, f32),
    pub soy_repartidor: bool,
}

// Mensaje que envia un comensal para solicitar un delivery
#[derive(Message, serde::Serialize, serde::Deserialize, Debug)]
#[rtype(result = "()")]
pub struct SolicitarPedido {
    pub comida: String,
    pub destino: (f32, f32),
}

// Mensaje que contiene la logica de recepción de un pedido
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct RecibirPedido {
    pub direccion_comensal_o: SocketAddr,
    pub comida: String,
    pub ubicacion_comensal: (f32, f32),
}

// Mensaje que contiene la logica de recepción de un pedido del lado del restaurante.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct PedidoAlRestaurante {
    pub direccion_comensal_r: SocketAddr,
    pub esta_aceptado: bool,
    pub comida: String,
    pub posicion_comensal: (f32, f32),
}
