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
