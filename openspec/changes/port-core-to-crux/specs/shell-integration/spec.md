# Spec Delta

## ADDED Requirements

### Requirement: Shells only perform effects
A shell SHALL interact with the game only by sending domain events (Skip, ActivateColor, ColorReceived) and by performing the effects the core requests: render, delay, random byte, play tone and BLE role. A shell SHALL NOT read game state for control flow or decide a match.

#### Scenario: Shell forwards every advertisement
- **WHEN** the board or browser decodes any valid color advertisement
- **THEN** it sends ColorReceived to the core without comparing it to the target

#### Scenario: Shell supplies randomness
- **WHEN** the core requests a random byte
- **THEN** the shell answers with a byte from its own random source and the core picks the target
