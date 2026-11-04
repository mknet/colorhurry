# Spec Delta

## MODIFIED Requirements

### Requirement: Feedback tones for player input
Skip SHALL play a short low tone (380 Hz, 55 ms) and ActivateColor SHALL play a higher confirmation tone (784 Hz, 130 ms).

#### Scenario: Skip sounds
- **GIVEN** the board in mode select
- **WHEN** the player presses Skip
- **THEN** a 380 Hz tone plays for 55 ms

#### Scenario: Apply sounds
- **GIVEN** the board in mode select
- **WHEN** the player presses ActivateColor
- **THEN** a 784 Hz tone plays for 130 ms

### Requirement: Descending countdown tones
A countdown SHALL consist of ten beats one second apart, each playing a 200 ms tone from a descending table: 680, 620, 560, 500, 450, 400, 350, 300, 250, 200 Hz.

#### Scenario: Countdown pitch falls
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** ten beats have passed
- **THEN** 200 ms tones of 680, 620, 560, 500, 450, 400, 350, 300, 250 and 200 Hz have played one second apart

### Requirement: Explosion sound
When a countdown expires, every shell SHALL play the same explosion: ten 100 ms steps sweeping from 180 Hz down to 45 Hz, as defined by the game rather than by the shell.

#### Scenario: Countdown expires
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** ten beats have passed
- **THEN** the explosion sweeps in ten 100 ms steps from 180 Hz down to 45 Hz

### Requirement: Timing next to the radio stack
Delays, blink cadence and countdown beats SHALL keep their period while the BLE radio stack is active.

#### Scenario: Countdown with BLE enabled
Manual: needs the board with the SoftDevice.
- **WHEN** a Caller round runs on firmware with the SoftDevice enabled
- **THEN** the countdown beats stay one second apart and the board keeps responding

### Requirement: Tones do not block input
Playing a feedback tone SHALL NOT delay button handling or LED updates.

#### Scenario: Fast skipping
- **GIVEN** the Picker on channel 0
- **WHEN** the player presses Skip three times without any time passing
- **THEN** the Yellow LED blinks
- **AND** three Skip tones have played

### Requirement: Game owns blink and beat periods
The game SHALL request its own timers with the `crux_time` operation `NotifyAfter`: a 50 ms blink tick (six ticks per blink phase) in mode select and picker, a 0 ms kick for the first countdown beat and 1000 ms between further beats. When the state changes, the core SHALL clear each timer of the previous state with `ClearTimer`. A cleared timer SHALL have no effect.

#### Scenario: Boot schedules a blink
- **WHEN** the board boots
- **THEN** a 50 ms timer is requested

#### Scenario: Six ticks per blink phase
- **GIVEN** the board in mode select
- **WHEN** 250 ms have passed
- **THEN** LED 0 is on
- **WHEN** 50 ms have passed
- **THEN** LED 0 is off

#### Scenario: Skip clears the blink timer
- **GIVEN** the board in mode select
- **AND** 120 ms have passed
- **WHEN** the player presses Skip
- **THEN** the core clears the pending blink timer
- **AND** a 50 ms timer is requested
- **WHEN** 299 ms have passed
- **THEN** LED 1 is off
- **WHEN** 1 ms has passed
- **THEN** LED 1 is on

### Requirement: Pause stops the timers and the input
`Pause` SHALL stop the blink and the countdown, and the core SHALL clear the timers that are pending at `Pause`. While the game is paused, the core SHALL ignore every event except `Resume`. `Resume` SHALL restart the blink on a screen that blinks. A second `Pause` and a `Resume` while the game is not paused SHALL have no effect.

#### Scenario: Timers at Pause
- **GIVEN** the Caller on channel 0 with target Red
- **AND** two beats have passed
- **WHEN** the app goes to the background
- **THEN** the core clears the pending beat timer
- **WHEN** 5000 ms have passed
- **THEN** no tone plays
- **AND** no timer is requested

#### Scenario: Input while paused
- **GIVEN** the Caller on channel 0 with target Red
- **AND** the app is in the background
- **WHEN** the player presses Skip
- **AND** the player presses ActivateColor
- **AND** Red arrives on channel 0
- **THEN** nothing changes

#### Scenario: Skip while paused keeps the choice
- **GIVEN** the board in mode select
- **AND** the app is in the background
- **WHEN** the player presses Skip
- **THEN** the choice is Picker on channel 0

#### Scenario: Blink after Resume in mode select
- **GIVEN** the board in mode select
- **AND** the app is in the background
- **WHEN** 500 ms have passed
- **THEN** no timer is requested
- **WHEN** the app comes back
- **THEN** a 50 ms timer is requested

#### Scenario: Blink after Resume in Picker
- **GIVEN** the Picker on channel 1
- **AND** the app is in the background
- **WHEN** the app comes back
- **THEN** a 50 ms timer is requested

#### Scenario: Resume without Pause
- **GIVEN** the board in mode select
- **WHEN** the app comes back
- **THEN** nothing changes

#### Scenario: A second Pause
- **GIVEN** the board in mode select
- **AND** the app is in the background
- **WHEN** the app goes to the background
- **THEN** nothing changes
