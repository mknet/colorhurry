# Proposal

## Why

Trying the game needs two boards and a flashing setup. A browser shell on the shared core lets anyone play the Receiver side on a laptop, and proves that the core really is platform-free. For the demo, a board as Picker should be able to drive a Receiver in the browser.

## What Changes

- New `crates/web`: the shared core compiled to WASM with Trunk, LED ring drawn as a CPB-style circle, Skip and Apply as buttons, tones via WebAudio.
- Start with two simulated boards in one page to exercise room play without hardware.
- Experimental live BLE receive: the user picks a Color Hurry board in the browser's device chooser, and its advertisements are fed into the core as ColorReceived.
- Once live BLE works, simplify to a single Receiver board (browsers can scan, but not advertise).

## Capabilities

### New Capabilities

### Modified Capabilities
- `shell-integration`: adds the browser Receiver shell

## Impact

- New crate `crates/web` (wasm-bindgen, web-sys with `web_sys_unstable_apis` for Web Bluetooth), `index.html`, `Trunk.toml`.
- `just web-serve`, `just web-build`; README notes the Chrome flag for live BLE.
