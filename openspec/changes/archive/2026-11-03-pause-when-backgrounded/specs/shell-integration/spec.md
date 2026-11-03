# Spec Delta

## ADDED Requirements

### Requirement: Apps pause the game in the background
An app shell SHALL send `Pause` to the core when the app goes to the background and `Resume` when it comes back. The iOS, macOS and watchOS apps SHALL use the background and active scene phases and SHALL ignore the inactive phase. The Android app SHALL use `onStop` and `onStart`. The web shell SHALL use the visibility of the page. The board SHALL NOT send `Pause` or `Resume`.

#### Scenario: Watch wrist is lowered
- **WHEN** the user lowers the wrist and the watch app becomes inactive but stays in the foreground
- **THEN** the game continues

#### Scenario: App comes back
- **WHEN** the user opens the app again after it went to the background during a countdown
- **THEN** the countdown continues at the step where it stopped

#### Scenario: Browser tab is hidden
- **WHEN** the user switches to another browser tab
- **THEN** the web shell sends `Pause`, and it sends `Resume` when the tab is visible again

#### Scenario: Board is never paused
- **WHEN** the board firmware runs
- **THEN** it sends no `Pause` or `Resume`, and its game behavior does not change

### Requirement: Pause stops the sound and the haptics
On `Pause`, an app shell SHALL stop the tone that plays, drop the tones that are queued, and cancel the haptic clicks that are scheduled.

#### Scenario: Watch app is dismissed during a countdown
- **WHEN** the user dismisses the watch app during a countdown
- **THEN** no tone and no haptic click comes after the dismissal, and the countdown does not continue

#### Scenario: Phone goes to the background during the explosion
- **WHEN** the iOS or Android app goes to the background while the explosion plays
- **THEN** the explosion stops at once

## MODIFIED Requirements

### Requirement: Shells only perform effects
A shell SHALL interact with the game only by sending domain events (Skip, ActivateColor, ColorReceived, Pause and Resume) and by performing the effects the core requests: render, delay, random byte, play tone and BLE role. A shell SHALL NOT read game state for control flow or decide a match.

#### Scenario: Shell forwards every advertisement
- **WHEN** the board or browser decodes any valid color advertisement
- **THEN** it sends ColorReceived to the core without comparing it to the target

#### Scenario: Shell supplies randomness
- **WHEN** the core requests a random byte
- **THEN** the shell answers with a byte from its own random source and the core picks the target

#### Scenario: Shell reports the app lifecycle
- **WHEN** an app goes to the background or comes back
- **THEN** the shell sends Pause or Resume and the core decides what stops and what continues
