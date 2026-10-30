# BLE Broadcast (Color Hurry CPB)

In parallel with the **Color Picker**, the board sends a BLE command via **non-connectable advertising** (manufacturer data in the advertising packet). No connection required.

> **Background / troubleshooting:** Why BLE was initially invisible and what it took to fix it (SoftDevice 7.0.1, flash layout, DFU package, async tasks) → [FINDINGS-BLE-FIX.md](FINDINGS-BLE-FIX.md)

## Important: SoftDevice version

| On the chip (Adafruit restore) | Rust library `nrf-softdevice-s140` |
|----------------------------------|----------------------------------------|
| **S140 6.1.1** (default)         | **S140 7.0.1**                         |

**This is why you don't see `ColorHurry-CPB`** even though the Color Picker is running: `Softdevice::enable()` / advertising fail silently with 6.1.1: nothing useful shows up on your phone (only foreign **N/A** entries).

### One-time setup for BLE

```bash
just setup-ble
```

This does:

1. `upgrade-softdevice-v7`: SoftDevice **7.0.1** via serial DFU (Button A + Reset)
2. `flash-v7`: app @ **0x27000** (`sd-v7`)

**Manually:**

```bash
just upgrade-softdevice-v7   # only swap the SoftDevice
just flash-v7                # build + flash the app
```

Without BLE (Color Picker only, as before): keep using `just restore-and-flash` + `just flash` (v6 / 6.1.1).

## Architecture

| Task | Module | Purpose |
|------|--------|---------|
| SoftDevice | `main.rs` | `sd.run()`: radio stack |
| BLE | `ble_broadcast.rs` | Advertising with command payload (picker) |
| UI | `game.rs` | Mode selection → picker or receiver |

### Picker vs. receiver

- **Picker:** sends `CMD_COLOR` only after **Button B** (all LEDs in that color), not during selection.
- **Receiver:** shows a random color, **scans** for a matching `CMD_COLOR`; on a hit, countdown → new random color.
- **Channel** (0-4, setup color on the left): the picker sends the channel in the payload; the receiver ignores other channels.

## Device name

**`ColorHurry-CPB`**: in the advertising packet (`full_name`) and in the GAP stack (`gap_device_name`). In nRF Connect the name should appear **by name** (not just N/A).

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

**Live updates while skipping:** advertising is restarted every **~50 ms** (`config.timeout = 5`) so that `set_color()` actually reaches the radio. Without the timeout, the manufacturer data would stay at the initial value (red).

## D13 diagnostics (without a phone)

After a reset, watch the **red D13 LED** (next to USB) once:

| D13 pattern | Meaning |
|-------------|---------|
| **4× short** at startup | Firmware **v7** (0x27000), correct for BLE |
| **2× short** at startup | Firmware **v6** (0x26000), **wrong for BLE** → `just flash-v7` |
| **1× long** (~0.5 s) | SoftDevice OK |
| **Quiet afterwards** | Advertising is running (payload refresh every ~50 ms, no D13 feedback) |
| **Fast strobe** (8×) | Advertising error: SD version doesn't match |
| **Only 4×, nothing else** | SoftDevice init fails (panic); SD upgrade needed |
| **Constant flickering** | Old firmware: D13 blink per adv timeout; `just flash-v7` |

NeoPixels (Color Picker) may blink **at the same time**; that is normal.

## Testing with nRF Connect

**Important:** Scan on an **iPhone/iPad** with the **nRF Connect** app, not on the Mac. macOS often shows only the MacBook's name; CPB devices are missing or appear as N/A.

1. `just setup-ble` (once) or, after an SD upgrade: `just flash-v7`
2. Scanner → **`ColorHurry-CPB`** or **`ColHurry`**
3. Manufacturer data: **`0x239A`**, payload e.g. **`53 03 03 00 FF 00 00`** (channel 0, red); changes while skipping in the Color Picker

## Error: "Device not configured" during upgrade

The Adafruit serial bootloader needs a **`softdevice_bootloader`** DFU package (SoftDevice + bootloader together), as with `just restore-and-flash`, not a SoftDevice-only package.

On **macOS**, USB often drops briefly (`Device not configured`). The script tries **3×**. Manually:

1. **Hold Button A → Reset 1× → release A** (DFU mode)
2. Immediately:

```bash
DFU_MANUAL=1 just upgrade-softdevice-v7
just flash-v7
```

Success: message `Device programmed.`, then unplug USB, wait 5 s, plug in, Reset 1×.

## Changing the command (firmware)

```rust
ble_broadcast::set_command(ble_broadcast::protocol::CMD_READY);
```

## Relevant files

| File | Role |
|------|------|
| [FINDINGS-BLE-FIX.md](FINDINGS-BLE-FIX.md) | Debug session: all fixes in chronological order |
| `src/ble_broadcast.rs` | SoftDevice, advertising, name, protocol |
| `src/main.rs` | Embassy tasks |
| `memory-s140-v7.x` | App @ `0x27000` (with S140 7.x) |
| `scripts/upgrade-softdevice-v7.sh` | SD 6.1.1 → 7.0.1 |
