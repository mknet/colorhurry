# Design

## Context

The core requests its blink ticks, countdown beats and the explosion wait with its own `Delay` operation. Each request carries a generation id, and the core ignores a `*Done` event whose id is old. Every shell has its own delay handler, and no shell can cancel a delay. See proposal.md for the motivation.

The pinned crux revision (`9d56ad1`, branch `spike/no-std`) ships `crux_time` 0.19 with the per-operation API `crux_time::clock::Time`. Without its `std` feature, `crux_time` keeps the operation types, `notify_after` and the timer handles. The crux example `counter-http` runs this subset on the same board.

## Goals / Non-Goals

**Goals:**
- The core uses the `crux_time` operations `NotifyAfter` and `ClearTimer`, and no operation of its own for time.
- The core clears each timer that it no longer wants. No generation id is necessary.
- The Apple and Android shells use the shipped handlers. The Rust shells answer the two operations themselves.
- The game behavior does not change.

**Non-Goals:**
- `Now` and `NotifyAt`. The game measures only durations, and both operations need `std`.
- The TypeScript and C# shell extensions. The web shell is a Rust shell, and there is no C# shell.
- A change to the board time base (`TIMER4`, `time.rs`) or to the 10 ms loop of the board shell.

## Decisions

### Effect variants
`Effect` gets `TimeNotifyAfter(time::NotifyAfter)` and `TimeClear(time::ClearTimer)`, the names that the `crux_time` docs use. `Delay` goes. The core re-exports the crate as `pub use crux_time as time;`, as the `counter-http` core does. Thus the Rust shells name `color_hurry_core::time::operation::NotifyAfter`, and they use the same `crux_time` version as the core.

### Two timer slots in the model
`Model` gets two fields:

- `blink_timer: Option<TimerHandle>`: the blink tick in mode select and in the Picker.
- `round_timer: Option<TimerHandle>`: the countdown beat or the explosion wait of the Caller.

The beat and the explosion wait are never pending at the same time. Today they share `beat_gen` for this reason. One slot for both keeps that relation, and one clear stops either of them.

`TimerHandle` derives `Debug`, `PartialEq` and `Eq`, but not `Clone`. Thus `Model` drops its `Clone` derive and keeps `Debug`. No code clones the model, and `App` puts no bound on `Model`. `TimerHandle` holds a `futures` oneshot `Sender`, which is `Send`.

### One helper starts a timer
A helper starts each timer:

1. It takes the old handle from the slot and clears it.
2. It calls `Time::notify_after` and stores the new handle in the slot.
3. It returns the command, which sends the outcome as `BlinkDone`, `BeatDone` or `ExplosionDone`.

A second helper clears a slot and leaves it empty. The `notes` example of crux uses the same pattern (`restart_timer`).

The core clears a timer at these places:

| Event | Slot that the core clears |
| --- | --- |
| Skip in mode select, and Skip while the Picker selects | `blink_timer`, through the helper that starts the next tick |
| ActivateColor from mode select | `blink_timer` (the Picker then starts a new one) |
| ActivateColor while the Picker selects | `blink_timer`, and no new one starts |
| Skip in a running Caller round | `round_timer` |
| A match | `round_timer` |
| The last beat | `round_timer` holds the explosion wait in place of the beat |
| Pause | both slots |
| Reset | both slots, and a new blink starts |

### Why the ids go
The core calls `TimerHandle::clear` inside `update`. Crux then runs the command of that timer in the same call to `process_event` or `resolve`. The command sees the clear and sends `ClearTimer`. After that, the command no longer listens for the answer to its `NotifyAfter`. Thus a `Completed` outcome can come only from the timer that is in the slot. The core does not compare the handle. When a `Completed` outcome arrives, the core takes the handle out of the slot, then acts on the event. If a reviewer wants a guard, `TimerHandle` compares with `CompletedTimerHandle`, so the core can compare the two in one line.

A clear is answered later, and then the command sends the event with `TimerOutcome::Cleared`. `update` ignores that outcome before it dispatches to a screen.

### Event shape
The three timer events become `BlinkDone(TimerOutcome)`, `BeatDone(TimerOutcome)` and `ExplosionDone(TimerOutcome)`. `TimerOutcome` has no `Facet` and no `serde` derive. Thus each variant gets `#[serde(skip)]` and `#[facet(skip)]`, and its field gets `#[facet(opaque)]`, as in the `notes` and `weather` examples. Only the core sends these events, so no shell needs them.

The three variants move to the end of `Event`. The generated Swift and Kotlin `Event` omit a skipped variant. If a skipped variant stays in the middle, the bincode index of each later variant can differ between Rust and the generated types. At the end of the enum, the indices of the shell events (`Boot` to `RandomByte`, then `Pause` to `CallerOnly`) are the same on both sides. The indices of `Pause`, `Resume`, `Reset` and `CallerOnly` change, and the regenerated types follow.

Alternative: keep unit variants and filter `Cleared` inside the command. Rejected, because the outcome is the documented interface of `crux_time`, and the core then reads in the same way as the crux examples.

### Native shells
The codegen registers `crux_time::shell::swift::TASK` and `crux_time::shell::kotlin::COROUTINE` with `.shell_extension(..)`. The generated module then holds the `TimeHandler` protocol, its types (`Now`, `NotifyAt`, `NotifyAfter`, `ClearTimer`, `Instant`, `Duration`, `TimerId`) and the shipped handler. Each `GameHandler` holds one handler and delegates the two methods to it in one line each. The Android shell gives `CoroutineTimeHandler` the `viewModelScope`, so that the timers stop with the `ViewModel`. The `ffi` crate gets `crux_time` as an optional dependency, and its `codegen` feature turns on `crux_time/facet_typegen`. The core feature `facet_typegen` also turns on `crux_time/facet_typegen`.

### Rust shells
The web shell, the board shell and the harness keep a list of pending timers, each with its `TimerId`:

- `TimeNotifyAfter`: the shell adds the request with its due time (web, harness) or its remaining milliseconds (board).
- When a timer is due, the shell resolves it with `request.operation.id`, then handles the effects that follow.
- `TimeClear`: the shell removes the timer that has the same id, and does not answer it. Then it resolves the clear with the same id and handles the effects that follow.

On the board, the timer list is the only timer state. The 10 ms loop polls it, and no hardware timer belongs to one request. Thus the removal from the list frees the timer. The shell converts the duration with `core::time::Duration::from(request.operation.duration)`.

### Harness
The harness keeps virtual time: `now_ms`, `due_ms`, `advance_ms` and the step `I advance time by N ms`. It logs the duration of each `NotifyAfter` and the id of each `ClearTimer`. The steps say "timer" in place of "delay". A new step `a timer should have been cleared` reads the log of clears.

### Unit tests
`CompletedTimerHandle` has private fields, so a test cannot make a `Completed` outcome. A unit test resolves the `NotifyAfter` request of the command with its own id, then reads the event from `cmd.events()`, as the `counter-http` tests do. The helpers that `#[effect]` generates change from `expect_only_delay` to `expect_only_time_notify_after`.

## Risks / Trade-offs

- [`crux_time` without `std` on the board] → The crux `counter-http` firmware builds the same subset for `thumbv7em-none-eabihf`. `get_timer_id` uses `AtomicUsize::fetch_add`, which this target has. The image gets `futures-channel` and `thiserror`. Check the size with `rust-size` after `just build`.
- [Version pins] → `crux_time` pins `facet = "=0.46.5"`, the same pin as the core. `crux_time` names `crux_core` by path inside the same git checkout, so Cargo resolves one `crux_core`.
- [Codegen with shell extensions] → The protocol types go into the generated modules whether the app sends them or not. Swift has its own `Duration`, and the generated `Duration` can hide it. No Apple shell source names `Duration` now, and the only use of `Task.sleep(for:)` is in the handler that this change deletes.
- [Generated names] → The method names of the generated `EffectHandler` follow the variant names (`timeNotifyAfter` and `timeClear` in the shipped docs). Verify them after the first `just ffi`. The Kotlin types go into the package `com.colorhurry.shared`. Verify that the shipped sources compile there, and that `Platform::Apple` covers watchOS.
- [Late answers] → `TaskTimeHandler` and `CoroutineTimeHandler` also answer a cleared `notifyAfter`. The bridge keeps the request until its answer arrives. Nothing listens for that answer, so `CoreFfi::resolve` does not fail.
- [An extra round trip for each clear] → Each clear sends `ClearTimer` and later gives an event with `Cleared`, on which `update` does no work.

## Migration Plan

1. Core and its tests, with the harness.
2. Board and web shells.
3. Codegen, then the Apple and Android shells.
4. Docs.
