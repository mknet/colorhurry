# Proposal

## Why

"Receiver" names what the board does on the radio: it receives advertisements. It does not name what the player does. At the table, the player with the target color calls the color out, and the Picker picks it. Nobody says "I am the receiver". The radio word leaked into the domain, the same way `ButtonLeft` once did. The role is the Caller.

## What Changes

- The second role is called **Caller** everywhere in the domain: mode select, the round, the specs, the Gherkin features, the core types (`Role::Caller`, `Screen::Caller`), the UI texts of all shells and the docs.
- Radio words stay where they mean the radio: `BleOp::Receiver` (scan for one channel and color), the radio mode in the board firmware, and the event `ColorReceived`.
- The capability `receiver-round` becomes `caller-round`. Requirements whose scenarios name the role get new names, because a modified requirement keeps its scenario names.
- No change to the game rules, the timing or the BLE protocol.

## Capabilities

### New Capabilities
- `caller-round`: the Caller's countdown round against a target color (the requirements of `receiver-round`, in the Caller's words)

### Modified Capabilities
- `mode-select`: steps 5 to 9 enter Caller
- `ble-protocol`: the Caller scans; the channel requirement is renamed
- `timing-and-tones`: acceptance moves to `caller.feature`
- `shell-integration`: browser and phone shells play as Caller
- `receiver-round`: retired, replaced by `caller-round`

## Impact

- `crates/core`: `Role`, `Screen`, `update_caller`, view and tests; `tests/features/receiver.feature` becomes `caller.feature`.
- Generated Swift and Kotlin types: `Screen.caller` / `Screen.CALLER`.
- Board, web, iOS and Android shells: screen matches and UI texts.
- `docs/RECEIVER.md` becomes `docs/CALLER.md`; README files and the OpenSpec project context.
