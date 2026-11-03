# Spec Delta

## ADDED Requirements

### Requirement: Apps offer a way back to mode select
The iOS, macOS, watchOS, Android and web shells SHALL offer a button that sends `Reset` to the core. The iOS, macOS and Android apps SHALL show a Mode button in the header, and the macOS app SHALL also send `Reset` for the Escape key. The watchOS app SHALL show a Change mode button on the status page and SHALL scroll back to the game page after the press. The web shell SHALL show a Mode button next to Skip and Apply. The board SHALL NOT send `Reset`.

#### Scenario: Phone Receiver changes the channel
- **WHEN** the user presses Mode during a countdown in the iOS or Android app
- **THEN** the tones stop, mode select shows the Receiver channel of the round, and the user can press Skip to choose a different channel

#### Scenario: Watch changes the mode
- **WHEN** the user presses Change mode on the watch status page
- **THEN** the watch shows the game page with mode select at the last choice

#### Scenario: Mac uses the keyboard
- **WHEN** the user presses Escape in the macOS app
- **THEN** the app goes back to mode select

#### Scenario: Board has no reset
- **WHEN** the board firmware runs
- **THEN** it sends no `Reset`, and its game behavior does not change

### Requirement: Reset stops the sound and the haptics
Before an app shell sends `Reset`, it SHALL stop the tone that plays, drop the tones that are queued, and cancel the haptic clicks that are scheduled.

#### Scenario: Reset during the explosion
- **WHEN** the user presses Mode in the iOS or Android app while the explosion plays
- **THEN** the explosion stops at once

## MODIFIED Requirements

### Requirement: Shells only perform effects
A shell SHALL interact with the game only by sending domain events (Skip, ActivateColor, ColorReceived, Pause, Resume and Reset) and by performing the effects the core requests: render, delay, random byte, play tone and BLE role. A shell SHALL NOT read game state for control flow or decide a match.

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
