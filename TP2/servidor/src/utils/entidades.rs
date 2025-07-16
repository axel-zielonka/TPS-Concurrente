//! Este módulo contiene la lógica de las entidades que modelan al sistema.

// Imports de crates externas.
use actix::Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

// Imports de funciones/estructuras propias.
use crate::server::client_server::EstadoRepartidor;
use common::tcp_enviador::TcpEnviador;

// Entidad que modela al 'Comensal'.
#[derive(Debug, Clone)]
pub struct EntidadComensal {
    pub posicion_comensal: (f32, f32),
    pub destino_comensal: (f32, f32),
    pub enviador_comensal: Option<Arc<Addr<TcpEnviador>>>,
}

// Entidad que modela al 'Restaurante'.
#[derive(Debug, Clone)]
pub struct EntidadRepartidor {
    pub posicion_repartidor: (f32, f32),
    pub id_actual_comensal: Option<SocketAddr>,
    pub restaurante_sender: Option<Arc<Addr<TcpEnviador>>>,
    pub estado: EstadoRepartidor,
    pub time_stamp: Instant,
}

// Entidad que modela al 'Repartidor'.
#[derive(Debug, Clone)]
pub struct EntidadRestaurante {
    pub posicion_restaurante: (f32, f32),
    pub repartidor_sender: Option<Arc<Addr<TcpEnviador>>>,
    pub estado: EstadoRepartidor,
    pub time_stamp: Instant,
}
