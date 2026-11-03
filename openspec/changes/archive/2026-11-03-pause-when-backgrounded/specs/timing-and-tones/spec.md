# Spec Delta

## ADDED Requirements

### Requirement: Pause stops the timers and the input
`Pause` SHALL stop the blink and the countdown, and the core SHALL ignore the delays that were pending at `Pause`. While the game is paused, the core SHALL ignore every event except `Resume`. `Resume` SHALL restart the blink on a screen that blinks. A second `Pause` and a `Resume` while the game is not paused SHALL have no effect.
Acceptance: `crates/core/tests/features/pause.feature`.

#### Scenario: Delays after Pause
- **WHEN** the game is paused and a delay from before `Pause` finishes
- **THEN** the delay has no effect and no new delay is requested

#### Scenario: Input while paused
- **WHEN** the player presses Skip or ActivateColor, or a color is received, while the game is paused
- **THEN** nothing changes

#### Scenario: Blink after Resume
- **WHEN** the app comes back in mode select or while the Picker selects a color
- **THEN** the core requests a 50 ms blink delay again

#### Scenario: Resume without Pause
- **WHEN** the shell sends `Resume` and the game is not paused
- **THEN** nothing changes and no effect is requested
