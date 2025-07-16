/// Struct usado para almacenar las estadísticas de un idioma
/// cantidad_reviews contiene la cantidad de reseñas en un idioma
/// top_reviews es un vector que almacena la reseña junto con sus votos en una tupla
#[derive(Debug, Default, Clone)]
pub struct Idioma {
    pub cantidad_reviews: usize,
    pub top_reviews: Vec<(String, u32)>,
}
