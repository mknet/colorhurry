# Spec Delta

## ADDED Requirements

### Requirement: Timing next to the radio stack
Delays, blink cadence and countdown beats SHALL keep their period while the BLE radio stack is active.

#### Scenario: Countdown with BLE enabled
- **WHEN** a Receiver round runs on firmware with the SoftDevice enabled
- **THEN** the countdown beats stay one second apart and the board keeps responding

### Requirement: Tones do not block input
Playing a feedback tone SHALL NOT delay button handling or LED updates.

#### Scenario: Fast skipping
- **WHEN** the player presses Skip several times in quick succession
- **THEN** every press moves the selection and the ring follows without waiting for the previous tone to end
