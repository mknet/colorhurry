# shell-integration Specification

## Purpose
Defines how platform shells (board, browser, phones) drive the game, so the rules exist only once and every shell plays the same game.

## Requirements

### Requirement: Game rules live in one shared core
Mode select, picker, receiver round and palette SHALL live in one platform-free `no_std` core. A shell SHALL only turn input into domain events, perform the effects the core returns and draw its view.

#### Scenario: Rules are testable on the host
- **WHEN** `cargo test -p color-hurry-core` runs on a laptop
- **THEN** mode select, picker and receiver behavior is exercised without a board

#### Scenario: Board firmware uses the core
- **WHEN** the board firmware is built
- **THEN** its game behavior comes from the shared core and the firmware only maps buttons, LEDs, speaker and radio

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
