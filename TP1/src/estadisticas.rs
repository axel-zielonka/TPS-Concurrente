use crate::idioma::Idioma;
use crate::juego::Juego;
use crate::review::Review;
use std::collections::HashMap;

/// Struct que almacena las estadísticas generales de los archivos csv
/// juegos es un HashMap donde la clave es el nombre del juego y el valor es una instancia del
///     struct 'Juego' (ver juego.rs)
/// idiomas es un HashMap donde la clave es un idioma y el valor es una instancia del struct
///     'Idioma' (ver idioma.rs)
#[derive(Debug, Default, Clone)]
pub struct Estadisticas {
    pub juegos: HashMap<String, Juego>,
    pub idiomas: HashMap<String, Idioma>,
}
impl Estadisticas {
    /// Función que recibe una instancia de una Review y la agrega a las estadísticas internas
    /// Agrega las entradas a los diccionarios en caso de que no existan, tanto para el idioma
    /// como para el juego.
    /// Actualiza las cantidades de reviews por juego y por idioma
    pub fn agregar_review(&mut self, review: Review) {
        // Actualizar juego
        let juego = self.juegos.entry(review.app_name.clone()).or_default();
        juego.reviews += 1;
        *juego.idiomas.entry(review.language.clone()).or_insert(0) += 1;

        juego
            .reviews_idiomas
            .entry(review.language.clone())
            .and_modify(|(texto_actual, votos_actual)| {
                if review.votes_helpful > *votos_actual {
                    *texto_actual = review.review.clone();
                    *votos_actual = review.votes_helpful;
                }
            })
            .or_insert((review.review.clone(), review.votes_helpful as u32));

        // Actualizar idioma
        let idioma = self.idiomas.entry(review.language.clone()).or_default();

        idioma.cantidad_reviews += 1;
        if review.votes_helpful > 0 {
            idioma
                .top_reviews
                .push((review.review, review.votes_helpful));
        }
    }
}

/// Función que recibe por parámetro 2 instancias del Struct Estadísticas y los combina en uno nuevo
/// De esta forma, la nueva instancia de Estadisticas contiene todos los datos de sus
/// "predecesoras", es decir, todas las reseñas de juegos e idiomas. En caso de que un juego o un
/// idioma esté en ambas predecesoras, se suman las cantidades para reflejar correctamente la
/// cantidad de reviews
pub fn combinar_estadisticas(a: Estadisticas, b: Estadisticas) -> Estadisticas {
    let mut resultado = Estadisticas {
        juegos: a.juegos,
        idiomas: a.idiomas,
    };

    for (key, juego_b) in b.juegos {
        let juego_a = resultado
            .juegos
            .entry(key.clone())
            .or_insert_with(|| Juego {
                reviews: 0,
                idiomas: HashMap::new(),
                reviews_idiomas: HashMap::new(),
            });

        juego_a.reviews += juego_b.reviews;

        for (idioma_b, count_b) in juego_b.idiomas {
            let count_a = juego_a.idiomas.entry(idioma_b.clone()).or_insert(0);
            *count_a += count_b;
        }

        for (idioma_b, (review_b, votos_b)) in juego_b.reviews_idiomas {
            let entry = juego_a
                .reviews_idiomas
                .entry(idioma_b.clone())
                .or_insert((review_b.clone(), votos_b));
            if votos_b > entry.1 {
                *entry = (review_b.clone(), votos_b);
            }
        }
    }

    for (idioma_b, idioma_b_info) in b.idiomas {
        let idioma_a = resultado
            .idiomas
            .entry(idioma_b.clone())
            .or_insert_with(|| Idioma {
                cantidad_reviews: 0,
                top_reviews: Vec::new(),
            });

        idioma_a.cantidad_reviews += idioma_b_info.cantidad_reviews;
        idioma_a.top_reviews.extend(idioma_b_info.top_reviews);
    }

    resultado
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn agregar_una_review() {
        let mut estadisticas = Estadisticas::default();
        let review = Review {
            app_name: "FIFA".to_string(),
            language: "Español".to_string(),
            review: "Alto juego".to_string(),
            votes_helpful: 100,
        };

        estadisticas.agregar_review(review.clone());

        assert!(estadisticas.juegos.contains_key("FIFA"));
        let juego = estadisticas.juegos.get("FIFA").unwrap();
        assert_eq!(juego.reviews, 1);
        assert_eq!(juego.idiomas.get("Español"), Some(&1));
        assert_eq!(
            juego.reviews_idiomas.get("Español"),
            Some(&("Alto juego".to_string(), 100))
        );

        assert!(estadisticas.idiomas.contains_key("Español"));
        let idioma = estadisticas.idiomas.get("Español").unwrap();
        assert_eq!(idioma.cantidad_reviews, 1);
        assert_eq!(idioma.top_reviews, vec![("Alto juego".to_string(), 100)]);
    }

    #[test]
    fn agregar_2_reviews_mismo_juego_distinto_idioma() {
        let mut estadisticas = Estadisticas::default();
        let r1 = Review {
            app_name: "FIFA".to_string(),
            language: "Español".to_string(),
            review: "Alto juego".to_string(),
            votes_helpful: 100,
        };
        let r2 = Review {
            app_name: "FIFA".to_string(),
            language: "Inglés".to_string(),
            review: "Very Good".to_string(),
            votes_helpful: 50,
        };

        estadisticas.agregar_review(r1.clone());
        estadisticas.agregar_review(r2.clone());

        let juego = estadisticas.juegos.get("FIFA").unwrap();
        assert_eq!(juego.reviews, 2);
        assert_eq!(juego.idiomas.get("Español"), Some(&1));
        assert_eq!(juego.idiomas.get("Inglés"), Some(&1));

        assert!(estadisticas.idiomas.contains_key("Español"));
        assert!(estadisticas.idiomas.contains_key("Inglés"));
        let esp = estadisticas.idiomas.get("Español").unwrap();
        assert_eq!(esp.cantidad_reviews, 1);
        assert_eq!(esp.top_reviews, vec![("Alto juego".to_string(), 100)]);

        let ing = estadisticas.idiomas.get("Inglés").unwrap();
        assert_eq!(ing.cantidad_reviews, 1);
        assert_eq!(ing.top_reviews, vec![("Very Good".to_string(), 50)]);
    }

    #[test]
    fn agregar_2_reviews_mismo_juego_mismo_idioma() {
        let mut estadisticas = Estadisticas::default();
        let r1 = Review {
            app_name: "FIFA".to_string(),
            language: "Español".to_string(),
            review: "Alto juego".to_string(),
            votes_helpful: 100,
        };
        let r2 = Review {
            app_name: "FIFA".to_string(),
            language: "Español".to_string(),
            review: "Aguante el modo carrera".to_string(),
            votes_helpful: 50,
        };

        estadisticas.agregar_review(r1.clone());
        estadisticas.agregar_review(r2.clone());

        let juego = estadisticas.juegos.get("FIFA").unwrap();
        assert_eq!(juego.reviews, 2);
        assert_eq!(juego.idiomas.get("Español"), Some(&2));
        assert_eq!(
            juego.reviews_idiomas.get("Español"),
            Some(&("Alto juego".to_string(), 100))
        );
        let esp = estadisticas.idiomas.get("Español").unwrap();
        assert_eq!(esp.cantidad_reviews, 2);
    }

    #[test]
    fn agregar_mismo_mismo_juego_mismo_idioma_con_mas_votos() {
        let mut estadisticas = Estadisticas::default();
        let r1 = Review {
            app_name: "FIFA".to_string(),
            language: "Español".to_string(),
            review: "Alto juego".to_string(),
            votes_helpful: 100,
        };
        let r2 = Review {
            app_name: "FIFA".to_string(),
            language: "Español".to_string(),
            review: "Aguante el modo carrera".to_string(),
            votes_helpful: 200,
        };

        estadisticas.agregar_review(r1.clone());
        estadisticas.agregar_review(r2.clone());
        let juego = estadisticas.juegos.get("FIFA").unwrap();
        assert_eq!(juego.reviews, 2);
        assert_eq!(juego.idiomas.get("Español"), Some(&2));
        assert_eq!(
            juego.reviews_idiomas.get("Español"),
            Some(&("Aguante el modo carrera".to_string(), 200))
        );
        let esp = estadisticas.idiomas.get("Español").unwrap();
        assert_eq!(esp.cantidad_reviews, 2);
    }
}
