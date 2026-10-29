# Spec Delta

## MODIFIED Requirements

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
