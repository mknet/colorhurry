# ble-protocol Specification

## Purpose
Defines how a Color Hurry board announces a color over BLE advertising, so any scanner can read it without a connection.

## Requirements

### Requirement: Non-connectable color advertising
The board SHALL advertise non-connectable under the name `ColorHurry-CPB` and SHALL carry the color in manufacturer data with the Adafruit company ID `0x239A`.

#### Scenario: Board is visible in a scanner
Manual: needs a board and a phone.
- **WHEN** a phone scans with nRF Connect
- **THEN** it lists `ColorHurry-CPB` with manufacturer data for company `0x239A`

### Requirement: Color payload
The manufacturer payload after the company ID SHALL be `[magic 'S', version 0x04, opcode CMD_COLOR 0x03, channel, color index]` (five bytes) with channel 0 to 4 and the index of a named spectrum color (0 White to 9 Pink). Payloads with another magic, version, opcode, channel or index SHALL be rejected.

#### Scenario: Red on the wire
- **GIVEN** the Picker on channel 2
- **WHEN** the player applies Red
- **THEN** the payload is `53 04 03 02 01`

#### Scenario: v3 payloads are rejected
- **WHEN** the 7-byte v3 payload `53 03 03 00 FF 00 00` arrives
- **THEN** it is not turned into a ColorReceived

### Requirement: Radio starts after mode select
The board SHALL keep the radio off during mode select and SHALL start advertising (Picker) or scanning (Caller) only after a role is chosen.

#### Scenario: No advertising during setup
- **GIVEN** the board in mode select
- **WHEN** the player presses Skip
- **THEN** no Color Hurry advertisement is sent

#### Scenario: Picker starts without a color
- **GIVEN** the board in mode select
- **WHEN** the player enters Picker on channel 1
- **THEN** the board advertises on channel 1 without a color

#### Scenario: Caller starts scanning
- **GIVEN** the board in mode select
- **WHEN** the player enters Caller on channel 3
- **AND** the shell answers the random request with 1
- **THEN** the board scans channel 3 for Red

### Requirement: Picker broadcasts only the applied color
The Picker SHALL advertise a color only after ActivateColor and SHALL withdraw it on the next Skip or ActivateColor.

#### Scenario: Apply starts the broadcast
- **GIVEN** the Picker on channel 0
- **WHEN** the player applies Red
- **THEN** a scanner sees the Red payload for channel 0

#### Scenario: Skip after apply withdraws it
- **GIVEN** the Picker on channel 0 with Red applied
- **WHEN** the player presses Skip
- **THEN** no color payload is advertised any more

#### Scenario: Apply after apply withdraws it
- **GIVEN** the Picker on channel 2 with Red applied
- **WHEN** the player presses ActivateColor
- **THEN** no color payload is advertised any more

### Requirement: One encoder and decoder for all shells
Encoding and decoding the payload SHALL be defined once in the shared core. Shells SHALL only extract the manufacturer data for company `0x239A` and pass it on.

#### Scenario: Roundtrip
- **WHEN** channel 2 and Orange are encoded
- **AND** the result is decoded again
- **THEN** the decoder returns channel 2 and Orange

### Requirement: Caller listens on its channel
During a countdown the Caller SHALL scan for Color Hurry payloads and SHALL treat only payloads on its own channel as ColorReceived.

#### Scenario: Other channel is ignored
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** Red arrives on channel 1
- **THEN** the countdown keeps running

### Requirement: Pause switches the radio off
`Pause` SHALL switch the radio off. `Resume` SHALL send the radio op of the current screen again: the Picker op for its channel and its current broadcast in Picker, and the Caller op for the target of a running round in Caller. In mode select, and in Caller with no running round, the radio SHALL stay off.

#### Scenario: Caller goes to the background
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** the app goes to the background
- **THEN** the radio is off

#### Scenario: Picker comes back
- **GIVEN** the Picker on channel 1 with Red applied
- **WHEN** the app goes to the background
- **AND** the app comes back
- **THEN** a scanner sees the Red payload for channel 1

#### Scenario: Picker comes back while selecting
- **GIVEN** the Picker on channel 1
- **WHEN** the app goes to the background
- **AND** the app comes back
- **THEN** the board advertises on channel 1 without a color

#### Scenario: Caller comes back
- **GIVEN** the Caller on channel 0 with target Red
- **WHEN** the app goes to the background
- **AND** the app comes back
- **THEN** the board scans channel 0 for Red

#### Scenario: Mode select comes back
- **GIVEN** the board in mode select
- **WHEN** the app goes to the background
- **AND** the app comes back
- **THEN** no Color Hurry advertisement is sent
