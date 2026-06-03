# Tasks

## 1. Radio stack

- [x] 1.1 Add SoftDevice S140 7.0.1 flash layout and the `just setup-ble` upgrade path, verify the board boots with the D13 "SoftDevice OK" pattern
- [x] 1.2 Advertise non-connectable as `ColorHurry-CPB` and verify the name in nRF Connect

## 2. Color Picker

- [x] 2.1 Drive the ten NeoPixels over PWM with the spectrum ring and blinking selection, verify on the board
- [x] 2.2 Map Button A to Skip and Button B to ActivateColor, verify cycling, wrap-around and apply

## 3. Broadcast

- [x] 3.1 Encode the v2 color payload and refresh advertising every ~50 ms, verify `53 02 03 FF 00 00` for red in nRF Connect
- [x] 3.2 Document the protocol and setup in `docs/BLE.md`
