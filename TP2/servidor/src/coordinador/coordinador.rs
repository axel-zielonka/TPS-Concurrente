//! Este módulo contiene la lógica del actor 'Coordinador'.

// Imports de crates externas.
use actix::prelude::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::io::split;
use tokio::net::TcpStream;

// Imports de funciones/estructuras propias.
use crate::coordinador::mensajes_coordinador::*;
use crate::coordinador::mensajes_coordinador::{ObtenerContadorPeer, ObtenerPeerDict};
use crate::utils::constantes::MAX_REINTENTOS;
use crate::utils::constantes::TIEMPO_MAXIMO_SIN_PING;
use crate::utils::errores_servidor::ServidorError;
use common::tcp_enviador::{MensajeTCP, TcpEnviador};

// Constantes.
const SLEEP_ENTRE_INTENTOS: u64 = 500;
const ERROR_CONECTAR_SOCKETS: &str = "Error al conectar los Sockets:";
const ERROR_SERIALIZAR: &str = "Error al serializar el mensaje del pedido:";
const ERROR_ACTUALIZAR_PEER: &str = "Error para actualizar el 'Peer':";
const ERROR_AL_CONECTAR: &str = "Error al conectar al";

// Estructura que encapsula al 'Actor' que representa al 'Servidor' coordinador del sistema distribuido.
pub struct Coordinador {
    pub direccion: SocketAddr,
    pub sockets: Vec<SocketAddr>,
    pub handlers_sockets: Peers,
    pub contador_sockets: u8,
}

// Implementa el trait 'Actor' para el actor 'Coordinador'.
impl Actor for Coordinador {
    type Context = Context<Self>;
}

// Constructor del 'Actor'.
impl Coordinador {
    pub fn new(direccion: SocketAddr, sockets: Vec<SocketAddr>) -> Addr<Self> {
        Coordinador::create(|_ctx| Coordinador {
            direccion,
            sockets,
            handlers_sockets: Arc::new(HashMap::new()),
            contador_sockets: 0,
        })
    }
}

// Handler que controla el proceso de convertirse en 'Coordinador' de una instancia del 'Servidor'.
impl Handler<ConvertirseEnCoordinador> for Coordinador {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, _msg: ConvertirseEnCoordinador, _ctx: &mut Self::Context) -> Self::Result {
        let direccion = self.direccion;
        let sockets = self.sockets.clone();
        let actor_addr = _ctx.address();
        Box::pin(
            async move {
                if let Err(e) = conectar_a_peers(direccion, sockets, actor_addr).await {
                    eprintln!("{} {:?}", ERROR_CONECTAR_SOCKETS, e);
                }
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        )
    }
}

// Handler que controla los pedidos que se realizan al 'Coordinador'.
impl Handler<HandlePedido> for Coordinador {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: HandlePedido, _ctx: &mut Self::Context) -> Self::Result {
        let sockets = self.sockets.clone();
        let actor_addr = _ctx.address();
        Box::pin(
            async move {
                let mut handle = None;
                let mut veces_reintentadas = sockets.len();
                while handle.is_none() && veces_reintentadas > 0 {
                    let contador = actor_addr
                        .send(ObtenerContadorPeer)
                        .await
                        .unwrap_or_default();
                    let peer = sockets[contador as usize % sockets.len()];
                    let handles = actor_addr.send(ObtenerPeerDict).await.unwrap_or_default();
                    if let Some(tupla) = handles.get(&peer) {
                        if tupla.1.elapsed().as_secs() >= TIEMPO_MAXIMO_SIN_PING {
                            veces_reintentadas -= 1;
                            continue;
                        }

                        handle = Some(tupla.clone());
                    }

                    veces_reintentadas -= 1;
                }

                if let Some(h) = handle {
                    match serde_json::to_string(&msg) {
                        Ok(handle_funcional_msj) => {
                            h.0.try_send(MensajeTCP(handle_funcional_msj))
                                .expect("Error al manejar el pedido.");
                        }

                        Err(err) => {
                            eprintln!("{} {}", ERROR_SERIALIZAR, err);
                        }
                    }
                }
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        )
    }
}

// Handler utilizado para actualizar el estado de los 'Comensales'.
impl Handler<ActualizarComensales> for Coordinador {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: ActualizarComensales, _ctx: &mut Self::Context) -> Self::Result {
        let msg = serde_json::to_string(&msg).expect("Error al convertir a JSON.");
        let actor_addr = _ctx.address();
        Box::pin(
            async move {
                actualizacion_broadcast(actor_addr, msg).await;
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        )
    }
}

// Handler utilizado para actualizar el estado de los 'Repartidores'.
impl Handler<ActualizarRepartidores> for Coordinador {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: ActualizarRepartidores, _ctx: &mut Self::Context) -> Self::Result {
        let msg = serde_json::to_string(&msg).expect("Error al convertir a JSON.");
        let actor_addr = _ctx.address();
        Box::pin(
            async move {
                actualizacion_broadcast(actor_addr, msg).await;
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        )
    }
}

// Handler para gestionar la obtención del contador de 'Peers'.
impl Handler<ObtenerContadorPeer> for Coordinador {
    type Result = u8;
    fn handle(&mut self, _msg: ObtenerContadorPeer, _ctx: &mut Self::Context) -> Self::Result {
        let actual = self.contador_sockets;
        self.contador_sockets += 1;
        actual
    }
}

// Handler para conectar nuevos servidores en el 'Anillo' de instancias.
impl Handler<ConectarNuevoServidor> for Coordinador {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: ConectarNuevoServidor, _ctx: &mut Self::Context) -> Self::Result {
        let direccion = self.direccion;
        let peer = msg.nuevo_servidor;
        let actor_addr = _ctx.address();
        Box::pin(
            async move {
                let handles = actor_addr.send(ObtenerPeerDict).await.unwrap_or_default();
                if handles.contains_key(&peer) {
                    if let Err(e) = actor_addr.try_send(AgregarServidorADiccionario {
                        serv_address: peer,
                        serv_enviador: handles[&peer].0.clone(),
                    }) {
                        println!("[{}] {} {:?}", direccion, ERROR_ACTUALIZAR_PEER, e);
                    }

                    return;
                }

                match conectar_a_peer(direccion, peer, actor_addr).await {
                    Ok(_) => (),
                    Err(e) => println!("[{}] {} {:?}: {:?}", direccion, ERROR_AL_CONECTAR, peer, e),
                }
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        )
    }
}

// Handler para agregar una nueva instancia a la 'Colección / Diccionario'.
impl Handler<AgregarServidorADiccionario> for Coordinador {
    type Result = ();
    fn handle(
        &mut self,
        msg: AgregarServidorADiccionario,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        if let Some(handles) = Arc::get_mut(&mut self.handlers_sockets) {
            handles.insert(msg.serv_address, (msg.serv_enviador, Instant::now()));
        }
    }
}

// Handler para obtener el 'Peer Dict'.
impl Handler<ObtenerPeerDict> for Coordinador {
    type Result = Peers;
    fn handle(&mut self, _msg: ObtenerPeerDict, _ctx: &mut Self::Context) -> Self::Result {
        self.handlers_sockets.clone()
    }
}

// Método para hacer broadcast en todas las instancias de servidores con la información actual.
pub async fn actualizacion_broadcast(coord_actor: Addr<Coordinador>, msg: String) {
    let handles = coord_actor.send(ObtenerPeerDict).await.unwrap_or_default();
    for handle in handles.values() {
        let res = handle.0.try_send(MensajeTCP(msg.clone()));
        if res.is_err() {
            continue;
        }
    }
}

// Método para conectarse a los 'Peers'.
async fn conectar_a_peers(
    direccion: SocketAddr,
    sockets: Vec<SocketAddr>,
    coord_actor: Addr<Coordinador>,
) -> Result<(), ServidorError> {
    println!("[{}] Conectandose a los sockets {:?}", direccion, sockets);
    for &peer in sockets.iter() {
        conectar_a_peer(direccion, peer, coord_actor.clone()).await?;
    }

    Ok(())
}

// Método para conectarse a un 'Peer'.
async fn conectar_a_peer(
    direccion: SocketAddr,
    peer: SocketAddr,
    coord_actor: Addr<Coordinador>,
) -> Result<(), ServidorError> {
    if direccion == peer {
        return Ok(()); // Se evita conectarse a si mismo.
    }

    let mut intentos = 0;
    let mut stream = None;
    while intentos < MAX_REINTENTOS {
        match TcpStream::connect(peer).await {
            Ok(s) => {
                stream = Some(s);
                break;
            }

            Err(_) => {
                intentos += 1;
                tokio::time::sleep(Duration::from_millis(SLEEP_ENTRE_INTENTOS)).await;
            }
        }
    }

    if let Some(stream) = stream {
        println!("[{}] Conectado a {:?}", direccion, peer);
        let (_, w_half) = split(stream);
        let escribir = Some(w_half);
        let sender_actor = TcpEnviador { escribir }.start();
        coord_actor
            .try_send(AgregarServidorADiccionario {
                serv_address: peer,
                serv_enviador: sender_actor,
            })
            .expect("Error al añadir el socket al 'Diccionario/Colección'.");
    } else {
        //println!("[{}] No se pudo conectar al 'Peer' {:?}", direccion, peer);
    }

    Ok(())
}

// Handler utilizado para actualizar el estado de los 'Repartidores'.
impl Handler<ActualizarRestaurantes> for Coordinador {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: ActualizarRestaurantes, _ctx: &mut Self::Context) -> Self::Result {
        let msg = serde_json::to_string(&msg).expect("Error al convertir a JSON.");
        let actor_addr = _ctx.address();
        Box::pin(
            async move {
                actualizacion_broadcast(actor_addr, msg).await;
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        )
    }
}
