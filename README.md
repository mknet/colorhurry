# Color Hurry

Cooperative 2-player game: shared logic in `crates/core` (Crux App), shells for **Circuit Playground Bluefruit**, **Browser (WASM)** and **iOS / Android** (FFI).

```text
crates/core    Event / Model / Effect / update / view  (no_std)
crates/cpb     Hardware shell (Embassy, SoftDevice, NeoPixels, Audio)
crates/web     Browser shell (one receiver board + experimental live BLE scan)
crates/ffi     Core over FFI for the native apps (Crux bridge + BoltFFI, facet typegen)
apps/ios       Native SwiftUI + CoreBluetooth shell on the Rust core
apps/android   Native Kotlin Compose + Android BLE shell on the Rust core
```

Tests: `just test` runs unit tests and Gherkin integration tests in `crates/core`.


## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- Target `thumbv7em-none-eabihf` (board) and `wasm32-unknown-unknown` (web)
- [just](https://github.com/casey/just), [trunk](https://trunkrs.dev/) (`brew install trunk` or `cargo install trunk`)
- [elf2flash](https://crates.io/crates/elf2flash) for flashing (`just install-tools`)

```bash
rustup target add thumbv7em-none-eabihf wasm32-unknown-unknown
just install-tools   # once
```

Optional (only with an external SWD debugger): [probe-rs](https://probe.rs/), `just flash-probe`

## BLE (SoftDevice S140 7.0.1)

Default build: feature **`sd-v7`**, app @ **0x27000**. Adafruit restore ships **S140 6.1.1**; BLE will **not** transmit with that. Once:

```bash
just setup-ble    # SD 6.1.1 → 7.0.1 + flash app
just flash-v7     # app only after that
cargo test -p color-hurry-core
just web-serve    # browser: http://127.0.0.1:8080
just ios / just android   # native apps (builds the core via `just ffi`, Xcode / JDK 17|21)
```

**Native apps (iOS + Android):** same entry as on the board (Skip/Apply → Picker/Receiver), UI like the web shell. Demo: CPB as picker, phone as receiver. See [apps/README.md](apps/README.md).

Browser **Live BLE** (unstable POC, receiver only): Chrome/Edge, flag
`chrome://flags/#enable-experimental-web-platform-features`, then “Live BLE scan”.
Picker advertising from the tab is not possible; only ads from real CPBs are received.

Details: [docs/BLE.md](docs/BLE.md), troubleshooting: [docs/FINDINGS-BLE-FIX.md](docs/FINDINGS-BLE-FIX.md)

Without BLE (color picker only): `just restore-and-flash` + `just flash` (6.1.1 @ 0x26000).

Audio (skip/apply tones): [docs/AUDIO.md](docs/AUDIO.md). Demo sequence: feature `speaker-demo`.

**Solid green** = bootloader, app is not running → [docs/RECOVERY.md](docs/RECOVERY.md)

**macOS: no access to CPLAYBTBOOT** → [docs/FLASH-MACOS.md](docs/FLASH-MACOS.md) (`just flash-serial`)

## Build & flash (USB)

Connect the board with a **USB data cable** (charge-only cables are not enough).

**Bootloader:** not button A/B: press the small **center reset** **twice quickly**, **or** **hold button A (left)** and **reset once**. **CPLAYBTBOOT** appears in Finder (NeoPixels: red → green). Details: [docs/BOOTLOADER.md](docs/BOOTLOADER.md).

Then:

```bash
just          # builds (release), creates .uf2, copies to CPLAYBTBOOT
# or:
just flash
just build    # compile only
```

`just` waits up to 60 s for the BOOT volume (reset can happen after start).

Alternatively: `cargo run --release` (same flow via `scripts/cargo-uf2-runner.sh`).

Without rebuild: `just flash-bin`.

Experimental (often broken on macOS): `just flash-deploy` (`elf2flash deploy` → `BlockDeviceOpenFail`).

**Note:** If `CPLAYBTBOOT` disappears during copy with `Input/output error`, the flash often still succeeded (board reboots immediately). The script now treats that as success.

## defmt logs

RTT only works with an **SWD debugger** (probe-rs), not via the UF2 USB bootloader alone:

```bash
probe-rs attach --chip nRF52840_xxAA
```

## Hardware (Bluefruit)

| Function  | Pin    |
|-----------|--------|
| NeoPixel  | P0.08  |
| Button A  | P1.02  |
| Button B  | P0.07  |
| Speaker   | P0.26  |
| Spk. En.  | P1.04  |

Pinout: [Circuit Playground Bluefruit](https://learn.adafruit.com/adafruit-circuit-playground-bluefruit/pinouts)

## Project phases

1. **Skeleton**: Embassy + defmt
2. **NeoPixel driver** (current): 10× WS2812 on P0.13, blink test with D13
3. Buttons with debouncing
4. Speaker (Destroy / Success)
5. BLE advertising + scan (`nrf-softdevice`)
6. Game state machine

## License

MIT OR Apache-2.0
