# ble-protocol Specification

## Purpose
Defines how a Color Hurry board announces a color over BLE advertising, so any scanner can read it without a connection.

## Requirements

### Requirement: Non-connectable color advertising
The board SHALL advertise non-connectable under the name `ColorHurry-CPB` and SHALL carry the color in manufacturer data with the Adafruit company ID `0x239A`.

#### Scenario: Board is visible in a scanner
- **WHEN** a phone scans with nRF Connect
- **THEN** it lists `ColorHurry-CPB` with manufacturer data for company `0x239A`

### Requirement: Color payload
The manufacturer payload after the company ID SHALL be `[magic 'S', version 0x04, opcode CMD_COLOR 0x03, channel, color index]` (five bytes) with channel 0 to 4 and the index of a named spectrum color (0 White to 9 Pink). Payloads with another magic, version, opcode, channel or index SHALL be rejected.

#### Scenario: Red on the wire
- **WHEN** the Picker on channel 2 applies Red
- **THEN** the payload is `53 04 03 02 01`

#### Scenario: v3 payloads are rejected
- **WHEN** a shell receives the 7-byte v3 payload `53 03 03 00 FF 00 00`
- **THEN** it is not turned into a ColorReceived

### Requirement: Radio starts after mode select
The board SHALL keep the radio off during mode select and SHALL start advertising (Picker) or scanning (Receiver) only after a role is chosen.

#### Scenario: No advertising during setup
- **WHEN** the board is in mode select
- **THEN** no Color Hurry advertisement is sent

### Requirement: Picker broadcasts only the applied color
The Picker SHALL advertise a color only after ActivateColor and SHALL withdraw it on the next Skip or ActivateColor.

#### Scenario: Apply starts the broadcast
- **WHEN** the Picker applies Red on channel 0
- **THEN** a scanner sees the Red payload for channel 0

#### Scenario: Skip after apply withdraws it
- **WHEN** the Picker presses Skip after applying a color
- **THEN** no color payload is advertised any more

### Requirement: Receiver listens on its channel
During a countdown the Receiver SHALL scan for Color Hurry payloads and SHALL treat only payloads on its own channel as ColorReceived.

#### Scenario: Other channel is ignored
- **WHEN** a Receiver on channel 0 hears a payload for channel 1
- **THEN** the payload has no effect on the round

### Requirement: One encoder and decoder for all shells
Encoding and decoding the payload SHALL be defined once in the shared core. Shells SHALL only extract the manufacturer data for company `0x239A` and pass it on.

#### Scenario: Roundtrip
- **WHEN** channel 2 and Orange are encoded and the result is decoded again
- **THEN** the decoder returns channel 2 and Orange
