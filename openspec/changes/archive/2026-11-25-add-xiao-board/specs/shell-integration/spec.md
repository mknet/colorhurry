# Spec Delta

## ADDED Requirements

### Requirement: One board shell for every board
Every board firmware SHALL share one board shell. The shell SHALL reach the hardware only through a `Board` trait (buttons, LEDs, delay, tones, radio), so boards differ only in their `Board` implementation.

#### Scenario: Shell tested on the host
Manual: this run of `cargo test` is the check.
- **WHEN** `cargo test -p color-hurry-shell` runs on a laptop
- **THEN** the button edges, LED dimming and advertisement parsing of the board shell are exercised without a board

#### Scenario: Firmware uses the shared shell
Manual: needs the firmware build.
- **WHEN** the board firmware is built
- **THEN** its shell comes from `crates/shell` and the firmware only implements `Board`

### Requirement: Seeed XIAO nRF52840 board
The firmware SHALL run on a Seeed XIAO nRF52840 (Sense) with a Grove RGB LED Stick (10 × WS2813) as the ten LEDs and a Grove Dual Button as button A (Skip) and button B (Apply). It SHALL use the same core, shell and radio protocol as the Circuit Playground Bluefruit. The XIAO has no speaker, so a tone SHALL last its full length without sound.

#### Scenario: XIAO plays mode select
Manual: needs a XIAO with the Grove parts.
- **WHEN** the XIAO boots
- **THEN** the LED stick shows mode select, and the two Grove buttons choose Picker or Caller as buttons A and B do on the CPB

#### Scenario: XIAO and CPB play together
Manual: needs a XIAO and a second board or app.
- **WHEN** the XIAO is Picker on channel N and applies the target color of a Caller on channel N
- **THEN** the Caller completes the task, as with a CPB Picker

#### Scenario: Silent tones keep the timing
Manual: needs a XIAO.
- **WHEN** the core asks the XIAO for a tone of N ms
- **THEN** the shell continues after N ms, as on a board with a speaker
