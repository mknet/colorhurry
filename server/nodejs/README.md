# Color Hurry — Node.js server shell

Runs the **same** Rust core (`crates/core`) inside Node via a [napi-rs](https://napi.rs) addon, speaks BLE on the host, and serves a browser UI (LED ring + Web Audio tones).

This is the demo that the core binds to Node without core changes — not via WASM.

## Architecture

- **napi addon** [`crates/nodejs`](../../crates/nodejs): `Core<ColorHurry>`, timers and random resolved in Rust; returns view, tones and BLE ops.
- **This package**: HTTP + WebSocket, `@abandonware/noble` scan (Caller), optional bleno advertise (Picker on Linux), static UI in `public/`.

## Requirements

- Node.js **18+**
- Rust toolchain (to build the addon)
- Bluetooth adapter for live play

| Role | macOS | Linux | Windows |
|------|-------|-------|---------|
| Caller (scan) | yes | yes (`libbluetooth-dev`) | best-effort |
| Picker (manufacturer advertise) | no (CoreBluetooth) | yes (`@abandonware/bleno` + EIR) | no |

Typical Mac demo: **this server = Caller**, **CPB = Picker** on the same channel.

### Linux packages

```bash
sudo apt-get install bluetooth bluez libbluetooth-dev libudev-dev
# optional, for Picker advertise:
npm install @abandonware/bleno
```

On Linux you may need capabilities for noble without root:

```bash
sudo setcap cap_net_raw+eip $(readlink -f $(which node))
```

## Run

From the repo root:

```bash
just node
```

Or:

```bash
cd server/nodejs
npm install
npm run build:native
npm start
```

Open **http://127.0.0.1:4747** — Skip / Apply cycle mode select (Picker or Caller + channel), Mode returns to mode select. Override port with `PORT=8080`.

Keys: `a` / ← Skip, `b` / Enter Apply, `Esc` / `m` Mode.

## Notes

- Game rules, tones and BLE payload **v4** come only from the Rust core.
- On macOS, choosing Picker in the UI is allowed; the status line explains that manufacturer advertise needs Linux. Use a board or Android as Picker instead.
