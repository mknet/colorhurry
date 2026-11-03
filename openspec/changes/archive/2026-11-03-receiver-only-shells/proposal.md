# Proposal

## Why

The Apple apps cannot be Picker. CoreBluetooth on iOS and macOS cannot advertise manufacturer data, and watchOS cannot advertise at all. But mode select lets the player choose Picker in these apps, and then the app sends nothing. A player put the watch in Picker, and the board Receiver saw no color.

## What Changes

- The core gets a new shell event, `ReceiverOnly`, at the end of `Event`. The bincode indices of the other events do not change.
- A shell that cannot broadcast a color sends `ReceiverOnly` once, before `Boot`.
- After `ReceiverOnly`, mode select offers only the Receiver choices. Skip cycles the Receiver channels in the ring order (4, 3, 2, 1, 0) and wraps from channel 0 to channel 4. The view model never names Picker, and Apply enters only Receiver.
- If the current choice is Picker when `ReceiverOnly` arrives, mode select moves to Receiver on channel 0. The board Picker starts on channel 0, so an Apple app opens on the matching Receiver. A Receiver choice does not change.
- `ReceiverOnly` is idempotent, and no event turns it off.
- The core accepts `ReceiverOnly` also while the game is paused. It is a fact about the shell, not a game action.
- `Reset` goes back to the last Receiver choice, as before.
- The mode-select LED ring does not change. A Receiver choice shows the five channel colors and blinks the LED of the choice in its channel color.
- The iOS, macOS and watchOS apps get `Platform.canBroadcast`, which is false. `GameController.boot()` sends `ReceiverOnly` before `Boot`.
- Without `ReceiverOnly`, nothing changes. The board, the Android app and the web shell never send it.
- The `ViewModel` does not change.

## Capabilities

### New Capabilities

### Modified Capabilities
- `mode-select`: a shell can limit mode select to the Receiver choices
- `shell-integration`: the Apple apps send `ReceiverOnly`, and `ReceiverOnly` joins the events that a shell can send

## Impact

- `crates/core/src/lib.rs`, `model.rs` and `palette.rs`: the new event, the flag and the Skip rule.
- `crates/core/tests/features/receiver_only.feature` and two new cucumber steps.
- Regenerated Swift and Kotlin types.
- Apple: `canBroadcast` in the three `Platform.swift` files, and `boot()` in `GameController`.
- `README.md` and `apps/README.md`.
- The board firmware, the Android app and the web shell do not change.
