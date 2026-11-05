# Tasks

## 1. Core

- [x] 1.1 Rename the role in `crates/core/src`: `Role`, `Screen`, `ModeRole`, `RoleKind`, `BleOp`, `caller_only`, `FIRST_CALLER_STEP`, `CALLER_ONLY_START_STEP`, `update_caller` and `caller_leds`
- [x] 1.2 Rename `Event::ReceiverOnly` to `Event::CallerOnly` and keep its position in `Event`
- [x] 1.3 Move `receiver.feature` to `caller.feature` and `receiver_only.feature` to `caller_only.feature`
- [x] 1.4 Change the cucumber steps, the step regexes and the helper `enter_caller`, and change the feature text in `mode_select.feature`, `pause.feature` and `reset.feature`
- [x] 1.5 Verify with `cargo test -p color-hurry-core -p color-hurry-ffi`

## 2. Board

- [x] 2.1 Rename `RadioMode::Receiver`, `RECEIVER_ARMED` and `set_radio_receiver` in `crates/cpb/src/ble_broadcast.rs`, and change `game.rs` and `shell.rs`
- [x] 2.2 Verify with `just build` and `rust-size target/thumbv7em-none-eabihf/release/color-hurry-cpb`

## 3. Shells

- [x] 3.1 Change the UI text in `crates/web`, and verify with `just web-build`
- [x] 3.2 Regenerate the Swift and Kotlin types with `just ffi`
- [x] 3.3 Change the Apple sources in `Shared`, `iOS`, `macOS` and `watchOS`, the three `Info.plist` files and `project.yml`
- [x] 3.4 Verify with `just mac-build`, `just ios-build` and `just watch-build`
- [x] 3.5 Change the Android sources, and verify with `just android-build`

## 4. Docs

- [x] 4.1 Change the domain language in `openspec/config.yaml` to Picker and Caller
- [x] 4.2 Move `docs/RECEIVER.md` to `docs/CALLER.md` and change every link to it
- [x] 4.3 Change `README.md`, `apps/README.md` and the other files in `docs`
- [x] 4.4 Change the deltas, the proposal and the tasks of the open change `run-board-bluetooth-on-both-softdevices` to say Caller
- [x] 4.5 Examine each hit of `rg -n -i receiver --glob '!target' --glob '!openspec/changes/archive/**'`, and keep only the verb

## 5. Archive

- [x] 5.1 Apply the deltas to `openspec/specs`
- [x] 5.2 Move `openspec/specs/receiver-round` to `openspec/specs/caller-round`
- [x] 5.3 Change the title and the Purpose of `caller-round/spec.md`, and the Purpose of `mode-select/spec.md`
- [x] 5.4 Move the change to `openspec/changes/archive/2026-11-04-rename-receiver-to-caller`
