#!/usr/bin/env bash
# Flash without a UF2 drive — via USB serial (adafruit-nrfutil)
# Default: S140 6.1.1 @ 0x26000 (--features sd-v6)
# Alternative: just flash-serial-v7  →  S140 7.x @ 0x27000
#
# On "Device not configured" (macOS): just flash-serial-dfu
# (put the board into DFU manually, then flash without --touch)

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
    # 0xFFFE = Adafruit default "any SoftDevice" — more reliable on CPB than 0x0101/0x0123
    sd_req="0xFFFE"
    flash_base="0x26000"
    layout="v6"
    features="sd-v6"
    ;;
esac

dfu_manual="${DFU_MANUAL:-0}"

if ! command -v adafruit-nrfutil &>/dev/null; then
    echo "adafruit-nrfutil is missing. Install it:" >&2
    echo "  pip3 install adafruit-nrfutil" >&2
    exit 1
fi

objcopy=""
for c in arm-none-eabi-objcopy llvm-objcopy; do
    command -v "$c" &>/dev/null && objcopy="$c" && break
done
if [[ -z "$objcopy" ]]; then
    echo "objcopy is missing (arm-none-eabi-objcopy or llvm-objcopy)." >&2
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
    echo "→ Touch $port @ 1200 baud (wakes the serial bootloader)"
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
        echo "→ (pyserial not found — use just flash-serial-dfu if needed)"
    fi
    echo "→ Waiting 5 s for USB reconnect …"
    sleep 5
}

# adafruit-nrfutil often returns exit code 0 on errors — check the output!
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

echo "→ Layout: $layout (app @ $flash_base, --sd-req $sd_req, --features $features)"

stamp="$root/target/.cpb-memory-layout"
if [[ -f "$stamp" ]] && [[ "$(cat "$stamp")" != "$layout" ]]; then
    echo "→ Different layout than last time — cargo clean"
    (cd "$root" && cargo clean)
fi
mkdir -p "$root/target"
echo "$layout" > "$stamp"

echo "→ cargo build --release -p color-hurry-cpb --target thumbv7em-none-eabihf --features $features"
(cd "$root" && cargo build --release -p color-hurry-cpb --target thumbv7em-none-eabihf --features "$features")

echo "→ ELF sections (vector table should start at $flash_base):"
arm-none-eabi-objdump -h "$elf" 2>/dev/null | grep -E "vector|Idx" || llvm-objdump -h "$elf" | grep -E "vector|Idx"

echo "→ ELF → Intel-Hex"
"$objcopy" -O ihex "$elf" "$hex"

echo "→ DFU package (--sd-req $sd_req)"
adafruit-nrfutil dfu genpkg \
    --dev-type 0x0052 \
    --sd-req "$sd_req" \
    --application "$hex" \
    --application-version 1 \
    "$zip"

echo ""
echo "=== Available USB serial ports ==="
ls /dev/cu.usb* /dev/cu.usbmodem* 2>/dev/null || ls /dev/cu.* 2>/dev/null | head -10 || true

if [[ "$dfu_manual" == "1" ]]; then
    echo ""
    echo "=== Manual DFU mode ==="
    echo "  Now: HOLD Button A (left) → Reset 1× → release A"
    echo "  You have 8 seconds …"
    sleep 8
fi

port="$(wait_for_port || true)"
if [[ -z "$port" ]]; then
    echo "" >&2
    echo "No port found. Connect the board via USB, then:" >&2
    echo "  SERIAL_PORT=/dev/cu.usbmodemXXXX just flash-serial" >&2
    echo "  or: just flash-serial-dfu" >&2
    exit 1
fi

echo ""
echo "→ Port: $port"

flashed=0
max_attempts=3

for attempt in $(seq 1 "$max_attempts"); do
    echo ""
    echo "=== Flash attempt $attempt/$max_attempts ==="

    port="$(wait_for_port || true)"
    if [[ -z "$port" ]]; then
        echo "→ No port — unplug/replug USB, Reset 1× if needed"
        sleep 3
        continue
    fi
    echo "→ Port: $port"

    if [[ "$dfu_manual" == "1" ]]; then
        echo "→ Flashing (board should be in DFU, no touch)"
        if do_flash "$port" 0 "$zip"; then
            flashed=1
            break
        fi
    elif [[ "$attempt" -eq 1 ]]; then
        echo "→ Touch, then flash without --touch"
        touch_dfu "$port" || true
        port="$(wait_for_port || true)"
        if [[ -n "$port" ]] && do_flash "$port" 0 "$zip"; then
            flashed=1
            break
        fi
    else
        echo "→ Retrying (board may need A+Reset for DFU)"
        if do_flash "$port" 0 "$zip"; then
            flashed=1
            break
        fi
    fi

    echo "→ Attempt failed"
    if [[ "$attempt" -lt "$max_attempts" && "$dfu_manual" != "1" ]]; then
        echo "→ Tip: hold Button A → Reset 1× → release A, then try again"
    fi
    sleep 3
done

echo ""
if [[ "$flashed" != "1" ]]; then
    echo "✗ Flash failed." >&2
    echo "" >&2
    echo "Most reliable on macOS:" >&2
    echo "  1. Hold Button A (left) → Reset 1× → release A" >&2
    echo "  2. just flash-serial-dfu" >&2
    echo "" >&2
    echo "Alternatives:" >&2
    echo "  just flash-serial-v7     (S140 7.x / flash @ 0x27000)" >&2
    echo "  just flash-finder        (Reset 2× → UF2 onto CPLAYBTBOOT)" >&2
    exit 1
fi

echo "✓ Flash OK (Device programmed)."
echo "→ Briefly unplug USB, plug it back in, Reset 1× (not 2×)."
echo "→ Expected: NeoPixels + D13 blink in sync (400 ms)."
echo ""
if [[ "$layout" == "v6" ]]; then
    echo "→ Nothing blinking? Very often a wrong flash address — then:"
    echo "     just flash-serial-dfu-v7"
    echo "  (app @ 0x27000 instead of 0x26000)"
fi
