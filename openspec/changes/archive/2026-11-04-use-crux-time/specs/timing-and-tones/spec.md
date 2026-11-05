# Spec Delta

## MODIFIED Requirements

### Requirement: Game owns blink and beat periods
The game SHALL request its own timers with the `crux_time` operation `NotifyAfter`: a 50 ms blink tick (six ticks per blink phase) in mode select and picker, a 0 ms kick for the first countdown beat and 1000 ms between further beats. When the state changes, the core SHALL clear each timer of the previous state with `ClearTimer`. A cleared timer SHALL have no effect.
Acceptance: `crates/core/tests/features/timing.feature`.

#### Scenario: Boot schedules a blink
- **WHEN** the game boots
- **THEN** a 50 ms timer is requested

#### Scenario: Skip clears the blink timer
- **WHEN** the player presses Skip while a blink timer is pending
- **THEN** the core clears that timer, requests a new 50 ms timer, and the old timer has no effect

### Requirement: Pause stops the timers and the input
`Pause` SHALL stop the blink and the countdown, and the core SHALL clear the timers that are pending at `Pause`. While the game is paused, the core SHALL ignore every event except `Resume`. `Resume` SHALL restart the blink on a screen that blinks. A second `Pause` and a `Resume` while the game is not paused SHALL have no effect.
Acceptance: `crates/core/tests/features/pause.feature`.

#### Scenario: Timers at Pause
- **WHEN** the game is paused while a timer is pending
- **THEN** the core clears that timer, the timer has no effect and no new timer is requested

#### Scenario: Input while paused
- **WHEN** the player presses Skip or ActivateColor, or a color is received, while the game is paused
- **THEN** nothing changes

#### Scenario: Blink after Resume
- **WHEN** the app comes back in mode select or while the Picker selects a color
- **THEN** the core requests a 50 ms blink timer again

#### Scenario: Resume without Pause
- **WHEN** the shell sends `Resume` and the game is not paused
- **THEN** nothing changes and no effect is requested
