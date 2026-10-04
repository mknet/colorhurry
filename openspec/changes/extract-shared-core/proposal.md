# Proposal

## Why

All game rules live in async firmware modules that talk to GPIO, PWM and the radio directly. They can only be tested on a board, and a second shell (the browser) would have to copy them. The rules should exist once, free of hardware, so every shell drives the same game.

## What Changes

- New `no_std` crate `color-hurry-core` with `Event`, `Model`, `Effect`, `update` and `view` for mode select, picker and receiver round, plus the palette.
- The firmware moves to `crates/cpb` and becomes a thin hardware shell: buttons and BLE become events, effects drive LEDs, speaker and radio.
- Host tests for the core.
- No change in game behavior.

## Capabilities

### New Capabilities
- `shell-integration`: how platform shells drive the game

### Modified Capabilities

## Impact

- Cargo workspace with `crates/core` and `crates/cpb`; mode modules removed from the firmware.
- `justfile` and scripts point at the new crate paths.
