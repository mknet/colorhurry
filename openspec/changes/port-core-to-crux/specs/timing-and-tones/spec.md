# Spec Delta

## MODIFIED Requirements

### Requirement: Explosion sound
When a countdown expires, every shell SHALL play the same explosion: ten 100 ms steps sweeping from 180 Hz down to 45 Hz, as defined by the game rather than by the shell.

#### Scenario: Countdown expires
- **WHEN** the tenth countdown beat has played
- **THEN** the explosion sweep plays for about one second on board and browser alike

## ADDED Requirements

### Requirement: Game owns blink and beat periods
The game SHALL request its own delays: a 50 ms blink tick (six ticks per blink phase) in mode select and picker, a 0 ms kick for the first countdown beat and 1000 ms between further beats. Delays from a previous state SHALL be ignored.

#### Scenario: Boot schedules a blink
- **WHEN** the game boots
- **THEN** a 50 ms delay is requested

#### Scenario: Stale delays after Skip
- **WHEN** the player presses Skip and an older blink delay finishes afterwards
- **THEN** the older delay has no effect
