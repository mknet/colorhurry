#!/usr/bin/env bash
# UF2 auf CPLAYBTBOOT — unter macOS oft nur per Finder möglich.

set -euo pipefail

uf2="${1:?Usage: copy-uf2.sh firmware.uf2}"
timeout_secs="${UF2_WAIT_SECONDS:-60}"
name="$(basename "$uf2")"
src_size=$(stat -f%z "$uf2" 2>/dev/null || stat -c%s "$uf2")

find_boot_volume() {
    if [[ -d /Volumes/CPLAYBTBOOT ]]; then
        echo "/Volumes/CPLAYBTBOOT"
        return 0
    fi
    local vol
    for vol in /Volumes/*BOOT; do
        [[ -d "$vol" ]] || continue
        echo "$vol"
        return 0
    done
    return 1
}

flash_ok() {
    local vol="$1"
    [[ ! -d "$vol" ]] && return 0
    return 1
}

try_copy() {
    local src="$1" dest="$2"
    # 1) cp ohne xattrs
    if COPYFILE_DISABLE=1 cp -X "$src" "$dest" 2>/dev/null; then
        return 0
    fi
    # 2) cat (umgeht manchmal cp/fcopyfile)
    if cat "$src" > "$dest" 2>/dev/null; then
        return 0
    fi
    # 3) Finder/AppleScript (macOS)
    if [[ "$(uname -s)" == Darwin ]] && [[ "$dest" == *CPLAYBTBOOT* ]]; then
        osascript -e "tell application \"Finder\" to duplicate POSIX file (POSIX file \"$src\") to disk \"CPLAYBTBOOT\" with replacing" 2>/dev/null && return 0
    fi
    return 1
}

open_finder_flash() {
    local uf2_abs
    uf2_abs=$(cd "$(dirname "$uf2")" && pwd)/$(basename "$uf2")
    echo ""
    echo "════════════════════════════════════════════════════════════"
    echo "  macOS blockiert Terminal-Kopieren auf CPLAYBTBOOT."
    echo "  Bitte im Finder ziehen:"
    echo ""
    echo "  $uf2_abs"
    echo "  → auf das Symbol „CPLAYBTBOOT“ (nicht in den Ordner)"
    echo "════════════════════════════════════════════════════════════"
    open -R "$uf2_abs" 2>/dev/null || true
    [[ -d /Volumes/CPLAYBTBOOT ]] && open /Volumes/CPLAYBTBOOT 2>/dev/null || true
}

echo "→ Warte auf CPLAYBTBOOT (NeoPixels grün) …"
deadline=$(( $(date +%s) + timeout_secs ))

while true; do
    if vol=$(find_boot_volume); then
        dest="$vol/$name"
        echo "→ Kopiere nach $vol/ …"

        if try_copy "$uf2" "$dest"; then
            sync 2>/dev/null || true
            sleep 1.5
            if flash_ok "$vol"; then
                echo "→ OK — Board startet neu."
                echo "→ Erwartung: 2 s ORANGE, dann cyan blinken."
                exit 0
            fi
        fi

        echo "→ Automatisch kopieren fehlgeschlagen (Permission denied ist normal auf dem Mac)." >&2
        open_finder_flash
        exit 1
    fi

    if (( $(date +%s) >= deadline )); then
        echo "Fehler: CPLAYBTBOOT nicht gefunden. Reset 2× (Mitte)." >&2
        exit 1
    fi
    sleep 0.5
done
