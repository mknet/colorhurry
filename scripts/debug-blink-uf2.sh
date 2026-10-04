#!/usr/bin/env bash
# Endlos-Blink-Test (pre_init only) als UF2 bauen + Host-Log

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
layout="${CPB_MEMORY:-v7}"
features="sd-v7,diag-preinit-only"
[[ "$layout" == "v6" ]] && features="sd-v6,diag-preinit-only"

elf="$root/target/thumbv7em-none-eabihf/release/color-hurry-cpb"
uf2="$root/target/thumbv7em-none-eabihf/release/color-hurry-cpb-blink-${layout}.uf2"

echo "→ Layout: $layout (Endlos-D13-Blink in pre_init)"
CPB_MEMORY="$layout" "$root/scripts/debug-host-log.sh" "blink-pre"

(cd "$root" && cargo build --release -p color-hurry-cpb --target thumbv7em-none-eabihf --no-default-features --features "$features")

CPB_MEMORY="$layout" "$root/scripts/debug-host-log.sh" "blink-post-build"

elf2flash convert --board circuit_playground_bluefruit "$elf" "$uf2"

python3 - "$uf2" <<'PY'
import struct, sys, json, time
p = sys.argv[1]
d = open(p, "rb").read()
_, target = struct.unpack_from("<II", d, 8)
log = "/Users/marcel/Development/mk/embedded/adafruit/.cursor/debug-55c9bc.log"
entry = {
    "sessionId": "55c9bc",
    "timestamp": int(time.time() * 1000),
    "hypothesisId": "H11",
    "location": "debug-blink-uf2.sh",
    "message": "uf2_target",
    "data": {"uf2": p, "block0_target": hex(target)},
    "runId": "blink-uf2",
}
with open(log, "a") as f:
    f.write(json.dumps(entry) + "\n")
print(f"→ UF2 block0 @ {hex(target)}")
PY

echo "→ UF2: $uf2"
open -R "$uf2" 2>/dev/null || true
open /Volumes/CPLAYBTBOOT 2>/dev/null || echo "→ Reset 2× (NeoPixels grün), dann UF2 auf CPLAYBTBOOT ziehen"
echo "→ Erwartung nach Reset 1×: D13 blinkt DAUERHAFT (~2 Hz)"
echo "→ Wenn nicht: SoftDevice/Bootloader reparieren (CircuitPython UF2)"
