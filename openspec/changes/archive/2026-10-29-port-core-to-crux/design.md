# Design

## Context

`docs/CORE.md` inventories what belongs in the core and what in the shells, and lists the violations in today's tree (match rule in the CPB radio layer, model peeks in both shells, duplicated LCG, diverging explosion, beat timing in the shells, overloaded `Event::Tick`, flags instead of states, shell words in the domain). See proposal.md for the motivation.

## Goals / Non-Goals

**Goals:**
- The core says WHAT, the shell says HOW: shells perform effects and draw the `ViewModel`, nothing else.
- The core stays `no_std` and links directly into the firmware (no FFI bridge on the device).

**Non-Goals:**
- Picker in the browser (browsers cannot advertise).
- Native mobile shells (separate change).

## Decisions

### Crux `App` from the `spike/no-std` branch
Follow `examples/cpb-counter`: `#[effect] enum Effect`, custom `Delay` operation via `#[derive(Operation)]`, `Command::new` with `request_from_shell`. The shells call `process_event` and `resolve`.
Alternative: keep the hand-rolled `update` / `Effect` list. Rejected, because it cannot stop shells from reading the model.

### Delays carry a generation id
`BlinkDone(id)` and `BeatDone(id)` only count if `id` matches the current generation. Skip, a match or a mode change bump the generation, so stale timers from an earlier state are ignored without cancelling anything in the shell.

### Randomness through an effect
`Random` resolves to one byte; the target is `color_from_entropy(byte)`. Board and web keep their seeded generator for now, but it sits behind the effect, so a hardware RNG can replace it without touching the core. Tests queue bytes and get deterministic rounds.

### Color on the wire: option C, protocol v4
Decided 2026-10-29. Options: A, RGB on the wire (v3, no cutover); B, named `Color` in the domain but RGB on the wire; C, named color index on the wire (v4). C is closest to "players shout Red!", makes the match an enum compare and keeps `Rgb` a pure display concern. Cost: board and web must be updated together.

### Tones in the core
Decided 2026-10-29. Meaning and parameters (Hz, ms) of Skip, Apply, countdown step and explosion live in the core as `PlayTone { sequence }`. The explosion is the CPB sweep (180 to 45 Hz, ten 100 ms steps) everywhere.

## Risks / Trade-offs

- [Crux `spike/no-std` is a branch, not a release] → path dependency on a local checkout, pinned by the workspace.
- [Heap on the nRF52840] → `embedded-alloc` heap sized as in the spike notes.
- [v3 and v4 boards in the same room] → v4 decoder rejects 7-byte v3 payloads explicitly.

## Migration Plan

Order from `docs/CORE.md` section 8: protocol module, tones, tests, round enum, event names, `App`, delays, CPB match path, model peeks, wire both shells, demo flow board Picker to browser Receiver.
