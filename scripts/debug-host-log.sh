#!/usr/bin/env bash
# Host-seitige Diagnose → NDJSON-Log (Hypothesen H1/H2/H5)

set -euo pipefail

LOG="/Users/marcel/Development/mk/embedded/adafruit/.cursor/debug-55c9bc.log"
SESSION="55c9bc"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PHASE="${1:-snapshot}"
LAYOUT="${CPB_MEMORY:-v6}"
ELF="$ROOT/target/thumbv7em-none-eabihf/release/color-hurry-cpb"
HEX="$ROOT/target/thumbv7em-none-eabihf/release/color-hurry-cpb.hex"
TS=$(($(date +%s) * 1000))

log() {
    local hid=$1
    local msg=$2
    local data=$3
    printf '{"sessionId":"%s","timestamp":%s,"hypothesisId":"%s","location":"debug-host-log.sh","message":"%s","data":%s,"runId":"%s"}\n' \
        "$SESSION" "$TS" "$hid" "$msg" "$data" "$PHASE" >> "$LOG"
}

mkdir -p "$(dirname "$LOG")"

# H1/H5: Link-Adresse vs. erwartete App-Basis
if [[ -f "$ELF" ]]; then
    vt=$(arm-none-eabi-readelf -l "$ELF" 2>/dev/null | awk '/LOAD/{print $3; exit}' || echo "missing")
    entry=$(arm-none-eabi-readelf -h "$ELF" 2>/dev/null | awk '/Entry/{print $4}' || echo "missing")
    expected="0x26000"
    [[ "$LAYOUT" == "v7" ]] && expected="0x27000"
    match="false"
    if [[ "$vt" != "missing" ]]; then
        [[ "$((vt))" -eq "$((expected))" ]] && match="true"
    fi
    log "H1" "elf_load_address" "{\"layout\":\"$LAYOUT\",\"vector_vma\":\"$vt\",\"entry\":\"$entry\",\"expected\":\"$expected\",\"match\":$match}"
    if [[ -f "$ELF" ]]; then
        tmp_sp=$(mktemp)
        arm-none-eabi-objcopy -O binary -j .vector_table "$ELF" "$tmp_sp" 2>/dev/null || true
        sp_hex=$(xxd -p "$tmp_sp" 2>/dev/null | head -c8 || echo "missing")
        rm -f "$tmp_sp"
        sp_norm="missing"
        if [[ "$sp_hex" != "missing" && ${#sp_hex} -ge 8 ]]; then
            b0=${sp_hex:6:2}; b1=${sp_hex:4:2}; b2=${sp_hex:2:2}; b3=${sp_hex:0:2}
            sp_norm=$(printf "0x%08x" $((0x${b0}${b1}${b2}${b3})))
        fi
        stack_ok="false"
        [[ "$sp_norm" == "0x20040000" ]] && stack_ok="true"
        log "H7" "vector_initial_sp" "{\"initial_sp\":\"$sp_norm\",\"expected_sp\":\"0x20040000\",\"stack_ok\":$stack_ok}"
    fi
else
    log "H1" "elf_missing" "{\"layout\":\"$LAYOUT\",\"path\":\"$ELF\"}"
fi

# H5: gleiche Basis wie ELF (Intel-Hex-Parser war unzuverlässig)
if [[ -f "$ELF" && "$vt" != "missing" ]]; then
    log "H5" "flash_base_from_elf" "{\"layout\":\"$LAYOUT\",\"flash_base\":\"$vt\"}"
fi

# H2: Bootloader / USB-Zustand
boot=""
if [[ -d /Volumes/CPLAYBTBOOT ]]; then
    boot="CPLAYBTBOOT_mounted"
    info=""
    if [[ -r /Volumes/CPLAYBTBOOT/INFO_UF2.TXT ]]; then
        info=$(tr '\n' '|' < /Volumes/CPLAYBTBOOT/INFO_UF2.TXT | head -c 400)
    fi
    log "H2" "bootloader_volume" "{\"state\":\"$boot\",\"info_uf2\":\"$info\"}"
else
    boot="not_mounted"
    ports=$(ls /dev/cu.usbmodem* 2>/dev/null | tr '\n' ',' || true)
    log "H2" "runtime_usb" "{\"bootloader\":\"$boot\",\"serial_ports\":\"$ports\"}"
fi

log "H0" "phase_complete" "{\"phase\":\"$PHASE\",\"layout\":\"$LAYOUT\"}"
