#!/usr/bin/env bash
# Restore bootloader + SoftDevice S140 6.1.1 via serial DFU (without UF2/CPLAYBTBOOT)
#
# Adafruit CPB: circuitplayground_nrf52840_bootloader @ 0.11.0, S140 6.1.1
# Afterwards flash the app with: just flash  (v6 @ 0x26000)

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cache="$root/tools/bootloader"
zip_name="circuitplayground_nrf52840_bootloader-0.11.0_s140_6.1.1.zip"
zip_url="https://github.com/adafruit/Adafruit_nRF52_Bootloader/releases/download/0.11.0/$zip_name"
zip_path="$cache/$zip_name"
dfu_manual="${DFU_MANUAL:-1}"

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

do_restore() {
    local port=$1
    local log
    log="$(mktemp "${TMPDIR:-/tmp}/cpb-restore.XXXX")"
    set +e
    adafruit-nrfutil --verbose dfu serial \
        --package "$zip_path" \
        -p "$port" \
        -b 115200 \
        --singlebank 2>&1 | tee "$log"
    set -e
    if grep -q "Device programmed\." "$log"; then
        rm -f "$log"
        return 0
    fi
    rm -f "$log"
    return 1
}

if ! command -v adafruit-nrfutil &>/dev/null; then
    echo "adafruit-nrfutil is missing: pip3 install adafruit-nrfutil" >&2
    exit 1
fi

mkdir -p "$cache"
if [[ ! -f "$zip_path" ]]; then
    echo "→ Downloading bootloader package …"
    curl -fsSL -o "$zip_path" "$zip_url"
fi

echo ""
echo "=== Restoring bootloader + SoftDevice ==="
echo "  Package: $zip_name"
echo "  Duration: ~30 s, do NOT unplug USB!"
echo ""
if [[ "$dfu_manual" == "1" ]]; then
    echo "  Now: hold Button A → Reset 1× → release A (DFU mode)"
    echo "  You have 8 seconds …"
    sleep 8
fi

port="$(wait_for_port || true)"
if [[ -z "$port" ]]; then
    echo "No serial port. Connect the board or set SERIAL_PORT." >&2
    exit 1
fi

echo "→ Port: $port"
restored=0
for attempt in $(seq 1 3); do
    echo ""
    echo "=== Restore attempt $attempt/3 ==="
    port="$(wait_for_port || true)"
    [[ -z "$port" ]] && { sleep 3; continue; }
    echo "→ Port: $port"
    if do_restore "$port"; then
        restored=1
        break
    fi
    echo "→ Failed: A+Reset for DFU if needed, then retry"
    sleep 3
done

if [[ "$restored" != "1" ]]; then
    echo "✗ Bootloader restore failed." >&2
    exit 1
fi

echo ""
echo "✓ Bootloader + SoftDevice restored."
echo "→ Unplug USB, wait 5 s, plug it back in, Reset 1×."
echo "→ Then flash the Rust app:  just flash"
