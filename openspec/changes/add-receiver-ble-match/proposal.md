# Proposal

## Why

The Receiver round has no way to win yet: the countdown always runs out. The core idea of the game is that the Picker sends the Receiver's target color in time. For that the Receiver has to hear the Picker, and several pairs in one room must not disturb each other.

## What Changes

- The radio starts only after mode select, in the chosen role.
- The Picker broadcasts only the applied color (after ActivateColor) and withdraws it on the next Skip or ActivateColor.
- The Receiver scans during the countdown. A matching color (ColorReceived) completes the task: the countdown stops, no explosion, next round.
- When the countdown expires without a match, the Receiver plays the explosion before the next round.
- Skip on the Receiver aborts the current round.
- **BREAKING** Protocol v3: the payload carries the channel (0 to 4) before the RGB bytes; Picker and Receiver only talk on the same channel.

## Capabilities

### New Capabilities

### Modified Capabilities
- `ble-protocol`: v3 payload with channel, broadcast only after apply, radio after mode select, receiver scan
- `receiver-round`: match, abort and explosion

## Impact

- `src/ble_broadcast.rs` (scanner, channel filter), `src/receiver.rs`, `src/countdown.rs` (abort callback), `src/game.rs`.
- Boards on v2 firmware cannot play with v3 boards.
