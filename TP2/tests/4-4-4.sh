#!/bin/bash

# Este script lanza 4 instancias de repartidores, 4 instancias de restaurantes y 4 instancias de comensales, en 12 terminales diferentes.
# Funciona para verificar el funcionamiento del sistema con una cantidad moderada de aplicaciones.

# Se define el tipo de terminal a utilizar.
TERMINAL="gnome-terminal"

# Se definen los comandos que se van a utilizar.
COMANDO_REPARTIDOR="cargo run --bin repartidor"
COMANDO_RESTAURANTE="cargo run --bin restaurante"
COMANDO_COMENSAL="cargo run --bin comensal"

# Ejecuta todos los comandos para armar el caso de análisis mencionado anteriormente.
for i in $(seq 0 3); do
    $TERMINAL --tab --title=Repartidor_$i -- bash -c "$COMANDO_REPARTIDOR; exec bash"
done

for i in $(seq 0 3); do
    $TERMINAL --tab --title=Restaurante_$i -- bash -c "$COMANDO_RESTAURANTE; exec bash"
done

for i in $(seq 0 3); do
    $TERMINAL --tab --title=Comensal_$i -- bash -c "$COMANDO_COMENSAL; exec bash"
done
