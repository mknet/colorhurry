# Spec Delta

## MODIFIED Requirements

### Requirement: Color payload
The manufacturer payload after the company ID SHALL be `[magic 'S', version 0x04, opcode CMD_COLOR 0x03, channel, color index]` (five bytes) with channel 0 to 4 and the index of a named spectrum color (0 White to 9 Pink). Payloads with another magic, version, opcode, channel or index SHALL be rejected.

#### Scenario: Red on the wire
- **WHEN** the Picker on channel 2 applies Red
- **THEN** the payload is `53 04 03 02 01`

#### Scenario: v3 payloads are rejected
- **WHEN** a shell receives the 7-byte v3 payload `53 03 03 00 FF 00 00`
- **THEN** it is not turned into a ColorReceived

## ADDED Requirements

### Requirement: One encoder and decoder for all shells
Encoding and decoding the payload SHALL be defined once in the shared core. Shells SHALL only extract the manufacturer data for company `0x239A` and pass it on.

#### Scenario: Roundtrip
- **WHEN** channel 2 and Orange are encoded and the result is decoded again
- **THEN** the decoder returns channel 2 and Orange
