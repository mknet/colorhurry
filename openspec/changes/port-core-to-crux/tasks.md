# Tasks

## 1. Inventory

- [ ] 1.1 Write `docs/CORE.md` with the core/shell boundary, violations, effect sketch and the decisions on color (v4) and tones, verify the checklist is confirmed

## 2. Core on Crux

- [ ] 2.1 Add `protocol.rs` (v4 encode/decode) with unit tests for roundtrip, bad magic, v3 payloads, bad channel and bad index
- [ ] 2.2 Add `tones.rs` with Skip, Apply, countdown and the explosion sweep
- [ ] 2.3 Implement `impl App for ColorHurry` with Round state, domain events and Delay/Random/PlayTone/Ble effects, verify `cargo test -p color-hurry-core`

## 3. Shells

- [ ] 3.1 Adapt the CPB shell: resolve Delay and Random from the board runtime, play tones and BLE ops from the effects, forward every parsed ad, verify `just build` and a round on the board
- [ ] 3.2 Adapt the web shell to `process_event` / `resolve`, verify `just web-build` and the demo flow board Picker to browser Receiver
