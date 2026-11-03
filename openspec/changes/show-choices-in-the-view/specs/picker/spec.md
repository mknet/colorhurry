# Spec Delta

## ADDED Requirements

### Requirement: The view names the selected color
In Picker, the view model SHALL carry the selected spectrum color and SHALL say whether Apply has locked it. On other screens the view model SHALL carry no picker color.
Acceptance: `crates/core/tests/features/picker.feature`.

#### Scenario: Entering Picker names the channel color
- **WHEN** the player enters Picker on channel 2
- **THEN** the view model names Orange as selected

#### Scenario: Skip names the next color
- **WHEN** the player presses Skip while Orange is selected
- **THEN** the view model names Yellow as selected

#### Scenario: Apply keeps the name
- **WHEN** the player presses ActivateColor while Orange is selected
- **THEN** the view model names Orange and marks it applied
