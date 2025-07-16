use crate::estadisticas::Estadisticas;
use crate::idioma_output::{IdiomaMasVotado, ReviewIdioma};
use crate::juego_output::{IdiomaPorJuego, JuegoMasVotado};
use serde::Serialize;

const PADRON: u32 = 110310;

/// Struct que contiene la información que va a ser luego escrita en el archivo de salida
/// padron es mi padrón personal: 110310
/// top3_juegos es un vector de JuegoMasVotado (ver juego_output.rs) que contiene los 3 juegos más
///     votados
/// top3_idiomas es un vector de IdiomaMasVotado (ver idioma_output.rs) que contiene los 3 idiomas
///     con mayor cantidad de reseñas
#[derive(Serialize)]
pub struct Output {
    pub padron: u32,
    #[serde(rename = "top_games")]
    pub top3_juegos: Vec<JuegoMasVotado>,
    #[serde(rename = "top_languages")]
    pub top3_idiomas: Vec<IdiomaMasVotado>,
}

impl Output {
    /// Recibe las estadísticas leídas y devuelve una instancia de Output con los juegos y los
    /// idiomas ya filtrados por cantidad de reviews
    pub fn new(e: &Estadisticas) -> Output {
        Output {
            padron: PADRON,
            top3_juegos: Self::filtrar_juegos(e),
            top3_idiomas: Self::filtrar_idiomas(e),
        }
    }

    /// Recibe las estadísticas leídas y devuelve un vector con los 3 juegos más votados
    /// `Vec<JuegoMasVotado>` es un vector que contiene los 3 juegos con más reseñas, y para cada
    /// juego se tiene un vector con los 3 idiomas con más reviews. Y para cada uno de esos idiomas,
    /// se obtiene la review con mayor votos junto con su contenido
    fn filtrar_juegos(e: &Estadisticas) -> Vec<JuegoMasVotado> {
        let mut juegos: Vec<JuegoMasVotado> = e
            .juegos
            .iter()
            .map(|(nombre, juego)| {
                let mut idiomas: Vec<IdiomaPorJuego> = juego
                    .idiomas
                    .iter()
                    .map(|(idioma, cant_reviews)| {
                        let (texto, votos) = juego
                            .reviews_idiomas
                            .get(idioma)
                            .cloned()
                            .unwrap_or_else(|| ("".to_string(), 0));

                        IdiomaPorJuego {
                            idioma: idioma.clone(),
                            reviews: *cant_reviews as u32,
                            top_review: texto,
                            top_review_votos: votos,
                        }
                    })
                    .collect();

                idiomas.sort_by(|a, b| {
                    b.reviews
                        .cmp(&a.reviews)
                        .then_with(|| a.idioma.cmp(&b.idioma))
                });
                idiomas.truncate(3);

                JuegoMasVotado {
                    nombre: nombre.clone(),
                    reviews: juego.reviews as u32,
                    idiomas,
                }
            })
            .collect();
        juegos.sort_by(|a, b| {
            b.reviews
                .cmp(&a.reviews)
                .then_with(|| a.nombre.cmp(&b.nombre))
        });
        juegos.truncate(3);

        juegos
    }

    ///Recibe las estadisticas leidas y devuelve un `Vec<IdiomaMasVotado>`
    /// `Vec<IdiomaMasVotado>` es un vector con los 3 idiomas que obtuvieron más reseñas, junto con
    /// su cantidad de reseñas
    /// Para cada uno de estos idiomas, se muestran las 10 reseñas con más votos junto con su texto
    /// y su cantidad de votos
    fn filtrar_idiomas(e: &Estadisticas) -> Vec<IdiomaMasVotado> {
        let mut idiomas: Vec<IdiomaMasVotado> = e
            .idiomas
            .iter()
            .map(|(nombre_idioma, datos_idioma)| {
                let mut top: Vec<ReviewIdioma> = datos_idioma
                    .top_reviews
                    .iter()
                    .map(|(texto, votos)| ReviewIdioma {
                        review: texto.clone(),
                        votos: *votos,
                    })
                    .collect();

                top.sort_by(|a, b| b.votos.cmp(&a.votos));
                top.truncate(10); // Máximo 10 reviews por idioma

                IdiomaMasVotado {
                    idioma: nombre_idioma.clone(),
                    reviews: datos_idioma.cantidad_reviews as u32,
                    top_reviews: top,
                }
            })
            .collect();

        idiomas.sort_by(|a, b| {
            b.reviews
                .cmp(&a.reviews)
                .then_with(|| a.idioma.cmp(&b.idioma))
        });
        idiomas.truncate(3);

        idiomas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::juego::Juego;
    use std::collections::HashMap;

    fn generar_estadisticas() -> Estadisticas {
        let mut estadisticas = Estadisticas::default();
        let mut juego1 = Juego {
            reviews: 5,
            idiomas: HashMap::new(),
            reviews_idiomas: HashMap::new(),
        };
        juego1.idiomas.insert("Español".to_string(), 3);
        juego1.idiomas.insert("Inglés".to_string(), 2);
        juego1
            .reviews_idiomas
            .insert("Español".to_string(), ("Muy bueno".to_string(), 50));
        juego1
            .reviews_idiomas
            .insert("Inglés".to_string(), ("Very good".to_string(), 100));

        let mut juego2 = Juego {
            reviews: 7,
            idiomas: HashMap::new(),
            reviews_idiomas: HashMap::new(),
        };
        juego2.idiomas.insert("Francés".to_string(), 6);
        juego2.idiomas.insert("Inglés".to_string(), 1);
        juego2
            .reviews_idiomas
            .insert("Francés".to_string(), ("Très bien!".to_string(), 200));
        juego2
            .reviews_idiomas
            .insert("Inglés".to_string(), ("Very good".to_string(), 20));

        let mut juego3 = Juego {
            reviews: 7,
            idiomas: HashMap::new(),
            reviews_idiomas: HashMap::new(),
        };
        juego3.idiomas.insert("Alemán".to_string(), 5);
        juego3.idiomas.insert("Español".to_string(), 1);
        juego3.idiomas.insert("Francés".to_string(), 1);
        juego3
            .reviews_idiomas
            .insert("Español".to_string(), ("Alto juego".to_string(), 70));
        juego3
            .reviews_idiomas
            .insert("Francés".to_string(), (":)".to_string(), 110));
        juego3
            .reviews_idiomas
            .insert("Alemán".to_string(), ("!!!".to_string(), 200));

        let mut juego4 = Juego {
            reviews: 2,
            idiomas: HashMap::new(),
            reviews_idiomas: HashMap::new(),
        };
        juego4.idiomas.insert("Inglés".to_string(), 2);
        juego4
            .reviews_idiomas
            .insert("Inglés".to_string(), ("Nice".to_string(), 105));

        estadisticas.juegos.insert("NBA 2k25".to_string(), juego1);
        estadisticas.juegos.insert("God of War".to_string(), juego2);
        estadisticas.juegos.insert("FIFA 17".to_string(), juego3);
        estadisticas.juegos.insert("SuperMario".to_string(), juego4);
        estadisticas
    }
    #[test]
    fn test_filtrar_juegos() {
        let estadisticas = generar_estadisticas();
        let resultado = Output::filtrar_juegos(&estadisticas);
        //deberian ser solo 3
        assert_eq!(resultado.len(), 3);

        //Por orden alfabético FIFA debería ir antes que God of War
        assert_eq!(resultado[0].nombre, "FIFA 17");
        assert_eq!(resultado[1].nombre, "God of War");
        assert_eq!(resultado[2].nombre, "NBA 2k25");
    }

    #[test]
    fn chequear_idiomas_en_top3_juegos() {
        let estadisticas = generar_estadisticas();
        let resultado = Output::filtrar_juegos(&estadisticas);

        let idiomas_fifa = &resultado[0].idiomas;
        let idiomas_gow = &resultado[1].idiomas;
        let idiomas_2k = &resultado[2].idiomas;

        assert_eq!(idiomas_fifa.len(), 3);
        assert_eq!(idiomas_gow.len(), 2);
        assert_eq!(idiomas_2k.len(), 2);

        assert_eq!(idiomas_fifa[0].idioma, "Alemán");
        assert_eq!(idiomas_fifa[0].top_review, "!!!");
        assert_eq!(idiomas_fifa[0].reviews, 5);
        assert_eq!(idiomas_fifa[0].top_review_votos, 200);
        assert_eq!(idiomas_fifa[1].idioma, "Español");
        assert_eq!(idiomas_fifa[1].reviews, 1);
        assert_eq!(idiomas_fifa[1].top_review, "Alto juego");
        assert_eq!(idiomas_fifa[1].top_review_votos, 70);
        assert_eq!(idiomas_fifa[2].idioma, "Francés");
        assert_eq!(idiomas_fifa[2].reviews, 1);
        assert_eq!(idiomas_fifa[2].top_review, ":)");
        assert_eq!(idiomas_fifa[2].top_review_votos, 110);

        assert_eq!(idiomas_gow[0].idioma, "Francés");
        assert_eq!(idiomas_gow[0].top_review, "Très bien!");
        assert_eq!(idiomas_gow[0].reviews, 6);
        assert_eq!(idiomas_gow[0].top_review_votos, 200);
        assert_eq!(idiomas_gow[1].idioma, "Inglés");
        assert_eq!(idiomas_gow[1].reviews, 1);
        assert_eq!(idiomas_gow[1].top_review, "Very good");
        assert_eq!(idiomas_gow[1].top_review_votos, 20);

        assert_eq!(idiomas_2k[0].idioma, "Español");
        assert_eq!(idiomas_2k[0].top_review, "Muy bueno");
        assert_eq!(idiomas_2k[0].reviews, 3);
        assert_eq!(idiomas_2k[0].top_review_votos, 50);
        assert_eq!(idiomas_2k[1].idioma, "Inglés");
        assert_eq!(idiomas_2k[1].reviews, 2);
        assert_eq!(idiomas_2k[1].top_review, "Very good");
        assert_eq!(idiomas_2k[1].top_review_votos, 100);
    }

    #[test]
    fn test_filtrar_idiomas() {
        use crate::estadisticas::*;
        use crate::idioma::*;
        use std::collections::HashMap;

        let mut estadisticas = Estadisticas {
            juegos: HashMap::new(),
            idiomas: HashMap::new(),
        };

        // Idioma italiano con 15 reviews, para que despues queden 10
        let mut reviews_it = Vec::new();
        for i in 1..=15 {
            reviews_it.push((format!("Review {}", i), i * 10)); // votos: 10, 20, ..., 150
        }
        let ita = Idioma {
            cantidad_reviews: 15,
            top_reviews: reviews_it,
        };

        let ingl = Idioma {
            cantidad_reviews: 5,
            top_reviews: vec![
                ("Great!".to_string(), 80),
                ("Loved it".to_string(), 95),
                ("Very fun".to_string(), 60),
                ("Nice game".to_string(), 75),
                ("Good enough".to_string(), 50),
            ],
        };

        let esp = Idioma {
            cantidad_reviews: 3,
            top_reviews: vec![
                ("Me encantó".to_string(), 70),
                ("Muy bueno".to_string(), 65),
                ("Excelente".to_string(), 85),
            ],
        };

        //Este deberia quedar afuera por ser el que menos reviews tiene
        let fra = Idioma {
            cantidad_reviews: 1,
            top_reviews: vec![("Incroyable".to_string(), 99)],
        };

        estadisticas.idiomas.insert("it".to_string(), ita);
        estadisticas.idiomas.insert("en".to_string(), ingl);
        estadisticas.idiomas.insert("es".to_string(), esp);
        estadisticas.idiomas.insert("fr".to_string(), fra);

        let resultado = Output::filtrar_idiomas(&estadisticas);

        // Solo deben quedar 3 idiomas
        assert_eq!(resultado.len(), 3);

        // Italiano deberia ir primero, pero solo las primeras 10 reviews tienen que aparecer
        assert_eq!(resultado[0].idioma, "it");
        assert_eq!(resultado[0].reviews, 15);
        assert_eq!(resultado[0].top_reviews.len(), 10);
        assert_eq!(resultado[0].top_reviews[0].review, "Review 15");
        assert_eq!(resultado[0].top_reviews[0].votos, 150);
        assert_eq!(resultado[0].top_reviews[9].votos, 60);

        assert_eq!(resultado[1].idioma, "en");
        assert_eq!(resultado[1].reviews, 5);
        assert_eq!(resultado[1].top_reviews.len(), 5);
        assert_eq!(resultado[1].top_reviews[0].votos, 95);

        assert_eq!(resultado[2].idioma, "es");
        assert_eq!(resultado[2].reviews, 3);
        assert_eq!(resultado[2].top_reviews.len(), 3);
        assert_eq!(resultado[2].top_reviews[0].votos, 85);

        // Chequeo que francés haya quedado afuera
        assert!(resultado.iter().all(|i| i.idioma != "fr"));
    }
}
