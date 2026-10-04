# receiver-round Specification

## Purpose
Defines the Receiver's round: a random target color, a visible and audible countdown, and what happens when time runs out.

## Requirements

### Requirement: Round starts with a random target color
When a round starts, the Receiver SHALL pick a random target color from the ten spectrum colors and SHALL show it on all ten LEDs. The channel color from mode select SHALL NOT influence the target.

#### Scenario: Entering Receiver starts a round
- **WHEN** the player enters Receiver
- **THEN** all LEDs show a random spectrum color and the countdown begins

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
