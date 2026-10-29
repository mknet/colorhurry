# timing-and-tones Specification

## Purpose
Defines the sounds of Color Hurry and their timing, so every input and every countdown beat is audible and recognizable.

## Requirements

### Requirement: Feedback tones for player input
Skip SHALL play a short low tone (380 Hz, 55 ms) and ActivateColor SHALL play a higher confirmation tone (784 Hz, 130 ms).
Acceptance: `crates/core/tests/features/mode_select.feature`, `crates/core/tests/features/picker.feature`.

#### Scenario: Skip sounds
- **WHEN** the player presses Skip
- **THEN** a 380 Hz tone plays for 55 ms

#### Scenario: Apply sounds
- **WHEN** the player presses ActivateColor
- **THEN** a 784 Hz tone plays for 130 ms

### Requirement: Descending countdown tones
A countdown SHALL consist of ten beats one second apart, each playing a 200 ms tone from a descending table: 680, 620, 560, 500, 450, 400, 350, 300, 250, 200 Hz.
Acceptance: `crates/core/tests/features/receiver.feature`.

#### Scenario: Countdown pitch falls
- **WHEN** a countdown runs to the end
- **THEN** ten tones play one second apart, each lower than the one before

### Requirement: Explosion sound
When a countdown expires, every shell SHALL play the same explosion: ten 100 ms steps sweeping from 180 Hz down to 45 Hz, as defined by the game rather than by the shell.
Acceptance: `crates/core/tests/features/receiver.feature`.

#### Scenario: Countdown expires
- **WHEN** the tenth countdown beat has played
- **THEN** the explosion sweep plays for about one second on board and browser alike

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

### Requirement: Game owns blink and beat periods
The game SHALL request its own delays: a 50 ms blink tick (six ticks per blink phase) in mode select and picker, a 0 ms kick for the first countdown beat and 1000 ms between further beats. Delays from a previous state SHALL be ignored.
Acceptance: `crates/core/tests/features/timing.feature`.

#### Scenario: Boot schedules a blink
- **WHEN** the game boots
- **THEN** a 50 ms delay is requested

#### Scenario: Stale delays after Skip
- **WHEN** the player presses Skip and an older blink delay finishes afterwards
- **THEN** the older delay has no effect
