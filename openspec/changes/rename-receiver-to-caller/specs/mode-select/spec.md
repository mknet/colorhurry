# Spec Delta

## REMOVED Requirements

### Requirement: ActivateColor enters the chosen role
**Reason**: The role is called Caller now; the requirement moves to a new name.
**Migration**: See "ActivateColor enters Picker or Caller".

## ADDED Requirements

### Requirement: ActivateColor enters Picker or Caller
ActivateColor SHALL enter Picker on channel `step` for steps 0 to 4 and Caller on channel `9 - step` for steps 5 to 9.
Acceptance: `crates/core/tests/features/mode_select.feature`.

#### Scenario: Left half enters Picker
- **WHEN** the player presses Skip twice and then ActivateColor
- **THEN** the board enters Picker on channel 2

#### Scenario: Right half enters Caller
- **WHEN** the player presses Skip six times and then ActivateColor
- **THEN** the board enters Caller on channel 3
