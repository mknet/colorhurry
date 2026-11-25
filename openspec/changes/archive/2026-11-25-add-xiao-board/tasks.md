# Tasks

## 1. Shared board shell

- [x] 1.1 Move the board shell to `crates/shell` behind the `Board` trait (from the `ese` branch) and implement `Board` for the CPB
- [x] 1.2 Verify `cargo test -p color-hurry-shell` and the CPB firmware build

## 2. XIAO board

- [x] 2.1 Probe the XIAO pins of the Grove LED stick and the Grove Dual Button with a throwaway firmware
- [x] 2.2 Add the `xiao` feature: LED stick pin, button pins and polarity, debug LED, device name, no speaker
- [x] 2.3 Add `just xiao-uf2`, `just xiao-flash` and `just xiao-flash-serial`
- [x] 2.4 Verify the CPB build (`sd-v6`, `sd-v7`) and the XIAO build
- [x] 2.5 Flash the XIAO and play mode select, Picker and Caller against a second board or app

## 3. Docs

- [x] 3.1 Add `docs/XIAO.md` (parts, wiring, flashing) and link it from `README.md`
