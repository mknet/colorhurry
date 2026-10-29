# Design

## Context

After `port-core-to-crux` the core is a Crux `App` that the board links directly and the browser compiles to WASM. Both shells are Rust. iOS and Android shells are not, so the core has to cross a language boundary. See proposal.md for the motivation.

## Goals / Non-Goals

**Goals:**
- No game rule, tone table or wire format in Swift or Kotlin.
- The core stays `no_std` and the firmware build is unaffected.

**Non-Goals:**
- Tuning the phone as Picker: it works through `color_advertisement`, but the demo uses the phone as Receiver.
- Publishing the apps to a store.

## Decisions

### Crux bridge over bytes, exported with BoltFFI
`CoreFfi` wraps `Bridge<ColorHurry>` with `update`, `resolve` and `view`, each taking and returning bincode bytes, as in the Crux `counter` example on `spike/no-std`. BoltFFI generates the Swift package and the Kotlin bindings plus jniLibs.
Alternative: UniFFI. Works as well, but the Crux spike and its typegen target BoltFFI, so following the example keeps us on the tested path.

### Generated types with facet typegen
A `codegen` binary registers `ColorHurry` and emits Swift (`App` package) and Kotlin (`com.colorhurry.shared`) types, including a `Core` wrapper with an `EffectHandler` protocol: one method per operation (`delay`, `random`, `playTone`, `ble`). The shells implement that protocol and nothing else.
Alternative: hand-written Swift/Kotlin mirrors of the types. Rejected, because they drift from the core with every change.

### Separate `crates/ffi`, core stays `no_std`
The core only derives `Serialize`, `Deserialize` (serde without std) and `Facet` (`alloc`). `#[effect(facet_typegen)]` only emits typegen code behind the core feature `facet_typegen`, which needs std and is switched on by `crates/ffi` alone. The firmware build never sees std.
Alternative: put the bridge into the core behind a feature. Rejected, because it mixes std and `no_std` concerns in one crate and makes the firmware build depend on feature hygiene.

### BLE payloads stay in Rust
The apps hand the raw manufacturer data (after the company ID) to `color_received_event` and get a serialized `Event` or nothing. For the Picker, `color_advertisement` builds the payload. The wire format exists once, in `protocol.rs`.

## Risks / Trade-offs

- [Generated code needs a Rust toolchain with iOS and Android targets] → `just ios` / `just android` run `just ffi-*` first; README lists the tools.
- [Bincode payloads are opaque in the debugger] → the generated types decode them on the Swift/Kotlin side for logging.
- [iOS deployment target rises to 17] → the generated `Core` and the controller use Observation (`@Observable`); acceptable for a demo app.
