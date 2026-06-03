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
After the tenth beat the Receiver SHALL start the next round with a new random target color.

#### Scenario: Countdown runs out
- **WHEN** the tenth beat has passed
- **THEN** a new random target color is shown on all LEDs and a new countdown begins
