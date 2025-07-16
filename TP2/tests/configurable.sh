#!/bin/bash

# Este script lanza una cantidad variable de instancias de repartidores, restaurantes y comensales.
# Uso: './script.sh <REPARTIDORES> <RESTAURANTES> <COMENSALES>'.

# Verificación de argumentos.
if [ "$#" -ne 3 ]; then
    echo "Uso: $0 <REPARTIDORES> <RESTAURANTES> <COMENSALES>"
    exit 1
fi

# Cantidades pasadas por consola.
CANT_REPARTIDORES=$1
CANT_RESTAURANTES=$2
CANT_COMENSALES=$3

# Tipo de terminal a usar.
TERMINAL="gnome-terminal"

# Comandos a ejecutar.
COMANDO_REPARTIDOR="cargo run --bin repartidor"
COMANDO_RESTAURANTE="cargo run --bin restaurante"
COMANDO_COMENSAL="cargo run --bin comensal"

# Ejecutar instancias de cada tipo según los argumentos recibidos por consola.

for i in $(seq 0 $((CANT_REPARTIDORES - 1))); do
    $TERMINAL --tab --title=Repartidor_$i -- bash -c "$COMANDO_REPARTIDOR; exec bash"
done

for i in $(seq 0 $((CANT_RESTAURANTES - 1))); do
    $TERMINAL --tab --title=Restaurante_$i -- bash -c "$COMANDO_RESTAURANTE; exec bash"
done

for i in $(seq 0 $((CANT_COMENSALES - 1))); do
    $TERMINAL --tab --title=Comensal_$i -- bash -c "$COMANDO_COMENSAL; exec bash"
done
