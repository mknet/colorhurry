#!/usr/bin/env bash
# SoftDevice S140 6.1.1 → 7.0.1 (Serial-DFU, Adafruit CPB)
#
# nrf-softdevice (Rust) bindet gegen S140 7.0.1. Adafruit-Restore liefert 6.1.1.
# Der Serial-Bootloader akzeptiert nur „softdevice_bootloader“-Pakete (wie restore),
# kein reines SoftDevice-Paket.
#
# Danach: just flash-v7  (App @ 0x27000)

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
bl_cache="$root/tools/bootloader"
sd_cache="$root/tools/softdevice"

bl_zip_name="circuitplayground_nrf52840_bootloader-0.11.0_s140_6.1.1.zip"
bl_zip_url="https://github.com/adafruit/Adafruit_nRF52_Bootloader/releases/download/0.11.0/$bl_zip_name"
bl_zip_path="$bl_cache/$bl_zip_name"

sd_hex_name="s140_nrf52_7.0.1_softdevice.hex"
sd_hex_url="https://github.com/NordicSemiconductor/nRF5-SDK-for-Mesh/raw/refs/heads/master/bin/softdevice/s140_nrf52_7.0.1_softdevice.hex"
sd_hex_path="$sd_cache/$sd_hex_name"

bl_bin_path="$bl_cache/cpb_bootloader_0.11.0.bin"
bl_hex_path="$bl_cache/cpb_bootloader_0.11.0.hex"
zip_path="$sd_cache/s140_v7_dfu.zip"
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
    for i in $(seq 1 60); do
        port="$(pick_port)"
        if [[ -n "$port" && -e "$port" ]]; then
            echo "$port"
            return 0
        fi
        sleep 0.5
    done
    return 1
}

do_flash() {
    local port=$1
    local log
    log="$(mktemp "${TMPDIR:-/tmp}/cpb-sd-v7.XXXX")"
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
    # macOS: USB bricht oft ab, obwohl der Flash danach OK ist
    if grep -q "Device not configured" "$log"; then
        echo "→ USB-Reconnect nach DFU (macOS) — prüfe gleich erneut …" >&2
        sleep 5
        if wait_for_port >/dev/null; then
            rm -f "$log"
            return 0
        fi
    fi
    rm -f "$log"
    return 1
}

extract_bootloader_hex() {
    python3 <<PY
import json, zipfile, struct
from pathlib import Path

bl_zip = Path("$bl_zip_path")
bl_bin = Path("$bl_bin_path")
bl_hex = Path("$bl_hex_path")

with zipfile.ZipFile(bl_zip) as zf:
    manifest = json.loads(zf.read("manifest.json"))
    info = manifest["manifest"]["softdevice_bootloader"]
    sd_size = info["sd_size"]
    bl_size = info["bl_size"]
    sd_bl = zf.read("sd_bl.bin")

bootloader = sd_bl[sd_size : sd_size + bl_size]
bl_bin.write_bytes(bootloader)

# Intel HEX @ 0xF4000 (Adafruit nRF52840 Bootloader)
origin = 0xF4000
lines = []
chunk = 16
for off in range(0, len(bootloader), chunk):
    piece = bootloader[off : off + chunk]
    addr = origin + off
    rec = bytes([len(piece), (addr >> 8) & 0xFF, addr & 0xFF, 0x00]) + piece
    cs = (-sum(rec)) & 0xFF
    lines.append(":" + rec.hex().upper() + f"{cs:02X}")
lines.append(":00000001FF")
bl_hex.write_text("\n".join(lines) + "\n")
print(f"Bootloader extrahiert: {len(bootloader)} Bytes → {bl_hex}")
PY
}

if ! command -v adafruit-nrfutil &>/dev/null; then
    echo "adafruit-nrfutil fehlt: pip3 install adafruit-nrfutil" >&2
    exit 1
fi

objcopy=""
for c in arm-none-eabi-objcopy llvm-objcopy; do
    command -v "$c" &>/dev/null && objcopy="$c" && break
done

mkdir -p "$bl_cache" "$sd_cache"

if [[ ! -f "$bl_zip_path" ]]; then
    echo "→ Lade Adafruit-Bootloader-Paket (zum Extrahieren des Bootloaders) …"
    curl -fsSL -o "$bl_zip_path" "$bl_zip_url"
fi

if [[ ! -f "$sd_hex_path" ]]; then
    echo "→ Lade $sd_hex_name …"
    curl -fsSL -o "$sd_hex_path" "$sd_hex_url"
fi

echo "→ Bootloader aus Adafruit-Zip extrahieren (0xF4000)"
extract_bootloader_hex

echo "→ DFU-Paket: S140 7.0.1 + Bootloader (softdevice_bootloader, wie restore)"
adafruit-nrfutil dfu genpkg \
    --dev-type 0x0052 \
    --dev-revision 52840 \
    --sd-req 0xB7,0xFFFE \
    --bootloader "$bl_hex_path" \
    --softdevice "$sd_hex_path" \
    "$zip_path"

echo ""
echo "=== SoftDevice S140 7.0.1 + Bootloader flashen ==="
echo "  Paket: $(basename "$zip_path") (SD 7.0.1, ~30 s)"
echo "  USB während DFU NICHT abziehen!"
echo ""
if [[ "$dfu_manual" == "1" ]]; then
    echo "  Jetzt: Button A halten → Reset 1× → A loslassen (DFU-Modus)"
    echo "  Du hast 12 Sekunden …"
    sleep 12
fi

flashed=0
for attempt in $(seq 1 3); do
    echo ""
    echo "=== Upgrade-Versuch $attempt/3 ==="
    port="$(wait_for_port || true)"
    if [[ -z "$port" ]]; then
        echo "→ Kein Port — USB prüfen, ggf. A+Reset für DFU"
        sleep 3
        continue
    fi
    echo "→ Port: $port"
    if do_flash "$port"; then
        flashed=1
        break
    fi
    echo "→ Fehlgeschlagen — erneut A+Reset für DFU, dann nächster Versuch"
    sleep 4
done

if [[ "$flashed" != "1" ]]; then
    echo "" >&2
    echo "✗ SoftDevice-Upgrade fehlgeschlagen." >&2
    echo "" >&2
    echo "Typisch auf macOS: USB trennt sich kurz (Device not configured)." >&2
    echo "Manuell:" >&2
    echo "  1. Button A halten → Reset 1× → A loslassen" >&2
    echo "  2. DFU_MANUAL=1 ./scripts/upgrade-softdevice-v7.sh" >&2
    echo "" >&2
    echo "Alternativ mit SWD-Debugger: docs/BLE.md" >&2
    exit 1
fi

echo ""
echo "✓ SoftDevice S140 7.0.1 installiert."
echo "→ USB abziehen, 5 s warten, wieder einstecken, Reset 1×."
echo "→ App flashen:  just flash-v7"
