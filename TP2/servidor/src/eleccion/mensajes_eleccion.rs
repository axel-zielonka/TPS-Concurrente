//! Este módulo contiene la lógica de los mensajes del 'EleccionCoordinador' para manejar elecciones y coordinadores en nuestro sistema distribuido.

// Imports de crates externas.
use actix::Message;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

// Mensaje interno para modelar un ping al coordinador.
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct PingCoordinador;

// Mensaje que permite a un servidor verificar si es el coordinador.
#[derive(Debug, Clone, Message)]
#[rtype(result = "bool")]
pub struct SoyCoordinador;

// Mensaje que permite a un servidor obtener la dirección del coordinador.
#[derive(Debug, Clone, Message)]
#[rtype(result = "Option<SocketAddr>")]
pub struct ObtenerDireccionLider;

// Mensaje que permite a un servidor iniciar una elección para nuevo lider.
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct IniciarEleccion;

// Mensaje que puede ser enviado y recibido para indicar que una elección está ocurriendo.
#[derive(Debug, Serialize, Deserialize, Clone, Message)]
#[rtype(result = "()")]
pub struct MensajeEleccion {
    pub candidatos: Vec<SocketAddr>,
}

// Este mensaje es recibido por el coordinador y responde con un ACK.
#[derive(Debug, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct MensajePing {
    pub id_enviador: SocketAddr,
}

// Mensaje que le notifica a todos los servidores quien es el nuevo coordinador.
#[derive(Debug, Serialize, Deserialize, Clone, Message)]
#[rtype(result = "()")]
pub struct MensajeCoordinador {
    pub coordinador: SocketAddr,
}

// Mensaje interno para asignar un nuevo coordinador.
#[derive(Message)]
#[rtype(result = "()")]
pub struct AsignarDireccionLider {
    pub id_coordinador: SocketAddr,
}

// Mensaje interno para obtener la dirección del coordinador actual.
#[derive(Message)]
#[rtype(result = "Option<SocketAddr>")]
pub struct ObtenerDireccionCoordinador;
