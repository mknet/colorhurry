# Spec Delta

## MODIFIED Requirements

### Requirement: Descending countdown tones
A countdown SHALL consist of ten beats one second apart, each playing a 200 ms tone from a descending table: 680, 620, 560, 500, 450, 400, 350, 300, 250, 200 Hz.
Acceptance: `crates/core/tests/features/caller.feature`.

#### Scenario: Countdown pitch falls
- **WHEN** a countdown runs to the end
- **THEN** ten tones play one second apart, each lower than the one before

### Requirement: Explosion sound
When a countdown expires, every shell SHALL play the same explosion: ten 100 ms steps sweeping from 180 Hz down to 45 Hz, as defined by the game rather than by the shell.
Acceptance: `crates/core/tests/features/caller.feature`.

#### Scenario: Countdown expires
- **WHEN** the tenth countdown beat has played
- **THEN** the explosion sweep plays for about one second on board and browser alike

### Requirement: Timing next to the radio stack
Delays, blink cadence and countdown beats SHALL keep their period while the BLE radio stack is active.

#### Scenario: Countdown with BLE enabled
- **WHEN** a Caller round runs on firmware with the SoftDevice enabled
- **THEN** the countdown beats stay one second apart and the board keeps responding
