//! Este módulo contiene la lógica del actor 'Almacenamiento'.

// Imports de crates externas.
use actix::Addr;
use actix::{Actor, Context, Handler};
use std::{collections::HashMap, net::SocketAddr};

// Imports de funciones/estructuras propias.
use crate::almacenamiento::mensajes_almacenamiento::HayRecursosDisponibles;
use crate::utils::{
    entidades::{EntidadComensal, EntidadRepartidor, EntidadRestaurante},
    errores_servidor::ServidorError,
};

// Constantes.
const INICIANDO_ACTOR_ALMACENAMIENTO: &str = "Iniciando actor de almacenamiento...";

// Esta estructura modela el actor de almacenamiento que maneja las entidades del sistema.
pub struct Almacenamiento {
    pub comensales: HashMap<SocketAddr, EntidadComensal>,
    pub repartidores_posta: HashMap<SocketAddr, EntidadRepartidor>,
    pub restaurantes: HashMap<SocketAddr, EntidadRestaurante>,
}

// Implementa el trait `Actor` para el actor `Almacenamiento`.
impl Actor for Almacenamiento {
    type Context = Context<Self>;
}

// Implementación del inicio del actor `Almacenamiento`.
impl Almacenamiento {
    pub fn start() -> Result<Addr<Almacenamiento>, ServidorError> {
        println!("{}", INICIANDO_ACTOR_ALMACENAMIENTO);
        let almacenamiento = Almacenamiento {
            comensales: HashMap::new(),
            repartidores_posta: HashMap::new(),
            restaurantes: HashMap::new(),
        };

        let almacenamiento_addr = almacenamiento.start();
        Ok(almacenamiento_addr)
    }
}

// Handler para ver si hay los recursos disponibles.
impl Handler<HayRecursosDisponibles> for Almacenamiento {
    type Result = bool;

    fn handle(&mut self, _msg: HayRecursosDisponibles, _: &mut Context<Self>) -> Self::Result {
        !(self.repartidores_posta.is_empty() || self.restaurantes.is_empty())
    }
}
