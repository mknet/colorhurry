# caller-round Specification

## Purpose
Defines the Caller's round: a random target color that the Caller calls out, a visible and audible countdown, and what happens when time runs out.

## Requirements

### Requirement: Round starts with a random target color
When a round starts, the Caller SHALL request one random byte from the shell and SHALL use spectrum color `byte mod 10` as the target, shown on all ten LEDs. The channel color from mode select SHALL NOT influence the target.
Acceptance: `crates/core/tests/features/caller.feature`.

#### Scenario: Entering Caller starts a round
- **WHEN** the player enters Caller on channel 0 and the shell answers the random request with 1
- **THEN** the target is Red, all LEDs show Red and the countdown begins

### Requirement: Countdown beats
The countdown SHALL run ten beats one second apart. Each beat SHALL play the next countdown tone and switch off the next LED counter-clockwise.
Acceptance: `crates/core/tests/features/caller.feature`.

#### Scenario: One LED per second
- **WHEN** three beats have passed
- **THEN** three LEDs are off and three descending tones have played

### Requirement: Next round after the countdown
When the tenth beat has passed without a match, the countdown expires (CountdownExpired): the Caller SHALL play the explosion and then start the next round with a new random target color.
Acceptance: `crates/core/tests/features/caller.feature`.

#### Scenario: Countdown runs out
- **WHEN** the tenth beat has passed without a matching color
- **THEN** the explosion plays and a new random target color is shown with a new countdown

### Requirement: Matching color completes the task
A ColorReceived whose channel equals the Caller's channel and whose named color equals the target SHALL complete the task (TaskCompleted): the countdown stops, no explosion plays, and a new random byte is requested for the next round. Only the core SHALL decide whether a color matches.
Acceptance: `crates/core/tests/features/caller.feature`.

#### Scenario: Picker sends the target in time
- **WHEN** the Caller on channel 0 with target Red receives Red on channel 0 during the countdown
- **THEN** the countdown stops without explosion and the next round starts from new entropy

#### Scenario: Wrong color is ignored
- **WHEN** the Caller with target Red receives Blue on its channel
- **THEN** the countdown keeps running

#### Scenario: Wrong channel is ignored
- **WHEN** the Caller on channel 0 with target Red receives Red on channel 1
- **THEN** the countdown keeps running

### Requirement: Skip aborts the round
Skip during a countdown SHALL abort the round without explosion and SHALL start the next round.
Acceptance: `crates/core/tests/features/caller.feature`.

#### Scenario: Player gives up on a color
- **WHEN** the player presses Skip during the countdown
- **THEN** a new round starts without explosion

### Requirement: Colors outside a running round are ignored
A ColorReceived while no countdown is running, or while the board is not a Caller, SHALL have no effect.
Acceptance: `crates/core/tests/features/caller.feature`.

#### Scenario: Color before the round starts
- **WHEN** a color is received while the board is still in mode select
- **THEN** nothing changes

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
