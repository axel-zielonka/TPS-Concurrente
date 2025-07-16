use std::env;

const ARGUMENTOS_ESPERADOS: usize = 4;
const RUTA_DATASET: usize = 1;
const N_THREADS: usize = 2;
const ARCHIVO_SALIDA: usize = 3;

/// Struct que almacena los argumentos introducidos por terminal.
/// ruta es el path del directorio del que se quieren leer los archivos
/// n_threads son la cantidad de threads en paralelo que se van a ejecutar
/// archivo_salida es el nombre del archivo en donde se va a encontrar el resultado del programa
///
pub struct Argumentos {
    pub ruta: String,
    pub n_threads: usize,
    pub archivo_salida: String,
}

/// Función que se encarga de parsear los comandos ingresados
/// n_threads debe ser un número natural >= 0
/// En caso de que el archivo de salida no tenga la extensión .json, se la agrega antes de procesar
pub fn parsear_argumentos() -> Option<Argumentos> {
    let args: Vec<String> = env::args().collect();
    if args.len() != ARGUMENTOS_ESPERADOS {
        eprintln!("Cantidad de argumentos inválida");
        return None;
    }
    let ruta = args[RUTA_DATASET].clone();
    let n_threads = args[N_THREADS].parse::<usize>().unwrap_or(1);
    let mut archivo_salida = args[ARCHIVO_SALIDA].clone();
    if !archivo_salida.ends_with(".json") {
        archivo_salida.push_str(".json");
    }
    Some(Argumentos {
        ruta,
        n_threads,
        archivo_salida,
    })
}
