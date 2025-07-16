//! Este módulo contiene la lógica del sitema de 'Reaper' entre las instancias del servidor.

// Imports de crates externas.
use actix::Addr;
use std::sync::Arc;
use std::time::Duration;

// Imports de funciones/estructuras propias.
use crate::almacenamiento::almacenamiento::Almacenamiento;
use crate::almacenamiento::mensajes_almacenamiento::BorrarRepartidoresAusentes;
use crate::coordinador::coordinador::Coordinador;
use crate::coordinador::mensajes_coordinador::{
    Accion, ActualizarComensales, ActualizarRepartidores,
};
use crate::server::client_server::EstadoRepartidor;
use common::mensajes::RechazarViaje;
use common::tcp_enviador::MensajeTCP;

// Funcion que mata a los repartidores que estan ausentes/muertos en el sistema.
async fn matar_repartidores_ausentes(
    actor_almacenamiento: Arc<Addr<Almacenamiento>>,
    direccion_coordinador: Arc<Addr<Coordinador>>,
) {
    let repartidores_muertos = actor_almacenamiento
        .send(BorrarRepartidoresAusentes)
        .await
        .unwrap();

    if !repartidores_muertos.is_empty() {
        println!(
            "Se borran {} repartidores ausentes.",
            repartidores_muertos.len()
        );

        for repartidor_ausente in &repartidores_muertos {
            direccion_coordinador
                .try_send(ActualizarRepartidores {
                    accion: Accion::Eliminar,
                    repartidor: repartidor_ausente.id_repartidor,
                    posicion: (0.0, 0.0),
                    id_comensal_actual: None,
                    status: EstadoRepartidor::Active,
                })
                .expect("Error al enviar la actualizacion de los repartidores");

            direccion_coordinador
                .try_send(ActualizarComensales {
                    accion: Accion::Eliminar,
                    comensal: repartidor_ausente
                        .id_comensal
                        .unwrap_or_else(|| "0.0.0.0:0".parse().unwrap()),
                    origen: (0.0, 0.0),
                    destino: (0.0, 0.0),
                })
                .expect("Error al enviar mensaje.");

            let tcp_message = MensajeTCP(
                serde_json::to_string(&RechazarViaje {
                    respuesta: "El repartidor esta desconectado, intente nuevamente".to_string(),
                })
                .unwrap(),
            );
            if let Some(sender) = repartidor_ausente.enviador_comensal.as_ref() {
                sender
                    .send(tcp_message)
                    .await
                    .expect("Error al enviar mensaje.");
            }

            println!(
                "Repartidor muerto con id {:?}",
                repartidor_ausente.id_repartidor
            );
        }
    }
}

// Método que spawnea la tarea de 'Reaper' de forma indefinida.
pub fn spawn_reaper_task(
    actor_almacenamiento: Arc<Addr<Almacenamiento>>,
    direccion_coordinador: Arc<Addr<Coordinador>>,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            matar_repartidores_ausentes(
                actor_almacenamiento.clone(),
                direccion_coordinador.clone(),
            )
            .await;
        }
    });
}
