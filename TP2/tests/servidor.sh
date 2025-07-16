#!/bin/bash

# Este script lanza una instancia del servidor, validando que el puerto esté dentro del rango permitido.
TERMINAL="gnome-terminal"
PORT=$1

# Validar que se haya pasado un argumento.
if [ -z "$PORT" ]; then
    echo "❌ Debes pasar un puerto como argumento. Ejemplo: ./servidor.sh 8081"
    exit 1
fi

# Validar rango permitido.
if [ "$PORT" -lt 8080 ] || [ "$PORT" -gt 8084 ]; then
    echo "❌ Puerto inválido: $PORT. Debe estar entre 8080 y 8084."
    exit 1
fi

COMANDO_SERVIDOR="cargo run --features log_funcionamiento --bin servidor $PORT"

# Verificar que la terminal esté disponible.
if ! command -v $TERMINAL &> /dev/null; then
    echo "❌ Error: $TERMINAL no está instalado."
    exit 1
fi

# Ejecutar el servidor.
$TERMINAL --title="Servidor_$PORT" -- bash -c "$COMANDO_SERVIDOR; exec bash"
