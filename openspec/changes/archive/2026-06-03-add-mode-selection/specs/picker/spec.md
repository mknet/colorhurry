# Spec Delta

## MODIFIED Requirements

### Requirement: Spectrum selection ring
The picker SHALL show the ten named spectrum colors White, Red, Orange, Yellow, LightGreen, Green, LightBlue, Blue, Purple and Pink, one per LED, and SHALL blink the LED of the currently selected color. Entering Picker on channel `n` SHALL select spectrum color `n`.

#### Scenario: Picker shows the spectrum
- **WHEN** the player enters Picker on channel 0
- **THEN** all ten LEDs show the spectrum and the White LED blinks
