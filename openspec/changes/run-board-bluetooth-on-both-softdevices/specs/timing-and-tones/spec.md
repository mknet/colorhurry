# Spec Delta

## MODIFIED Requirements

### Requirement: Timing next to the radio stack
Delays, blink cadence and countdown beats SHALL keep their period while the BLE radio stack is active.

#### Scenario: Countdown with BLE enabled
- **WHEN** a Caller round runs on firmware with the BLE radio stack active, on a board with S140 6.1.1 or 7.x
- **THEN** the countdown beats stay one second apart and the board keeps responding
