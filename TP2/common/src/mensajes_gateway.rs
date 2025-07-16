//! Este módulo contiene la definición de los mensajes del actor 'Gateway'.

// Imports de crates externas.
use actix::prelude::*;
use serde::{Deserialize, Serialize};

// Define los mensajes que puede recibir el actor 'Gateway'.
#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub enum MensajeGateway {
    Validar,
    Pagar,
}

// Mensaje para requerir una validación de pago o un cobro efectivo.
#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub enum RequerirPago {
    ValidarAutorizacionPago(ValidarAutorizacionPago),
    EfectivizarPago(EfectivizarPago),
}

// Mensaje para requerir una validación de autorización de pago.
#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct ValidarAutorizacionPago {
    pub id_comensal: String,
    pub valor: f32,
}

// Respuesta de autorización de pago, indicando si fue autorizado o no.
#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct RespuestaAutorizacion {
    pub id_comensal: String,
    pub autorizado: bool,
}

// Mensaje para requerir un cobro efectivo.
#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub struct EfectivizarPago {
    pub id_comensal: String,
    pub valor: f32,
}

// Respuesta al cobro efectivo, indicando si el pago fue realizado o si hubo un error.
#[derive(Message, Serialize, Deserialize, Debug)]
#[rtype(result = "()")]
pub enum RespuestaPago {
    PagoHecho,
    PaymentError(String),
}
