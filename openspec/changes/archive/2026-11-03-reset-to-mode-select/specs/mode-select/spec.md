# Spec Delta

## ADDED Requirements

### Requirement: Reset goes back to mode select
`Reset` SHALL be valid on every screen. It SHALL stop the blink, the countdown and the radio, and the core SHALL ignore the delays that were pending at `Reset`. It SHALL go to mode select with no explosion and no tone. `Reset` SHALL clear the Picker color. The next Receiver entry SHALL start a new round from a new random byte. While the game is paused, the core SHALL ignore `Reset`.
Acceptance: `crates/core/tests/features/reset.feature`.

#### Scenario: Reset from a Picker that broadcasts
- **WHEN** a Picker on channel 1 broadcasts Red and the player sends `Reset`
- **THEN** the radio stops, mode select is shown, and the view model names Picker on channel 1

#### Scenario: Reset from a Receiver countdown
- **WHEN** a Receiver on channel 3 counts down and the player sends `Reset`
- **THEN** the radio stops, mode select is shown with Receiver on channel 3, no explosion plays, and the beat that was pending has no effect

#### Scenario: Reset while paused
- **WHEN** the game is paused during a countdown and the shell sends `Reset`
- **THEN** nothing changes and no effect is requested

### Requirement: Mode select starts at the last choice after Reset
After `Reset`, mode select SHALL start at the role and channel that the player entered last. ActivateColor SHALL then enter them again, and Skip SHALL move on to the next step. In mode select, `Reset` SHALL keep the current choice and restart the blink.
Acceptance: `crates/core/tests/features/reset.feature`.

#### Scenario: Apply after Reset
- **WHEN** the player sends `Reset` from a Receiver on channel 3 and then presses ActivateColor
- **THEN** the board enters Receiver on channel 3 and starts a new round

#### Scenario: Skip after Reset
- **WHEN** the player sends `Reset` from a Picker on channel 4 and then presses Skip
- **THEN** the view model names Receiver on channel 4

#### Scenario: Reset in mode select
- **WHEN** the player presses Skip three times and then sends `Reset`
- **THEN** the view model still names Picker on channel 3, and the core requests a 50 ms blink delay
