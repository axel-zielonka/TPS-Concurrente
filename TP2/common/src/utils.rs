//! Este módulo contiene funciones utilitarias del sistema.

// Imports de crates externas.
use std::net::SocketAddr;

// Constantes.
const MAX_VALOR_RANDOM: f32 = 50.0; // Establece los margenes máximos del mapa emulado.
const CODIGO_ERROR: i32 = 1;
const ERROR_PARSEO: &str = "Error al parsear la dirección IP:";

// Conversión de una dirección IP de texto a `SocketAddr`.
pub fn socket_addr_desde_string(addr: String) -> SocketAddr {
    match addr.parse() {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("{} {}", ERROR_PARSEO, e);
            std::process::exit(CODIGO_ERROR);
        }
    }
}

// Generación de una tupla aleatoria de dos valores `f32` 'small ints' (para respetar el enunciado).
pub fn obtener_tupla_random() -> (f32, f32) {
    (
        (rand::random::<f32>() * MAX_VALOR_RANDOM).round(),
        (rand::random::<f32>() * MAX_VALOR_RANDOM).round(),
    )
}
