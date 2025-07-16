//! Este módulo contiene la lógica de los handlers  del actor 'Almacenamiento'.

// Imports de crates externas.
use actix::{Context, Handler};
use std::net::SocketAddr;

// Imports de funciones/estructuras propias.
use super::almacenamiento::Almacenamiento;
use super::mensajes_almacenamiento::{
    ActualizarRepartidor, BorrarComensal, BorrarRepartidor, BorrarRepartidoresAusentes, Finalizado,
    InsertarComensal, InsertarRepartidor, ObtenerComensal, ObtenerRepartidor,
    ObtenerRepartidorCercano,
};
use crate::almacenamiento::mensajes_almacenamiento::{
    ActualizarComensalUbicacionRestaurante, InsertarRestaurante, QuieroPedidoSoyComensal,
    RepartidorAusente,
};
use crate::server::client_server::EstadoRepartidor;
use crate::utils::entidades::{EntidadComensal, EntidadRepartidor, EntidadRestaurante};
use common::mensajes::FinalizarViaje;

// Constantes.
const MSJ_INSERTAR_RESTAURANTE: &str = "ALMACENAMIENTO - Insertando restaurante con id";
const INSERTANDO_COMENSAL_CON_ID: &str = "ALMACENAMIENTO - Insertando comensal con id";
const INSERTANDO_REPARTIDOR: &str = "ALMACENAMIENTO - Insertando repartidor con id";
const REPARTIDOR_AÑADIDO: &str = "ALMACENAMIENTO - Repartidor añadido con id";
const OBTENIENDO_COMENSAL: &str = "ALMACENAMIENTO - Obteniendo comensal con id";
const OBTENIENDO_REPARTIDOR: &str = "ALMACENAMIENTO - Obteniendo repartidor con id";
const REPARTIDOR_CON_ID_MSJ: &str = "ALMACENAMIENTO - Repartidor con id";
const AVISO_NO_ENCONTRADO: &str = " no encontrado.";
const ACTUALIZANDO_REPARTIDOR: &str = "ALMACENAMIENTO - Actualizando repartidor con id";
const BORRANDO_REPARTIDOR_CON_ID: &str = "ALMACENAMIENTO - Borrando repartidor con id";
const CLIENTE_BORRADO: &str = "ALMACENAMIENTO - Comensal borrado con id";
const OBTENIENDO_REPARTIDOR_CERCANO: &str = "ALMACENAMIENTO - Obteniendo repartidor cercano.";
const VERIFICANDO_REPARTIDOR_CON_ID: &str = "ALMACENAMIENTO - Verificando repartidor con id";
const EN_POSICION_MSJ: &str = " en posición";
const REPARTIDOR_MAS_CERCANO_MSJ: &str = "ALMACENAMIENTO - Repartidor más cercano encontrado";
const TERMINANDO_VIAJE_COMENSAL: &str =
    "ALMACENAMIENTO - Terminando el viaje del comensal a destino";
const COMENSAL_CON_ID: &str = "ALMACENAMIENTO - Comensal con id";
const CHEQUEANDO_FINALIZADO: &str = "ALMACENAMIENTO - Chequeando si el comensal con id";
const HA_FINALIZADO: &str = "ha finalizado su viaje.";
const ELIMINANDO_REPARTIDORES: &str = "ALMACENAMIENTO - Eliminando ";
const REPARTIDORES_AUSENTES: &str = " repartidores ausentes.";

// Handler para insertar un nuevo 'Restaurante' o actualizar uno existente en el almacenamiento.
impl Handler<InsertarRepartidor> for Almacenamiento {
    type Result = ();
    fn handle(&mut self, msj: InsertarRepartidor, _: &mut Self::Context) {
        println!("{} {:?}", INSERTANDO_REPARTIDOR, msj.id);
        match self.repartidores_posta.get_mut(&msj.id) {
            Some(restaurante) => {
                restaurante.posicion_repartidor = msj.posicion_restaurante;
                restaurante.estado = msj.estado;
                restaurante.time_stamp = msj.time_stamp;
                println!("{} {:?}", REPARTIDOR_AÑADIDO, msj.id);
            }

            None => {
                self.repartidores_posta.insert(
                    msj.id,
                    EntidadRepartidor {
                        posicion_repartidor: msj.posicion_restaurante,
                        id_actual_comensal: msj.id_comensal_actual,
                        restaurante_sender: msj.enviador_repartidor,
                        estado: msj.estado,
                        time_stamp: msj.time_stamp,
                    },
                );
            }
        }
    }
}

// Handler para insertar un nuevo 'Comensal' o actualizar uno existente en el almacenamiento.
impl Handler<InsertarComensal> for Almacenamiento {
    type Result = ();
    fn handle(&mut self, msj: InsertarComensal, _: &mut Self::Context) {
        println!("{} {:?}", INSERTANDO_COMENSAL_CON_ID, msj.id);
        self.comensales.insert(
            msj.id,
            EntidadComensal {
                posicion_comensal: msj.posicion_restaurante_del_pedido,
                destino_comensal: msj.ubicacion_comensal,
                enviador_comensal: msj.enviador_comensal,
            },
        );
    }
}

// Handler para insertar un nuevo 'Repartidor' o actualizar uno existente en el almacenamiento.
impl Handler<InsertarRestaurante> for Almacenamiento {
    type Result = ();
    fn handle(&mut self, msj: InsertarRestaurante, _: &mut Self::Context) {
        println!("{} {:?}", MSJ_INSERTAR_RESTAURANTE, msj.id);
        match self.restaurantes.get_mut(&msj.id) {
            Some(repartidor) => {
                repartidor.posicion_restaurante = msj.posicion_repartidor;
                repartidor.estado = msj.estado;
                repartidor.time_stamp = msj.time_stamp;
            }

            None => {
                self.restaurantes.insert(
                    msj.id,
                    EntidadRestaurante {
                        posicion_restaurante: msj.posicion_repartidor,
                        repartidor_sender: msj.enviador_repartidor,
                        estado: msj.estado,
                        time_stamp: msj.time_stamp,
                    },
                );
            }
        }
    }
}

// Handler para obtener un 'Comensal' del almacenamiento.
impl Handler<ObtenerComensal> for Almacenamiento {
    type Result = Option<EntidadComensal>;
    fn handle(&mut self, msj: ObtenerComensal, _: &mut Self::Context) -> Self::Result {
        println!("{} {:?}", OBTENIENDO_COMENSAL, msj.id);
        self.comensales.get(&msj.id).cloned()
    }
}

// Handler para obtener un 'Repartidor' del almacenamiento.
impl Handler<ObtenerRepartidor> for Almacenamiento {
    type Result = Option<EntidadRepartidor>;
    fn handle(&mut self, msj: ObtenerRepartidor, _: &mut Self::Context) -> Self::Result {
        println!("{} {:?}", OBTENIENDO_REPARTIDOR, msj.id);
        let repartidor = self.repartidores_posta.get(&msj.id).cloned();
        if repartidor.is_none() {
            eprintln!(
                "{} {:?} {}",
                REPARTIDOR_CON_ID_MSJ, msj.id, AVISO_NO_ENCONTRADO
            );
        }

        repartidor
    }
}

// Handler para actualizar el estado de un 'Repartidor' en el almacenamiento.
impl Handler<ActualizarRepartidor> for Almacenamiento {
    type Result = ();
    fn handle(&mut self, msj: ActualizarRepartidor, _: &mut Self::Context) {
        println!("{} {:?}", ACTUALIZANDO_REPARTIDOR, msj.id_repartidor);

        if let Some(repartidor) = self.repartidores_posta.get_mut(&msj.id_repartidor) {
            repartidor.estado = msj.estado;
            repartidor.id_actual_comensal = msj.id_comensal;
            repartidor.time_stamp = msj.time_stamp;
        } else {
            eprintln!(
                "{} {:?} {}",
                REPARTIDOR_CON_ID_MSJ, msj.id_repartidor, AVISO_NO_ENCONTRADO
            );
        }
    }
}

// Handler para actualizar el estado de un 'Repartidor' en el almacenamiento.
impl Handler<ActualizarComensalUbicacionRestaurante> for Almacenamiento {
    type Result = ();
    fn handle(&mut self, msj: ActualizarComensalUbicacionRestaurante, _: &mut Self::Context) {
        if let Some(repartidor) = self.comensales.get_mut(&msj.id_comensal) {
            repartidor.posicion_comensal = msj.nueva_ubicacion;
        } else {
            eprintln!("{} {}", REPARTIDOR_CON_ID_MSJ, AVISO_NO_ENCONTRADO);
        }
    }
}
// Handler para eliminar un 'Repartidor' del almacenamiento.
impl Handler<BorrarRepartidor> for Almacenamiento {
    type Result = ();
    fn handle(&mut self, msj: BorrarRepartidor, _: &mut Self::Context) {
        println!("{} {:?}", BORRANDO_REPARTIDOR_CON_ID, msj.id);
        self.repartidores_posta.remove(&msj.id);
    }
}

// Handler para eliminar un 'Comensal' del almacenamiento.
impl Handler<BorrarComensal> for Almacenamiento {
    type Result = ();
    fn handle(&mut self, msj: BorrarComensal, _: &mut Self::Context) {
        println!("{} {:?}", CLIENTE_BORRADO, msj.id);
        self.comensales.remove(&msj.id);
    }
}

// Handller para obtener un repartidor cercano.
impl Handler<ObtenerRepartidorCercano> for Almacenamiento {
    type Result = Option<SocketAddr>;
    fn handle(&mut self, _: ObtenerRepartidorCercano, _: &mut Self::Context) -> Self::Result {
        println!("{}", OBTENIENDO_REPARTIDOR_CERCANO);
        let mut repartidor_mas_cercano: Option<SocketAddr> = None;
        let mut menor_distancia = f32::INFINITY;
        for (addr, restaurante) in self.repartidores_posta.iter() {
            println!(
                "{} {:?} {} {:?}",
                VERIFICANDO_REPARTIDOR_CON_ID,
                addr,
                EN_POSICION_MSJ,
                restaurante.posicion_repartidor
            );

            if restaurante.estado == EstadoRepartidor::Active {
                let distancia = ((restaurante.posicion_repartidor.0).powi(2)
                    + (restaurante.posicion_repartidor.1).powi(2))
                .sqrt();

                if distancia < menor_distancia {
                    menor_distancia = distancia;
                    repartidor_mas_cercano = Some(*addr);
                }
            }
        }

        println!("{}", REPARTIDOR_MAS_CERCANO_MSJ);

        repartidor_mas_cercano
    }
}

// Handler para finalizar un viaje de un 'Comensal' y actualizar el estado del 'Repartidor'.
impl Handler<FinalizarViaje> for Almacenamiento {
    type Result = ();
    fn handle(&mut self, msj: FinalizarViaje, _: &mut Self::Context) {
        println!(
            "{} {:?}",
            TERMINANDO_VIAJE_COMENSAL, msj.direccion_comensal_f
        );

        if let Some(repartidor) = self.repartidores_posta.get_mut(&msj.direccion_conductor_f) {
            repartidor.estado = EstadoRepartidor::Active;
            repartidor.time_stamp = std::time::Instant::now();
            repartidor.posicion_repartidor = msj.pos_destino;
        }

        if self.comensales.remove(&msj.direccion_comensal_f).is_none() {
            eprintln!(
                "{} {:?} {}",
                COMENSAL_CON_ID, msj.direccion_comensal_f, AVISO_NO_ENCONTRADO
            );
        }
    }
}

// Handler para verificar si un 'Comensal' ha finalizado su viaje / espera del pedido.
impl Handler<Finalizado> for Almacenamiento {
    type Result = bool;
    fn handle(&mut self, msj: Finalizado, _: &mut Self::Context) -> Self::Result {
        println!(
            "{} {:?} {}",
            CHEQUEANDO_FINALIZADO, msj.id_comensal, HA_FINALIZADO
        );

        !self.comensales.contains_key(&msj.id_comensal)
    }
}

// Handler para eliminar repartidores ausentes (muertos) del almacenamiento.
impl Handler<BorrarRepartidoresAusentes> for Almacenamiento {
    type Result = Vec<RepartidorAusente>;
    fn handle(
        &mut self,
        _: BorrarRepartidoresAusentes,
        _: &mut Context<Self>,
    ) -> Vec<RepartidorAusente> {
        let mut repartidor_ausentes = Vec::new();
        for (id_repartidor, repartidor) in self.repartidores_posta.iter_mut() {
            if matches!(repartidor.estado, EstadoRepartidor::Waiting)
                && repartidor.time_stamp.elapsed().as_secs() > 3
            {
                let id_comensal = repartidor.id_actual_comensal;
                let enviador_comensal = self
                    .comensales
                    .get(&id_comensal.unwrap())
                    .unwrap()
                    .enviador_comensal
                    .clone();

                let repartidor_ausente = RepartidorAusente {
                    id_repartidor: *id_repartidor,
                    id_comensal,
                    enviador_comensal,
                };

                repartidor_ausentes.push(repartidor_ausente);
            }
        }

        if !repartidor_ausentes.is_empty() {
            println!(
                "{} {} {}",
                ELIMINANDO_REPARTIDORES,
                repartidor_ausentes.len(),
                REPARTIDORES_AUSENTES
            );
        }

        for repartidor_ausente in &repartidor_ausentes {
            self.repartidores_posta
                .remove(&repartidor_ausente.id_repartidor);
            self.comensales
                .remove(&repartidor_ausente.id_comensal.unwrap());
        }

        repartidor_ausentes
    }
}

// Handler que maneja el pedido que quiere un comensal.
impl Handler<QuieroPedidoSoyComensal> for Almacenamiento {
    type Result = Option<EntidadRestaurante>;
    fn handle(
        &mut self,
        quiero_pedido: QuieroPedidoSoyComensal,
        _: &mut Self::Context,
    ) -> Self::Result {
        println!("{}", OBTENIENDO_REPARTIDOR_CERCANO);
        let mut restaurante_mas_cercano: Option<EntidadRestaurante> = None;
        let mut menor_distancia = f32::INFINITY;

        for (addr, restaurante) in self.restaurantes.iter() {
            println!(
                "{} {:?} {} {:?}",
                VERIFICANDO_REPARTIDOR_CON_ID,
                addr,
                EN_POSICION_MSJ,
                restaurante.posicion_restaurante
            );

            if restaurante.estado == EstadoRepartidor::Active {
                let dx = restaurante.posicion_restaurante.0 - quiero_pedido.ubicacion_comensal.0;
                let dy = restaurante.posicion_restaurante.1 - quiero_pedido.ubicacion_comensal.1;
                let distancia = (dx.powi(2) + dy.powi(2)).sqrt();
                if distancia < menor_distancia {
                    menor_distancia = distancia;
                }

                restaurante_mas_cercano = Some((*restaurante).clone());
            }
        }

        restaurante_mas_cercano
    }
}
