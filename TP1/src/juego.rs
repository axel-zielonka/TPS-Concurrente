use std::collections::HashMap;
///Struct que almacena la información de un juego
/// reviews es la cantidad de reseñas escritas para un juego
/// idiomas es un HashMap donde las claves son los idiomas en los que se escribieron reseñas para
///     juego, y los valores son la cantidad de reseñas en dicho idioma
/// reviews_idiomas es un HashMap cuyas claves son los idiomas en los que se escribió una reseña para
///     ese juego y los valores son una dupla de texto y valor pertenecientes a la reseña con más
///     votos en dicho juego
#[derive(Debug, Default, Clone)]
pub struct Juego {
    pub reviews: usize,
    pub idiomas: HashMap<String, usize>,
    pub reviews_idiomas: HashMap<String, (String, u32)>,
}
