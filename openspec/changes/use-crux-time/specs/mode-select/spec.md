# Spec Delta

## MODIFIED Requirements

### Requirement: Reset goes back to mode select
`Reset` SHALL be valid on every screen. It SHALL stop the blink, the countdown and the radio, and the core SHALL clear the timers that are pending at `Reset`. It SHALL go to mode select with no explosion and no tone. `Reset` SHALL clear the Picker color. The next Caller entry SHALL start a new round from a new random byte. While the game is paused, the core SHALL ignore `Reset`.
Acceptance: `crates/core/tests/features/reset.feature`.

#### Scenario: Reset from a Picker that broadcasts
- **WHEN** a Picker on channel 1 broadcasts Red and the player sends `Reset`
- **THEN** the radio stops, mode select is shown, and the view model names Picker on channel 1

#### Scenario: Reset from a Caller countdown
- **WHEN** a Caller on channel 3 counts down and the player sends `Reset`
- **THEN** the radio stops, mode select is shown with Caller on channel 3, no explosion plays, and the core clears the beat timer that was pending

#### Scenario: Reset while paused
- **WHEN** the game is paused during a countdown and the shell sends `Reset`
- **THEN** nothing changes and no effect is requested

### Requirement: Mode select starts at the last choice after Reset
After `Reset`, mode select SHALL start at the role and channel that the player entered last. ActivateColor SHALL then enter them again, and Skip SHALL move on to the next step. In mode select, `Reset` SHALL keep the current choice and restart the blink.
Acceptance: `crates/core/tests/features/reset.feature`.

#### Scenario: Apply after Reset
- **WHEN** the player sends `Reset` from a Caller on channel 3 and then presses ActivateColor
- **THEN** the board enters Caller on channel 3 and starts a new round

#### Scenario: Skip after Reset
- **WHEN** the player sends `Reset` from a Picker on channel 4 and then presses Skip
- **THEN** the view model names Caller on channel 4

#### Scenario: Reset in mode select
- **WHEN** the player presses Skip three times and then sends `Reset`
- **THEN** the view model still names Picker on channel 3, and the core requests a 50 ms blink timer

### Requirement: CallerOnly moves a Picker choice and stays on
If the current choice is Picker when `CallerOnly` arrives, mode select SHALL move to Caller on channel 0 (ring step 9), the channel on which the board Picker starts. A Caller choice SHALL not change. `CallerOnly` SHALL be idempotent, and no event SHALL turn it off. The core SHALL accept `CallerOnly` also while the game is paused. `CallerOnly` SHALL not change the current screen.
Acceptance: `crates/core/tests/features/caller_only.feature`.

#### Scenario: CallerOnly when the choice is Picker
- **WHEN** the player presses Skip twice (Picker on channel 2) and the shell then sends `CallerOnly`
- **THEN** the view model names Caller on channel 0, and Skip then names Caller on channel 4

#### Scenario: CallerOnly a second time
- **WHEN** the shell sends `CallerOnly` again in a caller-only game
- **THEN** the choice does not change, and the core requests no tone, no timer and no BLE op

#### Scenario: CallerOnly while paused
- **WHEN** the game is paused in mode select and the shell sends `CallerOnly`
- **THEN** the core accepts it with no effect request, and after `Resume` mode select offers only Caller
