# Proposal

## Why

The time effect is a standard Crux capability (`crux_time`), so the core does not need an operation of its own for it. With `crux_time`, the Apple and Android shells use a timer handler that Crux ships, and they keep no timer code of their own. The core clears each timer that it no longer wants, so the shells can free that timer, and an old timer no longer runs to its end only for the core to ignore it.

## What Changes

- The core removes its own `Delay` operation. It sends the `crux_time` operations `NotifyAfter` and `ClearTimer` through `crux_time::clock::Time::notify_after`. The `Effect` variants are `TimeNotifyAfter` and `TimeClear`.
- The core keeps the `TimerHandle` of each pending timer in `Model`. It clears the blink timer and the round timer (the countdown beat or the explosion wait) when the state changes: Skip, a match, Pause, Reset, the explosion and each change of screen.
- `blink_gen` and `beat_gen` go. The events `BlinkDone`, `BeatDone` and `ExplosionDone` carry a `TimerOutcome` and no id. The core ignores `TimerOutcome::Cleared`.
- The three timer events move to the end of `Event` and are not serialized, because only the core sends them. They are no longer in the generated Swift and Kotlin `Event`.
- The codegen registers the shell extensions `crux_time::shell::swift::TASK` and `crux_time::shell::kotlin::COROUTINE`. The Apple shells delegate to `TaskTimeHandler`, and the Android shell delegates to `CoroutineTimeHandler`. The hand-written `delay` handlers go.
- The web shell, the board shell and the cucumber harness answer `NotifyAfter` and `ClearTimer` themselves, because they are Rust shells. A clear removes the pending timer, so the timer never fires.
- The specs and the Gherkin steps use the term "timer" for what the core requests. For example, the step "a delay of 50 ms should have been requested" becomes "a timer of 50 ms should have been requested". A new step checks that the core cleared a timer.
- Three places in the tests depend on the generation mechanism. They now check the same rule ("an old timer does not change the game") through a clear:
  - `timing.feature`, "Stale blink delays after Skip are ignored without crashing"
  - the unit test `after_beats_explosion_then_random`, which sends `ExplosionDone` with an old id
  - the comments in `pause.feature` and `reset.feature` that say the pending beat "expires here and has no effect"
- The game behavior does not change.

## Capabilities

### New Capabilities

### Modified Capabilities
- `timing-and-tones`: the core clears the timers of a previous state, and of the state at `Pause`, and a cleared timer has no effect
- `mode-select`: `Reset` clears the pending timers, and the scenarios name a blink timer
- `shell-integration`: the effect list names the timer and its clear, and a new requirement says how each shell answers the time effects

`caller-round` and `picker` have no delta. Their text does not name the delay or the generations, and it stays true.

### Overlap with an open change
The open change `run-board-bluetooth-on-both-softdevices` modifies the requirement "Timing next to the radio stack". Its first sentence names "Delays", in the general sense, and that sentence stays true. Thus this change does not modify that requirement, and the two changes do not touch the same requirement.

## Impact

- `Cargo.toml`: a workspace dependency on `crux_time` at the pinned crux revision, with `default-features = false`.
- `crates/core`: `Cargo.toml`, `lib.rs`, `model.rs` and `tests.rs`.
- `crates/core/tests`: `cucumber/harness.rs`, `cucumber/mod.rs`, and the features `caller.feature`, `caller_only.feature`, `pause.feature`, `reset.feature` and `timing.feature`.
- `crates/cpb/src/shell.rs` and `crates/web/src/lib.rs`.
- `crates/ffi`: `Cargo.toml` and `src/bin/codegen.rs`.
- Regenerated Swift and Kotlin types, which now also hold the `crux_time` protocol types and the shipped handler.
- Apple: `apps/apple/Shared/Shell/GameController.swift`.
- Android: `apps/android/app/src/main/java/com/colorhurry/app/shell/GameController.kt`.
- Docs: `apps/README.md` and the status line of `docs/CORE.md`.
