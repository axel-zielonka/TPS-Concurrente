//! Este módulo contiene la lógica de ejecución inicial del 'Servidor'.

// Imports de crates externas.
use common::utils::socket_addr_desde_string;
use server::server::Server;
use std::net::SocketAddr;
use utils::errores_servidor::ServidorError;

// Módulos locales utilizados.
mod almacenamiento;
mod coordinador;
mod eleccion;
mod server;
mod utils;

// Constantes.
const PORT_ARG: usize = 1;
const DIRECCION_IP_BASE: &str = "127.0.0.1:";
const PUERTO_POR_DEFECTO: &str = "8080";
const PUERTO_INICIAL: u16 = 8080;
const PUERTO_FINAL: u16 = 8085;

// Método 'main' que inicia el actor de cada servidor.
#[actix_rt::main]
async fn main() -> Result<(), ServidorError> {
    let puerto = std::env::args()
        .nth(PORT_ARG)
        .unwrap_or(PUERTO_POR_DEFECTO.to_string());
    let direccion = socket_addr_desde_string(format!("{}{}", DIRECCION_IP_BASE, puerto));
    let sockets: Vec<SocketAddr> = (PUERTO_INICIAL..PUERTO_FINAL)
        .map(|puerto| format!("{}{}", DIRECCION_IP_BASE, puerto))
        .map(socket_addr_desde_string)
        .collect();
    Server::start(direccion, sockets).await?;
    Ok(())
}
