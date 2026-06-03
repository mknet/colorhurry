#!/usr/bin/env bash
# Hängendes / kaputtes CPLAYBTBOOT auf macOS aufräumen

set -euo pipefail

echo "=== Gemountete BOOT-Laufwerke ==="
ls -la /Volumes/*BOOT 2>/dev/null || echo "(keins sichtbar)"

if [[ -d /Volumes/CPLAYBTBOOT ]]; then
    echo "→ Versuche Auswerfen …"
    diskutil eject /Volumes/CPLAYBTBOOT 2>/dev/null || diskutil unmount force /Volumes/CPLAYBTBOOT 2>/dev/null || true
fi

echo ""
echo "=== Externe Datenträger (diskutil) ==="
diskutil list external 2>/dev/null | head -40 || diskutil list | head -40

echo ""
echo "Nächste Schritte:"
echo "  1. USB am Board abziehen, 10 s warten"
echo "  2. Wieder einstecken"
echo "  3. Reset 2× (Mitte) → NeoPixels grün"
echo "  4. just boot-check"
echo ""
echo "Wenn CPLAYBTBOOT im Finder weiter fehlt → Serial-Flash (ohne Laufwerk):"
echo "  pip3 install adafruit-nrfutil"
echo "  just flash-serial"
