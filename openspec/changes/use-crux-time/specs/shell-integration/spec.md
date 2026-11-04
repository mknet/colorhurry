# Spec Delta

## MODIFIED Requirements

### Requirement: Shells only perform effects
A shell SHALL interact with the game only by sending domain events (Skip, ActivateColor, ColorReceived, Pause, Resume, Reset and CallerOnly) and by performing the effects the core requests: render, start a timer, clear a timer, random byte, play tone and BLE role. A shell SHALL NOT read game state for control flow or decide a match.

#### Scenario: Shell forwards every advertisement
- **WHEN** the board or browser decodes any valid color advertisement
- **THEN** it sends ColorReceived to the core without comparing it to the target

#### Scenario: Shell supplies randomness
- **WHEN** the core requests a random byte
- **THEN** the shell answers with a byte from its own random source and the core picks the target

#### Scenario: Shell reports the app lifecycle
- **WHEN** an app goes to the background or comes back
- **THEN** the shell sends Pause or Resume and the core decides what stops and what continues

#### Scenario: Shell forwards the Mode button
- **WHEN** the user presses the Mode button
- **THEN** the shell sends Reset and the core decides what stops and where mode select starts

#### Scenario: Shell reports that it cannot broadcast
- **WHEN** a shell that cannot broadcast a color starts
- **THEN** it sends CallerOnly before Boot and the core decides which choices mode select offers

## ADDED Requirements

### Requirement: Shells answer the time effects of crux_time
The core SHALL request its timers with the `crux_time` operations `NotifyAfter` and `ClearTimer`. A shell SHALL answer a `NotifyAfter` with its timer id when its duration has elapsed. A shell SHALL answer a `ClearTimer` with the same id, and SHALL free the timer that the id names. The iOS, macOS and watchOS apps SHALL use the shipped `TaskTimeHandler`, and the Android app SHALL use the shipped `CoroutineTimeHandler`. The web shell, the board and the acceptance test harness SHALL answer the two operations themselves.
Acceptance: `crates/core/tests/features/timing.feature`.

#### Scenario: A timer fires
- **WHEN** the core requests a 50 ms timer and 50 ms elapse
- **THEN** the shell answers the request with its timer id, and the core advances the blink

#### Scenario: The board frees a cleared timer
- **WHEN** the core clears a pending timer on the board
- **THEN** the board removes the timer from its list, answers the clear with the same id, and never answers the timer

#### Scenario: A late answer
- **WHEN** a native shell answers a timer after the core cleared it
- **THEN** the core ignores the answer, and the game does not change
