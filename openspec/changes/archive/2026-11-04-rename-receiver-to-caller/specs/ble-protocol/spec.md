# Spec Delta

## RENAMED Requirements
- FROM: `### Requirement: Receiver listens on its channel`
- TO: `### Requirement: Caller listens on its channel`

## MODIFIED Requirements

### Requirement: Radio starts after mode select
The board SHALL keep the radio off during mode select and SHALL start advertising (Picker) or scanning (Caller) only after a role is chosen.

#### Scenario: No advertising during setup
- **WHEN** the board is in mode select
- **THEN** no Color Hurry advertisement is sent

### Requirement: Caller listens on its channel
During a countdown the Caller SHALL scan for Color Hurry payloads and SHALL treat only payloads on its own channel as ColorReceived.
Acceptance: `crates/core/tests/features/caller.feature`.

#### Scenario: Other channel is ignored
- **WHEN** a Caller on channel 0 hears a payload for channel 1
- **THEN** the payload has no effect on the round

### Requirement: Pause switches the radio off
`Pause` SHALL switch the radio off. `Resume` SHALL send the radio op of the current screen again: the Picker op for its channel and its current broadcast in Picker, and the Caller op for the target of a running round in Caller. In mode select, and in Caller with no running round, the radio SHALL stay off.
Acceptance: `crates/core/tests/features/pause.feature`.

#### Scenario: Caller goes to the background
- **WHEN** the app goes to the background during a countdown
- **THEN** the radio stops scanning

#### Scenario: Picker comes back
- **WHEN** a Picker on channel 1 with Red applied goes to the background and comes back
- **THEN** the Picker on channel 1 broadcasts Red again

#### Scenario: Caller comes back
- **WHEN** a Caller on channel 0 with target Red goes to the background and comes back
- **THEN** the Caller scans on channel 0 for Red again
