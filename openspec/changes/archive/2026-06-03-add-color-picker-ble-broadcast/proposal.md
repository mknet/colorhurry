# Proposal

## Why

Color Hurry needs a first playable piece on the Circuit Playground Bluefruit: one player chooses a color on the LED ring and the board announces it over the air, so a second board (or a phone running nRF Connect) can pick it up without pairing.

## What Changes

- Color Picker on the ten NeoPixels: Skip (Button A) moves through a ring of ten spectrum colors, ActivateColor (Button B) applies the selected color to all LEDs.
- BLE broadcast of the color as non-connectable advertising with manufacturer data (protocol v2: magic, version, opcode, RGB).
- SoftDevice S140 7.0.1 as the radio stack, with flash layout and setup scripts for the upgrade from the factory 6.1.1.

## Capabilities

### New Capabilities
- `picker`: choosing and applying a color on the LED ring
- `ble-protocol`: how a Color Hurry board announces a color over BLE advertising

### Modified Capabilities

## Impact

- New firmware crate for the nRF52840 (Embassy, nrf-softdevice, NeoPixel PWM driver).
- Flashing and recovery tooling (`justfile`, `scripts/`, `docs/`).
