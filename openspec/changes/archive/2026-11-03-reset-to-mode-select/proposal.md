# Proposal

## Why

After a player enters Picker or Receiver in an app, the app has no way back to mode select. The player must close the app and open it again to change the role or the channel.

## What Changes

- The core gets a new shell event, `Reset`, at the end of `Event`. The bincode indices of the other events do not change.
- `Reset` is valid on every screen. It stops every timer, the round and the radio, and goes to mode select. No explosion plays.
- Mode select then starts at the role and channel that the player entered last. One Apply enters them again, and Skip moves on from them.
- `Reset` clears the Picker color. The next Receiver entry starts a new round, as it does today.
- `Reset` in mode select keeps the choice and restarts the blink.
- While the game is paused, the core ignores `Reset`, as it ignores every event except `Resume`.
- The iOS, macOS and Android apps get a small **Mode** button in the header. On macOS, the Escape key does the same.
- The watchOS app gets a **Change mode** button on the status page, under the Bluetooth status. After the press, the app scrolls back to the game page. The game page keeps only Skip and Apply.
- On `Reset`, the apps stop the tones that are queued and cancel the haptic clicks, as they do on `Pause`.
- The web shell gets a **Mode** button next to Skip and Apply.
- The board never sends `Reset`, and its behavior does not change.
- The `ViewModel` does not change.

## Capabilities

### New Capabilities

### Modified Capabilities
- `mode-select`: `Reset` goes back to mode select at the last choice
- `shell-integration`: the apps offer a Mode button, and `Reset` joins the events that a shell can send

## Impact

- `crates/core/src/lib.rs`: the new event and the reset logic.
- `crates/core/tests/features/reset.feature` and one new cucumber step.
- Regenerated Swift and Kotlin types.
- Apple: `reset()` in `GameController`, the Mode button in `ContentView`, the Escape shortcut in `MacContentView`, and the Change mode button and the page selection in `WatchContentView`.
- Android: `reset()` in `GameController` and the Mode button in `HomeScreen`.
- Web: the Mode button in `crates/web/src/lib.rs`.
- `README.md` and `apps/README.md`.
- The board firmware does not change.
