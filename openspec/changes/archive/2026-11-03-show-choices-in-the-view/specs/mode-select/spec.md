# Spec Delta

## ADDED Requirements

### Requirement: The view names the chosen role and channel
In mode select, the view model SHALL carry the role (Picker or Receiver) and the channel that ActivateColor would enter for the current step. On other screens the view model SHALL carry no choice.
Acceptance: `crates/core/tests/features/mode_select.feature`.

#### Scenario: Boot names Picker channel 0
- **WHEN** the board boots
- **THEN** the view model names Picker on channel 0

#### Scenario: Six Skips name Receiver channel 3
- **WHEN** the player presses Skip six times
- **THEN** the view model names Receiver on channel 3
