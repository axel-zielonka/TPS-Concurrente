use serde::Serialize;

/// Struct que almacena la información ya procesada de un Idioma
/// idioma es el nombre del idioma
/// reviews es la cantidad de reseñas que obtuvo
/// top_reviews es el vector con las reseñas escritas en dicho idioma, junto con sus votos
#[derive(Serialize)]
pub struct IdiomaMasVotado {
    #[serde(rename = "language")]
    pub idioma: String,
    #[serde(rename = "review_count")]
    pub reviews: u32,
    pub top_reviews: Vec<ReviewIdioma>,
}

/// Struct que almacena una reseña en un idioma determinado
/// review es el contenido de la reseña
/// votos es la cantidad de votos que obtuvo
#[derive(Serialize)]
pub struct ReviewIdioma {
    pub review: String,
    #[serde(rename = "votes")]
    pub votos: u32,
}
