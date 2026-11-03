# Tasks

## 1. Core

- [x] 1.1 Add `Event::Reset` at the end of `Event`
- [x] 1.2 Make `Reset` stop the timers, the round and the radio, and go to mode select at the last choice, and keep the rule that the core ignores it while paused
- [x] 1.3 Add `crates/core/tests/features/reset.feature` with the step definition, verify with `just test`

## 2. Shells

- [x] 2.1 Regenerate the Swift types with `just ffi-apple`
- [x] 2.2 Add `reset()` to the Apple `GameController`, which stops the queued tones and haptics and sends `Reset`
- [x] 2.3 Add the Mode button to `ContentView` (iOS and macOS), and the Escape shortcut to `MacContentView`
- [x] 2.4 Add the Change mode button to the watch status page, and scroll back to the game page after the press
- [x] 2.5 Verify the iOS, macOS and watchOS builds
- [x] 2.6 Regenerate the Kotlin types with `just ffi-android`, and add `reset()` and the Mode button to the Android app
- [ ] 2.7 Verify the Android build with `./gradlew :app:assembleDebug`
- [x] 2.8 Add the Mode button to the web shell, verify with `just web-build`
- [x] 2.9 Verify that the board firmware builds with `just build`
- [x] 2.10 Describe the Mode button in `README.md` and `apps/README.md`

## 3. Device checks

- [ ] 3.1 Press Change mode on the watch during a countdown, and check that the tones and haptic clicks stop and the game page shows mode select at the last choice
- [ ] 3.2 Press Mode on the iOS and Android apps during a countdown, and check that the tones stop and one Apply enters the same Receiver again
- [ ] 3.3 Press Mode on an Android Picker that broadcasts, and check that a board Receiver no longer sees the color
