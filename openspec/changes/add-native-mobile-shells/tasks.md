# Tasks

## 1. Core types for the bridge

- [ ] 1.1 Derive serde and facet on events, effect operations, view model, color and tones, gate typegen behind `facet_typegen`, verify `just build` (thumbv7em, `no_std`) and `just test`

## 2. FFI crate

- [ ] 2.1 Add `crates/ffi` with `CoreFfi` on the Crux bridge and BoltFFI exports, verify `cargo test -p color-hurry-ffi`
- [ ] 2.2 Add the `codegen` binary and `boltffi.toml`, verify `just ffi` produces the Swift package and the Kotlin bindings with jniLibs
- [ ] 2.3 Export `color_received_event` and `color_advertisement`, verify the roundtrip test

## 3. Native shells

- [ ] 3.1 iOS: `GameController` on the generated `Core`, `EffectHandler` for Delay, Random, PlayTone and Ble, verify `just ios-build`
- [ ] 3.2 Android: `GameController` on the generated `Core`, verify `just android-build`
- [ ] 3.3 Document build and shared logic in `apps/README.md` and `README.md`, verify the documented commands run as written
- [ ] 3.4 Verify the demo flow: board Picker to phone Receiver
