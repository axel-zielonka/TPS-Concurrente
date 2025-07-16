use csv::StringRecord;
use serde::Deserialize;

/// Constantes con las posiciones de los campos necesarios para el análisis del csv
const POSICION_APP_NAME: usize = 2;
const POSICION_LANGUAGE: usize = 4;
const POSICION_REVIEW: usize = 5;
const POSICION_VOTES_HELPFUL: usize = 9;
/// Como estoy tomando la cantidad de votos como un u32, y la lectura del archivo agarra el número
/// como un String, existe la posibilidad de que se haya ingresado un valor que al castearlo a u32
/// sea mayor al límite permitido. En este caso, la cantidad de votos no sería válida, y para no
/// perder la review asigno manualmente la cantidad de votos a 0
const VALOR_DEFAULT_VOTOS: u32 = 0;

/// Struct que almacena la información de una Review de Steam.
/// Los nombres de los atributos coinciden con los campos del csv
/// app_name es el nombre del juego
/// language es su idioma
/// review es el texto de la reseña
/// votes_helpful son los votos que tuvo dicha reseña
#[derive(Debug, Deserialize, Clone)]
pub struct Review {
    pub app_name: String,
    pub language: String,
    pub review: String,
    pub votes_helpful: u32,
}

impl Review {
    /// Recibe un StringRecord y lo convierte en una instancia de Review, retornando un Option
    /// por si llegara a fallar el parseo de algunos de sus elementos.
    pub fn parse_record(record: &StringRecord) -> Option<Review> {
        let app_name = record.get(POSICION_APP_NAME)?.to_string();
        let language = record.get(POSICION_LANGUAGE)?.to_string();
        let review = record.get(POSICION_REVIEW)?.to_string();
        let votes_helpful = record
            .get(POSICION_VOTES_HELPFUL)?
            .parse::<u32>()
            .unwrap_or(VALOR_DEFAULT_VOTOS);

        Some(Review {
            app_name,
            language,
            review,
            votes_helpful,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generar_vector() -> Vec<String> {
        let mut review_csv = Vec::new();
        for _ in 0..23 {
            review_csv.push("-".to_string());
        }
        review_csv
    }
    #[test]
    fn test_parsear_review() {
        let mut review = generar_vector();
        review[POSICION_APP_NAME] = "FIFA".to_string();
        review[POSICION_LANGUAGE] = "Español".to_string();
        review[POSICION_REVIEW] = "Es el mejor juego que jugue en mi vida".to_string();
        review[POSICION_VOTES_HELPFUL] = "2019".to_string();
        let sr = StringRecord::from(review);

        let r = Review::parse_record(&sr).unwrap();

        assert_eq!(r.app_name, "FIFA");
        assert_eq!(r.language, "Español");
        assert_eq!(r.review, "Es el mejor juego que jugue en mi vida");
        assert_eq!(r.votes_helpful, 2019);
    }

    #[test]
    fn test_parsear_con_votes_mayor_a_u32() {
        let mut review = generar_vector();
        // Un u32 va desde el 0 hasta el 4_294_967_295, por lo que al intentar cargar este numero
        // a la review debería asignarle 0
        review[POSICION_VOTES_HELPFUL] = "4294967296".to_string();
        let sr = StringRecord::from(review);
        let r = Review::parse_record(&sr).unwrap();
        assert_eq!(r.votes_helpful, 0);
    }
}
