# Color Picker (picker mode)

## Start: mode selection

After a reset, the **setup ring** appears:

### Mode + channel (setup)

| Display | **5 channel colors** on the left (blinking), **one** LED on the right (radio) |

| Button | Action |
|--------|--------|
| **A** | Next ring step (CCW, purple left → purple right) |
| **B** | Start (0–4 = picker, 5–9 = receiver) |

Channels: `palette::CHANNEL_COLORS` (5 colors).

### Color selection (picker mode)

All **10** LEDs = `palette::SPECTRUM` (white … pink), the selected one blinks off — **separate** from the channel setup.

## Picker runtime

| Button | Action |
|--------|--------|
| **A** | Next color (10 spectrum colors on the ring) |
| **B** | Confirm color → **all 10** LEDs; BLE sends `CMD_COLOR` (only then) |

Sounds: `speaker::play_skip` / `play_apply`.

## BLE

The picker sends the selected color via manufacturer data (`ble_broadcast::set_color`). See [BLE.md](BLE.md).

## Files

| File | Role |
|------|------|
| `src/mode_select.rs` | Setup UI |
| `src/color_picker.rs` | Picker |
| `src/game.rs` | Mode branching |
