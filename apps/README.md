# Color Hurry native apps

| App | Stack | Path |
|-----|--------|------|
| iOS | SwiftUI + CoreBluetooth | [`ios/`](ios/) |
| Android | Kotlin + Jetpack Compose + Android BLE | [`android/`](android/) |

Both mirror the board entry (**Skip** / **Apply** → Mode select → Picker or Receiver) and the web dark LED-ring UI. Game rules, tones and the BLE protocol **v4** come from the Rust core in `crates/core`.

## Typical play

1. Flash CPB with `just flash-v7`.
2. Phone: Skip to **Receiver** on the same channel, Apply.
3. CPB: Picker on that channel, Apply a color → phone countdown resets on match.

**iOS as Picker:** CoreBluetooth cannot put Adafruit manufacturer data on the air the way the board expects; use a CPB (or Android) as Picker. **Android as Picker** can advertise.

## Build / run

Both apps link the Rust core, so build it first (needs `boltffi`, see `just ffi`):

```bash
just ffi        # or just ffi-ios / just ffi-android
```

### iOS

```bash
just ffi-ios
cd apps/ios
xcodegen generate          # regenerates ColorHurry.xcodeproj from project.yml
open ColorHurry.xcodeproj  # or: just ios
```

Needs a Development Team in Xcode signing for a physical device (BLE).

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

Both apps link the Rust core through the Crux bridge and BoltFFI (`crates/ffi`). The shells contain no game logic:

- `GameController` holds the generated `Core`, maps buttons and BLE ads onto domain events (`Skip`, `ActivateColor`, `ColorReceived`) and draws the `ViewModel`.
- An `EffectHandler` performs what the core asks for: `Delay` (timer), `Random` (secure random byte), `PlayTone` (audio), `Ble` (scan / advertise).
- BLE payloads are parsed and built in Rust (`color_received_event`, `color_advertisement`), so the wire format exists only once.

Swift and Kotlin types (`Event`, `Effect`, `ViewModel`, `Color`, …) are generated from the core with facet typegen (`crates/ffi/src/bin/codegen.rs`). The core itself stays `no_std`; only `crates/ffi` turns on the std bridge.
