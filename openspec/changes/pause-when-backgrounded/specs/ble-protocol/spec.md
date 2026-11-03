# Spec Delta

## ADDED Requirements

### Requirement: Pause switches the radio off
`Pause` SHALL switch the radio off. `Resume` SHALL send the radio op of the current screen again: the Picker op for its channel and its current broadcast in Picker, and the Receiver op for the target of a running round in Receiver. In mode select, and in Receiver with no running round, the radio SHALL stay off.
Acceptance: `crates/core/tests/features/pause.feature`.

#### Scenario: Receiver goes to the background
- **WHEN** the app goes to the background during a countdown
- **THEN** the radio stops scanning

#### Scenario: Picker comes back
- **WHEN** a Picker on channel 1 with Red applied goes to the background and comes back
- **THEN** the Picker on channel 1 broadcasts Red again

#### Scenario: Receiver comes back
- **WHEN** a Receiver on channel 0 with target Red goes to the background and comes back
- **THEN** the Receiver scans on channel 0 for Red again
