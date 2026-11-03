# Proposal

## Why

The board firmware uses the `nrf-softdevice` crate for Bluetooth. That crate needs the resident SoftDevice S140 7.x. A board with the factory S140 6.1.1 has no Bluetooth, so it cannot be a Picker or a Receiver. The owner of such a board cannot test a board Picker against the Mac and watch Receivers.

The goal is one firmware source that has Bluetooth on both boards. No irreversible change to a board is allowed, so the work must not update the SoftDevice or the bootloader. Players must still play without pairing: the Picker broadcasts and the Receiver listens, and no connection is ever made.

## What Changes

- The board firmware gets its Bluetooth stack from `nrf-sdc` (the SoftDevice Controller), `nrf-mpsl`, `trouble-host` and `bt-hci`. The firmware links this stack into its own image. It never enables the resident SoftDevice, so the version of that SoftDevice does not matter to the code.
- The Picker keeps its non-connectable advertisements and the Receiver keeps scanning only. The firmware enables no connection, GATT or security feature.
- The public API of `ble_broadcast.rs` stays the same, so `shell.rs` and `game.rs` do not change.
- The firmware keeps two memory layouts (`sd-v6` and `sd-v7`) from one source, because a board jumps to the application just after its SoftDevice. Both layouts now have Bluetooth.
- A new UF2 flash recipe reads the SoftDevice version from `INFO_UF2.TXT` and picks the matching layout. The serial recipes `just flash` and `just flash-v7` stay.
- Scanning has no prior art on this stack, so the first task is a go / no-go check with a throwaway binary.
- The docs no longer say that S140 6.1.1 boards have no Bluetooth.

## Capabilities

### New Capabilities

### Modified Capabilities
- `ble-protocol`: Bluetooth on the board works on S140 6.1.1 and 7.x boards, with no pairing or connection, and flashing matches the memory layout to the board
- `timing-and-tones`: the timing requirement names the radio stack, not the SoftDevice, because the board no longer enables the resident SoftDevice

## Impact

- `crates/cpb`: `Cargo.toml`, `build.rs`, `ble_broadcast.rs`, `main.rs`, `time.rs`, and both memory files (`memory-s140-v6.x` and `memory-s140-v7.x`).
- The flash recipes in the `justfile`, and `scripts/flash-serial.sh` (unchanged `--sd-req` values).
- `README.md`, `docs/BLE.md`, `docs/RECOVERY.md`, `docs/CORE.md`, `AGENTS.md` if it mentions the SoftDevice, and the crate doc comments.
- The core, the web shell and the FFI crates do not change.
- Risks: peripheral and interrupt clashes with the existing PAC-level code, the executor upgrade (timing), RAM use of the `nrf-sdc` memory pool, and a v7 build that only a v7 board can run.
