# timing-and-tones Specification

## Purpose
Defines the sounds of Color Hurry and their timing, so every input and every countdown beat is audible and recognizable.

## Requirements

### Requirement: Feedback tones for player input
Skip SHALL play a short low tone (380 Hz, 55 ms) and ActivateColor SHALL play a higher confirmation tone (784 Hz, 130 ms).

#### Scenario: Skip sounds
- **WHEN** the player presses Skip
- **THEN** a 380 Hz tone plays for 55 ms

#### Scenario: Apply sounds
- **WHEN** the player presses ActivateColor
- **THEN** a 784 Hz tone plays for 130 ms

### Requirement: Descending countdown tones
A countdown SHALL consist of ten beats one second apart, each playing a 200 ms tone from a descending table: 680, 620, 560, 500, 450, 400, 350, 300, 250, 200 Hz.

#### Scenario: Countdown pitch falls
- **WHEN** a countdown runs to the end
- **THEN** ten tones play one second apart, each lower than the one before

### Requirement: Explosion sound
When a countdown expires, the game SHALL play an explosion: ten 100 ms steps sweeping from 180 Hz down to 45 Hz.

#### Scenario: Countdown expires
- **WHEN** the tenth countdown beat has played
- **THEN** the explosion sweep plays for about one second

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
