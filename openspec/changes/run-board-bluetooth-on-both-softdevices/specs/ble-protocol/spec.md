# Spec Delta

## ADDED Requirements

### Requirement: Board Bluetooth works on S140 6.1.1 and 7.x boards
The board firmware SHALL provide Bluetooth on a board with SoftDevice S140 6.1.1 and on a board with S140 7.x, built from the same source, without an update of the SoftDevice or the bootloader. The Picker SHALL broadcast non-connectable advertisements and the Receiver SHALL only scan, so no pairing or connection is ever needed. A flash SHALL use the memory layout that matches the SoftDevice of the board, either detected from the board or chosen by the user.

#### Scenario: Picker on an S140 6.1.1 board
- **WHEN** a board with S140 6.1.1 is Picker on channel N and applies a color
- **THEN** a Receiver on channel N gets a ColorReceived for that color

#### Scenario: Picker on an S140 7.x board
- **WHEN** a board with S140 7.x is Picker on channel N and applies a color
- **THEN** a Receiver on channel N gets a ColorReceived for that color, as it does with an S140 6.1.1 board

#### Scenario: Receiver on either board
- **WHEN** a board with S140 6.1.1 or 7.x is Receiver on channel N and a Picker on channel N applies a color
- **THEN** the board gets a ColorReceived for that color

#### Scenario: No pairing or connection
- **WHEN** a Picker and a Receiver play a round
- **THEN** the Picker only sends non-connectable advertisements, the Receiver only scans, and no pairing prompt or connection occurs on any device

#### Scenario: Flash picks the layout that matches the SoftDevice
- **WHEN** the user flashes a board through its UF2 bootloader and `INFO_UF2.TXT` reports S140 6.x or 7.x
- **THEN** the flash uses the memory layout for that version (`sd-v6` or `sd-v7`)

#### Scenario: Unknown SoftDevice version
- **WHEN** `INFO_UF2.TXT` reports a SoftDevice version that is not 6.x or 7.x, or none
- **THEN** the flash stops with a message and the user chooses the layout

#### Scenario: User chooses the layout
- **WHEN** the user flashes through the serial recipe `just flash` or `just flash-v7`
- **THEN** the flash uses the layout that the recipe names, because serial DFU cannot see the SoftDevice version
