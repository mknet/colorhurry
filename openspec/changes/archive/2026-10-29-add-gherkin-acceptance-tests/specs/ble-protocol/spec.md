# Spec Delta

## MODIFIED Requirements

### Requirement: Picker broadcasts only the applied color
The Picker SHALL advertise a color only after ActivateColor and SHALL withdraw it on the next Skip or ActivateColor.
Acceptance: `crates/core/tests/features/picker.feature`.

#### Scenario: Apply starts the broadcast
- **WHEN** the Picker applies Red on channel 0
- **THEN** a scanner sees the Red payload for channel 0

#### Scenario: Skip after apply withdraws it
- **WHEN** the Picker presses Skip after applying a color
- **THEN** no color payload is advertised any more

### Requirement: Receiver listens on its channel
During a countdown the Receiver SHALL scan for Color Hurry payloads and SHALL treat only payloads on its own channel as ColorReceived.
Acceptance: `crates/core/tests/features/receiver.feature`.

#### Scenario: Other channel is ignored
- **WHEN** a Receiver on channel 0 hears a payload for channel 1
- **THEN** the payload has no effect on the round
