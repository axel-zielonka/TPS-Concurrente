//! Este módulo contiene la lógica de ejecución inicial del 'Comensal'.
mod comensal;

// Imports de crates externas.
use std::net::SocketAddr;

// Módulos locales utilizados.
use comensal::Comensal;

// Constantes.
const DIRECCION_IP_BASE: &str = "127.0.0.1:";
const PUERTO_INICIAL: u16 = 8080;
const PUERTO_FINAL: u16 = 8084;

// Método 'main' que inicia el actor de cada comensal.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut servidores: Vec<SocketAddr> = Vec::new();
    for puerto in PUERTO_INICIAL..=PUERTO_FINAL {
        let servidor: SocketAddr = format!("{}{}", DIRECCION_IP_BASE, puerto).parse()?;
        servidores.push(servidor);
    }

    let mut comensal = Comensal::new(servidores).await;
    comensal.run().await;
    Ok(())
}
