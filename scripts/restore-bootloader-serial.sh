#!/usr/bin/env bash
# Bootloader + SoftDevice S140 6.1.1 per Serial-DFU wiederherstellen (ohne UF2/CPLAYBTBOOT)
#
# Adafruit CPB: circuitplayground_nrf52840_bootloader @ 0.11.0, S140 6.1.1
# Danach App mit: just flash  (v6 @ 0x26000)

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
    echo "adafruit-nrfutil fehlt: pip3 install adafruit-nrfutil" >&2
    exit 1
fi

mkdir -p "$cache"
if [[ ! -f "$zip_path" ]]; then
    echo "→ Lade Bootloader-Paket …"
    curl -fsSL -o "$zip_path" "$zip_url"
fi

echo ""
echo "=== Bootloader + SoftDevice wiederherstellen ==="
echo "  Paket: $zip_name"
echo "  Dauer: ~30 s — USB NICHT abziehen!"
echo ""
if [[ "$dfu_manual" == "1" ]]; then
    echo "  Jetzt: Button A halten → Reset 1× → A loslassen (DFU-Modus)"
    echo "  Du hast 8 Sekunden …"
    sleep 8
fi

port="$(wait_for_port || true)"
if [[ -z "$port" ]]; then
    echo "Kein Serial-Port. Board verbinden oder SERIAL_PORT setzen." >&2
    exit 1
fi

echo "→ Port: $port"
restored=0
for attempt in $(seq 1 3); do
    echo ""
    echo "=== Restore-Versuch $attempt/3 ==="
    port="$(wait_for_port || true)"
    [[ -z "$port" ]] && { sleep 3; continue; }
    echo "→ Port: $port"
    if do_restore "$port"; then
        restored=1
        break
    fi
    echo "→ Fehlgeschlagen — ggf. A+Reset für DFU, dann erneut"
    sleep 3
done

if [[ "$restored" != "1" ]]; then
    echo "✗ Bootloader-Restore fehlgeschlagen." >&2
    exit 1
fi

echo ""
echo "✓ Bootloader + SoftDevice wiederhergestellt."
echo "→ USB abziehen, 5 s warten, wieder einstecken, Reset 1×."
echo "→ Dann Rust-App flashen:  just flash"
