# Spec Delta

## Purpose

Lets the Color Picker player choose one color from a fixed spectrum on the LED ring and commit to it.

## ADDED Requirements

### Requirement: Spectrum selection ring
The picker SHALL show a fixed spectrum of ten distinct colors, one per LED, and SHALL blink the LED of the currently selected color.

#### Scenario: Picker shows the spectrum
- **WHEN** the board starts
- **THEN** all ten LEDs show the spectrum and the LED of the first color blinks

### Requirement: Skip cycles the selection
While the player is selecting, Skip SHALL move the selection to the next spectrum color and SHALL wrap around after the last one.

#### Scenario: Skip moves to the next color
- **WHEN** the player presses Skip while selecting
- **THEN** the next LED becomes the blinking selection

#### Scenario: Selection wraps
- **WHEN** the player presses Skip with the last spectrum color selected
- **THEN** the first spectrum color becomes the selection

### Requirement: ActivateColor applies the selected color
ActivateColor SHALL apply the selected color: all ten LEDs show it and blinking stops. Skip or ActivateColor on an applied color SHALL return to selecting.

#### Scenario: Apply fills the ring
- **WHEN** the player presses ActivateColor while selecting
- **THEN** all ten LEDs show the selected color

#### Scenario: Leaving the applied color
- **WHEN** the player presses Skip or ActivateColor while a color is applied
- **THEN** the spectrum ring with the blinking selection is shown again
