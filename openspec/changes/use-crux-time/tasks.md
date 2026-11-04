# Tasks

## 1. Core

- [ ] 1.1 Add `crux_time` to the workspace dependencies at the pinned crux revision, with `default-features = false`, and add it to `crates/core/Cargo.toml`. The core feature `facet_typegen` also turns on `crux_time/facet_typegen`
- [ ] 1.2 Re-export the crate as `color_hurry_core::time`
- [ ] 1.3 Remove the `Delay` operation, and add the effect variants `TimeNotifyAfter(time::NotifyAfter)` and `TimeClear(time::ClearTimer)`
- [ ] 1.4 Change `BlinkDone`, `BeatDone` and `ExplosionDone` to carry a `TimerOutcome`, skip them for `serde` and `facet`, and move them to the end of `Event`
- [ ] 1.5 Replace `blink_gen` and `beat_gen` with `blink_timer` and `round_timer` in `Model`, and remove the `Clone` derive from `Model`
- [ ] 1.6 Add the helpers that start a timer and clear a slot, and use them in place of `schedule_blink`, `schedule_beat_ms` and `schedule_explosion_done`
- [ ] 1.7 Clear the slots at Skip, a match, Pause, Reset, the explosion and each change of screen, as `design.md` lists them
- [ ] 1.8 Ignore `TimerOutcome::Cleared` in `update`, and take the handle out of its slot for `TimerOutcome::Completed`
- [ ] 1.9 Change the doc comments that name `Delay` or a generation (`model.rs`, `pause`, `reset`)
- [ ] 1.10 Change the unit tests in `tests.rs` to the generated `time_notify_after` helpers, and resolve the timer requests to get the events. Change the old id in `after_beats_explosion_then_random` to a test that clears the explosion wait
- [ ] 1.11 Verify with `cargo test -p color-hurry-core`

## 2. Harness and features

- [ ] 2.1 Answer `TimeNotifyAfter` and `TimeClear` in `cucumber/harness.rs`, with virtual time, and log the durations and the clears
- [ ] 2.2 Change the steps from "delay" to "timer", and add the step `a timer should have been cleared`
- [ ] 2.3 Change the feature text in `caller.feature`, `caller_only.feature`, `pause.feature`, `reset.feature` and `timing.feature`
- [ ] 2.4 Rewrite the `timing.feature` scenario "Stale blink delays after Skip are ignored without crashing" to check that Skip clears the blink timer and the game continues
- [ ] 2.5 Change the comments in `pause.feature` and `reset.feature` that say the pending beat expires, and check that Pause and Reset clear a timer
- [ ] 2.6 Verify with `cargo test -p color-hurry-core -p color-hurry-ffi`

## 3. Board and web

- [ ] 3.1 Answer `TimeNotifyAfter` and `TimeClear` in `crates/cpb/src/shell.rs`. A clear removes the timer from the list
- [ ] 3.2 Verify with `just build` and `rust-size target/thumbv7em-none-eabihf/release/color-hurry-cpb`
- [ ] 3.3 Answer `TimeNotifyAfter` and `TimeClear` in `crates/web/src/lib.rs`, and verify with `just web-build`

## 4. Native shells

- [ ] 4.1 Add `crux_time` to `crates/ffi/Cargo.toml` for the `codegen` feature, and register `crux_time::shell::swift::TASK` and `crux_time::shell::kotlin::COROUTINE` in `crates/ffi/src/bin/codegen.rs`
- [ ] 4.2 Regenerate the Swift and Kotlin types with `just ffi`, and examine the generated `EffectHandler` and the location of the shipped handlers
- [ ] 4.3 Delegate the time operations to `TaskTimeHandler` in `GameController.swift`, and remove the `delay` handler
- [ ] 4.4 Verify with `just mac-build`, `just ios-build` and `just watch-build`
- [ ] 4.5 Delegate the time operations to `CoroutineTimeHandler` in `GameController.kt`, with `viewModelScope`, and remove the `delay` handler
- [ ] 4.6 Verify with `just android-build`

## 5. Docs

- [ ] 5.1 Change the effect list in `apps/README.md` and the status line of `docs/CORE.md`
- [ ] 5.2 Examine each hit of `rg -n 'Delay|_gen\b' --glob '!target' --glob '!openspec/changes/archive/**' --glob '!apps/*/generated/**'`

## 6. Archive

- [ ] 6.1 Apply the deltas to `openspec/specs`
- [ ] 6.2 Move the change to `openspec/changes/archive/2026-11-04-use-crux-time`
