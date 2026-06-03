#!/usr/bin/env bash
# Cargo runner: ELF → UF2 → Boot-Laufwerk kopieren

set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
elf="${1:?}"
uf2="${elf}.uf2"

elf2flash convert --board circuit_playground_bluefruit "$elf" "$uf2"
exec "$root/scripts/copy-uf2.sh" "$uf2"
