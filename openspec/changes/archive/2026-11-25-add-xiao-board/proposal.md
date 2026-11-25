# Proposal

## Why

The Circuit Playground Bluefruit is the only board that runs the game. The Seeed XIAO nRF52840 (Sense) has the same chip, is small and cheap, and with two Grove parts it has everything a player needs: the Grove RGB LED Stick (10 × WS2813) gives the ten LEDs and the Grove Dual Button gives Skip and Apply. A XIAO should play against a CPB, a phone or the browser, with the same rules and the same radio protocol.

## What Changes

- The board shell moves to `crates/shell` behind a `Board` trait (buttons, LEDs, delay, tones, radio), as on the `ese` branch, so a second board only needs its own `Board` implementation.
- The firmware crate `crates/cpb` gets a `xiao` feature. It selects the XIAO pins: the LED stick on D0, the buttons on the Grove port next to it (active low, pull-up), and the on-board red LED for diagnostics. Radio, time base and memory layout stay shared, because both boards are nRF52840 with S140 7.x and the same UF2 bootloader family.
- The XIAO has no speaker. Its `play_tone` waits for the tone length without sound, so the countdown keeps its timing.
- The XIAO advertises the scan-response name `ColorHurry-XIAO`. The Color Hurry advertisement itself does not change.
- New `just` recipes build the XIAO UF2 and copy it to the XIAO bootloader drive.

## Capabilities

### New Capabilities

### Modified Capabilities
- `shell-integration`: the board shell is shared through a `Board` trait, and the Seeed XIAO nRF52840 with the Grove LED stick and dual button is a supported board

## Impact

- New crate `crates/shell`; `crates/cpb` implements `Board` and gets the `xiao` feature (`board.rs`, `buttons.rs`, `neopixel_pwm.rs`, `debug_led.rs`, `ble_broadcast.rs`, `main.rs`).
- `justfile` (XIAO recipes), `README.md` and a new `docs/XIAO.md`.
- No change to the core, the radio protocol, the web shell or the native apps.
