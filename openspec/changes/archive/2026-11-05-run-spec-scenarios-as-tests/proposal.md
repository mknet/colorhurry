# Proposal

## Why

Every example exists twice. The specs in `openspec/specs` hold 94 scenarios in declarative WHEN/THEN form. The Gherkin features in `crates/core/tests/features` hold 67 other scenarios (91 runs with the examples of one outline) in imperative form, with other names and other cuts. The only link is one line per requirement, `Acceptance: <feature>`. Nothing checks that both texts agree. Archiving a change moves the change, not the living spec, so the duplication is permanent.

## What Changes

- The spec scenarios become the tests. A parser for cucumber-rs reads `openspec/specs/*/spec.md` and hands Gherkin features to the existing runner: capability becomes Feature, requirement becomes Rule with its SHALL text as description, scenario stays Scenario.
- The `.feature` files are deleted. Harness and step mechanics stay; the steps take the declarative wording of the specs.
- Scenarios that only lived in a feature and are about the game join the spec, among them the ones for Pause, Reset and CallerOnly.
- The spec scenarios get one style: GIVEN for the context, WHEN for the action, THEN for the outcome, with recurring phrases ("the Caller on channel 0 with target Red", "Red arrives on channel 0", "the countdown stops").
- Examples that only lived in a feature and are about the game join the spec. Technical examples become unit tests in `crates/core/src/tests.rs`.
- Scenarios that need a board, a browser or a phone are marked `Manual:` and show up as skipped.
- The `Acceptance:` lines are removed.

## Capabilities

### New Capabilities

### Modified Capabilities
- `ble-protocol`: scenarios in GIVEN/WHEN/THEN; Picker and Caller start their radio role after mode select; apply after apply withdraws the broadcast; the scanner check is manual; Resume in a selecting Picker and in mode select
- `caller-round`: scenarios in GIVEN/WHEN/THEN; a color while picking is ignored; ActivateColor has no effect on the Caller (new requirement); the next round starts only after the explosion, and Skip during the explosion starts no second round
- `mode-select`: scenarios in GIVEN/WHEN/THEN; entering a role clears the choice; Reset from a selecting Picker, Apply after Reset into a Picker, Skip tone and Apply without Skip in a caller-only game, CallerOnly keeps a Caller choice
- `picker`: scenarios in GIVEN/WHEN/THEN; the channel picks the first selection; ActivateColor on an applied color also returns to selecting; mode select names no picker color
- `shell-integration`: scenarios that need a shell are manual; the core picks the target from the shell's byte; a fired timer requests the next blink tick
- `timing-and-tones`: scenarios in GIVEN/WHEN/THEN; six ticks per blink phase; the countdown table and the explosion sweep are checked step by step; Skip clears the blink timer, checked to the millisecond; Skip while paused keeps the choice; Resume in Picker restarts the blink

## Impact

- `crates/core/tests/cucumber`: new `spec.rs` (parser), declarative steps in `mod.rs`, harness fires timers only when time advances and keeps a random request open until a step answers it.
- `crates/core/tests/features`: deleted.
- `crates/core/Cargo.toml`: test target `spec_parser` for the parser's unit test.
- `README.md`, `justfile`: test comments.
- No change to the game rules, the timing or the BLE protocol. The requirement "Next round after the countdown" now names the order that the code already has: the random byte for the next round comes after the explosion.
