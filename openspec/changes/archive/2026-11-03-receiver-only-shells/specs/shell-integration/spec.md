# Spec Delta

## ADDED Requirements

### Requirement: Apple apps are Receiver only
The iOS, macOS and watchOS apps SHALL declare `Platform.canBroadcast` as false, because CoreBluetooth on iOS and macOS cannot advertise manufacturer data and watchOS cannot advertise at all. When `canBroadcast` is false, `GameController.boot()` SHALL send `ReceiverOnly` to the core before `Boot`. The Android app, the web shell and the board SHALL NOT send `ReceiverOnly`.

#### Scenario: Watch opens on a Receiver choice
- **WHEN** the user opens the watch app
- **THEN** mode select names Receiver on channel 0, and Skip cycles only the Receiver channels

#### Scenario: Android keeps Picker
- **WHEN** the user opens the Android app
- **THEN** mode select offers Picker and Receiver as before

#### Scenario: Board is not changed
- **WHEN** the board firmware runs
- **THEN** it sends no `ReceiverOnly`, and its game behavior does not change

## MODIFIED Requirements

### Requirement: Shells only perform effects
A shell SHALL interact with the game only by sending domain events (Skip, ActivateColor, ColorReceived, Pause, Resume, Reset and ReceiverOnly) and by performing the effects the core requests: render, delay, random byte, play tone and BLE role. A shell SHALL NOT read game state for control flow or decide a match.

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

#### Scenario: Shell reports that it cannot broadcast
- **WHEN** a shell that cannot broadcast a color starts
- **THEN** it sends ReceiverOnly before Boot and the core decides which choices mode select offers

### Requirement: Native macOS and watchOS shells on the shared core
The macOS and watchOS apps SHALL run the shared Rust core through FFI, like the iOS and Android apps. They SHALL contain no game rules, tone tables or BLE payload format of their own: they map input and BLE advertisements to domain events, perform the requested effects and draw the view. The watchOS app SHALL run on Apple Watch Series 6 (`arm64_32`) and later.

#### Scenario: Mac as Receiver
- **WHEN** the Mac app is in Receiver on a channel and a board Picker on the same channel applies the Mac's target color
- **THEN** the app completes the task and starts the next round, with the same tones as board, browser and phone

#### Scenario: Watch as Receiver
- **WHEN** the watch app is in Receiver on a channel and a board Picker on the same channel applies the watch's target color
- **THEN** the app completes the task and starts the next round, with the same tones as the other shells, and the watch gives a haptic on each tone step

#### Scenario: Picker is not available
- **WHEN** the user presses Skip in mode select in the macOS or watchOS app
- **THEN** mode select names only Receiver choices, as in the iOS app, and the user must use a board or an Android phone as Picker
