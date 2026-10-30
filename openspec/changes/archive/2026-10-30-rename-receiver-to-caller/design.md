# Design

## Context

See proposal.md for the motivation. The word "receiver" appears on two levels today: as the player's role and as the radio mode that scans for advertisements.

## Goals / Non-Goals

**Goals:**
- One word per concept: Caller for the role, receiver/receive for the radio.

**Non-Goals:**
- Renaming the BLE operations or the protocol. `BleOp` describes what the radio does (`Off`, `Picker`, `Receiver`, `Broadcast`), and a scanning radio is a receiver.

## Decisions

### Caller, not Announcer or Spotter
Caller names the action at the table ("call out the color") and pairs with Picker: one calls, one picks. Announcer is longer and formal; Spotter stresses seeing instead of calling out.

### Radio words stay
`BleOp::Receiver`, `RadioMode::Receiver`, `set_radio_receiver` and `ColorReceived` keep their names. They describe the radio or an event of the round, not the player. The Gherkin step "BLE should have received a Receiver op" keeps its wording for the same reason.

### Capability rename by retire and add
OpenSpec cannot rename a capability. The change adds `caller-round` with all requirements in the Caller's words and removes every requirement from `receiver-round`; `retire_capabilities: true` deletes the empty spec at archive time. A MODIFIED requirement must keep its scenario names, so requirements whose scenarios name the role are removed and added under a new name.

## Risks / Trade-offs

- [Historic docs and archived changes still say Receiver] → They describe the state at their time and stay unchanged; `docs/CORE.md` is the dated inventory of the Crux port.
