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

### Requirement: A shell can limit mode select to Receiver
A shell that cannot broadcast a color SHALL send `ReceiverOnly` once, before `Boot`. After `ReceiverOnly`, mode select SHALL offer only the Receiver choices (ring steps 5 to 9). Skip SHALL move to the next Receiver choice in the ring order (Receiver on channel 4, 3, 2, 1 and 0) and SHALL wrap from channel 0 back to channel 4. The view model SHALL never name Picker, and ActivateColor SHALL enter only Receiver.
Acceptance: `crates/core/tests/features/receiver_only.feature`.

#### Scenario: Boot names Receiver on channel 0
- **WHEN** the shell sends `ReceiverOnly` and then `Boot`
- **THEN** mode select is shown, and the view model names Receiver on channel 0

#### Scenario: Skip wraps among the Receiver channels
- **WHEN** the shell sends `ReceiverOnly` and `Boot`, and the player presses Skip once
- **THEN** the view model names Receiver on channel 4, and four more Skips name Receiver on channel 0 again

#### Scenario: Apply enters Receiver
- **WHEN** the shell sends `ReceiverOnly` and `Boot`, and the player presses Skip twice and then ActivateColor
- **THEN** the game enters Receiver on channel 3 and starts a round

### Requirement: Reset and the full ring stay the same
In a receiver-only game, `Reset` SHALL go back to the last Receiver choice, and Skip SHALL then move to the next Receiver choice. Without `ReceiverOnly`, mode select SHALL offer all ten ring steps as before.
Acceptance: `crates/core/tests/features/receiver_only.feature`.

#### Scenario: Reset goes back to the last Receiver choice
- **WHEN** a receiver-only game is in Receiver on channel 3 and the player sends `Reset`
- **THEN** mode select names Receiver on channel 3, and Skip then names Receiver on channel 2

#### Scenario: Picker stays available without ReceiverOnly
- **WHEN** the board boots without `ReceiverOnly`
- **THEN** the view model names Picker on channel 0

### Requirement: ReceiverOnly moves a Picker choice and stays on
If the current choice is Picker when `ReceiverOnly` arrives, mode select SHALL move to Receiver on channel 0 (ring step 9), the channel on which the board Picker starts. A Receiver choice SHALL not change. `ReceiverOnly` SHALL be idempotent, and no event SHALL turn it off. The core SHALL accept `ReceiverOnly` also while the game is paused. `ReceiverOnly` SHALL not change the current screen.
Acceptance: `crates/core/tests/features/receiver_only.feature`.

#### Scenario: ReceiverOnly when the choice is Picker
- **WHEN** the player presses Skip twice (Picker on channel 2) and the shell then sends `ReceiverOnly`
- **THEN** the view model names Receiver on channel 0, and Skip then names Receiver on channel 4

#### Scenario: ReceiverOnly a second time
- **WHEN** the shell sends `ReceiverOnly` again in a receiver-only game
- **THEN** the choice does not change, and the core requests no tone, no delay and no BLE op

#### Scenario: ReceiverOnly while paused
- **WHEN** the game is paused in mode select and the shell sends `ReceiverOnly`
- **THEN** the core accepts it with no effect request, and after `Resume` mode select offers only Receiver
