//! Este módulo contiene la lógica de los mensajes del 'Coordinador'.

// Imports de crates externas.
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

// Imports de funciones/estructuras propias.
pub type Peers = Arc<HashMap<SocketAddr, (Addr<TcpEnviador>, Instant)>>;
use crate::server::client_server::EstadoRepartidor;
use common::tcp_enviador::TcpEnviador;

// Este mensaje conecta al Coordinador con los Servidores para que puedan enviar mensajes y actualizaciones.
#[derive(Message)]
#[rtype(result = "()")]
pub struct ConvertirseEnCoordinador;

// Este mensaje conecta al Coordinador con un nuevo Servidor si es que este se conecto mas tarde que el Coordinador.
#[derive(Message)]
#[rtype(result = "()")]
pub struct ConectarNuevoServidor {
    pub nuevo_servidor: SocketAddr,
}

// Enum que representa las acciones que se pueden realizar en actualizaciones.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Accion {
    Insertar,
    Eliminar,
    Actualizar,
}

// Mensaje que se usa para actualizar a los comensales de los Servidores no coordinadores.
#[derive(Message, Serialize, Deserialize, Clone, Debug)]
#[rtype(result = "()")]
pub struct ActualizarComensales {
    pub accion: Accion,
    pub comensal: SocketAddr,
    pub origen: (f32, f32),
    pub destino: (f32, f32),
}

// Mensaje para actuailzar a los repartidores en Servidores no coordinadores.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct ActualizarRepartidores {
    pub accion: Accion,
    pub repartidor: SocketAddr,
    pub posicion: (f32, f32),
    pub id_comensal_actual: Option<SocketAddr>,
    pub status: EstadoRepartidor,
}

// Este mensaje le dice a los Servidores que manejen un pedido de un comensal.
#[derive(Message, Serialize, Deserialize)]
#[rtype(result = "()")]
pub struct HandlePedido {
    pub id_comensal_ht: SocketAddr,
}

// Mensaje interno para agregar un nuevo servidor al diccionario 'TCP'.
#[derive(Message)]
#[rtype(result = "()")]
pub struct AgregarServidorADiccionario {
    pub serv_address: SocketAddr,
    pub serv_enviador: Addr<TcpEnviador>,
}

// Mensaje intero para obtener el diccionario 'TCP'.
#[derive(Message)]
#[rtype(result = "Peers")]
pub struct ObtenerPeerDict;

// Mensaje interno apra obtener la cantidad de peers y aumentarlo se usa para balanceo de carga en round robin.
#[derive(Message)]
#[rtype(result = "u8")]
pub struct ObtenerContadorPeer;

// Mensaje para actualizar el estado de los 'Repartidores'.
#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct UpdateRepartidores {
    pub accion: Accion,
    pub repartidor: SocketAddr,
    pub posicion: (f32, f32),
    pub id_comensal_actual: Option<SocketAddr>,
    pub status: EstadoRepartidor,
    pub repartidor_activo: bool,
}

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
#[rtype(result = "()")]
pub struct ActualizarRestaurantes {
    pub accion: Accion,
    pub restaurante: SocketAddr,
    pub posicion: (f32, f32),
    pub id_comensal_actual: Option<SocketAddr>,
    pub status: EstadoRepartidor,
}
