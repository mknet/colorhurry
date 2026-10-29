#!/usr/bin/env bash
# SoftDevice S140 6.1.1 → 7.0.1 (serial DFU, Adafruit CPB)
#
# nrf-softdevice (Rust) binds against S140 7.0.1. The Adafruit restore ships 6.1.1.
# The serial bootloader only accepts "softdevice_bootloader" packages (like restore),
# not a SoftDevice-only package.
#
# Afterwards: just flash-v7  (app @ 0x27000)

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
    # macOS: USB often drops even though the flash is OK afterwards
    if grep -q "Device not configured" "$log"; then
        echo "→ USB reconnect after DFU (macOS) — checking again shortly …" >&2
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
print(f"Bootloader extracted: {len(bootloader)} Bytes → {bl_hex}")
PY
}

if ! command -v adafruit-nrfutil &>/dev/null; then
    echo "adafruit-nrfutil is missing: pip3 install adafruit-nrfutil" >&2
    exit 1
fi

objcopy=""
for c in arm-none-eabi-objcopy llvm-objcopy; do
    command -v "$c" &>/dev/null && objcopy="$c" && break
done

mkdir -p "$bl_cache" "$sd_cache"

if [[ ! -f "$bl_zip_path" ]]; then
    echo "→ Downloading Adafruit bootloader package (to extract the bootloader) …"
    curl -fsSL -o "$bl_zip_path" "$bl_zip_url"
fi

if [[ ! -f "$sd_hex_path" ]]; then
    echo "→ Downloading $sd_hex_name …"
    curl -fsSL -o "$sd_hex_path" "$sd_hex_url"
fi

echo "→ Extracting bootloader from Adafruit zip (0xF4000)"
extract_bootloader_hex

echo "→ DFU package: S140 7.0.1 + bootloader (softdevice_bootloader, like restore)"
adafruit-nrfutil dfu genpkg \
    --dev-type 0x0052 \
    --dev-revision 52840 \
    --sd-req 0xB7,0xFFFE \
    --bootloader "$bl_hex_path" \
    --softdevice "$sd_hex_path" \
    "$zip_path"

echo ""
echo "=== Flashing SoftDevice S140 7.0.1 + bootloader ==="
echo "  Package: $(basename "$zip_path") (SD 7.0.1, ~30 s)"
echo "  Do NOT unplug USB during DFU!"
echo ""
if [[ "$dfu_manual" == "1" ]]; then
    echo "  Now: hold Button A → Reset 1× → release A (DFU mode)"
    echo "  You have 12 seconds …"
    sleep 12
fi

flashed=0
for attempt in $(seq 1 3); do
    echo ""
    echo "=== Upgrade attempt $attempt/3 ==="
    port="$(wait_for_port || true)"
    if [[ -z "$port" ]]; then
        echo "→ No port — check USB, A+Reset for DFU if needed"
        sleep 3
        continue
    fi
    echo "→ Port: $port"
    if do_flash "$port"; then
        flashed=1
        break
    fi
    echo "→ Failed — A+Reset for DFU again, then next attempt"
    sleep 4
done

if [[ "$flashed" != "1" ]]; then
    echo "" >&2
    echo "✗ SoftDevice upgrade failed." >&2
    echo "" >&2
    echo "Typical on macOS: USB disconnects briefly (Device not configured)." >&2
    echo "Manually:" >&2
    echo "  1. Hold Button A → Reset 1× → release A" >&2
    echo "  2. DFU_MANUAL=1 ./scripts/upgrade-softdevice-v7.sh" >&2
    echo "" >&2
    echo "Alternatively with an SWD debugger: docs/BLE.md" >&2
    exit 1
fi

echo ""
echo "✓ SoftDevice S140 7.0.1 installed."
echo "→ Unplug USB, wait 5 s, plug it back in, Reset 1×."
echo "→ Flash the app:  just flash-v7"
