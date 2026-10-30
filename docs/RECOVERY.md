# Default program / CircuitPython instead of Rust

If **CircuitPython** or the **old factory program** is still running after flashing (e.g. `CIRCUITPY` in Finder, colorful demo, no 2× orange startup):

1. **Bootloader:** Reset **2×** → NeoPixels **green**
2. **Finder** (almost always necessary on the Mac; Terminal: `Permission denied`):
   ```bash
   just flash-finder
   ```
   Then drag the `.uf2` onto **CPLAYBTBOOT** (onto the **icon**, not into the folder)
3. The drive must **disappear**, otherwise the flash failed
4. Unplug USB for 5 s, plug in, Reset **1×**

**Rust detected?** At startup, **all NeoPixels orange for 2 seconds**, then **blinking cyan** + red D13.

`just` used to wrongly report "OK" even though CPLAYBTBOOT was still there; it now only reports success when the drive is gone.

---

# Board stays green (bootloader): recovery

**Green = UF2 bootloader.** Your Rust app is not running.

## Quick test after flashing

1. **Unplug USB completely**, wait 5 seconds, plug in again  
2. Do **not** touch any buttons  
3. Only **Reset (middle) 1×**, **not** 2×  
4. Expected: **red LED D13** blinks (~500 ms), **not** permanently green

## If it's still only green

### A) GPREGRET / bootloader mode is "stuck"

Earlier firmware could jump into the bootloader via a button (`GPREGRET = 0x57`). This survives soft resets.

→ **Really cut power** (unplug USB), not just reset.

### B) SoftDevice damaged by a wrong flash

If it was previously flashed at address **0x0**, the **SoftDevice** region may be corrupted. The bootloader (green) still works, the app at **0x27000** does not.

**Repair (UF2, NeoPixels green):**

1. [Circuit Playground Bluefruit bootloader update](https://learn.adafruit.com/adafruit-circuit-playground-bluefruit/update-bootloader-use-uf2)  
2. File `update-*circuitplayground*` / `*CPLAYBT*` from [Adafruit_nRF52_Bootloader releases](https://github.com/adafruit/Adafruit_nRF52_Bootloader/releases)  
3. Drag it onto **CPLAYBTBOOT** via Finder (don't `cd` into the drive from the terminal)

**Or test with CircuitPython:**

- [CircuitPython UF2 for CPB](https://circuitpython.org/board/circuitplayground_bluefruit/)  
- If **CIRCUITPY** appears afterwards: hardware is OK → flash Rust again with `just flash-now`

### C) Flash didn't actually write

`just` reports "UF2 accepted", but `dd` can fail on the Mac.

→ In **Finder**: drag `target/thumbv7em-none-eabihf/release/color-hurry-cpb.uf2` onto **CPLAYBTBOOT**  
→ NeoPixels briefly **red**, drive disappears, board restarts

## Then flash Rust

```bash
just flash-clean
```

(or, while in the bootloader: `just flash-now`)

## Re-enabling NeoPixels

Once D13 blinks, re-enable `mod neopixel` and the NeoPixel code in `main.rs` (older Git version).
