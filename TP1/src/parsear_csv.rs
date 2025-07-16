use crate::estadisticas;
use crate::estadisticas::Estadisticas;
use crate::review::Review;
use csv::ReaderBuilder;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// Tamaño en registros csv de cada chunk a procesar
const CHUNK_SIZE: usize = 200_000;

/// Recibe el path del arhcivo y lo abre. Separa el archivo en Chunks y realiza el procesamiento de
/// cada chunk en paralelo, tanto para el parseo de las Reviews como para el procesamiento de las
/// mismas. Al final, combina todas las Estadísticas obtenidas de los distintos chunks en una misma
/// Estadística que es devuelta al terminar la función
pub fn procesar_csv(path: &Path) -> Estadisticas {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            return Estadisticas {
                juegos: HashMap::new(),
                idiomas: HashMap::new(),
            };
        }
    };

    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(file);
    let mut chunks = Vec::new();
    let mut actual = Vec::with_capacity(CHUNK_SIZE);

    for result in reader.records() {
        if let Ok(record) = result {
            actual.push(record);
            if actual.len() >= CHUNK_SIZE {
                chunks.push(actual);
                actual = Vec::with_capacity(CHUNK_SIZE);
            }
        }
    }

    if !actual.is_empty() {
        chunks.push(actual);
    }

    chunks
        .into_par_iter()
        .map(|chunk| {
            let mut estadisticas = Estadisticas {
                juegos: HashMap::new(),
                idiomas: HashMap::new(),
            };
            for record in chunk {
                if let Some(review) = Review::parse_record(&record) {
                    estadisticas.agregar_review(review);
                }
            }
            estadisticas
        })
        .reduce(
            || Estadisticas {
                juegos: HashMap::new(),
                idiomas: HashMap::new(),
            },
            |a, b| estadisticas::combinar_estadisticas(a, b),
        )
}
