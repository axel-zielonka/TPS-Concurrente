//! Este módulo contiene la lógica de ejecución inicial del 'Gateway'.

// Módulos locales utilizados.
mod gateway;
use gateway::GatewayActor;

// Constantes.
const DIRECCION_STR: &str = "127.0.0.1:8085";
const MSJ_ERROR: &str = "Error al parsear la dirección:";

// Método 'main' que inicia el actor del gateway de pagos.
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let direccion = DIRECCION_STR.to_string().parse();
    let direccion = match direccion {
        Ok(direccion) => direccion,
        Err(e) => {
            eprintln!("{} {}", MSJ_ERROR, e);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, e));
        }
    };

    GatewayActor::start(direccion).await?;
    Ok(())
}
