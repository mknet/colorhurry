# Spec Delta

## ADDED Requirements

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
