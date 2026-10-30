# Audio (CPB speaker)

## Hardware

| Pin | Function |
|-----|----------|
| P0.26 (D12) | PWM1 → speaker |
| P1.04 (D11) | Amplifier enable (HIGH = on) |

PWM0 is reserved for NeoPixels.

## Color Picker: UI sounds (default)

| Action | Button | Tone | Character |
|--------|--------|------|-----------|
| **Change color** | A (skip, selection) | 380 Hz, 55 ms, duty 22 % | subtle, low |
| **Confirm color** | B (all LEDs) | 784 Hz, 130 ms, duty 34 % | higher, enter-like |

Implementation: `speaker::play_skip()` / `speaker::play_apply()` in `color_picker.rs`.

Adjust the constants in `src/speaker.rs`.

## Receiver countdown (default in receiver mode)

One descending tone per second, in sync with the NeoPixels switching off counter-clockwise (CCW):

| Second | Tone (approx.) |
|--------|----------------|
| 1-10 | 680 → 200 Hz (`src/countdown.rs`) |

Implementation: `countdown::run_once` after the random color is shown (`src/receiver.rs`).

## 12-second demo (kept)

The test sequence (guitar reset → 10 descending tones → explosion) lives in **`src/speaker_demo.rs`** and is **off** by default.

To test, build with the feature:

```bash
cargo build --release --features speaker-demo
# or extend default in Cargo.toml; then flash-v7 as usual
```

`speaker_demo_task` is then started in `main.rs`. **Not** intended for use in parallel with the UI (both use PWM1).

### Demo sequence

| Second | Content |
|--------|---------|
| 1 | Guitar strum g-b-e |
| 2-11 | Descending scale (680 → 200 Hz) |
| 12 | Explosion (sweep, full second) |

## Relevant files

| File | Role |
|------|------|
| `src/speaker.rs` | PWM driver, UI sounds |
| `src/speaker_demo.rs` | Demo sequence (feature `speaker-demo`) |
| `src/color_picker.rs` | Calls UI sounds on button presses |
