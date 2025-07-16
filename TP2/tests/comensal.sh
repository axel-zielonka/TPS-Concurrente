#!/bin/bash

# Lanza una instancia de comensal en una nueva terminal.
gnome-terminal --title="Comensal" -- bash -c "cargo run --bin comensal; exec bash"
