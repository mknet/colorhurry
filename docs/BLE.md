# BLE Broadcast (Color Hurry CPB)

In parallel with the **Color Picker**, the board sends a BLE command via **non-connectable advertising** (manufacturer data in the advertising packet). No connection required.

> **Earlier stack:** The debug notes for the `nrf-softdevice` stack are in [FINDINGS-BLE-FIX.md](FINDINGS-BLE-FIX.md).

## SoftDevice versions

Bluetooth works on boards with S140 6.1.1 (the Adafruit factory version) and on boards with S140 7.x. You do not update the SoftDevice or the bootloader.

The firmware uses Nordic's SoftDevice Controller (`nrf-sdc` with `nrf-mpsl`) and the `trouble-host` host. The firmware links this stack into its own image and never enables the resident SoftDevice. The version of the resident SoftDevice therefore does not matter to the code.

A board starts the app right after its SoftDevice, so only the memory layout depends on the version:

| SoftDevice | App @ | Cargo feature | Serial DFU |
|------------|-------|---------------|------------|
| S140 6.1.1 | **0x26000** | `sd-v6` | `just flash` |
| S140 7.x | **0x27000** | `sd-v7` (default) | `just flash-v7` |

RAM starts at `0x20006000` in both layouts.

### Flash the matching layout

With the board in the UF2 bootloader (`CPLAYBTBOOT` mounted), the UF2 recipes read `INFO_UF2.TXT` and pick the layout:

```bash
just board       # print the SoftDevice of the board and the matching layout
just flash-uf2   # build, convert and copy the UF2
just flash-uf2 v6   # choose the layout yourself
```

Serial DFU (`just flash`, `just flash-v7`) cannot read the SoftDevice. You choose the recipe that matches your board.

### Radio resources

MPSL owns RADIO, RTC0, TIMER0, TEMP, CLOCK/POWER, EGU0, RNG and PPI channels 17 to 31. `embassy-time` uses RTC1. TIMER4 is the app time base.

MPSL keeps the 16 MHz crystal on for the life of the firmware, so TIMER4 and the PWM tones keep their timing.

## Architecture

| Task | Module | Purpose |
|------|--------|---------|
| MPSL | `main.rs` | `mpsl.run()`: radio scheduler |
| BLE | `ble_broadcast.rs` | Advertising with command payload (picker), passive scan (caller) |
| UI | `game.rs` | Mode selection → picker or caller |

### Picker vs. caller

- **Picker:** sends `CMD_COLOR` only after **Button B** (all LEDs in that color), not during selection.
- **Caller:** shows a random color, **scans** for a matching `CMD_COLOR`; on a hit, countdown → new random color.
- **Channel** (0-4, setup color on the left): the picker sends the channel in the payload; the caller ignores other channels.

## Device name

The advertisement carries the short name **`ColHurry`**. The scan response carries the full name **`ColorHurry-CPB`**. The advertisement is non-connectable and scannable, so a scanner that asks for the scan response shows the full name. A passive scanner shows the short name.

## Protocol (manufacturer data)

Company ID: **Adafruit `0x239A`**, followed by:

| Offset | Field | Value |
|--------|-------|-------|
| 0 | Magic | `'S'` (`0x53`) |
| 1 | Version | `0x03` |
| 2 | Opcode | `0x03` = CMD_COLOR |
| 3 | Channel | 0-4 (like `palette::CHANNEL_COLORS` in setup) |
| 4 | Red | 0-255 (palette, undimmed) |
| 5 | Green | 0-255 |
| 6 | Blue | 0-255 |

Example channel 2 (green), color red: `53 03 03 02 FF 00 00`. Only the **confirmed** color is sent (picker, "Applied" phase).

### Firmware

```rust
ble_broadcast::set_picker_broadcast(Some(Rgb { r: 255, g: 120, b: 0 }));
```

**Live updates while skipping:** the advertisement stays on until the color changes. A new color replaces the advertiser, so `set_color()` reaches the radio. The Picker advertises about every 30 ms, and the Receiver scans passively with a 100 ms interval and a 100 ms window.

## D13 diagnostics (without a phone)

After a reset, watch the **red D13 LED** (next to USB) once:

| D13 pattern | Meaning |
|-------------|---------|
| **2× short** at startup | Firmware **v6** layout (0x26000) |
| **4× short** at startup | Firmware **v7** layout (0x27000) |
| **1× long** (~0.5 s) | Bluetooth is ready |
| **Quiet afterwards** | Advertising or scanning is running (no D13 feedback) |
| **Fast strobe** (8×) | The radio set-up or the radio failed. The game runs without Bluetooth. |

Both layouts have Bluetooth. A 2× or 4× pattern only tells you which layout the firmware uses. If the pattern does not match the SoftDevice of your board, the app does not start. Flash the layout that matches the board.

NeoPixels (Color Picker) may blink **at the same time**; that is normal.

## Testing with nRF Connect

**Important:** Scan on an **iPhone/iPad** with the **nRF Connect** app, not on the Mac. macOS often shows only the MacBook's name; CPB devices are missing or appear as N/A.

1. `just flash-uf2` (or `just flash` for v6, `just flash-v7` for v7)
2. Scanner → **`ColHurry`** or **`ColorHurry-CPB`**
3. Manufacturer data: **`0x239A`**, payload e.g. **`53 03 03 00 FF 00 00`** (channel 0, red); changes when the Picker applies a color

## Changing the command (firmware)

```rust
ble_broadcast::set_command(ble_broadcast::protocol::CMD_READY);
```

## Relevant files

| File | Role |
|------|------|
| [FINDINGS-BLE-FIX.md](FINDINGS-BLE-FIX.md) | History: the debug session of the earlier `nrf-softdevice` stack |
| `crates/cpb/src/ble_broadcast.rs` | Host, advertising, scanning, name, protocol |
| `crates/cpb/src/main.rs` | MPSL and SoftDevice Controller start, Embassy tasks |
| `crates/cpb/memory-s140-v6.x` | App @ `0x26000` (S140 6.x) |
| `crates/cpb/memory-s140-v7.x` | App @ `0x27000` (S140 7.x) |
