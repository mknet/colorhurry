# Spec Delta

## MODIFIED Requirements

### Requirement: Round starts with a random target color
When a round starts, the Receiver SHALL request one random byte from the shell and SHALL use spectrum color `byte mod 10` as the target, shown on all ten LEDs. The channel color from mode select SHALL NOT influence the target.

#### Scenario: Entering Receiver starts a round
- **WHEN** the player enters Receiver on channel 0 and the shell answers the random request with 1
- **THEN** the target is Red, all LEDs show Red and the countdown begins

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

## ADDED Requirements

### Requirement: Colors outside a running round are ignored
A ColorReceived while no countdown is running, or while the board is not a Receiver, SHALL have no effect.

#### Scenario: Color before the round starts
- **WHEN** a color is received while the board is still in mode select
- **THEN** nothing changes
