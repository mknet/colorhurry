# picker Specification

## Purpose
Lets the Color Picker player choose one color from a fixed spectrum on the LED ring and commit to it.

## Requirements

### Requirement: Spectrum selection ring
The picker SHALL show the ten named spectrum colors White, Red, Orange, Yellow, LightGreen, Green, LightBlue, Blue, Purple and Pink, one per LED, and SHALL blink the LED of the currently selected color. Entering Picker on channel `n` SHALL select spectrum color `n`.

#### Scenario: Picker shows the spectrum
- **GIVEN** the board in mode select
- **WHEN** the player enters Picker on channel 0
- **THEN** the ring shows the spectrum
- **AND** the White LED blinks

#### Scenario: Channel picks the first selection
- **GIVEN** the board in mode select
- **WHEN** the player enters Picker on channel 2
- **THEN** the Orange LED blinks

### Requirement: Skip cycles the selection
While the player is selecting, Skip SHALL move the selection to the next spectrum color and SHALL wrap around after the last one.

#### Scenario: Skip moves to the next color
- **GIVEN** the Picker on channel 0
- **WHEN** the player presses Skip
- **THEN** the Red LED blinks

#### Scenario: Selection wraps
- **GIVEN** the Picker on channel 0 with Pink selected
- **WHEN** the player presses Skip
- **THEN** the White LED blinks

### Requirement: ActivateColor applies the selected color
ActivateColor SHALL apply the selected color: all ten LEDs show it and blinking stops. Skip or ActivateColor on an applied color SHALL return to selecting.

#### Scenario: Apply fills the ring
- **GIVEN** the Picker on channel 0 with Orange selected
- **WHEN** the player presses ActivateColor
- **THEN** all LEDs show Orange

#### Scenario: Leaving the applied color
- **GIVEN** the Picker on channel 0 with Orange applied
- **WHEN** the player presses Skip
- **THEN** the ring shows the spectrum
- **AND** the Orange LED blinks

#### Scenario: Applying twice leaves the applied color
- **GIVEN** the Picker on channel 0 with Orange applied
- **WHEN** the player presses ActivateColor
- **THEN** the ring shows the spectrum
- **AND** the Orange LED blinks

### Requirement: The view names the selected color
In Picker, the view model SHALL carry the selected spectrum color and SHALL say whether Apply has locked it. On other screens the view model SHALL carry no picker color.

#### Scenario: Entering Picker names the channel color
- **GIVEN** the board in mode select
- **WHEN** the player enters Picker on channel 2
- **THEN** the view names Orange as selected

#### Scenario: Skip names the next color
- **GIVEN** the Picker on channel 2
- **WHEN** the player presses Skip
- **THEN** the view names Yellow as selected

#### Scenario: Apply keeps the name
- **GIVEN** the Picker on channel 2
- **WHEN** the player presses ActivateColor
- **THEN** the view names Orange as applied

#### Scenario: Mode select names no color
- **WHEN** the board boots
- **THEN** the view names no Picker color
