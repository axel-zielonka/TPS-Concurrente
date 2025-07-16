#!/bin/bash

# Este script lanza 2 instancias de servidor, mas una instancia del gateway, en 3 terminales diferentes, cada una con un puerto diferente.
# Además tiene activados los logs relacionados a los viajes, para ver como funciona el sistema de forma productiva.

# Se determina la cantidad de servidores a lanzar y el puerto inicial.
N_TERMINALES=2
PUERTO_INICIAL=8080

# Se define el listado de comando a ejecutar en cada terminal.
COMANDO_GATEWAY="cargo run --bin gateway"
TITULO_GATEWAY="Gateway"
COMANDOS_SERVIDOR=(
    "cargo run --features log_funcionamiento --bin servidor 8080"
    "cargo run --features log_funcionamiento --bin servidor 8081"
)

# Se define el tipo de terminal a utilizar.
TERMINAL="gnome-terminal"

# Ejecuta todos los comandos para armar el caso de análisis mencionado anteriormente.
$TERMINAL --tab --title=$TITULO_GATEWAY -- bash -c "$COMANDO_GATEWAY; exec bash"
for i in $(seq $((N_TERMINALES - 1)) -1 0); do
    if [ $i -lt ${#COMANDOS_SERVIDOR[@]} ]; then
        PORT=$((PUERTO_INICIAL + $i))
        $TERMINAL --tab --title=Servidor_$PORT -- bash -c "${COMANDOS_SERVIDOR[$i]}; exec bash"
    fi
done
