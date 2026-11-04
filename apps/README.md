# Color Hurry native apps

| App | Stack | Path |
|-----|--------|------|
| iOS | SwiftUI + CoreBluetooth | [`apple/`](apple/) |
| macOS (14+) | SwiftUI + CoreBluetooth | [`apple/`](apple/) |
| watchOS (10+) | SwiftUI + CoreBluetooth | [`apple/`](apple/) |
| Android | Kotlin + Jetpack Compose + Android BLE | [`android/`](android/) |

All four apps mirror the board entry (**Skip** / **Apply** → Mode select → Picker or Caller) and the web dark LED-ring UI. The **Mode** button (**Change mode** on the watch) goes back to mode select from any screen. Mode select then starts at the role and channel that you entered last, so one Apply enters them again. Game rules, tones and the BLE protocol **v4** come from the Rust core in `crates/core`.

## Typical play

1. Flash CPB with `just flash-v7`.
2. Phone, Mac or watch: Skip to **Caller** on the same channel, Apply.
3. CPB: Picker on that channel, Apply a color → the countdown on the phone, Mac or watch resets on match.

**iOS, macOS and watchOS are Caller only.** CoreBluetooth on iOS and macOS cannot advertise manufacturer data, and watchOS cannot advertise at all. Thus mode select in these apps offers only Caller. It starts at Caller on channel 0, which matches the channel on which a board Picker starts. Skip then cycles the Caller channels 4, 3, 2, 1 and 0. Use a CPB (or Android) as Picker. **Android as Picker** can advertise.

## Build / run

All apps link the Rust core, so build it first (needs `boltffi`, see `just ffi`):

```bash
just ffi        # or just ffi-apple (alias ffi-ios) / just ffi-android
```

### Apple (iOS, macOS, watchOS)

One xcodegen project in `apps/apple` holds three apps: `ColorHurry` (iOS), `ColorHurryMac` (macOS 14+) and `ColorHurryWatch` (standalone, watchOS 10+, Apple Watch Series 6 and later).

BoltFFI 0.30.1 has no watchOS target. The `arm64_32` watch slice needs nightly Rust with `rust-src`. `scripts/xcframework_watchos.rs` adds the watchOS slices. Install the prerequisite once:

```bash
rustup component add rust-src --toolchain nightly
```

Then build the core and open the project:

```bash
just ffi-apple             # alias: just ffi-ios
cd apps/apple
xcodegen generate          # regenerates ColorHurry.xcodeproj from project.yml
open ColorHurry.xcodeproj  # or: just apple (alias: just ios)
```

To check that one app builds without signing, run `just ios-build`, `just mac-build` or `just watch-build`.

To run the watch app on a 40 mm watch simulator (the screen size of a Series 6), run `just watch-sim`. The recipe creates the simulator the first time and opens Device Hub.

To run an app on a device, or to use Bluetooth on the Mac, set your signing team. Create `apps/apple/Local.xcconfig` with this line:

```
DEVELOPMENT_TEAM = <your team ID>
```

Git ignores `Local.xcconfig` and `ColorHurry.xcodeproj`. `xcodegen generate` makes the project again from `project.yml`, so do not set the team in Xcode. Xcode does not keep it.

- **iOS:** needs the signing team for a physical device (BLE).
- **macOS:** needs the signing team for Bluetooth (sandbox and Bluetooth entitlement). Press `a` or the left arrow for Skip, `b` or Return for Apply, and Escape for Mode.
- **watchOS:** page 1 shows the ring at full screen with the mode line in its centre. Skip (bottom left) and Apply (bottom right) are corner buttons. Scroll down for the Bluetooth status and the **Change mode** button on page 2. **Change mode** goes back to mode select and scrolls back to page 1. Each tone step gives a haptic click. A real watch needs the signing team.

### Android

Needs **JDK 17 or 21** (not Homebrew OpenJDK 27, because Gradle Kotlin DSL fails to parse that version string):

```bash
export JAVA_HOME=$(/usr/libexec/java_home -v 21)
just ffi-android
cd apps/android
./gradlew :app:assembleDebug
# or: just android
```

Install the debug APK from `app/build/outputs/apk/debug/`.

## Shared logic

All four shells link the Rust core through the Crux bridge and BoltFFI (`crates/ffi`). The shells contain no game logic:

- `GameController` holds the generated `Core`, maps buttons and BLE ads onto domain events (`Skip`, `ActivateColor`, `Reset`, `ColorReceived`) and draws the `ViewModel`.
- An `EffectHandler` performs what the core asks for: `TimeNotifyAfter` and `TimeClear` (timers, through the `TaskTimeHandler` or `CoroutineTimeHandler` that `crux_time` ships), `Random` (secure random byte), `PlayTone` (audio), `Ble` (scan / advertise).
- BLE payloads are parsed and built in Rust (`color_received_event`, `color_advertisement`), so the wire format exists only once.

The iOS, macOS and watchOS shells share their Swift code in `apps/apple/Shared`. Swift and Kotlin types (`Event`, `Effect`, `ViewModel`, `Color`, …) are generated from the core with facet typegen (`crates/ffi/src/bin/codegen.rs`). The core itself stays `no_std`. Only `crates/ffi` turns on the std bridge.
