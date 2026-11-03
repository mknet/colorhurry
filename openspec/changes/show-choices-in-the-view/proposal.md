# Proposal

## Why

In mode select, Skip moves a blinking LED through ten ring steps. Steps 0 to 4 mean Picker on channel `step`, and steps 5 to 9 mean Receiver on channel `9 - step`. Players cannot remember this mapping, so they cannot tell what Apply will enter.

## What Changes

- The core adds `mode_choice` to the `ViewModel`. It names the role and channel that ActivateColor would enter, and it is empty on other screens.
- The core also adds `picker_color` and `picker_applied` to the `ViewModel`. In Picker they name the selected spectrum color and say whether Apply locked it. They are empty on other screens. The Receiver target stays unnamed.
- The iOS, macOS, watchOS and Android shells show both as text, for example "Apply enters Receiver ch0 · Skip for next". The shells only format the value and do not read game state.

## Capabilities

### New Capabilities

### Modified Capabilities
- `mode-select`: the view names the chosen role and channel
- `picker`: the view names the selected color

## Impact

- `crates/core/src/view.rs`: new `ModeChoice` and `ModeRole` types and the `mode_choice`, `picker_color` and `picker_applied` fields.
- Regenerated Swift and Kotlin types, and `modeHint` in both `GameController` files.
