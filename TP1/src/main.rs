mod configurar_argumentos;
mod estadisticas;
mod formatear_datos_json;
mod idioma;
mod idioma_output;
mod juego;
mod juego_output;
mod parsear_csv;
mod review;

use crate::estadisticas::Estadisticas;

use crate::formatear_datos_json::Output;
use rayon::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

/// Obtiene todos los archivos que se encuentren en la ruta del directorio recibida por parámetro
/// y los devuelve cómo un vector de Paths
fn obtener_archivos(ruta_archivo: &str) -> Vec<PathBuf> {
    let files = fs::read_dir(ruta_archivo).expect("Failed to read ruta archivos");
    files
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "csv").unwrap_or(false))
        .collect::<Vec<_>>()
}

/// Recibe la cantidad de threads que se quieren correr en paralelo y crea la ThreadPool
fn lanzar_thread_pool(n: usize) -> ThreadPool {
    ThreadPoolBuilder::new()
        .num_threads(n)
        .build()
        .expect("Failed to create thread pool")
}

/// Recibe el vector con los archivos y una referencia mutable de una instancia de Estadisticas,
/// así como también un ThreadPool.
/// Lanza el ThreadPool y en paralelo recorre los archivos en 'archivos' usando par_iter()
/// A cada archivo de 'archivos' le aplica de forma concurrente la funcion 'procesar_csv()'
/// Luego al final, combina todas las estadísticas recibidas de los distintos archivos en una
/// instancia de Estadisticas y crea el Output para realizar la salida
fn procesar_archivos(archivos: Vec<PathBuf>, mut e: Estadisticas, pool: ThreadPool) -> Output {
    pool.install(|| {
        e = archivos
            .par_iter()
            .map(|path| parsear_csv::procesar_csv(path))
            .reduce(
                || Estadisticas {
                    juegos: std::collections::HashMap::new(),
                    idiomas: std::collections::HashMap::new(),
                },
                estadisticas::combinar_estadisticas,
            )
    });

    let o = Output::new(&e);
    o
}

/// Recibe el Output resultante de procesar todos los archivos csv y lo escribe en el archivo de
/// salida que se recibió por línea de comandos al principio de la ejecución del programa
fn escribir_resultado(resultado: Output, salida: &str) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(&resultado).expect("Error al serializar salida");
    let mut archivo = File::create(&salida)?;
    archivo.write_all(json.as_bytes())?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let start = Instant::now();

    let args = match configurar_argumentos::parsear_argumentos() {
        Some(a) => a,
        None => std::process::exit(1),
    };

    let archivos = obtener_archivos(&args.ruta);

    let estadisticas_totales = Estadisticas::default();

    let pool = lanzar_thread_pool(args.n_threads);

    let resultado = procesar_archivos(archivos, estadisticas_totales, pool);

    escribir_resultado(resultado, &args.archivo_salida)?;

    println!("Duración del programa: {} segs", start.elapsed().as_secs());
    Ok(())
}
