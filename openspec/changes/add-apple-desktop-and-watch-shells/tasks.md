# Tasks

## 1. Move the iOS shell to apps/apple

- [ ] 1.1 Move `apps/ios` to `apps/apple`, with the shared code in `Shared/Shell` and `Shared/UI` and the iOS-only files in `iOS/`
- [ ] 1.2 Update `crates/ffi/boltffi.toml`, `justfile`, `README.md`, `apps/README.md` and `.gitignore`, and replace the committed `ColorHurry.xcodeproj` with one generated from `project.yml`, verify `just ios-build`

## 2. macOS and watchOS slices of the FFI core

- [ ] 2.1 Check first that `crux_core`, `boltffi` and `bincode` build and run on `arm64_32`, and stop to review the options if they do not
- [ ] 2.2 Enable macOS in `boltffi.toml`, add the platform chain to `codegen.rs` and the targets to `rust-toolchain.toml`
- [ ] 2.3 Add `scripts/xcframework_watchos.rs`, which builds the three watchOS libraries and rebuilds the xcframework, verify with `ls`, the xcframework `Info.plist` and `lipo -info`
- [ ] 2.4 Add the `.watchOS(.v10)` and `.macOS(.v13)` platforms to the generated `Package.swift` if it declares platforms

## 3. justfile

- [ ] 3.1 Add `ffi-apple` (keep `ffi-ios` as an alias), `apple`, `mac-build`, `watch-build` and `ios-build`, verify each recipe runs

## 4. Xcode project

- [ ] 4.1 Add the `ColorHurryMac` and `ColorHurryWatch` targets to `project.yml` with Bluetooth usage strings, the macOS sandbox entitlement, and device `ARCHS` that include `arm64_32`, verify the build log

## 5. Shared Swift changes

- [ ] 5.1 Add the per-platform `Platform` files (audio setup, tone feedback, platform name) without compiler directives, verify `just test` and `just ios-build`

## 6. Per-platform UI

- [ ] 6.1 macOS: window size, keyboard shortcuts for Skip and Apply, cannot-broadcast status when the core requests the Picker broadcast, verify `just mac-build`
- [ ] 6.2 watchOS: full-screen LED ring, Skip and Apply buttons, BLE status line, haptic on each tone step, cannot-broadcast status when the core requests the Picker broadcast, verify `just watch-build`

## 7. Docs and demo

- [ ] 7.1 Add the macOS and watchOS rows, the build commands and the Picker limit to `apps/README.md` and `README.md`, and rename the iOS README
- [ ] 7.2 Verify the demo flow: board Picker to Mac Receiver
- [ ] 7.3 Verify on a Series 6: the app launches, and a board Picker on the same channel completes a Receiver round with tones and haptics
