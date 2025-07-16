#!/bin/bash

# Uso: ./comparar_json.sh archivo1.json archivo2.json

if [ "$#" -ne 2 ]; then
  echo "Uso: $0 archivo1.json archivo2.json"
  exit 1
fi

ARCHIVO1="$1"
ARCHIVO2="$2"

if cmp -s "$ARCHIVO1" "$ARCHIVO2"; then
  echo "✅ Los archivos JSON son estrictamente iguales."
else
  echo "❌ Los archivos JSON son diferentes. Diferencias:"
  echo "---------------------------------------------"
  diff -u "$ARCHIVO1" "$ARCHIVO2"
  echo "---------------------------------------------"
fi