# Tasks

## 1. Core

- [x] 1.1 Add `Event::Pause` and `Event::Resume` at the end of `Event`, and add the `paused` flag to `Model`
- [x] 1.2 Make `Pause` invalidate the blink and beat generations and send `Ble` `Off`, and make the core ignore every event except `Resume` while it is paused
- [x] 1.3 Make `Resume` send the radio op of the current screen again, restart the blink, continue the countdown one beat period later, and request a random byte for a Receiver with no running round
- [x] 1.4 Add `crates/core/tests/features/pause.feature` with the step definitions, and let the harness hold a `Random` request in flight, verify with `just test`

## 2. Shells

- [x] 2.1 Regenerate the Swift types with `just ffi-apple`
- [x] 2.2 Add the shared `pausesInBackground` view modifier, and use it in `ContentView` (iOS and macOS) and `WatchContentView`
- [x] 2.3 Add `pause()` and `resume()` to the Apple `GameController`, and `stop()` to `TonePlayer`, which drops the queued buffers and cancels the scheduled haptic clicks
- [x] 2.4 Verify the iOS, macOS and watchOS builds
- [x] 2.5 Regenerate the Kotlin types with `just ffi-android`, and pause and resume the Android game in `onStop` and `onStart`
- [ ] 2.6 Verify the Android build with `./gradlew :app:assembleDebug`
- [x] 2.7 Send `Pause` and `Resume` from the web shell on `visibilitychange`, verify with `just web-build`
- [x] 2.8 Verify that the board firmware builds with `just build`

## 3. Device checks

- [ ] 3.1 Dismiss the watch app during a countdown, and check that no haptic click or tone comes after it
- [ ] 3.2 Lower the wrist during a countdown, and check that the countdown continues
- [ ] 3.3 Open the watch app again, and check that the countdown continues at the same step
- [ ] 3.4 Send the iOS and Android apps to the background during a countdown, and check that the tones stop and the countdown continues when they come back
