# Color Hurry shared core (inventory for the Crux port)

Status: confirmed for port (Marcel, 2026-10-29). Color on wire = C/v4; tones (Hz/ms) in core; inventory and effect shape accepted.

This note inventories what belongs in the shared core versus the shells, checks that list against the current tree, records known violations with file and function, sketches the Crux target shape (from Stuart's `spike/no-std` / `examples/cpb-counter`), and lists migration steps. Guiding rule: the core says WHAT, the shell says HOW.

Two meanings of "core":

- **Core Domain (DDD):** the game rules that set Color Hurry apart.
- **Shared core (Crux):** everything that must not differ between platforms (rules plus supporting pieces such as the palette and the BLE payload layout).

---

## 1. What belongs in the shared core (verified)

| Area | Current home | Notes |
|------|--------------|--------|
| Roles and mode selection | `crates/core/src/model.rs`, `update.rs`, `palette.rs` | `Role::Picker/Receiver { channel }`, channels 0..4, `mode_step`, `ring_step_led` / `ring_step_channel` / `ring_step_is_picker` |
| Picker phases | `model::PickerPhase`, `update_picker` | Selecting / Applied, spectrum cycle, apply and withdraw broadcast |
| Receiver round | `update_receiver`, `start_round` | Match on channel + RGB, abort on left button, explosion after `COUNTDOWN_STEPS` (10), then wait for entropy |
| Palette | `palette.rs` | `CHANNEL_COLORS`, `SPECTRUM`, `color_from_entropy`. **Decided:** match identity is named/`Color` (or spectrum index) on the wire (v4); `Rgb` is only for display, 0..255 before shell dimming |
| View | `view.rs` | LED colors per screen, countdown fill, visible channel → `ViewModel` |
| Blink counter | `BLINK_TICKS = 6` in `model.rs` | Half-period in ticks; UI tick period lives in shells today (50 ms both) |
| Tone kinds + parameters | `effects::ToneKind` today; Hz/ms still in shells | **Decided:** meaning and frequency/duration live in core so Skip, Apply, CountdownStep, and Explosion sound the same everywhere. Shell only plays (PWM / WebAudio) |
| BLE protocol | duplicated in shells | Company ID `0x239A`, magic `'S'`, version 3, `CMD_COLOR`, payload `[magic, version, opcode, channel, r, g, b]`, channel validation. Encode/decode should be pure functions in core; today they are not |

Beat duration, blink period in milliseconds, tone Hz/ms, and Random are intended for the core (as constants and/or `Delay` / `Tone` / `Random` effects) but are still shell-owned. See section 3.

---

## 2. What belongs in the shell (verified)

| Concern | CPB | Web |
|---------|-----|-----|
| Clocks / timers | `crates/cpb/src/time.rs`, Embassy delays in `shell.rs` | `setInterval` 50 ms / 1000 ms in `web/src/lib.rs` |
| Entropy source | LCG in `shell::next_entropy` (should become real RNG + `Random` effect) | Same LCG in `Shell::next_entropy` |
| Radio stack | SoftDevice, `ble_broadcast::radio_loop`, advertise/scan | Web Bluetooth chooser, `watchAdvertisements` |
| Device name / filters | `DEVICE_NAME`, short name `ColHurry` | Name prefixes and company-id filters in `web/src/ble.rs` |
| GPIO, debounce | `buttons.rs` | DOM buttons |
| NeoPixel PWM + **dimming** | `neopixel_pwm::scale_channel`, `MAX_CHANNEL = 8` | DOM/CSS LED circle (layout is shell) |
| Audio output | `speaker.rs` PWM | WebAudio oscillator |

Shells must turn input into domain `Event`s, execute effects, and draw the `ViewModel`. They must not decide matches, hold game constants, or read model fields for control flow.

---

## 3. Inventory corrections and additions

Checked against the code. Items that refine or extend the brief:

1. **Match filtering is CPB-only.** The web shell already forwards every parsed `(channel, color)` via `Shell::ingest_live_ble` → `Event::BleColorReceived` without comparing to the target. Only the board path decides the match in the radio layer.
2. **Explosion tones diverge.** Skip and Apply match across platforms (380 Hz / 55 ms, 784 Hz / 130 ms). Countdown steps share the same `DESCENDING_HZ` table and 200 ms. Explosion does **not**: CPB `speaker::play_explosion` is a 10-step sweep 180→45 Hz × 100 ms (~1 s); web uses a single 120 Hz / 400 ms beep. Putting tone parameters in the core is not only a recommendation for Skip/Apply; it is required for Explosion parity.
3. **No shared encode.** CPB mutates a `static mut CMD_PAYLOAD` in `ble_broadcast::set_color`. Web only has `parse_color_payload`. Core has neither encode nor decode. Protocol tests today live under `web/src/ble.rs` (`parses_cmd_color_payload`, `rejects_wrong_magic`).
4. **Misleading protocol comment on CPB.** `ble_broadcast::protocol` says payload is `[opcode, channel, r, g, b]`; the bytes on the air are `[magic, version, opcode, channel, r, g, b]` (7 bytes after company ID).
5. **Web ignores BLE effects.** `apply_effect` treats `SetBlePicker`, `SetBleReceiver`, `SetPickerBroadcast`, and `ClearBle` as no-ops. Live receive is wired outside the effect loop. After the port, the web Receiver still needs a way to feed ads into `process_event`; Picker-on-web remains out of scope for the demo (board Picker → browser Receiver).
6. **`Effect::Render` is not the paint trigger.** Both shells paint after every `update` (CPB always calls `neopixel_pwm::show_pixels`; web `paint` after dispatch). Crux will use `render()` / `RenderOperation` as in the spike.
7. **`NUM_LEDS` is duplicated** in `cpb/src/neopixel_pwm.rs` (`pub const NUM_LEDS: usize = 10`) alongside `color_hurry_core::NUM_LEDS`. Hardware buffer size can stay local; the game constant should have a single source of truth in core.
8. **Beat length is approximately 1 s but assembled differently.** Web: 1000 ms `setInterval` → `Event::Tick`. CPB: `dispatch(Tick)` (plays ~200 ms countdown tone) then `poll_match_or_abort(800)`. Same product, different ownership; should become core-requested `Delay { millis }`.
9. **UI blink period:** both shells use 50 ms; with `BLINK_TICKS = 6` that is ~300 ms per half blink. The 50 ms value is not in core today.
10. **SoftDevice bootstrap reads the model.** After mode select, `Shell::bootstrap_radio` uses `view(...).screen` and `model.channel()` to re-apply radio setup. That is another model peek the Crux port should replace with effects already emitted (or re-emitted) when BLE becomes ready.

---

## 4. Violations (file and function)

### a. Match rule duplicated (CPB)

- `crates/cpb/src/ble_broadcast.rs` → `on_adv_report`: compares scanned channel and RGB to `receiver_target` / `receiver_channel`, sets `matched`.
- `crates/cpb/src/ble_broadcast.rs` → `take_color_match`: returns that flag.
- `crates/cpb/src/shell.rs` → `poll_match_or_abort`: on match, sends `Event::BleColorReceived { channel: self.model.channel(), color: self.model.receiver_target }` (the **stored target**, not the bytes heard).
- Core `update_receiver` for `BleColorReceived` then checks channel and color again, so it only rubber-stamps the shell.

**Fix:** parse ads to `(channel, color)` only; forward every valid parse; only the core decides match / ignore.

### b. Shells read model internals

| Location | Fields / API used for control flow |
|----------|--------------------------------------|
| `cpb/src/shell.rs` → `run_game` | `awaiting_entropy`, `countdown_active` |
| `cpb/src/shell.rs` → `poll_match_or_abort` | `receiver_target`, `channel()` |
| `cpb/src/shell.rs` → `bootstrap_radio` | `screen` (via `view`), `channel()` |
| `web/src/lib.rs` → `dispatch` | `awaiting_entropy`, `screen` |
| `web/src/lib.rs` → `ui_ticks` / `countdown_ticks` | `screen`, `countdown_active` |

In Crux the model is private. Replace with effects (`Random`, `Delay`, BLE ops) and reading only `ViewModel` for display.

### c. Random generator duplicated

Identical LCG (`wrapping_mul(1_103_515_245).wrapping_add(12_345)`, high byte):

- `cpb/src/shell.rs` → `Shell::next_entropy`
- `web/src/lib.rs` → `Shell::next_entropy`

Replace with a `Random` effect; shells supply nRF RNG / `crypto.getRandomValues`.

### d. Tone table duplicated; Explosion not aligned

| Kind | CPB | Web |
|------|-----|-----|
| Skip | `speaker.rs` 380 Hz / 55 ms | `play_tone` 380 / 55 |
| Apply | `speaker.rs` 784 Hz / 130 ms | 784 / 130 |
| CountdownStep | `shell.rs` `DESCENDING_HZ` + 200 ms; also `speaker_demo.rs` | literal table + 200 ms |
| Explosion | `play_explosion` sweep ~1 s | 120 Hz / 400 ms |

**Decided (Marcel, 2026-10-29):** core owns meaning **and** Hz/ms (including a single Explosion definition; pick the CPB sweep or a simplified form during the port). Shell only plays.

### e. Beat timing in the shells

- Web: 1000 ms interval → `countdown_ticks` → `Event::Tick`.
- CPB: Tick then 800 ms poll window (plus tone duration inside Tick handling).

Move into the core as `Delay { millis }` with `.then_send(...)` (spike pattern).

### f. `Event::Tick` overloaded

- ModeSelect / Picker: blink cadence (~50 ms).
- Receiver: countdown beat (~1 s).

Same enum variant, different meaning. Split (e.g. domain `BlinkTick` / `CountdownBeat`) and/or let the core request both rhythms via `Delay`.

### g. BLE protocol constants duplicated

- `cpb/src/ble_broadcast.rs` → `COMPANY_ID`, `protocol::{MAGIC, VERSION, CMD_*}`, `parse_color_from_adv`, `set_color` / `CMD_PAYLOAD`.
- `web/src/ble.rs` → same constants, `parse_color_payload`, tests.

Merge encode/decode + constants into one core module. Shells keep AD structure walking (CPB) and Web Bluetooth map access (web).

### h. Flags instead of states

`countdown_active` + `awaiting_entropy` on `Model` allow combinations that do not mean a valid round. Target shape:

```text
enum Round {
    Waiting,
    Running { target: Rgb, step: u8 },
    Exploded,
}
```

(`Exploded` may be transient if the next command immediately requests `Random` and returns to `Waiting`; exact enum is for the port.)

### i. Shell words in the domain

Current: `Event::ButtonLeft`, `Event::ButtonRight`, `BleColorReceived`, `Tick`, `Entropy`.

Talk / target domain names (shell maps hardware or HTML onto them): Skip, ActivateColor (Apply), ColorReceived, CountdownBeat, TaskCompleted, CountdownExpired, plus entropy/random completion and blink as needed.

---

## 5. Proposed effect list (Crux shape)

Aligned with `examples/cpb-counter` on branch `spike/no-std`: `impl App`, `update -> Command<Effect, Event>`, `#[effect]`, custom `Delay`, shell calls `core.process_event(...)` and `core.resolve(...)`.

Sketch (adapt names to the real API when wiring the dependency):

```text
#[effect]
enum Effect {
    Render(RenderOperation),
    Delay(Delay),           // millis; shell: Embassy timer / setTimeout
    Random(Random),         // e.g. request 1 byte; shell: RNG
    Ble(BleOperation),      // Off | Picker { channel } | Receiver { channel, color } | Broadcast { color: Option<Rgb> }
    Tone(ToneOperation),    // kind and/or { hz, ms }; shell: PWM / WebAudio
}
```

Composition (spike style):

- `Command::all([...])` or `render().and(other)`
- `Command::request_from_shell(op).then_send(|out| Event::...)` or `Command::new(|ctx| async { ... })` as in `cpb-counter` `flash`

`no_std` + `alloc` as in the spike; CPB links the core directly (no FFI bridge); web via WASM (bridge/typegen later if needed; spike notes that `#[effect(facet_typegen)]` needs std `bridge`).

Suggested domain events after rename:

```text
enum Event {
    Skip,
    ActivateColor,
    ColorReceived { channel: u8, color: Color }, // v4: named/index, not Rgb
    BlinkTick,           // or driven only by Delay completion
    CountdownBeat,       // or Delay completion carrying step
    RandomByte(u8),      // resolve of Random
    DelayDone(/* id */), // if multiple delays need correlation
}
```

Exact Delay/Random correlation IDs follow the spike (`FlashDone(u32)` pattern) once the round state machine is drafted.

---

## 6. Design decision: color on the wire

**Question:** Should the core speak `enum Color { White, Red, Orange, ... }` instead of `Rgb`, and should BLE send a color index instead of `r,g,b` (protocol version 4)?

### Decision (Marcel, 2026-10-29): Option C

Send a fixed color concept on the wire, not RGB triples. Domain and protocol use a named/indexed color; shells map that to display RGB (and dimming) locally via the shared palette.

**Consequences for the port:**

- Bump manufacturer payload to **protocol version 4** (board and web must ship together; no mixed v3/v4 demo unless a temporary dual decoder is added).
- Core match rule: same channel and same `Color` (or spectrum index), not byte-equal `r,g,b`.
- `Rgb` remains a view/palette concern (how it looks on NeoPixels or CSS), not the BLE identity of a color.
- Encode/decode in core take `(channel, Color)` / index; payload shape to be fixed in the protocol module (e.g. `[magic, version, opcode, channel, color_index]`).

### Options considered

**A — `Rgb` on the wire (v3):** no cutover; match = exact bytes; domain stays hardware-ish.

**B — Named `Color` in the domain, RGB still on the wire (v3):** talk can say "Red"; wire stays compatible; two builds with different palettes can still disagree on the air.

**C — Named `Color` / index on the wire (v4):** chosen. Closest to "players shout Red!"; match is an index/enum compare; requires coordinated firmware + web update.

---

## 7. Tests the core should have after the port

Today: 5 host tests in `crates/core/src/tests.rs` (mode select, entropy start, match happy path, picker apply). Missing relative to the contract:

- Match **only** through the core: correct channel and color → next round, no explosion
- Wrong channel → no state change / no explosion
- Wrong color → no state change / no explosion
- After 10 beats → explosion, then a new target requested via `Random` (command assertion)
- Palette identity: picker spectrum index `i` equals `color_from_entropy(b)` when `i == b % 10`
- Protocol roundtrip: `decode(encode(channel, color)) == (channel, color)`; reject invalid channel, wrong magic, wrong version
- Prefer Crux test helpers (`assert_effect!`, `expect_only_render()`, etc.) over hand-rolled effect matching once the app is on Crux

Keep green after each migration step:

- `cargo test -p color-hurry-core`
- thumbv7em build: `just build`
- web: Trunk build (`just web-build` or equivalent)
- Demo: board as Picker → browser as Receiver over BLE

---

## 8. Migration steps (order)

Do not start until Marcel confirms this document.

1. **Lock the inventory:** confirm sections 1-2 and 4 (and migration if still open). Already decided: color on wire **C / v4** (section 6); tones (Hz/ms including Explosion) in core.
2. **Core protocol module (v4):** define `Color` (or spectrum index), `encode` / `decode` for `(channel, Color)` in `color-hurry-core`; point CPB and web at them; move protocol unit tests into core; keep AD/WebBluetooth framing in shells. Ship board + web together.
3. **Tone parameters in core:** single table for Skip, Apply, CountdownStep, Explosion (unify Explosion; today CPB sweep vs web beep); shells only play Hz/ms from the effect.
4. **Expand core tests** for match / mismatch / explosion / palette / protocol (still on the pre-Crux `update` API if needed, then rehome to `App` tests).
5. **Round state enum** replace `countdown_active` + `awaiting_entropy`; keep public API minimal.
6. **Rename domain events** (Skip, ActivateColor, ColorReceived, …); shells map buttons.
7. **Introduce Crux `App`** behind the spike dependency (`no_std` + `alloc`): `Model` private, `ViewModel` public, `Command<Effect, Event>`, `Delay` + `Random` first (timing and entropy), still stubbing BLE/Tone as today's side effects if needed.
8. **Split Tick / own delays:** core requests blink and beat delays; remove shell intervals that branch on model flags.
9. **Fix CPB match path:** `on_adv_report` only parses; shell forwards `(channel, color)`; delete `matched` / `take_color_match` rubber-stamp.
10. **Stop model peeks:** shells handle effects and `ViewModel` only; entropy via `Random` resolve; BLE via `BleOperation`.
11. **Wire web and CPB** to `Core::process_event` / `resolve` as in `cpb-counter`; verify demo flow.
12. **Optional:** protocol v4 / named `Color` if Marcel chose option C.

---

## 9. Spike reference (for implementers)

- Repo: `/Users/marcel/Development/crux`, branch `spike/no-std`
- Example: `examples/cpb-counter`
  - `app.rs`: `impl App for Counter`, `Delay` via `#[derive(Operation)]`, `#[effect] enum Effect`, `render().and(blink)`, `Command::new` + `request_from_shell`
  - `main.rs`: `Core<Counter>`, `process_event`, outstanding delays, `resolve` when due
- Notes: `examples/cpb-counter/SPIKE_NOTES.md` (heap size, `critical-section`, no FFI on device)

---

## 10. Confirmation checklist

Confirmed:

- [x] Boundary core vs shell (sections 1-2) looks right (Inventar)
- [x] Violations a-i match reality (section 4)
- [x] Effect list (section 5) is the intended first cut (incl. Ble as a normal Crux effect; ads in as Events)
- [x] Color on the wire: **C** (named/index, protocol v4) — section 6
- [x] Migration order (section 8) is acceptable
- [x] Tone parameters (including Explosion) move into the core — shell only plays
