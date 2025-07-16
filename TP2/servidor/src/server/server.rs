//! Este módulo contiene la lógica del actor 'Servidor'.

// Imports de crates externas.
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{split, AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::LinesStream;

// Imports de funciones/estructuras propias.
use super::server_coordinador::HacerPedido;
use crate::almacenamiento::almacenamiento::Almacenamiento;
use crate::coordinador::coordinador::Coordinador;
use crate::coordinador::mensajes_coordinador::{
    ActualizarComensales, ActualizarRepartidores, HandlePedido, UpdateRepartidores,
};
use crate::eleccion::eleccion::EleccionCoordinador;
use crate::eleccion::mensajes_eleccion::{MensajeCoordinador, MensajeEleccion, MensajePing};
use crate::server::ping::{spawn_ping_task, WhoIsCoordinator};
use crate::server::reaper::spawn_reaper_task;
use crate::server::server_almacenamiento::{
    HacerActualizacionComensal, HacerActualizacionRepartidor, HacerActualizacionRepartidores,
};
use crate::utils::errores_servidor::ServidorError;
use crate::utils::logs::{log_eleccion, log_funcionamiento};
use actix::prelude::*;
use common::mensajes::{
    FinalizarViaje, Posicion, RespuestaOfertaViaje, SolicitarPedido, SolicitarRepartidor,
};
use common::tcp_enviador::{MensajeTCP, TcpEnviador};
pub type CoordEleccion = Arc<Addr<EleccionCoordinador>>;

// Estructura que encapsula al 'Actor' que representa a una instancia del 'Servidor' del programa.
pub struct Server {
    pub addr: SocketAddr,
    pub client_addr: SocketAddr,
    pub tcp_enviador: Arc<Addr<TcpEnviador>>,
    pub eleccion_coordinador: CoordEleccion,
    pub coordinador: Arc<Addr<Coordinador>>,
    pub almacenamiento_addr: Arc<Addr<Almacenamiento>>,
}

// Implementación del trait actor.
impl Actor for Server {
    type Context = Context<Self>;
}

// Implementación de handlers y constructores del servidor.
impl Server {
    // Constructor del servidor.
    pub fn new(
        stream: TcpStream,
        client_addr: SocketAddr,
        addr: SocketAddr,
        eleccion_coordinador: CoordEleccion,
        coordinador: Arc<Addr<Coordinador>>,
        almacenamiento_addr: Arc<Addr<Almacenamiento>>,
    ) -> Addr<Self> {
        Server::create(|ctx| {
            let (r_half, w_half) = split(stream);
            Server::add_stream(LinesStream::new(BufReader::new(r_half).lines()), ctx);
            let escribir = Some(w_half);
            let sender_actor = TcpEnviador { escribir }.start();
            let tcp_enviador = Arc::new(sender_actor);

            Server {
                addr,
                client_addr,
                tcp_enviador,
                eleccion_coordinador,
                coordinador,
                almacenamiento_addr,
            }
        })
    }

    // Método iniciador del actor.
    pub async fn start(addr: SocketAddr, peers: Vec<SocketAddr>) -> Result<(), ServidorError> {
        if peers.is_empty() {
            return Err(ServidorError::InvalidSockets("No hay peers".to_string()));
        }

        let listener = TcpListener::bind(addr).await.map_err(|e| {
            ServidorError::BindError(format!("Failed to bind to {}: {:?}", addr, e))
        })?;

        let coordinador = Arc::new(Coordinador::new(addr, peers.clone()));

        let eleccion_coordinador = Arc::new(EleccionCoordinador::new(
            addr,
            Arc::clone(&coordinador),
            Arc::new(peers),
        ));

        let almacenamiento_actor = Arc::new(Almacenamiento::start()?);

        spawn_ping_task(eleccion_coordinador.clone());

        spawn_reaper_task(almacenamiento_actor.clone(), coordinador.clone());

        accept_connections(
            listener,
            addr,
            eleccion_coordinador,
            coordinador,
            almacenamiento_actor,
        )
        .await
    }
}

// Handlers de mensajes del servidor.
impl StreamHandler<Result<String, std::io::Error>> for Server {
    fn handle(&mut self, read: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        if let Ok(line) = read {
            let mensaje = line.trim();

            // Handler del mensaje 'Ping'.
            if let Ok(ping_msg) = serde_json::from_str::<MensajePing>(mensaje) {
                log_eleccion(format!("[{:?}] Recibido Ping, enviando Ack", self.addr));
                ctx.address()
                    .try_send(ping_msg)
                    .expect("No se pudo enviar MensajePing");
            }

            // Handler del mensaje 'QuienEsElCoordinador'.
            if mensaje == "WhoIsCoordinator" {
                ctx.address()
                    .try_send(WhoIsCoordinator {})
                    .expect("No se pudo enviar WhoIsCoordinator");
            }

            // Handler del mensaje 'Eleccion'.
            if let Ok(mensaje_eleccion) = serde_json::from_str::<MensajeEleccion>(mensaje) {
                let tcp_enviador_clone = self.tcp_enviador.clone();

                tcp_enviador_clone
                    .try_send(MensajeTCP("Ack".to_string()))
                    .expect("No se pudo enviar la respuesta");

                self.eleccion_coordinador
                    .try_send(MensajeEleccion {
                        candidatos: mensaje_eleccion.candidatos,
                    })
                    .expect("No se pudo enviar el mensaje de elección");
            }

            // Handler del mensaje que indica si esta instancia es o no un coordinador.
            if let Ok(mensaje_coordinador) = serde_json::from_str::<MensajeCoordinador>(mensaje) {
                let tcp_enviador_clone = self.tcp_enviador.clone();
                tcp_enviador_clone
                    .try_send(MensajeTCP("Ack".to_string()))
                    .expect("No se pudo enviar la respuesta");
                self.eleccion_coordinador
                    .try_send(MensajeCoordinador {
                        coordinador: mensaje_coordinador.coordinador,
                    })
                    .expect("No se pudo enviar el mensaje de coordinador");
            }

            // Handler del mensaje para solicitar un repartidor.
            if let Ok(solicitar_repartidor) = serde_json::from_str::<SolicitarRepartidor>(mensaje) {
                log_funcionamiento(format!(
                    "[Pedido DESDE EL RESTAURANTE: {:?}]",
                    solicitar_repartidor.comida
                ));
                ctx.address()
                    .try_send(solicitar_repartidor)
                    .expect("Error en SolicitarViaje");
            }

            // Handler del mensaje para solicitar un comensal.
            if let Ok(solicitar_una_comida) = serde_json::from_str::<SolicitarPedido>(mensaje) {
                log_funcionamiento(format!(
                    "[Pedido de comida DESDE EL COMENSAL: {:?}]",
                    solicitar_una_comida.comida
                ));
                ctx.address()
                    .try_send(solicitar_una_comida)
                    .expect("Error en SolicitarViaje");
            }

            // Handler del mensaje que indica que hay un repartidor listo.
            if let Ok(repartidor_listo) = serde_json::from_str::<Posicion>(mensaje) {
                ctx.address()
                    .try_send(repartidor_listo)
                    .expect("Error en PosicionRepartidor");
            }

            // Handler del mensaje para indicar que hay un nuevo restaurante.
            if let Ok(mensaje_identidad) =
                serde_json::from_str::<common::mensajes::MensajeIdentidad>(mensaje)
            {
                log_funcionamiento(format!(
                    "[Identidad recibida, es repartidor: {:?}]",
                    mensaje_identidad.soy_repartidor
                ));
                ctx.address()
                    .try_send(mensaje_identidad)
                    .expect("Error en PosicionRepartidor");
            }

            // Handler del mensaje para indicar que finalizo un viaje.
            if let Ok(pedido_finalizado) = serde_json::from_str::<FinalizarViaje>(mensaje) {
                ctx.address()
                    .try_send(pedido_finalizado)
                    .expect("Error en FinalizarViaje");
            }

            // Handler del mensaje para manejar los pedidos.
            if let Ok(manejar_pedido) = serde_json::from_str::<HandlePedido>(mensaje) {
                ctx.address()
                    .try_send(manejar_pedido)
                    .expect("Error en HandlePedido");
            }

            // Handler del mensaje que indica que puede aceptar un pedido.
            if let Ok(respuesta_aceptar_pedido) =
                serde_json::from_str::<RespuestaOfertaViaje>(mensaje)
            {
                ctx.address()
                    .try_send(respuesta_aceptar_pedido)
                    .expect("Error en RespuestaOfertaViaje");
            }

            // Handler del mensaje para hacer un pedido.
            if let Ok(hacer_pedido) = serde_json::from_str::<HacerPedido>(mensaje) {
                self.tcp_enviador
                    .try_send(MensajeTCP("Ack".to_string()))
                    .expect("No se pudo enviar la respuesta");
                ctx.address()
                    .try_send(hacer_pedido)
                    .expect("Error en MakeTrip");
            }

            // Handler del mensaje para indicar actualizaciones internas del coordinador.
            if let Ok(actualizar_comensal) = serde_json::from_str::<ActualizarComensales>(mensaje) {
                ctx.address()
                    .try_send(HacerActualizacionComensal {
                        upt_msg: actualizar_comensal,
                    })
                    .expect("No se pudo enviar HacerActualizacionComensal");
            }

            // Handler del mensaje para indicar actualizaciones en los repartidores.
            if let Ok(actualizar_repartidor) =
                serde_json::from_str::<ActualizarRepartidores>(mensaje)
            {
                ctx.address()
                    .try_send(HacerActualizacionRepartidores {
                        upt_msg: actualizar_repartidor,
                    })
                    .expect("No se pudo enviar MakeUpdateRepartidor");
            }

            if let Ok(buscando_trabajo) = serde_json::from_str::<UpdateRepartidores>(mensaje) {
                println!("[{:?}] Recibido ActualizarRepartidores", self.addr);
                ctx.address()
                    .try_send(HacerActualizacionRepartidor {
                        upt_msg: buscando_trabajo,
                    })
                    .expect("Error en ActualizarRepartidores");
            }
        } else {
            println!("[{:?}] Error al leer línea {:?}", self.addr, read);
        }
    }
}

// Método para aceptar conexiones.
async fn accept_connections(
    listener: TcpListener,
    addr: SocketAddr,
    eleccion_coordinador: Arc<Addr<EleccionCoordinador>>,
    coordinador: Arc<Addr<Coordinador>>,
    almacenamiento_actor: Arc<Addr<Almacenamiento>>,
) -> Result<(), ServidorError> {
    loop {
        match listener.accept().await {
            Ok((stream, client_addr)) => {
                log_eleccion(format!("[{}] Conexión recibida de {:?}", addr, client_addr));
                Server::new(
                    stream,
                    client_addr,
                    addr,
                    eleccion_coordinador.clone(),
                    Arc::clone(&coordinador),
                    almacenamiento_actor.clone(),
                );
            }
            Err(e) => {
                println!("[{}] Error al aceptar conexión: {:?}", addr, e);
            }
        }
    }
}
