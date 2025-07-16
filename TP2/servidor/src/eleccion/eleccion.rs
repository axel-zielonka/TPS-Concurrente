//! Este módulo contiene la lógica del actor 'EleccionCoordinador'.

// Imports de crates externas.
use actix::prelude::*;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{split, BufReader};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

// Imports de funciones/estructuras propias.
use crate::coordinador::coordinador::Coordinador;
use crate::coordinador::mensajes_coordinador::ConvertirseEnCoordinador;
use crate::eleccion::mensajes_eleccion::{
    AsignarDireccionLider, IniciarEleccion, MensajeCoordinador, MensajeEleccion, MensajePing,
    ObtenerDireccionCoordinador, ObtenerDireccionLider, PingCoordinador, SoyCoordinador,
};
use crate::utils::logs::log_eleccion;
use common::mensajes::QuienEsCoordinador;
use common::tcp_enviador::MensajeTCP;

// Constantes.
const MIN_PEER_PORT: u16 = 8080;
const MAX_PEER_PORT: u16 = 8084;
const MIN_INTENTOS: usize = 0;
const MAX_INTENTOS: usize = 3;
const TIMEOUT_SEGUNDOS: u64 = 3;
const MSJ_ELECCION_RECIBIDO: &str = "Mensaje de elección recibido.";
const INICIANDO_ELECCION: &str = "Iniciando elección de coordinador.";
const RECIBIO_MSJ_ELECCION: &str = "Recibió mensaje de elección.";
const CANDIDATOS_FINALES: &str = "Candidatos finales: ";
const TRATANDO_CONECTAR_PROX_SOCK: &str = "Tratando de conectar al siguiente socket: ";
const CONECTADO_AL_PROX: &str = "Conectado al siguiente peer de elección: ";
const RECIBIDO: &str = "Recibido: ";
const ERROR_LEYENDO_LINEA: &str = "Error leyendo línea del socket.";
const NUEVO_COORDINADOR: &str = "Nuevo coordinador: ";
const ERROR_SETEANDO_CONFIG: &str = "Error al setear la configuración del coordinador.";
const ERROR_BROADCAST: &str = "Error al enviar mensaje de broadcast:";
const PINGING_COORDINADOR: &str = "Haciendo ping al coordinador.";
const ERROR_ESCRIBIENDO_MENSAJE: &str = "Error escribiendo mensaje de elección.";
const VOLVIENDOSE_COORDINADOR: &str = "Volviéndose coordinador.";
const ACK: &str = "Ack";

// Actor responsable de la elección del coordinador en un sistema distribuido.
// Este actor implementa el algoritmo de elección de anillo (Ring Election).
#[derive(Debug, Clone)]
pub struct EleccionCoordinador {
    pub id: SocketAddr,
    pub id_coordinador: Option<SocketAddr>,
    pub coordinador: Arc<Addr<Coordinador>>,
    pub peers: Arc<Vec<SocketAddr>>,
    pub en_eleccion: bool,
}

// Implementa el trait `Actor` para el actor `EleccionCoordinador`.
impl Actor for EleccionCoordinador {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        _ctx.notify(AskForCoordinator);
    }
}

// Constructor del actor `EleccionCoordinador`.
impl EleccionCoordinador {
    pub fn new(
        id: SocketAddr,
        coordinador: Arc<Addr<Coordinador>>,
        peers: Arc<Vec<SocketAddr>>,
    ) -> Addr<Self> {
        EleccionCoordinador::create(|_ctx| EleccionCoordinador {
            id,
            id_coordinador: None,
            coordinador,
            peers,
            en_eleccion: false,
        })
    }
}

// Implementación del handler para asignar la dirección del coordinador.
impl Handler<AsignarDireccionLider> for EleccionCoordinador {
    type Result = ();
    fn handle(&mut self, msg: AsignarDireccionLider, _ctx: &mut Self::Context) {
        self.id_coordinador = Some(msg.id_coordinador);
    }
}

// Implementación del handler para obtener la dirección del coordinador.
impl Handler<ObtenerDireccionCoordinador> for EleccionCoordinador {
    type Result = MessageResult<ObtenerDireccionCoordinador>;
    fn handle(
        &mut self,
        _msg: ObtenerDireccionCoordinador,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        MessageResult(self.id_coordinador)
    }
}

// Mensaje y estructura para preguntar quién es el coordinador.
#[derive(Message)]
#[rtype(result = "()")]
struct AskForCoordinator;
impl Handler<AskForCoordinator> for EleccionCoordinador {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, _msg: AskForCoordinator, ctx: &mut Self::Context) -> Self::Result {
        let mut eleccion = self.clone();
        let direccion = ctx.address();
        Box::pin(
            async move {
                if eleccion.en_eleccion || eleccion.es_coordinador(direccion.clone()).await {
                    return;
                }

                eleccion
                    .preguntar_quien_coordina(eleccion.peers.to_vec(), eleccion.id, direccion)
                    .await;
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        )
    }
}

// Handler para manejar el mensaje que consulta si la instancia es el coordinador.
impl Handler<SoyCoordinador> for EleccionCoordinador {
    type Result = ResponseFuture<bool>;
    fn handle(&mut self, _msg: SoyCoordinador, ctx: &mut Self::Context) -> Self::Result {
        let direccion = ctx.address();
        let eleccion = self.clone();
        Box::pin(async move { eleccion.es_coordinador(direccion).await })
    }
}

// Handler para obtener la dirección del líder actual.
impl Handler<ObtenerDireccionLider> for EleccionCoordinador {
    type Result = MessageResult<ObtenerDireccionLider>;
    fn handle(&mut self, _msg: ObtenerDireccionLider, _ctx: &mut Self::Context) -> Self::Result {
        MessageResult(self.id_coordinador)
    }
}

// Handler para manejar el mensaje de ping al coordinador.
impl Handler<PingCoordinador> for EleccionCoordinador {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, _msg: PingCoordinador, ctx: &mut Self::Context) -> Self::Result {
        let eleccion = self.clone();
        let direccion_actor = ctx.address();
        Box::pin(
            async move {
                if eleccion.es_coordinador(direccion_actor.clone()).await || eleccion.en_eleccion {
                    return;
                }
                eleccion.ping_coordinador(direccion_actor).await;
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        )
    }
}

// Handler para iniciar una elección de coordinador.
impl Handler<IniciarEleccion> for EleccionCoordinador {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, _msg: IniciarEleccion, _ctx: &mut Self::Context) -> Self::Result {
        let mut eleccion = self.clone();
        let direccion = _ctx.address();
        Box::pin(
            async move {
                if eleccion.en_eleccion {
                    return;
                }

                eleccion.iniciar_eleccion(direccion).await;
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        )
    }
}

// Handler para manejar mensajes de elección.
impl Handler<MensajeEleccion> for EleccionCoordinador {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: MensajeEleccion, ctx: &mut Self::Context) -> Self::Result {
        println!("{}", MSJ_ELECCION_RECIBIDO);
        let mut eleccion = self.clone();
        let direccion = ctx.address();
        Box::pin(
            async move {
                eleccion.handle_election(msg, direccion).await;
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        )
    }
}

// Handler para manejar mensajes del coordinador.
impl Handler<MensajeCoordinador> for EleccionCoordinador {
    type Result = ResponseActFuture<Self, ()>;
    fn handle(&mut self, msg: MensajeCoordinador, ctx: &mut Self::Context) -> Self::Result {
        let mut eleccion = self.clone();
        let direccion_actor = ctx.address();
        Box::pin(
            async move {
                eleccion
                    .recibir_mensaje_coordinador(msg, direccion_actor)
                    .await;
            }
            .into_actor(self)
            .map(|_, _, _| ()),
        )
    }
}

// Conjunto de funciones para manejar la lógica de elección del coordinador.
impl EleccionCoordinador {
    // Inicia una elección de coordinador.
    pub async fn iniciar_eleccion(&mut self, addr: Addr<EleccionCoordinador>) {
        log_eleccion(INICIANDO_ELECCION.to_string());
        let candidates = vec![self.id];
        self.en_eleccion = true;
        self.enviar_mensaje_eleccion(candidates, addr).await;
    }

    // Maneja el mensaje de elección recibido.
    pub async fn handle_election(&mut self, msg: MensajeEleccion, addr: Addr<EleccionCoordinador>) {
        if self.en_eleccion && !msg.candidatos.contains(&self.id) {
            return;
        }

        log_eleccion(format!("[{:?}] {}", self.id, RECIBIO_MSJ_ELECCION));
        self.en_eleccion = true;
        if msg.candidatos.contains(&self.id) {
            log_eleccion(format!("{} {:?}", CANDIDATOS_FINALES, msg.candidatos));
            self.manejar_nuevo_coordinador(msg.candidatos, addr).await;
            return;
        }

        let mut candidatos = msg.candidatos;
        candidatos.push(self.id);
        self.enviar_mensaje_eleccion(candidatos, addr).await;
    }

    // Envía un mensaje de elección a los siguientes nodos en el anillo.
    pub async fn enviar_mensaje_eleccion(
        &mut self,
        candidates: Vec<SocketAddr>,
        direccion: Addr<EleccionCoordinador>,
    ) {
        let msg = MensajeEleccion {
            candidatos: candidates.clone(),
        };
        let msg = MensajeTCP(format!(
            "{}\n",
            serde_json::to_string(&msg).expect("Error convirtiendo a JSON.")
        ));

        let mut proximo_puerto_socket = self.id.port();
        let mut tiene_ack = false;
        while !tiene_ack && proximo_puerto_socket != self.id.port() - 1 {
            proximo_puerto_socket = if proximo_puerto_socket >= MAX_PEER_PORT {
                MIN_PEER_PORT
            } else {
                proximo_puerto_socket + 1
            };

            let siguiente_socket = SocketAddr::new(self.id.ip(), proximo_puerto_socket);
            log_eleccion(format!(
                "{} {:?}",
                TRATANDO_CONECTAR_PROX_SOCK, siguiente_socket
            ));
            for _ in MIN_INTENTOS..MAX_INTENTOS {
                if let Ok(stream) = TcpStream::connect(siguiente_socket).await {
                    let (lector, mut escritor) = split(stream);
                    log_eleccion(format!("{} {:?}", CONECTADO_AL_PROX, siguiente_socket));
                    if escritor.write_all(msg.0.as_bytes()).await.is_err() {
                        eprintln!("Error escribiendo mensaje de elección.");
                        continue;
                    }

                    let mut lector = BufReader::new(lector);
                    let mut line = String::new();
                    match timeout(
                        Duration::from_secs(TIMEOUT_SEGUNDOS),
                        lector.read_line(&mut line),
                    )
                    .await
                    {
                        Ok(Ok(_)) => {
                            log_eleccion(format!("{} {:?}", RECIBIDO, line));
                            tiene_ack = true;
                        }

                        Ok(Err(e)) => {
                            println!("[{:?}] {} {:?}", siguiente_socket, ERROR_LEYENDO_LINEA, e);
                        }

                        Err(_) => {
                            log_eleccion("Timeout".to_string());
                        }
                    }

                    break;
                }
            }

            if !tiene_ack {
                log_eleccion("Error para conectarse al siguiente peer.".to_string());
            }
        }

        if !tiene_ack {
            self.manejar_nuevo_coordinador(candidates, direccion).await;
        }
    }

    // Maneja la elección de un nuevo coordinador.
    pub async fn manejar_nuevo_coordinador(
        &mut self,
        candidates: Vec<SocketAddr>,
        addr: Addr<EleccionCoordinador>,
    ) {
        let nuevo_coordinador = match candidates.iter().min_by_key(|&x| x.port()) {
            Some(coordinador) => coordinador,
            None => {
                panic!("Sin candidatos encontrados para coordinar.");
            }
        };

        let msg = MensajeCoordinador {
            coordinador: *nuevo_coordinador,
        };

        self.recibir_mensaje_coordinador(msg.clone(), addr).await;
        self.broadcast_coordinador(msg).await;
    }

    // Recibe un mensaje del coordinador y actualiza el estado del actor.
    pub async fn recibir_mensaje_coordinador(
        &mut self,
        msg: MensajeCoordinador,
        addr: Addr<EleccionCoordinador>,
    ) {
        log_eleccion(format!(
            "[{:?}] {} {:?}",
            self.id, NUEVO_COORDINADOR, msg.coordinador
        ));

        if let Err(e) = addr.try_send(AsignarDireccionLider {
            id_coordinador: msg.coordinador,
        }) {
            println!("{} {:?}", ERROR_SETEANDO_CONFIG, e);
        }

        if self.en_eleccion {
            self.en_eleccion = false;
        }

        if self.es_coordinador(addr.clone()).await {
            self.volverse_coordinador().await;
        }
    }

    // Envía un mensaje de coordinador a todos los peers.
    pub async fn broadcast_coordinador(&self, new_coord: MensajeCoordinador) {
        let msg = format!(
            "{}\n",
            serde_json::to_string(&new_coord).expect("Error al convertir a JSON.")
        );

        let msg = MensajeTCP(msg);
        for &peer in self.peers.iter() {
            if peer == self.id {
                continue;
            }

            for _ in MIN_INTENTOS..MAX_INTENTOS {
                match TcpStream::connect(peer).await {
                    Ok(s) => {
                        let stream = Some(s);
                        let (_, mut escritor) = match stream {
                            Some(stream) => split(stream),
                            None => panic!("Stream es NULL"),
                        };

                        if let Err(e) = escritor.write_all(msg.0.as_bytes()).await {
                            eprintln!("{}: {}", ERROR_BROADCAST, e);
                        }

                        log_eleccion(format!(
                            "[{:?}] Envia coordinador {} mensaje a {:?}",
                            self.id,
                            new_coord.coordinador.port(),
                            peer
                        ));

                        break;
                    }

                    Err(_) => {
                        continue;
                    }
                }
            }
        }
    }

    // Hace ping al coordinador para verificar su estado.
    pub async fn ping_coordinador(&self, direccion_actor: Addr<Self>) {
        log_eleccion(format!("[{:?}] {}", self.id, PINGING_COORDINADOR));
        let coord_id = direccion_actor
            .send(ObtenerDireccionCoordinador)
            .await
            .expect("Falló al obtener dirección del coordinador");

        let coord = match coord_id {
            Some(coord) => coord,
            None => {
                log_eleccion("No hay coordinador".to_string());
                return;
            }
        };

        let mut tiene_ack = false;
        for _ in MIN_INTENTOS..MAX_INTENTOS {
            match TcpStream::connect(coord).await {
                Ok(s) => {
                    let stream = Some(s);
                    let (lector, mut escritor) = match stream {
                        Some(stream) => split(stream),
                        None => panic!("Stream es NULL"),
                    };

                    // ENVIANDO PING Y ESPERANDO AL ACK.
                    let ping_msg = MensajePing {
                        id_enviador: self.id,
                    };
                    let msg =
                        MensajeTCP(format!("{}\n", serde_json::to_string(&ping_msg).unwrap()));
                    if let Err(e) = escritor.write_all(msg.0.as_bytes()).await {
                        eprintln!("{} {}", ERROR_ESCRIBIENDO_MENSAJE, e);
                    }

                    let mut lector = BufReader::new(lector);
                    let mut line = String::new();
                    match timeout(
                        Duration::from_secs(TIMEOUT_SEGUNDOS),
                        lector.read_line(&mut line),
                    )
                    .await
                    {
                        Ok(Ok(_)) => {
                            log_eleccion(format!("[{:?}] Recibido: {:?}", self.id, line));
                            tiene_ack = true;
                        }

                        Ok(Err(e)) => {
                            log_eleccion(format!("[{:?}] Falló al leer línea: {:?}", self.id, e));
                        }

                        Err(_) => {
                            log_eleccion(format!("[{:?}] Timeout", self.id));
                            tiene_ack = false;
                        }
                    }

                    break;
                }

                Err(_) => {
                    continue;
                }
            }
        }

        if !tiene_ack {
            log_eleccion(format!("[{:?}] coordinador caído.", self.id));

            direccion_actor
                .try_send(IniciarEleccion)
                .expect("Error al iniciar elección.");
        }
    }

    // Convierte este actor en coordinador.
    pub async fn volverse_coordinador(&self) {
        log_eleccion(format!("[{:?}] {}", self.id, VOLVIENDOSE_COORDINADOR));
        self.coordinador
            .send(ConvertirseEnCoordinador)
            .await
            .expect("Error al enviar 'ConvertirseEnCoordinador'.");
    }

    // Verifica si este actor es el coordinador actual.
    pub async fn es_coordinador(&self, addr: Addr<EleccionCoordinador>) -> bool {
        let coord_id = addr
            .send(ObtenerDireccionCoordinador)
            .await
            .expect("Error al obtener dirección del coordinador.");
        match coord_id {
            Some(coord_id) => self.id == coord_id,
            None => false,
        }
    }

    // Pregunta a los peers quién es el coordinador.
    pub async fn preguntar_quien_coordina(
        &mut self,
        peers: Vec<SocketAddr>,
        id: SocketAddr,
        addr: Addr<EleccionCoordinador>,
    ) {
        log_eleccion("Preguntando quien es el coordinador".to_string());
        let mut got_res = false;
        for &peer in peers.iter().filter(|&&peer| peer != id) {
            for _ in MIN_INTENTOS..MAX_INTENTOS {
                if let Ok(stream) = TcpStream::connect(peer).await {
                    let (lector, mut escritor) = split(stream);
                    let msg = MensajeTCP("WhoIsCoordinator\n".to_string());
                    if escritor.write_all(msg.0.as_bytes()).await.is_err() {
                        eprintln!("Error leyendo mensaje de elección.");
                        continue;
                    }

                    let mut lector = BufReader::new(lector);
                    let mut line = String::new();
                    match timeout(
                        Duration::from_secs(TIMEOUT_SEGUNDOS),
                        lector.read_line(&mut line),
                    )
                    .await
                    {
                        Ok(Ok(_)) => {
                            if let Ok(who_is_coord_msg) =
                                serde_json::from_str::<QuienEsCoordinador>(&line)
                            {
                                self.recibir_mensaje_coordinador(
                                    MensajeCoordinador {
                                        coordinador: who_is_coord_msg.direccion_coordinador,
                                    },
                                    addr.clone(),
                                )
                                .await;

                                got_res = true;
                                break;
                            } else if line.trim() == ACK {
                                break;
                            }
                        }

                        Ok(Err(e)) => {
                            println!("Error al leer la línea: {:?}", e);
                        }

                        Err(_) => {
                            log_eleccion("Timeout".to_string());
                        }
                    }

                    break;
                }
            }

            if got_res {
                break;
            }
        }

        if !got_res {
            log_eleccion("No hay respuesta de los peers".to_string());
            self.manejar_nuevo_coordinador(vec![self.id], addr.clone())
                .await;
        }
    }
}
