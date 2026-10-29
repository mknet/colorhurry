# Proposal

## Why

The spec scenarios describe the game in WHEN/THEN, but nothing checks them. The core tests in `src/tests.rs` exercise functions, not the stories players tell. Gherkin features that drive the core only through domain events make the scenarios executable, and they read like the domain stories from the Event Storming.

## What Changes

- Cucumber test harness in `crates/core/tests` that drives `ColorHurry` through `Event`s and resolves `Delay` and `Random` itself (virtual time, queued random bytes).
- Four feature files: `mode_select.feature`, `picker.feature`, `receiver.feature`, `timing.feature`.
- Every game requirement names the feature file that accepts it. BLE payload encoding stays in unit tests (`protocol.rs`), so there is no protocol feature.
- `just test` runs unit tests and features.

## Capabilities

### New Capabilities

### Modified Capabilities
- `mode-select`: requirements reference `mode_select.feature`
- `picker`: requirements reference `picker.feature`
- `receiver-round`: requirements reference `receiver.feature`
- `ble-protocol`: broadcast and channel requirements reference their features
- `timing-and-tones`: tone and timing requirements reference their features

## Impact

- `crates/core/Cargo.toml` dev-dependencies (`cucumber`, `futures`, `tokio`), new `crates/core/tests/`.
- `justfile` `test` recipe.
