#!/usr/bin/env bash
# Упаковывает исходники в WorldEdit-мод: dist/worldedit-<version>.hnmod
set -euo pipefail
cd "$(dirname "$0")"

VERSION=$(grep -m1 '"version"' horizon-native.json | sed -E 's/.*"version": *"([^"]+)".*/\1/')
OUT="dist/worldedit-${VERSION}.hnmod"

mkdir -p dist
rm -f "$OUT"
zip -X -r "$OUT" horizon-native.json mixins
echo "Готово: $OUT"
