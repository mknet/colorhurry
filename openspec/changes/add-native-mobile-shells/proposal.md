# Proposal

## Why

For the demo, a phone should be able to play the Receiver against a board Picker, without a Chrome flag. Native apps get CoreBluetooth and Android BLE. Writing the game again in Swift and Kotlin would bring back exactly what the Crux port removed: rules, tones and wire format in several places that drift apart. The apps should run the same Rust core as the board and the browser.

## What Changes

- Core types (`Event`, effect operations, `ViewModel`, `Color`, tones) become serializable with serde and describable with facet, without giving up `no_std`.
- New crate `crates/ffi`: the core behind the Crux bridge, exported with BoltFFI, plus a `codegen` binary that generates the Swift and Kotlin types with facet typegen.
- BLE payload helpers exported from Rust (`color_received_event`, `color_advertisement`), so the apps never parse or build payloads themselves.
- Native shells: SwiftUI + CoreBluetooth (`apps/ios`) and Kotlin Compose + Android BLE (`apps/android`). Same entry as the board (Skip / Apply, mode select, Picker or Receiver), LED ring UI like the web shell.
- `just ffi`, `just ffi-ios`, `just ffi-android`; `just ios` and `just android` build the core first.

## Capabilities

### New Capabilities

### Modified Capabilities
- `shell-integration`: adds the native iOS and Android shells on the shared core

## Impact

- `crates/core`: serde and facet derives, feature `facet_typegen` (only enabled by `crates/ffi`).
- New `crates/ffi`, `apps/ios`, `apps/android`; generated code under `apps/*/generated` is not checked in.
- Tooling: `boltffi_cli` 0.30.1, Xcode with xcodegen, JDK 17 or 21 and the Android NDK.
