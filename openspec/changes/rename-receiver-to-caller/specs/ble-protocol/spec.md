# Spec Delta

## MODIFIED Requirements

### Requirement: Radio starts after mode select
The board SHALL keep the radio off during mode select and SHALL start advertising (Picker) or scanning (Caller) only after a role is chosen.

#### Scenario: No advertising during setup
- **WHEN** the board is in mode select
- **THEN** no Color Hurry advertisement is sent

## REMOVED Requirements

### Requirement: Receiver listens on its channel
**Reason**: The role is called Caller now; the requirement moves to a new name.
**Migration**: See "Caller listens on its channel".

## ADDED Requirements

### Requirement: Caller listens on its channel
During a countdown the Caller SHALL scan for Color Hurry payloads and SHALL treat only payloads on its own channel as ColorReceived.
Acceptance: `crates/core/tests/features/caller.feature`.

#### Scenario: Other channel is ignored
- **WHEN** a Caller on channel 0 hears a payload for channel 1
- **THEN** the payload has no effect on the round
