## Introducción

Steam es una plataforma de distribución digital de videojuegos.

Publica una API para obtener, entre otras cosas, los reviews que los usuarios publican sobre estos juegos.
https://partner.steamgames.com/doc/store/getreviews

Queremos analizar un dataset con dumps de estos reviews par encontrar los juegos mejor calificados, y las 
caracteristicas de su comunidad.

Dicho dataset se encuentra publicado en Kaggle, en formato CSV donde cada linea correspodne a una reseña hecha por un 
usuario sobre un juego.

## Objetivo
Implementar una aplicación en Rust para procesamiento de información, aprovechando las ventajas del modelo Fork-Join, 
utilizando el dataset https://www.kaggle.com/datasets/najzeko/steam-reviews-2021

La información a obtener del mismo incluye:

* El top 3 de juegos con más reseñas (campo `app_name`). En caso de empate, resolver alfabeticamente.
  * De cada juego, la cantidad reseñas que obtuvo.
  * Los 3 idiomas más utilizados para hacer reseñas del juego, junto con la cantidad de reseñas en ese idioma. En caso 
  de empate, resolver alfabeticamente.
    * Y por cada idioma, el texto de la reseña más votada (`votes_helpful`) con su cantidad de votos.
    
* El top 3 de idiomas utilizados para hacer reseñas (campo `language`). En caso de empate, resolver alfabeticamente.
  * De cada idioma, la cantidad total de reseñas escritas en ese idioma.
  * El top 10 de reseñas más votadas en ese idioma junto con la cantidad de votos.
  
## Requerimientos
  
* La aplicación debe recibir como parámetro de linea de comandos el path a un directorio, y debe procesar todos los 
archivos `.csv` en el mismo. Los archivos a procesar corresponden con el formato del archivo `stream_reviews.csv` dentro 
del dataset.

* Debe recibir un segundo parámetro entero por linea de comandos indicando la cantidad de worker threads con la cual 
procesar la información

* Debe recibir un tercer parámetro con el nombre del archivo de salida como resultado del procesamiento.

* En resumen, la aplicación será ejecutada como `cargo run <input-path> <num-threads> <output-file-name>`

El formato del archivo de salida debe ser
```
{
"padron": <número de padron del alumno>,
"top_games": [{
"game": "<nombre juego 1>",
"review_count": <cantidad total reseñas para ese juego>
"languages": [
{
"language": "<nombre idioma 1>",
"review_count": <cantidad de reseñas para ese juego en ese idioma>,
"top_review": "<texto del comentario mas votado>",
"top_review_votes": <cantidad de votos que recibio el comentario mas votado>
},
...,
{
"language": "<nombre idioma 3>",
...
},
]
},
...,
{
"game": "<nombre juego 3>",
...
}],
"top_languages": [{
{
"language": "<nombre idioma 1>"
"review_count": <cantidad total reseñas para ese juego>,
"top_reviews": [{
"review": "<texto de la reseña mas votada>",
"votes": cantidad de votos que obtuvo
},
...,
{
"review": "<texto de la reseña N>",
...
}]
},
...
{
"language": "<nombre idioma 3>"
...
},
}]
}
```

## Requerimientos no funcionales
Los siguientes son los requerimientos no funcionales para la resolución de los ejercicios:

* El proyecto deberá ser desarrollado en lenguaje Rust, usando las herramientas de la biblioteca estándar.

* El archivo Cargo.toml se debe encontrar en la raíz del repositorio, para poder ejecutar correctamente los tests 
automatizados

* Se deberán utilizar las herramientas de concurrencia correspondientes al modelo forkjoin

* No se permite utilizar **crates** externos, salvo los explícitamente mencionados en este enunciado, en los ejemplos 
de la materia, o autorizados expresamente por los profesores. Para el procesamiento de JSON se puede utilizar el crate 
`serde_json`

* El código fuente debe compilarse en la última versión stable del compilador y no se permite utilizar bloques unsafe.

* El código deberá funcionar en ambiente Unix / Linux.

* El programa deberá ejecutarse en la línea de comandos.

* La compilación no debe arrojar **warnings** del compilador, ni del linter **clippy**.

* Las funciones y los tipos de datos (**struct**) deben estar documentadas siguiendo el estándar de **cargo doc**.

* El código debe formatearse utilizando **cargo fmt**.

* Cada tipo de dato implementado debe ser colocado en una unidad de compilación (archivo fuente) independiente.

## Entrega
La resolución del presente proyecto es individual.

La entrega del proyecto se realizará mediante Github Classroom.
Cada estudiante tendrá un repositorio disponible para hacer diferentes commits con el objetivo de resolver el problema
propuesto. Se recomienda iniciar tempranamente y hacer commits pequeños agreguen funcionalidad incrementalmente.

Se podrán hacer commits hasta el día de la entrega a las 19 hs Arg, luego el sistema automáticamente quitará el acceso 
de escritura.

El archivo README.md en el repositorio debe incluir el padrón del alumno 
y un enlace a un video de entre 5 y 7 minutos donde el alumno explique su resolución, detallando las decisiones de
diseño y mostrando en vivo el código que escribió. El video debe mostrar claramente la cara del alumno y se debe poder 
escuchar correctamente y con buen volumen su voz.

## Evaluación

# Principios teóricos y corrección de bugs
La evaluación se realizará sobre Github, pudiendo el profesor hacer comentarios en el repositorio y solicitar cambios o 
mejoras cuando lo encuentre oportuno, especialmente debido al uso incorrecto de herramientas de concurrencia.

# Casos de prueba
Se someterá a la aplicación a diferentes casos de prueba que validen la correcta aplicación de las 
herramientas de concurrencia, por ejemplo, la ausencia de deadlocks.

Además la aplicación deberá respetar los formatos de salida y valores esperados de los resultados, y deberá mostrar 
algún incremento en performance cuando la ejecución de la misma se hace con varios hilos en un ambiente multiprocesador.

# Organización del código
El código debe organizarse respetando los criterios de buen diseño y en particular aprovechando las herramientas 
recomendadas por Rust. Se prohibe el uso de bloques `unsafe`.

# Tests automatizados
La presencia de tests automatizados que prueben diferentes casos, 
en especial sobre el uso de las herramientas de concurrencia es un plus.

# Presentación en término
El trabajo deberá entregarse para la fecha estipulada. 
La presentación fuera de término sin coordinación con antelación con el profesor influye negativamente en la nota final.