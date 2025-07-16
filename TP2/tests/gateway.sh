#!/bin/bash

# Este script lanza una instancia del gateway en una terminal separada.
TERMINAL="gnome-terminal"
COMANDO_GATEWAY="cargo run --bin gateway"

# Ejecutar el gateway.
$TERMINAL --title="Gateway" -- bash -c "$COMANDO_GATEWAY; exec bash"
