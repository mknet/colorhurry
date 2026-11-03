# Tasks

## 1. Scan gate (go / no-go)

- [ ] 1.1 Build a throwaway binary that starts MPSL and the SoftDevice Controller with the scan role, runs an observer loop, and logs or blinks on a manufacturer-data report
- [ ] 1.2 Confirm that `nrf-sdc 0.4` has `support_scan()`, name the cargo feature that enables it, and report the flash and RAM cost of the roles
- [ ] 1.3 Confirm that `trouble-host 0.8` builds with no connection in use and that its scan API gives the raw AD bytes for `parse_color_from_adv`
- [ ] 1.4 Confirm that passive scanning receives the Picker payload
- [ ] 1.5 If `trouble-host` does not fit, stop and report before the `bt-hci` fallback

## 2. Port to the SoftDevice Controller

- [ ] 2.1 Replace `nrf-softdevice` with `nrf-sdc`, `nrf-mpsl`, `trouble-host` and `bt-hci`, and enable only advertising and scanning
- [ ] 2.2 Port `ble_broadcast.rs` with the same public API and the Off, Picker and Receiver modes, and keep the 30 ms advertising interval and the 100 ms / 100 ms scan
- [ ] 2.3 Port `main.rs` and `time.rs`, remove only the HFCLK start, and keep the TIMER4 time base
- [ ] 2.4 List every peripheral that MPSL, `embassy-nrf` and the app use, and fix each clash (NeoPixel PWM, speaker PWM, buttons, debug LED)
- [ ] 2.5 Add `memory-s140-v6.x` (FLASH `0x26000`) and `memory-s140-v7.x` (FLASH `0x27000`), keep the `sd-v6` / `sd-v7` features and `build.rs`, and remove the text that says v6 has no BLE
- [ ] 2.6 Size the `nrf-sdc` memory pool, and report the RAM use and the stack use
- [ ] 2.7 Verify `cargo build --release -p color-hurry-cpb --target thumbv7em-none-eabihf` with `--features sd-v7` and with `--no-default-features --features sd-v6`
- [ ] 2.8 Verify `just test`, and that the web and FFI crates still build

## 3. Flash with the matching layout

- [ ] 3.1 Add the UF2 recipe `just flash-uf2` (name to confirm) that reads `SoftDevice: S140 x.y.z` from `INFO_UF2.TXT`
- [ ] 3.2 Build a UF2 for `0x26000` or `0x27000` with `cargo objcopy` and `uf2conv.py` (family `0xADA52840`), and copy it to `CPLAYBTBOOT`
- [ ] 3.3 Stop with a clear message if the version is unknown
- [ ] 3.4 Keep `just flash` (v6) and `just flash-v7`, and keep the `--sd-req` values in `scripts/flash-serial.sh`

## 4. Documentation

- [ ] 4.1 Update `README.md`, `docs/BLE.md`, `docs/RECOVERY.md`, `docs/CORE.md`, `AGENTS.md` (if it mentions the SoftDevice) and the crate doc comments to say that both SoftDevice versions have Bluetooth
- [ ] 4.2 Add one note to the `docs/FINDINGS-*.md` files that they describe the earlier `nrf-softdevice` stack, and change nothing else in them

## 5. Hardware checks (S140 6.1.1 board)

- [ ] 5.1 Flash the board with `just flash-uf2` or `just flash`, and check that it boots and shows the mode select
- [ ] 5.2 Check that a board Picker completes a round with a Mac Receiver
- [ ] 5.3 Check that a board Picker completes a round with an Apple Watch Series 6 Receiver, with tones and ticks
- [ ] 5.4 Check that the board as Receiver completes a round, with a second board or with the help of Marcel, or note it as not tested
- [ ] 5.5 Check that no pairing prompt appears on the Mac or the watch at any time
- [ ] 5.6 Check that the countdown and the tones keep the same rhythm after the executor upgrade
- [ ] 5.7 Check that `just flash` in crux `examples/counter-http/cpb` puts the crux example back on the board
- [ ] 5.8 Build the `sd-v7` layout, and run it on an S140 7.x board if one is available
