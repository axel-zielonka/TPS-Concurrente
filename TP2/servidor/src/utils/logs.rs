//! Este módulo contiene la lógica de la diferenciación de 'logs' .

// Usado para los logs de la elección del coordinador.
pub fn log_eleccion(_msg: String) {
    #[cfg(feature = "log_eleccion")]
    {
        println!("{}", _msg);
    }
}

// Usado para los logs de funcionamiento del sistema.
pub fn log_funcionamiento(_msg: String) {
    #[cfg(feature = "log_funcionamiento")]
    {
        println!("{}", _msg);
    }
}
