# Spec Delta

## Purpose

Defines how a Color Hurry board announces a color over BLE advertising, so any scanner can read it without a connection.

## ADDED Requirements

### Requirement: Non-connectable color advertising
The board SHALL advertise non-connectable under the name `ColorHurry-CPB` and SHALL carry the color in manufacturer data with the Adafruit company ID `0x239A`.

#### Scenario: Board is visible in a scanner
- **WHEN** a phone scans with nRF Connect
- **THEN** it lists `ColorHurry-CPB` with manufacturer data for company `0x239A`

### Requirement: Color payload
The manufacturer payload after the company ID SHALL be `[magic 'S', version 0x02, opcode CMD_COLOR 0x03, r, g, b]` with undimmed palette RGB values.

#### Scenario: Red on the wire
- **WHEN** the selected color is red
- **THEN** the payload is `53 02 03 FF 00 00`

### Requirement: Advertised color follows the selection
The board SHALL advertise the currently selected picker color and SHALL refresh the advertising data within about 50 ms of a change.

#### Scenario: Skip updates the advertisement
- **WHEN** the player presses Skip
- **THEN** a scanner sees the RGB of the new selection within about 50 ms
