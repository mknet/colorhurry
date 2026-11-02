# Color Hurry

Cooperative 2-player game: shared logic in `crates/core` (Crux App), shells for **Circuit Playground Bluefruit**, **Browser (WASM)** and **iOS / Android** (FFI).

```text
crates/core    Event / Model / Effect / update / view  (no_std)
crates/cpb     Hardware shell (Embassy, Bluetooth, NeoPixels, Audio)
crates/web     Browser shell (one receiver board + experimental live BLE scan)
crates/ffi     Core over FFI for the native apps (Crux bridge + BoltFFI, facet typegen)
apps/apple     Native SwiftUI + CoreBluetooth shell on the Rust core
apps/android   Native Kotlin Compose + Android BLE shell on the Rust core
```

Tests: `just test` runs unit tests and Gherkin integration tests in `crates/core`.


## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- Target `thumbv7em-none-eabihf` (board) and `wasm32-unknown-unknown` (web)
- [just](https://github.com/casey/just), [trunk](https://trunkrs.dev/) (`brew install trunk` or `cargo install trunk`)
- [elf2flash](https://crates.io/crates/elf2flash) for `cargo run --release` and the experimental `just flash-deploy` (`just install-tools`)
- `rust-objcopy` (from `cargo install cargo-binutils`) and `python3` for the UF2 recipes

```bash
rustup target add thumbv7em-none-eabihf wasm32-unknown-unknown
just install-tools   # once
```

Optional (only with an external SWD debugger): [probe-rs](https://probe.rs/), `just flash-probe`

## BLE (works on S140 6.1.1 and 7.x)

The board runs Bluetooth on Nordic's SoftDevice Controller (`nrf-sdc`), which is linked into the app. The app never enables the resident SoftDevice. Boards with the factory **S140 6.1.1** and boards with **S140 7.x** both have Bluetooth. You do not update the SoftDevice or the bootloader.

A board starts the app right after its SoftDevice, so the memory layout must match the board:

| SoftDevice | App @ | Feature | Serial DFU |
|------------|-------|---------|------------|
| S140 6.1.1 | **0x26000** | `sd-v6` | `just flash` |
| S140 7.x | **0x27000** | `sd-v7` (default) | `just flash-v7` |

With the board in the UF2 bootloader (`CPLAYBTBOOT` mounted), `just board` prints the SoftDevice and the matching layout. `just flash-uf2` reads `INFO_UF2.TXT`, picks the layout, and copies the UF2 to the board. To choose the layout, run `just flash-uf2 v6` or `just flash-uf2 v7`.

```bash
just board        # print the SoftDevice of the board and the matching layout
just flash-uf2    # build, convert and copy the UF2 with the matching layout
cargo test -p color-hurry-core
just web-serve    # browser: http://127.0.0.1:8080
just ios / just android   # native apps (builds the core via `just ffi`, Xcode / JDK 17|21)
```

**Native apps (iOS + Android):** same entry as on the board (Skip/Apply → Picker/Receiver), UI like the web shell. Demo: CPB as picker, phone as receiver. See [apps/README.md](apps/README.md).

Browser **Live BLE** (unstable POC, receiver only): Chrome/Edge, flag
`chrome://flags/#enable-experimental-web-platform-features`, then “Live BLE scan”.
Picker advertising from the tab is not possible; only ads from real CPBs are received.

Details: [docs/BLE.md](docs/BLE.md)

Audio (skip/apply tones): [docs/AUDIO.md](docs/AUDIO.md). Demo sequence: feature `speaker-demo`.

**Solid green** = bootloader, app is not running → [docs/RECOVERY.md](docs/RECOVERY.md)

**macOS: no access to CPLAYBTBOOT** → [docs/FLASH-MACOS.md](docs/FLASH-MACOS.md) (`just flash-serial`)

## Build & flash (USB)

Connect the board with a **USB data cable** (charge-only cables are not enough).

**Bootloader:** not button A/B: press the small **center reset** **twice quickly**, **or** **hold button A (left)** and **reset once**. **CPLAYBTBOOT** appears in Finder (NeoPixels: red → green). Details: [docs/BOOTLOADER.md](docs/BOOTLOADER.md).

Then:

```bash
just flash-uf2   # builds (release), creates the .uf2 for the layout of the board, copies it to CPLAYBTBOOT
just flash       # serial DFU, v6 layout (S140 6.1.1)
just flash-v7    # serial DFU, v7 layout (S140 7.x), also the default for `just`
just build       # compile only
```

`just flash-uf2` stops with a message if `CPLAYBTBOOT` is not mounted. Serial DFU cannot read the SoftDevice, so you choose the layout with `just flash` or `just flash-v7`.

Alternatively: `cargo run --release` (same flow via `scripts/cargo-uf2-runner.sh`).

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
