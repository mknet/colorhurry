# mode-select Specification

## Purpose
Lets each player choose a role (Picker or Receiver) and a channel on the board, so several pairs can play in one room.

## Requirements

### Requirement: Boot into mode select
After boot the board SHALL show mode select: the five channel colors on the ring, with the LED of the current step blinking. The first step is Picker on channel 0.
Acceptance: `crates/core/tests/features/mode_select.feature`.

#### Scenario: Boot lands on mode select
- **WHEN** the board boots
- **THEN** mode select is shown with step 0 (Picker, channel 0) blinking

### Requirement: Skip cycles the ring step
Skip SHALL move to the next of ten ring steps and SHALL wrap from step 9 back to step 0.
Acceptance: `crates/core/tests/features/mode_select.feature`.

#### Scenario: Ten Skips wrap around
- **WHEN** the player presses Skip ten times and then ActivateColor
- **THEN** the board enters Picker on channel 0

### Requirement: ActivateColor enters the chosen role
ActivateColor SHALL enter Picker on channel `step` for steps 0 to 4 and Receiver on channel `9 - step` for steps 5 to 9.
Acceptance: `crates/core/tests/features/mode_select.feature`.

#### Scenario: Left half enters Picker
- **WHEN** the player presses Skip twice and then ActivateColor
- **THEN** the board enters Picker on channel 2

#### Scenario: Right half enters Receiver
- **WHEN** the player presses Skip six times and then ActivateColor
- **THEN** the board enters Receiver on channel 3

### Requirement: The view names the chosen role and channel
In mode select, the view model SHALL carry the role (Picker or Receiver) and the channel that ActivateColor would enter for the current step. On other screens the view model SHALL carry no choice.
Acceptance: `crates/core/tests/features/mode_select.feature`.

#### Scenario: Boot names Picker channel 0
- **WHEN** the board boots
- **THEN** the view model names Picker on channel 0

#### Scenario: Six Skips name Receiver channel 3
- **WHEN** the player presses Skip six times
- **THEN** the view model names Receiver on channel 3

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
