# Spec Delta

## ADDED Requirements

### Requirement: A paused round continues where it stopped
After `Resume`, a countdown that ran at `Pause` SHALL continue at the same step, and its next beat SHALL come one full beat period after `Resume`. If no round ran at `Pause`, `Resume` SHALL request a new random byte and start the next round from it. A pause during the explosion SHALL NOT play the explosion again. A random byte that arrives while the game is paused SHALL have no effect.
Acceptance: `crates/core/tests/features/pause.feature`.

#### Scenario: Pause in the middle of a countdown
- **WHEN** the app goes to the background at countdown step 2 and comes back later
- **THEN** the countdown shows step 2, and the next beat comes one second after the app came back

#### Scenario: Pause during the explosion
- **WHEN** the app goes to the background while the explosion plays and comes back later
- **THEN** the next round starts with a new random target color, and the explosion does not play again

#### Scenario: Random byte while paused
- **WHEN** the answer to a random request arrives while the game is paused
- **THEN** the answer has no effect, and `Resume` requests a new random byte and starts the next round

#### Scenario: Random byte after Resume
- **WHEN** the answer to a random request from before `Pause` arrives after `Resume`
- **THEN** exactly one round starts
