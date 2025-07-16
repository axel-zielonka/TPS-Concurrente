#!/bin/bash

# Lanza una instancia de restaurante en una nueva terminal.
gnome-terminal --title="Restaurante" -- bash -c "cargo run --bin restaurante; exec bash"
