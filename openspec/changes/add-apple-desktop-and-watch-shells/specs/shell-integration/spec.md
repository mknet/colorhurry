# Spec Delta

## ADDED Requirements

### Requirement: Native macOS and watchOS shells on the shared core
The macOS and watchOS apps SHALL run the shared Rust core through FFI, like the iOS and Android apps. They SHALL contain no game rules, tone tables or BLE payload format of their own: they map input and BLE advertisements to domain events, perform the requested effects and draw the view. The watchOS app SHALL run on Apple Watch Series 6 (`arm64_32`) and later.

#### Scenario: Mac as Receiver
- **WHEN** the Mac app is in Receiver on a channel and a board Picker on the same channel applies the Mac's target color
- **THEN** the app completes the task and starts the next round, with the same tones as board, browser and phone

#### Scenario: Watch as Receiver
- **WHEN** the watch app is in Receiver on a channel and a board Picker on the same channel applies the watch's target color
- **THEN** the app completes the task and starts the next round, with the same tones as the other shells, and the watch gives a haptic on each tone step

#### Scenario: Picker is not available
- **WHEN** the user enters Picker in the macOS or watchOS app
- **THEN** the app shows that it cannot broadcast the color, and tells the user to use a board or an Android phone as Picker, as the iOS app does
