# Tasks

## 1. Core

- [x] 1.1 Add `Event::ReceiverOnly` at the end of `Event`
- [x] 1.2 Store the flag in the model, move a Picker choice to Receiver on channel 0, and accept the event also while paused
- [x] 1.3 Make Skip in mode select cycle only the Receiver steps when the flag is set
- [x] 1.4 Add `crates/core/tests/features/receiver_only.feature` with the step definitions, verify with `just test`

## 2. Shells

- [x] 2.1 Regenerate the Swift types with `just ffi-apple`
- [x] 2.2 Add `canBroadcast` to `iOS/Platform.swift`, `macOS/Platform.swift` and `watchOS/Platform.swift`
- [x] 2.3 Make `GameController.boot()` send `ReceiverOnly` before `Boot` when `canBroadcast` is false
- [x] 2.4 Verify the iOS, macOS and watchOS builds
- [x] 2.5 Regenerate the Kotlin types with `just ffi-android`
- [ ] 2.6 Verify the Android build with `./gradlew :app:assembleDebug`
- [x] 2.7 Verify the web shell with `just web-build`
- [x] 2.8 Verify that the board firmware builds with `just build`
- [x] 2.9 Describe the Receiver-only Apple apps in `README.md` and `apps/README.md`

## 3. Device checks

- [ ] 3.1 Open the watch app, and check that mode select names Receiver on channel 0 and that Skip cycles only the Receiver channels
- [ ] 3.2 Open the iOS app, and check that mode select names only Receiver and that a board Picker on the same channel completes a round
- [ ] 3.3 Open the Android app, and check that mode select still offers Picker
