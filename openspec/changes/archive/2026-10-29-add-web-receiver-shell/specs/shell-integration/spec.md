# Spec Delta

## ADDED Requirements

### Requirement: Browser Receiver on the shared core
The browser shell SHALL run the shared core compiled to WebAssembly as one Receiver board, draw the ten LEDs as a CPB-style circle and offer Skip and Apply as buttons.

#### Scenario: Playing a round in the browser
- **WHEN** the user opens the web shell and enters Receiver
- **THEN** the ring shows the target color and counts down with the same beats and tones as the board

### Requirement: Live BLE receive in the browser
Where Web Bluetooth advertisement watching is available, the browser shell SHALL let the user choose a Color Hurry board and SHALL feed every valid color advertisement from it into the core as ColorReceived.

#### Scenario: Board Picker drives the browser Receiver
- **WHEN** a board Picker on the Receiver's channel applies the target color
- **THEN** the browser Receiver completes the task and starts the next round
