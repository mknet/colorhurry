# Spec Delta

## RENAMED Requirements
- FROM: `### Requirement: A shell can limit mode select to Receiver`
- TO: `### Requirement: A shell can limit mode select to Caller`
- FROM: `### Requirement: ReceiverOnly moves a Picker choice and stays on`
- TO: `### Requirement: CallerOnly moves a Picker choice and stays on`

## MODIFIED Requirements

### Requirement: ActivateColor enters the chosen role
ActivateColor SHALL enter Picker on channel `step` for steps 0 to 4 and Caller on channel `9 - step` for steps 5 to 9.
Acceptance: `crates/core/tests/features/mode_select.feature`.

#### Scenario: Left half enters Picker
- **WHEN** the player presses Skip twice and then ActivateColor
- **THEN** the board enters Picker on channel 2

#### Scenario: Right half enters Caller
- **WHEN** the player presses Skip six times and then ActivateColor
- **THEN** the board enters Caller on channel 3

### Requirement: The view names the chosen role and channel
In mode select, the view model SHALL carry the role (Picker or Caller) and the channel that ActivateColor would enter for the current step. On other screens the view model SHALL carry no choice.
Acceptance: `crates/core/tests/features/mode_select.feature`.

#### Scenario: Boot names Picker channel 0
- **WHEN** the board boots
- **THEN** the view model names Picker on channel 0

#### Scenario: Six Skips name Caller channel 3
- **WHEN** the player presses Skip six times
- **THEN** the view model names Caller on channel 3

### Requirement: Reset goes back to mode select
`Reset` SHALL be valid on every screen. It SHALL stop the blink, the countdown and the radio, and the core SHALL ignore the delays that were pending at `Reset`. It SHALL go to mode select with no explosion and no tone. `Reset` SHALL clear the Picker color. The next Caller entry SHALL start a new round from a new random byte. While the game is paused, the core SHALL ignore `Reset`.
Acceptance: `crates/core/tests/features/reset.feature`.

#### Scenario: Reset from a Picker that broadcasts
- **WHEN** a Picker on channel 1 broadcasts Red and the player sends `Reset`
- **THEN** the radio stops, mode select is shown, and the view model names Picker on channel 1

#### Scenario: Reset from a Caller countdown
- **WHEN** a Caller on channel 3 counts down and the player sends `Reset`
- **THEN** the radio stops, mode select is shown with Caller on channel 3, no explosion plays, and the beat that was pending has no effect

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
- **THEN** the view model still names Picker on channel 3, and the core requests a 50 ms blink delay

### Requirement: A shell can limit mode select to Caller
A shell that cannot broadcast a color SHALL send `CallerOnly` once, before `Boot`. After `CallerOnly`, mode select SHALL offer only the Caller choices (ring steps 5 to 9). Skip SHALL move to the next Caller choice in the ring order (Caller on channel 4, 3, 2, 1 and 0) and SHALL wrap from channel 0 back to channel 4. The view model SHALL never name Picker, and ActivateColor SHALL enter only Caller.
Acceptance: `crates/core/tests/features/caller_only.feature`.

#### Scenario: Boot names Caller on channel 0
- **WHEN** the shell sends `CallerOnly` and then `Boot`
- **THEN** mode select is shown, and the view model names Caller on channel 0

#### Scenario: Skip wraps among the Caller channels
- **WHEN** the shell sends `CallerOnly` and `Boot`, and the player presses Skip once
- **THEN** the view model names Caller on channel 4, and four more Skips name Caller on channel 0 again

#### Scenario: Apply enters Caller
- **WHEN** the shell sends `CallerOnly` and `Boot`, and the player presses Skip twice and then ActivateColor
- **THEN** the game enters Caller on channel 3 and starts a round

### Requirement: Reset and the full ring stay the same
In a caller-only game, `Reset` SHALL go back to the last Caller choice, and Skip SHALL then move to the next Caller choice. Without `CallerOnly`, mode select SHALL offer all ten ring steps as before.
Acceptance: `crates/core/tests/features/caller_only.feature`.

#### Scenario: Reset goes back to the last Caller choice
- **WHEN** a caller-only game is in Caller on channel 3 and the player sends `Reset`
- **THEN** mode select names Caller on channel 3, and Skip then names Caller on channel 2

#### Scenario: Picker stays available without CallerOnly
- **WHEN** the board boots without `CallerOnly`
- **THEN** the view model names Picker on channel 0

### Requirement: CallerOnly moves a Picker choice and stays on
If the current choice is Picker when `CallerOnly` arrives, mode select SHALL move to Caller on channel 0 (ring step 9), the channel on which the board Picker starts. A Caller choice SHALL not change. `CallerOnly` SHALL be idempotent, and no event SHALL turn it off. The core SHALL accept `CallerOnly` also while the game is paused. `CallerOnly` SHALL not change the current screen.
Acceptance: `crates/core/tests/features/caller_only.feature`.

#### Scenario: CallerOnly when the choice is Picker
- **WHEN** the player presses Skip twice (Picker on channel 2) and the shell then sends `CallerOnly`
- **THEN** the view model names Caller on channel 0, and Skip then names Caller on channel 4

#### Scenario: CallerOnly a second time
- **WHEN** the shell sends `CallerOnly` again in a caller-only game
- **THEN** the choice does not change, and the core requests no tone, no delay and no BLE op

#### Scenario: CallerOnly while paused
- **WHEN** the game is paused in mode select and the shell sends `CallerOnly`
- **THEN** the core accepts it with no effect request, and after `Resume` mode select offers only Caller
