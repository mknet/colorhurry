# Tasks

## 1. WASM shell

- [ ] 1.1 Add `crates/web` with two simulated boards on the shared core, verify a picker board completes a receiver round in one page
- [ ] 1.2 Add `just web-serve` / `just web-build`, verify the Trunk release build
- [ ] 1.3 Document the workspace layout and browser setup in `README.md`

## 2. Live BLE

- [ ] 2.1 Receive board advertisements via Web Bluetooth and feed them into the core, verify with a CPB Picker in Chrome
- [ ] 2.2 Document the Chrome experimental flag in `README.md`
- [ ] 2.3 Prefer the device chooser plus `watchAdvertisements` over `requestLEScan`, verify a selected board feeds events reliably

## 3. Single Receiver

- [ ] 3.1 Reduce the page to one Receiver board fed by live BLE, verify the demo flow board Picker to browser Receiver
- [ ] 3.2 Lay out the LEDs as a CPB-style circle that fills the viewport, verify on desktop and narrow windows
