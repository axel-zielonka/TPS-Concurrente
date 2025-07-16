#!/bin/bash

# Este script lanza 5 instancias de servidor, en 5 terminales diferentes, cada una con un puerto diferente.
# Además tiene activados los logs relacionados al proceso de elecciones para ver como funciona este frente a la caída del servidor líder.

# Se determina la cantidad de servidores a lanzar y el puerto inicial.
N_TERMINALES=5
PUERTO_INICIAL=8080

# Se define el listado de comando a ejecutar en cada terminal.
COMANDOS=(
    "cargo run --features log_eleccion --bin servidor 8080"
    "cargo run --features log_eleccion --bin servidor 8081"
    "cargo run --features log_eleccion --bin servidor 8082"
    "cargo run --features log_eleccion --bin servidor 8083"
    "cargo run --features log_eleccion --bin servidor 8084"
)

# Se define el tipo de terminal a utilizar.
TERMINAL="gnome-terminal"

# Ejecuta todos los comandos para armar el caso de análisis mencionado anteriormente.
for i in $(seq $((N_TERMINALES - 1)) -1 0); do
    if [ $i -lt ${#COMANDOS[@]} ]; then
        PORT=$((PUERTO_INICIAL + $i))
        $TERMINAL --tab --title=Servidor_$PORT -- bash -c "${COMANDOS[$i]}; exec bash"
    fi
done
