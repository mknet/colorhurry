# caller-round Specification

## Purpose
Defines the Caller's round: a random target color, a visible and audible countdown, and what happens when time runs out.

## Requirements

### Requirement: Round starts with a random target color
When a round starts, the Caller SHALL request one random byte from the shell and SHALL use spectrum color `byte mod 10` as the target, shown on all ten LEDs. The channel color from mode select SHALL NOT influence the target.

#### Scenario: Entering Caller starts a round
- **GIVEN** the board in mode select
- **WHEN** the player enters Caller on channel 0
- **AND** the shell answers the random request with 1
- **THEN** the target is Red
- **AND** all LEDs show Red
- **AND** the countdown begins

### Requirement: Countdown beats
The countdown SHALL run ten beats one second apart. Each beat SHALL play the next countdown tone and switch off the next LED counter-clockwise.

#### Scenario: One LED per second
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** three beats have passed
- **THEN** three LEDs are off
- **AND** three descending tones have played

### Requirement: Next round after the countdown
When the tenth beat has passed without a match, the countdown expires (CountdownExpired): the Caller SHALL play the explosion and then start the next round with a new random target color. The Caller SHALL request the random byte for the next round only after the explosion has played, and Skip during the explosion SHALL NOT start a second round.

#### Scenario: Countdown runs out
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** ten beats have passed
- **THEN** the explosion plays
- **AND** the countdown stops
- **AND** no random byte is requested
- **WHEN** the explosion has played
- **AND** the shell answers the random request with 5
- **THEN** the target is Green
- **AND** the countdown begins

#### Scenario: Skip during the explosion
- **GIVEN** the Caller on channel 0 with target Red
- **AND** ten beats have passed
- **WHEN** the player presses Skip
- **AND** the explosion has played
- **AND** the shell answers the random request with 5
- **THEN** the target is Green
- **AND** a new random byte is requested

### Requirement: Matching color completes the task
A ColorReceived whose channel equals the Caller's channel and whose named color equals the target SHALL complete the task (TaskCompleted): the countdown stops, no explosion plays, and a new random byte is requested for the next round. Only the core SHALL decide whether a color matches.

#### Scenario: Picker sends the target in time
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** Red arrives on channel 0
- **THEN** the countdown stops
- **AND** no explosion plays
- **AND** a new random byte is requested

#### Scenario: Wrong color is ignored
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** Blue arrives on channel 0
- **THEN** the countdown keeps running

#### Scenario: Wrong channel is ignored
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** Red arrives on channel 1
- **THEN** the countdown keeps running

### Requirement: Skip aborts the round
Skip during a countdown SHALL abort the round without explosion and SHALL start the next round.

#### Scenario: Player gives up on a color
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** the player presses Skip
- **THEN** the countdown stops
- **AND** no explosion plays
- **AND** a new random byte is requested

### Requirement: Colors outside a running round are ignored
A ColorReceived while no countdown is running, or while the board is not a Caller, SHALL have no effect.

#### Scenario: Color before the round starts
- **GIVEN** the board in mode select
- **WHEN** Red arrives on channel 0
- **THEN** nothing changes

#### Scenario: Color while picking
- **GIVEN** the Picker on channel 0
- **WHEN** Red arrives on channel 0
- **THEN** nothing changes

### Requirement: ActivateColor has no effect on the Caller
ActivateColor SHALL NOT change a running round: the Caller only reacts to Skip and to colors.

#### Scenario: Apply during the countdown
- **GIVEN** the Caller on channel 1 with target Yellow
- **WHEN** the player presses ActivateColor
- **THEN** the countdown keeps running

### Requirement: A paused round continues where it stopped
After `Resume`, a countdown that ran at `Pause` SHALL continue at the same step, and its next beat SHALL come one full beat period after `Resume`. If no round ran at `Pause`, `Resume` SHALL request a new random byte and start the next round from it. A pause during the explosion SHALL NOT play the explosion again. A random byte that arrives while the game is paused SHALL have no effect.

#### Scenario: Pause in the middle of a countdown
- **GIVEN** the Caller on channel 0 with target Red
- **AND** two beats have passed
- **WHEN** the app goes to the background
- **AND** 5000 ms have passed
- **AND** the app comes back
- **THEN** two LEDs are off
- **AND** no tone plays
- **WHEN** 999 ms have passed
- **THEN** two LEDs are off
- **WHEN** 1 ms has passed
- **THEN** three LEDs are off

#### Scenario: Pause during the explosion
- **GIVEN** the Caller on channel 0 with target Red
- **AND** ten beats have passed
- **WHEN** the app goes to the background
- **AND** 2000 ms have passed
- **AND** the app comes back
- **AND** the shell answers the random request with 5
- **THEN** the target is Green
- **AND** no explosion plays

#### Scenario: Random byte while paused
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** the player presses Skip
- **AND** the app goes to the background
- **AND** the shell answers the random request with 2
- **THEN** the countdown stops
- **WHEN** the app comes back
- **AND** the shell answers the random request with 3
- **THEN** the target is Yellow
- **AND** the board scans channel 0 for Yellow

#### Scenario: Random byte after Resume
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** the player presses Skip
- **AND** the app goes to the background
- **AND** the app comes back
- **AND** the shell answers the random request with 2
- **THEN** the target is Orange
- **WHEN** the shell answers the random request with 3
- **THEN** the target is Orange
