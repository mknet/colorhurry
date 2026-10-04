# Spec Delta

## MODIFIED Requirements

### Requirement: Next round after the countdown
When the tenth beat has passed without a match, the countdown expires (CountdownExpired): the Receiver SHALL play the explosion and then start the next round with a new random target color.

#### Scenario: Countdown runs out
- **WHEN** the tenth beat has passed without a matching color
- **THEN** the explosion plays and a new random target color is shown with a new countdown

## ADDED Requirements

### Requirement: Matching color completes the task
A ColorReceived on the Receiver's channel whose color equals the target (same RGB) SHALL complete the task (TaskCompleted): the countdown stops, no explosion plays, and the next round starts.

#### Scenario: Picker sends the target in time
- **WHEN** the Receiver on channel 0 with target Red hears Red on channel 0 during the countdown
- **THEN** the countdown stops without explosion and a new round starts

#### Scenario: Wrong color is ignored
- **WHEN** the Receiver with target Red hears Blue on its channel
- **THEN** the countdown keeps running

### Requirement: Skip aborts the round
Skip during a countdown SHALL abort the round without explosion and SHALL start the next round.

#### Scenario: Player gives up on a color
- **WHEN** the player presses Skip during the countdown
- **THEN** a new round starts without explosion
