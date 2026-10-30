# Spec Delta

## MODIFIED Requirements

### Requirement: Game rules live in one shared core
Mode select, picker, caller round and palette SHALL live in one platform-free `no_std` core. A shell SHALL only turn input into domain events, perform the effects the core returns and draw its view.

#### Scenario: Rules are testable on the host
- **WHEN** `cargo test -p color-hurry-core` runs on a laptop
- **THEN** mode select, picker and caller behavior is exercised without a board

#### Scenario: Board firmware uses the core
- **WHEN** the board firmware is built
- **THEN** its game behavior comes from the shared core and the firmware only maps buttons, LEDs, speaker and radio

## REMOVED Requirements

### Requirement: Browser Receiver on the shared core
**Reason**: The role is called Caller now; the requirement moves to a new name.
**Migration**: See "Browser Caller on the shared core".

### Requirement: Live BLE receive in the browser
**Reason**: The role is called Caller now; the requirement moves to a new name.
**Migration**: See "Live BLE colors in the browser".

### Requirement: Native mobile shells on the shared core
**Reason**: The role is called Caller now; the requirement moves to a new name.
**Migration**: See "Native iOS and Android shells on the shared core".

## ADDED Requirements

### Requirement: Browser Caller on the shared core
The browser shell SHALL run the shared core compiled to WebAssembly as one Caller board, draw the ten LEDs as a CPB-style circle and offer Skip and Apply as buttons.

#### Scenario: Playing a round in the browser
- **WHEN** the user opens the web shell and enters Caller
- **THEN** the ring shows the target color and counts down with the same beats and tones as the board

### Requirement: Live BLE colors in the browser
Where Web Bluetooth advertisement watching is available, the browser shell SHALL let the user choose a Color Hurry board and SHALL feed every valid color advertisement from it into the core as ColorReceived.

#### Scenario: Board Picker drives the browser Caller
- **WHEN** a board Picker on the Caller's channel applies the target color
- **THEN** the browser Caller completes the task and starts the next round

### Requirement: Native iOS and Android shells on the shared core
The iOS and Android apps SHALL run the shared Rust core through FFI. They SHALL contain no game rules, tone tables or BLE payload format of their own: they map taps and BLE advertisements to domain events, perform the requested effects and draw the view.

#### Scenario: Phone as Caller
- **WHEN** the phone app is in Caller on channel 0 and a board Picker on channel 0 applies the phone's target color
- **THEN** the app completes the task and starts the next round, with the same tones as board and browser

#### Scenario: Payloads are decoded by the core
- **WHEN** the app scans manufacturer data for company `0x239A`
- **THEN** it passes the bytes to the core and only valid v4 payloads become ColorReceived
