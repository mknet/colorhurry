# Spec Delta

## MODIFIED Requirements

### Requirement: Color payload
The manufacturer payload after the company ID SHALL be `[magic 'S', version 0x03, opcode CMD_COLOR 0x03, channel, r, g, b]` with channel 0 to 4 and undimmed palette RGB values.

#### Scenario: Red on the wire
- **WHEN** the Picker on channel 2 applies Red
- **THEN** the payload is `53 03 03 02 FF 00 00`

## REMOVED Requirements

### Requirement: Advertised color follows the selection
**Reason**: A Receiver must only react to a color the Picker has committed to, not to every step while skipping.
**Migration**: See "Picker broadcasts only the applied color".

## ADDED Requirements

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
