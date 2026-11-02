# Proposal

## Why

The game runs on the board, in the browser, on iOS and on Android. We also want it on a Mac and on an Apple Watch. The target watch is a Series 6, which a user can wear and tap during a round. Writing the game again in Swift for these devices would bring back rules, tones and wire format in several places that drift apart. The new apps must run the same Rust core as the other shells.

## What Changes

- The iOS app moves from `apps/ios` to `apps/apple`. One xcodegen project holds an iOS, a macOS and a watchOS app. The three apps share the shell code (BLE, game controller, tone player, LED ring) and keep platform differences in separate per-platform source folders.
- The FFI crate builds macOS and watchOS slices. BoltFFI 0.30.1 has no watchOS target, so a local cargo script builds the watchOS static libraries and rebuilds the xcframework. The script is small, so we can delete it when BoltFFI supports watchOS.
- The watchOS slices cover `arm64_32-apple-watchos` (Series 6 and later), `aarch64-apple-watchos` and `aarch64-apple-watchos-sim`. The `arm64_32` target is tier 3 in Rust, so it builds with nightly and `-Zbuild-std`.
- A macOS app (SwiftUI + CoreBluetooth) with the same entry as iOS (Skip / Apply, mode select, Receiver) and keyboard shortcuts.
- A standalone watchOS app (SwiftUI + CoreBluetooth) with a full-screen LED ring, Skip and Apply buttons, and a haptic on each tone step.
- Neither app can broadcast as a Picker. CoreBluetooth cannot advertise manufacturer data on macOS or watchOS (the same limit as iOS). The apps still let the user enter Picker, because mode select is in the core. When the core requests the broadcast, each app shows that it cannot broadcast the color and tells the user to use a board or an Android phone as Picker.
- `just ffi-apple` replaces `just ffi-ios` (which stays as an alias). New `just` recipes build the three apps.

## Capabilities

### New Capabilities

### Modified Capabilities
- `shell-integration`: adds the native macOS and watchOS shells on the shared core

## Impact

- `apps/ios` becomes `apps/apple`. The references in `crates/ffi/boltffi.toml`, `justfile`, `README.md`, `apps/README.md` and `.gitignore` change. The committed `ColorHurry.xcodeproj` is deleted and regenerated from `project.yml`.
- `crates/ffi`: macOS and watchOS platforms in `boltffi.toml` and `codegen.rs`. New script `scripts/xcframework_watchos.rs`. New targets in `rust-toolchain.toml`. The `arm64_32` target is not listed there, because stable cannot install it.
- Risk: `crux_core` with `bridge`, `boltffi` and `bincode` must work on `arm64_32`, which has 32-bit pointers. We check this first. If BoltFFI is not 32-bit safe, we stop and review the options.
- Tooling: nightly Rust with the `rust-src` component, Xcode with xcodegen, and a Series 6 for the manual check.
