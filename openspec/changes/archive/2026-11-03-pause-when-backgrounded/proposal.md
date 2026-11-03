# Proposal

## Why

The apps keep the game running when they go to the background. On the watch, the user dismisses the app, but the rounds continue and the watch plays a haptic click for each tone step. The other apps also keep their timers and the radio active when nobody looks at them.

## What Changes

- The core gets two shell events, `Pause` and `Resume`.
- `Pause` stops every timer of the core and switches the radio off. While the game is paused, the core ignores every event except `Resume`.
- `Resume` sends the radio op of the current screen again and restarts its timers. A countdown continues at the same step, and the next beat comes one full beat period after `Resume`. A Receiver with no running round requests a new random byte. Thus a pause during the explosion starts the next round on `Resume`, and the explosion does not play again.
- A random byte that arrives while the game is paused has no effect. `Resume` requests a new byte, so the game cannot stop in the waiting state.
- The iOS, macOS and watchOS apps send `Pause` when the scene goes to the background and `Resume` when it is active again. They ignore the inactive phase, so a lowered wrist or the iOS Control Center does not pause the game. On `Pause`, the apps stop the tones that are queued and cancel the haptic clicks that are scheduled.
- The Android app sends `Pause` in `onStop` and `Resume` in `onStart`, and stops the tone that plays.
- The web shell sends `Pause` when the tab is hidden and `Resume` when it is visible again.
- The board never sends `Pause` or `Resume`, and its behavior does not change.
- The `ViewModel` does not change.

## Capabilities

### New Capabilities

### Modified Capabilities
- `shell-integration`: the apps pause the game in the background, and `Pause` and `Resume` join the events that a shell can send
- `receiver-round`: a paused round continues where it stopped
- `ble-protocol`: `Pause` switches the radio off, and `Resume` restores the radio op of the screen
- `timing-and-tones`: `Pause` stops the blink and beat timers, and the core ignores input while it is paused

## Impact

- `crates/core/src/lib.rs` and `crates/core/src/model.rs`: the new events, the `paused` flag, and the pause and resume logic.
- `crates/core/tests/features/pause.feature` and the cucumber steps. The test harness can hold a `Random` request in flight.
- Regenerated Swift and Kotlin types.
- Apple: a shared view modifier in `apps/apple/Shared/UI/PauseInBackground.swift`, `pause()` and `resume()` in `GameController`, `stop()` in `TonePlayer`, and a cancellation check for the haptic clicks in each `Platform.swift`.
- Android: `MainActivity`, `GameController` and `TonePlayer`.
- Web: a `visibilitychange` handler in `crates/web/src/lib.rs`.
- The board firmware does not change.
