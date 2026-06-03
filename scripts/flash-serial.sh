#!/usr/bin/env bash
# Flashen ohne UF2-Laufwerk — per USB-Serial (adafruit-nrfutil)
# Standard: S140 6.1.1 @ 0x26000 (--features sd-v6)
# Alternative: just flash-serial-v7  →  S140 7.x @ 0x27000
#
# Bei „Device not configured“ (macOS): just flash-serial-dfu
# (Board manuell in DFU, dann ohne --touch flashen)

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
layout="${CPB_MEMORY:-v6}"
elf="$root/target/thumbv7em-none-eabihf/release/color-hurry-cpb"
hex="$root/target/thumbv7em-none-eabihf/release/color-hurry-cpb.hex"
zip="$root/target/thumbv7em-none-eabihf/release/color-hurry-cpb-dfu-${layout}.zip"

case "$layout" in
  v7)
    sd_req="0xFFFE"
    flash_base="0x27000"
    features="sd-v7"
    ;;
  v6|*)
    # 0xFFFE = Adafruit-Standard „any SoftDevice“ — auf CPB zuverlässiger als 0x0101/0x0123
    sd_req="0xFFFE"
    flash_base="0x26000"
    layout="v6"
    features="sd-v6"
    ;;
esac

dfu_manual="${DFU_MANUAL:-0}"

if ! command -v adafruit-nrfutil &>/dev/null; then
    echo "adafruit-nrfutil fehlt. Installieren:" >&2
    echo "  pip3 install adafruit-nrfutil" >&2
    exit 1
fi

objcopy=""
for c in arm-none-eabi-objcopy llvm-objcopy; do
    command -v "$c" &>/dev/null && objcopy="$c" && break
done
if [[ -z "$objcopy" ]]; then
    echo "objcopy fehlt (arm-none-eabi-objcopy oder llvm-objcopy)." >&2
    exit 1
fi

pick_port() {
    if [[ -n "${SERIAL_PORT:-}" ]]; then
        echo "$SERIAL_PORT"
        return
    fi
    ls /dev/cu.usbmodem* 2>/dev/null | head -1 \
        || ls /dev/cu.usb* 2>/dev/null | grep -vi bluetooth | head -1 \
        || true
}

wait_for_port() {
    local i port
    for i in $(seq 1 40); do
        port="$(pick_port)"
        if [[ -n "$port" && -e "$port" ]]; then
            echo "$port"
            return 0
        fi
        sleep 0.5
    done
    return 1
}

touch_dfu() {
    local port=$1
    echo "→ Touch $port @ 1200 Baud (weckt Serial-Bootloader)"
    local py=""
    for candidate in \
        "${ADAFRUIT_NRFUTIL_PYTHON:-}" \
        "${HOME}/.local/pipx/venvs/adafruit-nrfutil/bin/python" \
        python3; do
        [[ -z "$candidate" || ! -x "$candidate" ]] && continue
        if "$candidate" -c "import serial" 2>/dev/null; then
            py=$candidate
            break
        fi
    done
    if [[ -n "$py" ]]; then
        "$py" - "$port" <<'PY'
import sys, time
import serial
s = serial.Serial(sys.argv[1], 1200, timeout=0.5)
time.sleep(0.3)
s.close()
PY
    else
        echo "→ (pyserial nicht gefunden — nutze ggf. just flash-serial-dfu)"
    fi
    echo "→ Warte 5 s auf USB-Reconnect …"
    sleep 5
}

# adafruit-nrfutil gibt bei Fehlern oft Exit-Code 0 — Ausgabe prüfen!
do_flash() {
    local port=$1
    local use_touch=${2:-0}
    local pkg=$3
    local log
    log="$(mktemp "${TMPDIR:-/tmp}/cpb-dfu.XXXX")"

    local touch_flag=()
    if [[ "$use_touch" == "1" ]]; then
        touch_flag=(--touch 1200)
    fi

    set +e
    adafruit-nrfutil --verbose dfu serial \
        --package "$pkg" \
        -p "$port" \
        -b 115200 \
        --singlebank \
        "${touch_flag[@]}" 2>&1 | tee "$log"
    set -e

    if grep -q "Device programmed\." "$log"; then
        rm -f "$log"
        return 0
    fi
    rm -f "$log"
    return 1
}

echo "→ Layout: $layout (App @ $flash_base, --sd-req $sd_req, --features $features)"

stamp="$root/target/.cpb-memory-layout"
if [[ -f "$stamp" ]] && [[ "$(cat "$stamp")" != "$layout" ]]; then
    echo "→ Anderes Layout als letztes Mal — cargo clean"
    (cd "$root" && cargo clean)
fi
mkdir -p "$root/target"
echo "$layout" > "$stamp"

echo "→ cargo build --release --features $features"
(cd "$root" && cargo build --release --features "$features")

echo "→ ELF-Sektionen (Vector-Tabelle soll bei $flash_base beginnen):"
arm-none-eabi-objdump -h "$elf" 2>/dev/null | grep -E "vector|Idx" || llvm-objdump -h "$elf" | grep -E "vector|Idx"

echo "→ ELF → Intel-Hex"
"$objcopy" -O ihex "$elf" "$hex"

echo "→ DFU-Paket (--sd-req $sd_req)"
adafruit-nrfutil dfu genpkg \
    --dev-type 0x0052 \
    --sd-req "$sd_req" \
    --application "$hex" \
    --application-version 1 \
    "$zip"

echo ""
echo "=== Verfügbare USB-Serial-Ports ==="
ls /dev/cu.usb* /dev/cu.usbmodem* 2>/dev/null || ls /dev/cu.* 2>/dev/null | head -10 || true

if [[ "$dfu_manual" == "1" ]]; then
    echo ""
    echo "=== Manueller DFU-Modus ==="
    echo "  Jetzt: Button A (links) HALTEN → Reset 1× → A loslassen"
    echo "  Du hast 8 Sekunden …"
    sleep 8
fi

port="$(wait_for_port || true)"
if [[ -z "$port" ]]; then
    echo "" >&2
    echo "Kein Port gefunden. Board per USB verbinden, dann:" >&2
    echo "  SERIAL_PORT=/dev/cu.usbmodemXXXX just flash-serial" >&2
    echo "  oder: just flash-serial-dfu" >&2
    exit 1
fi

echo ""
echo "→ Port: $port"

flashed=0
max_attempts=3

for attempt in $(seq 1 "$max_attempts"); do
    echo ""
    echo "=== Flash-Versuch $attempt/$max_attempts ==="

    port="$(wait_for_port || true)"
    if [[ -z "$port" ]]; then
        echo "→ Kein Port — USB ab/zu, ggf. Reset 1×"
        sleep 3
        continue
    fi
    echo "→ Port: $port"

    if [[ "$dfu_manual" == "1" ]]; then
        echo "→ Flashe (Board soll in DFU sein, kein Touch)"
        if do_flash "$port" 0 "$zip"; then
            flashed=1
            break
        fi
    elif [[ "$attempt" -eq 1 ]]; then
        echo "→ Touch, dann Flash ohne --touch"
        touch_dfu "$port" || true
        port="$(wait_for_port || true)"
        if [[ -n "$port" ]] && do_flash "$port" 0 "$zip"; then
            flashed=1
            break
        fi
    else
        echo "→ Erneuter Versuch (Board ggf. A+Reset für DFU)"
        if do_flash "$port" 0 "$zip"; then
            flashed=1
            break
        fi
    fi

    echo "→ Versuch fehlgeschlagen"
    if [[ "$attempt" -lt "$max_attempts" && "$dfu_manual" != "1" ]]; then
        echo "→ Tipp: Button A halten → Reset 1× → A loslassen, dann nochmal"
    fi
    sleep 3
done

echo ""
if [[ "$flashed" != "1" ]]; then
    echo "✗ Flash fehlgeschlagen." >&2
    echo "" >&2
    echo "Am zuverlässigsten auf macOS:" >&2
    echo "  1. Button A (links) halten → Reset 1× → A loslassen" >&2
    echo "  2. just flash-serial-dfu" >&2
    echo "" >&2
    echo "Alternativen:" >&2
    echo "  just flash-serial-v7     (S140 7.x / Flash @ 0x27000)" >&2
    echo "  just flash-finder        (Reset 2× → UF2 auf CPLAYBTBOOT)" >&2
    exit 1
fi

echo "✓ Flash OK (Device programmed)."
echo "→ USB kurz abziehen, wieder an, Reset 1× (nicht 2×)."
echo "→ Erwartung: NeoPixels + D13 blinken synchron (400 ms)."
echo ""
if [[ "$layout" == "v6" ]]; then
    echo "→ Blinkt nichts? Sehr oft falsche Flash-Adresse — dann:"
    echo "     just flash-serial-dfu-v7"
    echo "  (App @ 0x27000 statt 0x26000)"
fi
