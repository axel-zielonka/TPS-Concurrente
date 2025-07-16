#!/bin/bash

# Lanza una instancia de repartidor en una nueva terminal.
gnome-terminal --title="Repartidor" -- bash -c "cargo run --bin repartidor; exec bash"
