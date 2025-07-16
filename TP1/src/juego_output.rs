use serde::Serialize;

/// Struct que representa la salida de un juego.
/// nombre es el nombre del juego
/// reviews es la cantidad de reseñas que obtuvo
/// idiomas es un vector con los idiomas en los que se escribieron reseñas para dicho juego,
///     junto con sus reseñas y votos
#[derive(Serialize)]
pub struct JuegoMasVotado {
    #[serde(rename = "game")]
    pub nombre: String,
    #[serde(rename = "review_count")]
    pub reviews: u32,
    #[serde(rename = "languages")]
    pub idiomas: Vec<IdiomaPorJuego>,
}

/// Struct que representa los idiomas en los que se escribieron reseñas para un determinado juego
/// idioma es el nombre del idioma
/// reviews es la cantidad totaL de reseñas escritas para un mismo juego en un idioma
/// top_review es el contenido de la review en un idioma con la mayor cantidad de votos
/// top_review_votos es la cantidad de votos que obtuvo la reseña con más votos para un juego en
///     un idioma
#[derive(Serialize)]
pub struct IdiomaPorJuego {
    #[serde(rename = "language")]
    pub idioma: String,
    #[serde(rename = "review_count")]
    pub reviews: u32,
    #[serde(rename = "top_review")]
    pub top_review: String,
    #[serde(rename = "top_review_votes")]
    pub top_review_votos: u32,
}
