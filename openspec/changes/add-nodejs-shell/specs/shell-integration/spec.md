# Spec Delta

## ADDED Requirements

### Requirement: Node.js server shell on the shared core
The Node.js server shell SHALL run the shared Rust core through a Node-API (napi-rs) addon without changing the core. It SHALL map UI and BLE input to domain events, perform the effects the core returns (timers and random inside the addon; BLE and tones in the shell) and draw the view in a browser UI served by the server. Game rules, tone tables and the BLE payload format SHALL come only from the core.

#### Scenario: Core over Node-API
Manual: needs the Node.js server build.
- **WHEN** `just node` builds the addon and starts the server
- **THEN** the browser UI shows mode select from the shared core and Skip / Apply / Mode send domain events through the addon

#### Scenario: Caller scans manufacturer data
Manual: needs a board Picker and a machine with BLE scan.
- **WHEN** the server is in Caller on a channel and a Picker on that channel applies the target color
- **THEN** the shell feeds every valid Color Hurry manufacturer advertisement into the core as ColorReceived and the round continues with the same tones as the other shells
