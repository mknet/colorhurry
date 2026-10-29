# Proposal

## Why

The shared core is shared in name only. The board decides matches in its radio layer, both shells run their own random generator and their own beat timers, the explosion sounds different on board and browser, and the shells read model fields to decide what to do next. The inventory in `docs/CORE.md` lists every violation. Porting the core to a Crux `App` makes the model private and turns every outside interaction into an effect the shell has to perform.

## What Changes

- `impl App for ColorHurry` on Crux (`spike/no-std`, `no_std` + `alloc`): private `Model`, public `ViewModel`, `Command<Effect, Event>`.
- Domain events instead of hardware words: `Skip`, `ActivateColor`, `ColorReceived { channel, color }`, `RandomByte`, `BlinkDone`, `BeatDone`.
- Effects: `Render`, `Delay`, `Random`, `PlayTone`, `Ble`. The core owns blink and beat periods and requests randomness from the shell.
- Round as a state: `Waiting` or `Running { target, step }` instead of two flags.
- Tones (Hz and ms, including the explosion) defined once in the core.
- **BREAKING** Protocol v4: named color index on the wire instead of RGB. Encode and decode live in the core; the match compares named colors.
- Board and web shell only parse advertisements and forward them; the core alone decides a match.

## Capabilities

### New Capabilities

### Modified Capabilities
- `ble-protocol`: v4 payload with color index, encode and decode shared by all shells
- `receiver-round`: target from shell entropy, match on named color decided only by the core
- `timing-and-tones`: one tone definition for all shells, blink and beat periods owned by the game
- `shell-integration`: shells only perform effects and never read game state

## Impact

- `crates/core` rewritten around `ColorHurry`; new `protocol.rs`, `tones.rs`.
- `crates/cpb` and `crates/web` adapt to `process_event` / `resolve`.
- v3 boards and v4 boards cannot play together; board firmware and web shell ship together.
