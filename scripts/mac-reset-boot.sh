#!/usr/bin/env bash
# Clean up a hung / broken CPLAYBTBOOT on macOS

set -euo pipefail

echo "=== Mounted BOOT drives ==="
ls -la /Volumes/*BOOT 2>/dev/null || echo "(none visible)"

if [[ -d /Volumes/CPLAYBTBOOT ]]; then
    echo "→ Trying to eject …"
    diskutil eject /Volumes/CPLAYBTBOOT 2>/dev/null || diskutil unmount force /Volumes/CPLAYBTBOOT 2>/dev/null || true
fi

echo ""
echo "=== External disks (diskutil) ==="
diskutil list external 2>/dev/null | head -40 || diskutil list | head -40

echo ""
echo "Next steps:"
echo "  1. Unplug USB from the board, wait 10 s"
echo "  2. Plug it back in"
echo "  3. Reset 2× (middle) → NeoPixels green"
echo "  4. just boot-check"
echo ""
echo "If CPLAYBTBOOT is still missing in Finder → serial flash (no drive needed):"
echo "  pip3 install adafruit-nrfutil"
echo "  just flash-serial"
