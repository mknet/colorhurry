# Bootloader / flashing (CPLAYBTBOOT)

## Important: which button?

| Button | Position | Function |
|--------|----------|----------|
| **Reset** | **Middle** (small) | 1× = restart, **2× fast** = UF2 bootloader |
| **Button A** | left | In-game / **hold during reset** → bootloader |
| **Button B** | right | In-game |

Do **not** press Button A or B twice; that does **not** start a bootloader.

## Starting the bootloader (3 ways)

### 1. Double reset (default)

1. USB **data** cable (not a charge-only cable)
2. Press **Reset in the middle** twice **quickly** in a row (< 500 ms)
3. NeoPixels: briefly **red**, then **green**
4. In Finder: drive **CPLAYBTBOOT**

### 2. Button A + Reset (more reliable)

1. **Hold Button A (left)**
2. While holding, tap **Reset (middle) once** (or briefly unplug and replug USB, keep holding A)
3. **Release A** when the NeoPixels turn green
4. **CPLAYBTBOOT** should appear

*(With current firmware: **hold Button A**, plug in USB or **Reset 1×** → starts directly in the bootloader.)*

### 3. Firmware already on the board, bootloader via code

With flashed firmware: **hold Button A**, **Reset 1×** → `GPREGRET` → UF2.

## Checking on the Mac

```bash
just boot-check
```

### `Permission denied` in `/Volumes/CPLAYBTBOOT`

Happens often on the Mac; the bootloader is active anyway.

- Do **not** `cd` into the drive and run `ls`
- Instead, copy from the outside:

```bash
cp /path/to/file.uf2 /Volumes/CPLAYBTBOOT/
# or, in the project:
just flash
```

Or in **Finder**: drag the `.uf2` onto the **CPLAYBTBOOT** icon (without opening the folder).

If `cp` fails too: eject the drive, Reset 2×, try again.  
Optional: **System Settings → Privacy → Full Disk Access** → enable Terminal/iTerm.

USB device (vendor 0x239A):

```bash
system_profiler SPUSBDataType | grep -A6 -i "239A\|Circuit"
```

## All NeoPixels stay green (after a successful `just`)

The board is still in the **UF2 bootloader**, the app isn't running. Common causes:

1. **Wrong flash address**: the app must be linked at `0x27000` (`memory.x`). After a change: `just flash-clean`
2. **GPREGRET / Button A**: older firmware jumped into the bootloader immediately when A was held. Current firmware sets GPREGRET to 0 at startup.
3. **Hands off Button A** during the reset after flashing.

Expected after OK: **cyan blinking** edge LEDs, **red** D13 blinks too, **not** permanently green.

## If no drive appears

- Different USB cable / directly on the Mac (no hub)
- Different USB port
- NeoPixels stay **red** → often cable/port (no data contact)
- Vary the double-reset rhythm (faster/slower)
- Try **Button A + Reset**

## Flashing

```bash
just
```

Waits up to 60 s for **CPLAYBTBOOT** (the reset can happen after `just` has started).
