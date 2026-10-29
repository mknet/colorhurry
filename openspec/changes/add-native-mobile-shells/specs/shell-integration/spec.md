# Spec Delta

## ADDED Requirements

### Requirement: Native mobile shells on the shared core
The iOS and Android apps SHALL run the shared Rust core through FFI. They SHALL contain no game rules, tone tables or BLE payload format of their own: they map taps and BLE advertisements to domain events, perform the requested effects and draw the view.

#### Scenario: Phone as Receiver
- **WHEN** the phone app is in Receiver on channel 0 and a board Picker on channel 0 applies the phone's target color
- **THEN** the app completes the task and starts the next round, with the same tones as board and browser

#### Scenario: Payloads are decoded by the core
- **WHEN** the app scans manufacturer data for company `0x239A`
- **THEN** it passes the bytes to the core and only valid v4 payloads become ColorReceived
