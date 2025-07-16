//! Este módulo contiene la lógica de los mensajes del actor 'Almacenamiento'.

// Imports de crates externas.
use actix::{Addr, Message};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

// Imports de funciones/estructuras propias.
use crate::server::client_server::EstadoRepartidor;
use crate::utils::entidades::{EntidadComensal, EntidadRepartidor, EntidadRestaurante};
use common::tcp_enviador::TcpEnviador;

// Implementación de la estructura `RepartidorAusente` que representa un repartidor muerto.
pub struct RepartidorAusente {
    pub id_repartidor: SocketAddr,
    pub id_comensal: Option<SocketAddr>,
    pub enviador_comensal: Option<Arc<Addr<TcpEnviador>>>,
}

// Mensaje para obtener un 'Comensal' del almacenamiento.
#[derive(Message)]
#[rtype(result = "Option<EntidadComensal>")]
pub struct ObtenerComensal {
    pub id: SocketAddr,
}

// Mensaje para obtener un 'Repartidor' del almacenamiento.
#[derive(Message)]
#[rtype(result = "Option<EntidadRepartidor>")]
pub struct ObtenerRepartidor {
    pub id: SocketAddr,
}

// Mensaje para actualizar un 'Repartidor' del almacenamiento.
#[derive(Message)]
#[rtype(result = "()")]
pub struct ActualizarRepartidor {
    pub id_repartidor: SocketAddr,
    pub id_comensal: Option<SocketAddr>,
    pub estado: EstadoRepartidor,
    pub time_stamp: Instant,
}

// Mensaje para saber si están disponibles los recursos necesarios.
#[derive(Message)]
#[rtype(result = "bool")]
pub struct HayRecursosDisponibles;

// Mensaje para insertar un 'Restaurante' en el almacenamiento.
#[derive(Message)]
#[rtype(result = "()")]
pub struct InsertarRepartidor {
    pub id: SocketAddr,
    pub posicion_restaurante: (f32, f32),
    pub id_comensal_actual: Option<SocketAddr>,
    pub enviador_repartidor: Option<Arc<Addr<TcpEnviador>>>,
    pub estado: EstadoRepartidor,
    pub time_stamp: Instant,
}

// Mensaje para insertar un 'Comensal' en el almacenamiento.
#[derive(Message)]
#[rtype(result = "()")]
pub struct InsertarComensal {
    pub id: SocketAddr,
    pub posicion_restaurante_del_pedido: (f32, f32),
    pub ubicacion_comensal: (f32, f32),
    pub enviador_comensal: Option<Arc<Addr<TcpEnviador>>>,
}

// Mensaje para insertar un 'Repartidor' en el almacenamiento.
#[derive(Message)]
#[rtype(result = "()")]
pub struct InsertarRestaurante {
    pub id: SocketAddr,
    pub posicion_repartidor: (f32, f32),
    pub enviador_repartidor: Option<Arc<Addr<TcpEnviador>>>,
    pub estado: EstadoRepartidor,
    pub time_stamp: Instant,
}

// Mensaje para remover un 'Comensal' del almacenamiento.
#[derive(Message)]
#[rtype(result = "()")]
pub struct BorrarComensal {
    pub id: SocketAddr,
}

// Mensaje para remover un 'Repartidor' del almacenamiento.
#[derive(Message)]
#[rtype(result = "()")]
pub struct BorrarRepartidor {
    pub id: SocketAddr,
}

// Mensaje para obtener el 'Repartidor' más cercano a un 'Comensal'.
#[derive(Message)]
#[rtype(result = "Option<SocketAddr>")]
pub struct ObtenerRepartidorCercano;

// Mensaje para saber si un 'Comensal' pudo terminar de recibir su pedido.
#[derive(Message)]
#[rtype(result = "bool")]
pub struct Finalizado {
    pub id_comensal: SocketAddr,
}

// Mensaje para detectar y eliminar los 'Repartidores' ausentes.
#[derive(Message)]
#[rtype(result = "Vec<RepartidorAusente>")]
pub struct BorrarRepartidoresAusentes;

// Mensaje con el que un comensal notifica que quiere un pedido.
#[derive(Message)]
#[rtype(result = "Option<EntidadRestaurante>")]
pub struct QuieroPedidoSoyComensal {
    pub ubicacion_comensal: (f32, f32),
}

// Mensaje para actualizar un 'Repartidor' del almacenamiento.
#[derive(Message)]
#[rtype(result = "()")]
pub struct ActualizarComensalUbicacionRestaurante {
    pub nueva_ubicacion: (f32, f32),
    pub id_comensal: SocketAddr,
}
