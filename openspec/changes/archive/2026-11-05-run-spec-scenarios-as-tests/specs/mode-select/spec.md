# Spec Delta

## MODIFIED Requirements

### Requirement: Boot into mode select
After boot the board SHALL show mode select: the five channel colors on the ring, with the LED of the current step blinking. The first step is Picker on channel 0.

#### Scenario: Boot lands on mode select
- **WHEN** the board boots
- **THEN** mode select is shown
- **AND** LED 0 blinks

### Requirement: Skip cycles the ring step
Skip SHALL move to the next of ten ring steps and SHALL wrap from step 9 back to step 0.

#### Scenario: Ten Skips wrap around
- **GIVEN** the board in mode select
- **WHEN** the player presses Skip ten times
- **AND** the player presses ActivateColor
- **THEN** the board is the Picker on channel 0

### Requirement: ActivateColor enters the chosen role
ActivateColor SHALL enter Picker on channel `step` for steps 0 to 4 and Caller on channel `9 - step` for steps 5 to 9.

#### Scenario: Left half enters Picker
- **GIVEN** the board in mode select
- **WHEN** the player presses Skip twice
- **AND** the player presses ActivateColor
- **THEN** the board is the Picker on channel 2

#### Scenario: Right half enters Caller
- **GIVEN** the board in mode select
- **WHEN** the player presses Skip six times
- **AND** the player presses ActivateColor
- **THEN** the board is the Caller on channel 3

### Requirement: The view names the chosen role and channel
In mode select, the view model SHALL carry the role (Picker or Caller) and the channel that ActivateColor would enter for the current step. On other screens the view model SHALL carry no choice.

#### Scenario: Boot names Picker channel 0
- **WHEN** the board boots
- **THEN** the choice is Picker on channel 0

#### Scenario: Six Skips name Caller channel 3
- **GIVEN** the board in mode select
- **WHEN** the player presses Skip six times
- **THEN** the choice is Caller on channel 3

#### Scenario: Entering a role clears the choice
- **GIVEN** the board in mode select
- **WHEN** the player presses ActivateColor
- **THEN** the view names no choice

### Requirement: Reset goes back to mode select
`Reset` SHALL be valid on every screen. It SHALL stop the blink, the countdown and the radio, and the core SHALL clear the timers that are pending at `Reset`. It SHALL go to mode select with no explosion and no tone. `Reset` SHALL clear the Picker color. The next Caller entry SHALL start a new round from a new random byte. While the game is paused, the core SHALL ignore `Reset`.

#### Scenario: Reset from a Picker that selects a color
- **GIVEN** the Picker on channel 2 with Yellow selected
- **WHEN** the player sends Reset
- **THEN** mode select is shown
- **AND** the choice is Picker on channel 2
- **AND** the view names no Picker color
- **AND** the radio is off
- **AND** a 50 ms timer is requested
- **AND** no tone plays

#### Scenario: Reset from a Picker that broadcasts
- **GIVEN** the Picker on channel 1 with Red applied
- **WHEN** the player sends Reset
- **THEN** mode select is shown
- **AND** the choice is Picker on channel 1
- **AND** the view names no Picker color
- **AND** the radio is off

#### Scenario: Reset from a Caller countdown
- **GIVEN** the Caller on channel 3 with target Red
- **AND** two beats have passed
- **WHEN** the player sends Reset
- **THEN** mode select is shown
- **AND** the choice is Caller on channel 3
- **AND** the countdown stops
- **AND** the radio is off
- **AND** no explosion plays
- **AND** the core clears the pending beat timer
- **WHEN** 12000 ms have passed
- **THEN** the choice is Caller on channel 3
- **AND** no tone plays

#### Scenario: Reset while paused
- **GIVEN** the Caller on channel 0 with target Red
- **AND** the app is in the background
- **WHEN** the player sends Reset
- **THEN** nothing changes

### Requirement: Mode select starts at the last choice after Reset
After `Reset`, mode select SHALL start at the role and channel that the player entered last. ActivateColor SHALL then enter them again, and Skip SHALL move on to the next step. In mode select, `Reset` SHALL keep the current choice and restart the blink.

#### Scenario: Apply after Reset enters the same Caller
- **GIVEN** the Caller on channel 3 with target Red
- **AND** three beats have passed
- **WHEN** the player sends Reset
- **AND** the player presses ActivateColor
- **AND** the shell answers the random request with 4
- **THEN** the board is the Caller on channel 3
- **AND** the target is LightGreen
- **AND** the countdown begins

#### Scenario: Apply after Reset enters the same Picker
- **GIVEN** the Picker on channel 4 with LightGreen applied
- **WHEN** the player sends Reset
- **AND** the player presses ActivateColor
- **THEN** the board is the Picker on channel 4
- **AND** the view names LightGreen as selected
- **AND** the board advertises on channel 4 without a color

#### Scenario: Skip after Reset
- **GIVEN** the Picker on channel 4
- **WHEN** the player sends Reset
- **AND** the player presses Skip
- **THEN** the choice is Caller on channel 4

#### Scenario: Reset in mode select
- **GIVEN** the board in mode select with Picker on channel 3 chosen
- **WHEN** the player sends Reset
- **THEN** the choice is Picker on channel 3
- **AND** a 50 ms timer is requested
- **AND** no tone plays

### Requirement: A shell can limit mode select to Caller
A shell that cannot broadcast a color SHALL send `CallerOnly` once, before `Boot`. After `CallerOnly`, mode select SHALL offer only the Caller choices (ring steps 5 to 9). Skip SHALL move to the next Caller choice in the ring order (Caller on channel 4, 3, 2, 1 and 0) and SHALL wrap from channel 0 back to channel 4. The view model SHALL never name Picker, and ActivateColor SHALL enter only Caller.

#### Scenario: Boot names Caller on channel 0
- **WHEN** the shell sends CallerOnly
- **AND** the board boots
- **THEN** mode select is shown
- **AND** the choice is Caller on channel 0

#### Scenario: Skip wraps among the Caller channels
- **GIVEN** a caller-only game in mode select
- **WHEN** the player presses Skip once
- **THEN** the choice is Caller on channel 4
- **WHEN** the player presses Skip four times
- **THEN** the choice is Caller on channel 0
- **WHEN** the player presses Skip once
- **THEN** the choice is Caller on channel 4

#### Scenario: Skip sounds in a caller-only game
- **GIVEN** a caller-only game in mode select
- **WHEN** the player presses Skip
- **THEN** a 380 Hz tone plays for 55 ms

#### Scenario: Apply enters Caller
- **GIVEN** a caller-only game in mode select
- **WHEN** the player presses Skip twice
- **AND** the player presses ActivateColor
- **AND** the shell answers the random request with 1
- **THEN** the board is the Caller on channel 3
- **AND** the board scans channel 3 for Red

#### Scenario: Apply without Skip enters Caller on channel 0
- **GIVEN** a caller-only game in mode select
- **WHEN** the player presses ActivateColor
- **THEN** the board is the Caller on channel 0

### Requirement: Reset and the full ring stay the same
In a caller-only game, `Reset` SHALL go back to the last Caller choice, and Skip SHALL then move to the next Caller choice. Without `CallerOnly`, mode select SHALL offer all ten ring steps as before.

#### Scenario: Reset goes back to the last Caller choice
- **GIVEN** a caller-only game in mode select
- **WHEN** the player presses Skip twice
- **AND** the player presses ActivateColor
- **AND** the player sends Reset
- **THEN** mode select is shown
- **AND** the choice is Caller on channel 3
- **WHEN** the player presses Skip
- **THEN** the choice is Caller on channel 2

#### Scenario: Reset after Caller on channel 0 wraps to channel 4
- **GIVEN** a caller-only game in mode select
- **WHEN** the player presses ActivateColor
- **AND** the player sends Reset
- **AND** the player presses Skip
- **THEN** the choice is Caller on channel 4

#### Scenario: Picker stays available without CallerOnly
- **GIVEN** the board in mode select
- **WHEN** the player presses Skip four times
- **THEN** the choice is Picker on channel 4
- **WHEN** the player presses Skip
- **THEN** the choice is Caller on channel 4

### Requirement: CallerOnly moves a Picker choice and stays on
If the current choice is Picker when `CallerOnly` arrives, mode select SHALL move to Caller on channel 0 (ring step 9), the channel on which the board Picker starts. A Caller choice SHALL not change. `CallerOnly` SHALL be idempotent, and no event SHALL turn it off. The core SHALL accept `CallerOnly` also while the game is paused. `CallerOnly` SHALL not change the current screen.

#### Scenario: CallerOnly when the choice is Picker
- **GIVEN** the board in mode select with Picker on channel 2 chosen
- **WHEN** the shell sends CallerOnly
- **THEN** mode select is shown
- **AND** the choice is Caller on channel 0
- **WHEN** the player presses Skip
- **THEN** the choice is Caller on channel 4

#### Scenario: CallerOnly keeps a Caller choice
- **GIVEN** the board in mode select with Caller on channel 2 chosen
- **WHEN** the shell sends CallerOnly
- **THEN** the choice is Caller on channel 2

#### Scenario: CallerOnly a second time
- **GIVEN** a caller-only game in mode select with Caller on channel 2 chosen
- **WHEN** the shell sends CallerOnly
- **THEN** nothing changes

#### Scenario: CallerOnly while paused
- **GIVEN** the board in mode select
- **AND** the app is in the background
- **WHEN** the shell sends CallerOnly
- **THEN** the choice is Caller on channel 0
- **AND** no effect is requested
- **WHEN** the app comes back
- **AND** the player presses Skip
- **THEN** the choice is Caller on channel 4
