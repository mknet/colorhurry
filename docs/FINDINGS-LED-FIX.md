# Findings: No LED output → NeoPixels + D13 blink

Documentation of the debug session for the **Circuit Playground Bluefruit (CPB)** in this repo.
Status: after color picker, button fix, and brightness adjustment.

---

## Summary

There were **three separate problems**, not one:

| # | Problem | Symptom | Fix |
|---|---------|---------|-----|
| 1 | SoftDevice/MBR broken | Flash OK, but app does not start | `just restore-and-flash` |
| 2 | WS2812 bitbang mistimed | NeoPixels **white** instead of orange | PWM driver `neopixel_pwm.rs` |
| 3 | No blink logic for NeoPixels | Orange **solid**, only D13 blinks | `show_orange()` / `show_off()` in the loop |
| 4 | NeoPixels too bright | Colors correct, but uncomfortably harsh | Hard channel cap `MAX_CHANNEL = 8` |

**Step 1 was decisive.** Without a working SoftDevice, no Rust app starts — no matter which firmware you flash.

---

## Problem 1: App does not start (SoftDevice damaged)

### Symptoms

- **No** D13 blink after reset once
- NeoPixels **only briefly** on USB plug-in (bootloader/power-on) — **no** solid orange
- Serial DFU reports `Device programmed`, board leaves DFU mode
- Host logs: ELF correct @ `0x26000` or `0x27000`, stack pointer OK — still no visible app

### Cause

The **SoftDevice/MBR region** on the nRF52840 was damaged, likely by earlier flashes at **address 0x0** (instead of app base `0x26000` / `0x27000`).

- The **UF2/serial bootloader** still ran (brief NeoPixel flash on plug-in)
- The **jump to the application** at the correct app address did not work
- Firmware updates alone could not fix this

### Fix

Restore bootloader + SoftDevice via **serial DFU** (without CPLAYBTBOOT):

```bash
just restore-and-flash
```

The script `scripts/restore-bootloader-serial.sh` flashes:

- `circuitplayground_nrf52840_bootloader-0.11.0_s140_6.1.1.zip`
- SoftDevice **S140 6.1.1** → app then belongs at **`0x26000`** (feature `sd-v6`)

**DFU mode:** hold button A → reset once → release A. During restore (~30 s) do **not** unplug USB.

### How to recognize success

- D13 blinks (slowly)
- NeoPixels respond (initially maybe still white — see problem 2)

---

## Problem 2: NeoPixels white instead of orange (WS2812 timing)

### Symptoms

- App runs (D13 blinks)
- NeoPixels glow **extremely bright white**

### Cause

**CPU bitbang** for WS2812 with `delay_cycles()` assuming ~64 MHz CPU clock.

With SoftDevice running, effective timing for software delays is wrong. “0” pulses become too long → the WS2812 chip reads almost all bits as “1” → **white**.

### Fix

Hardware **PWM @ 16 MHz** (independent of CPU clock), analogous to `src/neopixel.rs` / Embassy:

- Driver: `src/neopixel_pwm.rs`
- PWM0 on **P0.13** (D8), power **P0.06** (D35, LOW = on)

### How to recognize success

- NeoPixels glow **orange** (not white)

---

## Problem 3: NeoPixels do not blink (missing logic)

### Symptoms

- NeoPixels **solid orange**
- Only D13 blinks

### Cause

Not a bug — `show_orange()` was called **only once in `main()` at start**. The main loop only toggled D13.

### Fix

In the blink loop, alternate:

```rust
neopixel_pwm::show_orange();
// … D13 on, wait …
neopixel_pwm::show_off();
// … D13 off, wait …
```

### How to recognize success

- NeoPixels and D13 blink **in sync** (400 ms)

---

## Problem 4: NeoPixels too bright (brightness)

### Symptoms

- WS2812 timing and colors are correct (no more white)
- Color selection (color picker) works
- LEDs feel **clearly too bright**, even after several reduction attempts

### What was tried before (not enough)

Several stages of **percentage scaling** in `show_pixels()`:

| Variant | Formula | Max channel at palette 255 |
|---------|---------|----------------------------|
| ~60 % | `153/256` | ~153 |
| ~40 % | `102/256` | ~102 |
| ~20 % | `51/256` | ~51 |

The palette in `color_picker.rs` intentionally stayed at full RGB (0–255) so the ten spectrum colors remain distinguishable. Dimming happened only when sending via `Rgb::dimmed()` in `show_pixels()`.

**Why that still felt too bright:** At 20 %, the max channel is still ~51 — about 20 % of WS2812 resolution. Ten NeoPixels in a ring add visually; in a dark room it still looks harsh. Percentage scaling without a hard cap still allows high individual values.

### Fix (final step)

Instead of only a percentage factor, there is now an **absolute per-channel ceiling**:

```rust
// src/neopixel_pwm.rs
pub const MAX_CHANNEL: u8 = 8;  // ≈ 3 % of 255

pub fn scale_channel(v: u8) -> u8 {
    if v == 0 {
        return 0;
    }
    ((v as u32 * MAX_CHANNEL as u32) / 255) as u8
}
```

**Behavior:**

- Each channel is scaled **linearly**: full red (255) → **8**, half green (128) → **4**, black (0) → **0**
- No channel can exceed **8** — independent of the input palette
- `show_pixels()` applies `dimmed()` to all pixels before building the PWM sequence
- Color ratios (e.g. orange vs yellow) are preserved; only overall brightness drops

**Tuning:** raise `MAX_CHANNEL` in `src/neopixel_pwm.rs` (e.g. `12` or `16`) if too dark; lower for even softer light.

### How to recognize success

- Colors still distinguishable, but **clearly dimmed** and comfortable in the dark

---

## What was *not* the main problem

| Guess | Result |
|-------|--------|
| Flashing fails | **Disproven** — DFU reports success, board leaves flash mode |
| Wrong link address (v6 vs v7) | **Not enough alone** — correctly linked ELFs only helped after SoftDevice restore |
| CPLAYBTBOOT / UF2 on macOS | Often `Permission denied` / volume does not mount — **serial DFU** more reliable |
| Wrong D13 pin | **Unlikely** — P1.14, active high; D13 blinked once the app ran |
| defmt / panic-probe | Simplification useful, but **not** the cause of missing app start |

---

## Hardware (CPB, verified)

| Function | Pin |
|----------|-----|
| NeoPixels (10×) | P0.13 (D8) |
| NeoPixel/sensor power (D35) | P0.06, **LOW = on** |
| Red LED D13 | P1.14, active high |
| Button A / B | P1.02 / P1.15 |

---

## Flash workflow (macOS, recommended)

Default without UF2 volume:

```bash
just flash          # v6 @ 0x26000 (after S140 6.1.1 restore)
just flash-v7       # only if board has S140 7.x / app @ 0x27000
```

If SoftDevice is broken:

```bash
just restore-and-flash
```

UF2 (only if CPLAYBTBOOT mounts): `just flash-uf2`

See also: [RECOVERY.md](RECOVERY.md), [FLASH-MACOS.md](FLASH-MACOS.md)

---

## Firmware evolution in this repo

| Phase | Code | Result |
|-------|------|--------|
| Start | Embassy, defmt, NeoPixel bitbang, diagnostic `pre_init` | App did not start (SoftDevice broken) |
| Serial DFU + minimal | D13-only, `panic-halt` | Still no start without restore |
| After `restore-and-flash` | Bitbang orange | App runs, NeoPixels **white** |
| PWM driver | `neopixel_pwm.rs` | NeoPixels **orange** (static) |
| Blink loop | `show_orange` / `show_off` | NeoPixels + D13 **in sync** |
| Color picker | `color_picker.rs`, buttons active-high | Color select, skip with button A |
| Brightness | `MAX_CHANNEL = 8` in `neopixel_pwm.rs` | Comfortably dimmed (~3 % max) |

---

## Distinguishing bootloader vs our app

| Observation after reset once | Meaning |
|------------------------------|---------|
| NeoPixels **briefly** on plug-in, then nothing | Bootloader/power-on, **app not running** |
| D13 blinks, NeoPixels respond | **Our app is running** |
| NeoPixels **solid green** | UF2 bootloader mode (reset 2×) |
| Only 2× short D13 | Bootloader pulse, not our blink loop |

---

## Relevant files

| File | Role |
|------|------|
| `src/main.rs` | D13 + NeoPixel blink loop |
| `src/neopixel_pwm.rs` | WS2812 via nRF PWM (16 MHz), global brightness (`MAX_CHANNEL`) |
| `src/color_picker.rs` | Color select (full palette, dims in `show_pixels`) |
| `src/neopixel.rs` | Embassy PWM variant (for later) |
| `memory-s140-v6.x` | App @ `0x26000`, S140 6.1.1 |
| `memory-s140-v7.x` | App @ `0x27000`, S140 7.x |
| `scripts/restore-bootloader-serial.sh` | SoftDevice + bootloader repair |
| `scripts/flash-serial.sh` | App flash via adafruit-nrfutil |

---

## Next steps (development)

- Re-enable Embassy (port `neopixel.rs` or PWM driver)
- Color Hurry game logic (buttons, BLE, speaker)
- Optional: remove old debug scripts (`debug-host-log.sh`, …) or move to `scripts/debug/`
