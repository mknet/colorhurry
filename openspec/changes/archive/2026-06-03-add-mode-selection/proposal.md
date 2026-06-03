# Proposal

## Why

Color Hurry is a cooperative game for two roles: the Picker announces a color, the Receiver has to get it before its countdown runs out. Today every board is a picker. Players need to choose a role and a channel on the board itself, so several pairs can play in the same room.

## What Changes

- Boot into mode select: the LED ring shows ten steps, Skip moves to the next step, ActivateColor enters the chosen role.
- Steps 0 to 4 enter Picker on channel 0 to 4, steps 5 to 9 enter Receiver on channel 4 to 0.
- A first Receiver round without radio: random target color from the spectrum, ten-second countdown (one LED off per beat, descending tone), then the next round.
- One shared palette: five channel colors for mode select, ten named spectrum colors (White to Pink) for the picker and the receiver's random pool.

## Capabilities

### New Capabilities
- `mode-select`: choosing role and channel at boot
- `receiver-round`: the Receiver's countdown round against a target color

### Modified Capabilities
- `picker`: the spectrum becomes the shared named palette; the picker is entered from mode select and starts on its channel index

## Impact

- New `mode_select`, `receiver`, `countdown`, `palette`, `game` modules in the firmware.
- `docs/COLOR-PICKER.md`, `docs/RECEIVER.md`.
