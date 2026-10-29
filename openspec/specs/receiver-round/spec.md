# receiver-round Specification

## Purpose
Defines the Receiver's round: a random target color, a visible and audible countdown, and what happens when time runs out.

## Requirements

### Requirement: Round starts with a random target color
When a round starts, the Receiver SHALL request one random byte from the shell and SHALL use spectrum color `byte mod 10` as the target, shown on all ten LEDs. The channel color from mode select SHALL NOT influence the target.

#### Scenario: Entering Receiver starts a round
- **WHEN** the player enters Receiver on channel 0 and the shell answers the random request with 1
- **THEN** the target is Red, all LEDs show Red and the countdown begins

### Requirement: Countdown beats
The countdown SHALL run ten beats one second apart. Each beat SHALL play the next countdown tone and switch off the next LED counter-clockwise.

#### Scenario: One LED per second
- **WHEN** three beats have passed
- **THEN** three LEDs are off and three descending tones have played

### Requirement: Next round after the countdown
When the tenth beat has passed without a match, the countdown expires (CountdownExpired): the Receiver SHALL play the explosion and then start the next round with a new random target color.

#### Scenario: Countdown runs out
- **WHEN** the tenth beat has passed without a matching color
- **THEN** the explosion plays and a new random target color is shown with a new countdown

### Requirement: Matching color completes the task
A ColorReceived whose channel equals the Receiver's channel and whose named color equals the target SHALL complete the task (TaskCompleted): the countdown stops, no explosion plays, and a new random byte is requested for the next round. Only the core SHALL decide whether a color matches.

#### Scenario: Picker sends the target in time
- **WHEN** the Receiver on channel 0 with target Red receives Red on channel 0 during the countdown
- **THEN** the countdown stops without explosion and the next round starts from new entropy

#### Scenario: Wrong color is ignored
- **WHEN** the Receiver with target Red receives Blue on its channel
- **THEN** the countdown keeps running

#### Scenario: Wrong channel is ignored
- **WHEN** the Receiver on channel 0 with target Red receives Red on channel 1
- **THEN** the countdown keeps running

### Requirement: Skip aborts the round
Skip during a countdown SHALL abort the round without explosion and SHALL start the next round.

#### Scenario: Player gives up on a color
- **WHEN** the player presses Skip during the countdown
- **THEN** a new round starts without explosion

### Requirement: Colors outside a running round are ignored
A ColorReceived while no countdown is running, or while the board is not a Receiver, SHALL have no effect.

#### Scenario: Color before the round starts
- **WHEN** a color is received while the board is still in mode select
- **THEN** nothing changes
