# Proposal

## Why

The players say "Caller", not "Receiver". The Caller calls out the color, and the Picker picks it.

## What Changes

- The role Receiver gets the name Caller in the specs, the core, the shells, the tests and the docs. The roles are now Picker and Caller.
- The core renames the role in every case form: `Role::Caller`, `Screen::Caller`, `ModeRole::Caller`, `RoleKind::Caller`, `BleOp::Caller`, `caller_only`, `FIRST_CALLER_STEP`, `CALLER_ONLY_START_STEP`, `update_caller` and `caller_leds`.
- The shell event `ReceiverOnly` gets the name `CallerOnly`. Its position in `Event` does not change, so the bincode indices of the events do not change.
- The board firmware renames `RadioMode::Receiver`, `RECEIVER_ARMED` and `set_radio_receiver` in `ble_broadcast.rs`.
- The acceptance tests move to `caller.feature` and `caller_only.feature`. The Gherkin steps say "caller" (for example `When I enter caller on channel 0`), and the test helper is `enter_caller`.
- The web, Apple and Android shells show "Caller" in their UI text.
- `docs/RECEIVER.md` moves to `docs/CALLER.md`.
- The verb does not change. `ColorReceived`, `color_received`, "receives" and "is received" stay as they are.
- The game behavior does not change.

## Capabilities

### New Capabilities

### Modified Capabilities
- `receiver-round`: the requirements name the Caller and `caller.feature`. The archive step moves this capability to `caller-round` (see below)
- `mode-select`: the roles are Picker and Caller, and a shell sends `CallerOnly`
- `ble-protocol`: the Caller scans for payloads on its channel
- `shell-integration`: the shells run as Caller, and `CallerOnly` replaces `ReceiverOnly` in the shell events
- `timing-and-tones`: the countdown requirements name `caller.feature` and the Caller round

### Capability directory rename
A delta cannot rename a capability. The deltas in this change apply to `openspec/specs/receiver-round`. Then the archive step moves `openspec/specs/receiver-round` to `openspec/specs/caller-round` by hand. A delta also cannot change a spec title or a `## Purpose` section. The archive step edits these lines by hand:

- `receiver-round/spec.md`: the title becomes `# caller-round Specification`, and the Purpose names "the Caller's round".
- `mode-select/spec.md`: the Purpose names the roles "Picker or Caller".

### Overlap with an open change
The open change `run-board-bluetooth-on-both-softdevices` also modifies `Timing next to the radio stack`, and its deltas name the Receiver. This change updates the text of that open change to say Caller. This change archives first. When the Bluetooth change archives later, its version of the requirement replaces this version, and it already says Caller.

## Impact

- `crates/core`: `lib.rs`, `model.rs`, `palette.rs`, `view.rs` and `tests.rs`.
- `crates/core/tests`: the cucumber steps in `cucumber/mod.rs` and `cucumber/parse.rs`, and the features `caller.feature`, `caller_only.feature`, `mode_select.feature`, `pause.feature` and `reset.feature`.
- `crates/cpb`: `ble_broadcast.rs`, `game.rs` and `shell.rs`.
- `crates/web/src/lib.rs`.
- Regenerated Swift and Kotlin types.
- Apple: `Shared` (`BleService.swift`, `GameController.swift`, `ContentView.swift`), the three `Platform.swift` files, `WatchHint.swift`, the three `Info.plist` files and `project.yml`.
- Android: `BleService.kt`, `GameController.kt` and `HomeScreen.kt`.
- `openspec/config.yaml`, `README.md`, `apps/README.md`, `docs/AUDIO.md`, `docs/BLE.md`, `docs/COLOR-PICKER.md`, `docs/CORE.md` and `docs/CALLER.md`.
- The open change `run-board-bluetooth-on-both-softdevices`.
- The archived changes in `openspec/changes/archive` do not change. The archived change `receiver-only-shells` keeps its name.
