# Proposal

## Why

Color Hurry already runs the same Rust core on the board, in the browser (WASM) and in the native apps (BoltFFI). There is no Node.js shell yet. A notebook server that drives the unchanged core over Node-API and speaks BLE shows that the core binds the same way outside WASM and mobile FFI.

## What Changes

- `crates/nodejs`: a thin [napi-rs](https://napi.rs) addon that holds `Core<ColorHurry>`, resolves timers and random inside Rust, and returns view, tones and BLE ops to JavaScript. The core crate stays unchanged.
- `server/nodejs`: HTTP + WebSocket shell with a static Web UI (LED ring, Skip/Apply/Mode, Web Audio tones). BLE scan via `@abandonware/noble` (Caller on macOS/Linux; Windows best-effort). Picker advertise via bleno EIR on Linux only (manufacturer data); on macOS the UI can still choose Picker and reports when advertise is unavailable.
- A short `shell-integration` delta for the Node.js server shell.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `shell-integration`: Node.js server shell on the shared core

## Impact

- New `crates/nodejs` workspace member and `server/nodejs` package; `just node` recipe.
- No changes to `crates/core` or existing shells.
