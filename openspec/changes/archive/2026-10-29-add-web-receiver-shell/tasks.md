# Tasks

## 1. WASM shell

- [x] 1.1 Add `crates/web` with two simulated boards on the shared core, verify a picker board completes a receiver round in one page
- [x] 1.2 Add `just web-serve` / `just web-build`, verify the Trunk release build
- [x] 1.3 Document the workspace layout and browser setup in `README.md`

## 2. Live BLE

- [x] 2.1 Receive board advertisements via Web Bluetooth and feed them into the core, verify with a CPB Picker in Chrome
- [x] 2.2 Document the Chrome experimental flag in `README.md`
- [x] 2.3 Prefer the device chooser plus `watchAdvertisements` over `requestLEScan`, verify a selected board feeds events reliably

## 3. Single Receiver

- [x] 3.1 Reduce the page to one Receiver board fed by live BLE, verify the demo flow board Picker to browser Receiver
- [x] 3.2 Lay out the LEDs as a CPB-style circle that fills the viewport, verify on desktop and narrow windows
