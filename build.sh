#!/usr/bin/env bash
# Упаковывает исходники в WorldEdit-мод:
# worldedit-horizon/target/worldedit_horizon-<version>.hnmod
# Сборку пакета делает tools/package.py — там же собирается манифест.
set -euo pipefail
cd "$(dirname "$0")/worldedit-horizon"

python3 tools/package.py
