//! Módulo que encapsula los distintos.

// 'Enum' que contiene los distintos errores que pueden ocurrir en el servidor.
#[derive(Debug, PartialEq)]
pub enum ServidorError {
    IoError(String),
    BindError(String),
    InvalidSockets(String),
}

// Implementación del trait 'From'.
impl From<std::io::Error> for ServidorError {
    fn from(err: std::io::Error) -> ServidorError {
        ServidorError::IoError(err.to_string())
    }
}
