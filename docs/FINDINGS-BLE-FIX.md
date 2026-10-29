# Findings: BLE sends nothing → advertising works

Documentation of the debug session for **BLE broadcast** on the **Circuit Playground Bluefruit (CPB)** in this repo.

---

## Summary

There were **several separate problems**. The color picker already worked — **BLE stayed invisible** because the Nordic SoftDevice stack on the chip did not match the Rust library.

| # | Problem | Symptom | Fix |
|---|---------|---------|-----|
| 1 | **SoftDevice version mismatch** | No `ColorHurry-CPB` in scanner; D13: only **4× short**, no long blink, no ~3×/s advertising | S140 **6.1.1 → 7.0.1** (`just setup-ble`) |
| 2 | **App at wrong flash address** | App starts, BLE code not at expected location | Feature **`sd-v7`**, link @ **0x27000** (`just flash-v7`) |
| 3 | **Wrong DFU package for SD upgrade** | `adafruit-nrfutil` rejects a SoftDevice-only package | Build **`softdevice_bootloader`** ZIP (`scripts/upgrade-softdevice-v7.sh`) |
| 4 | **Blocking delays in executor** | SoftDevice gets no CPU time, advertising hangs | `delay_ms_async` + `yield_now` instead of `delay_ms` in tasks |
| 5 | **Task architecture** | Radio stack needs its own async task | `softdevice_task` with `sd.run()` + separate `advertise_loop` |
| 6 | **Wrong test client** | “Nothing to see” on Mac | **nRF Connect on iPhone/iPad**, not macOS Bluetooth |

**Steps 1 + 2 were decisive.** `nrf-softdevice-s140` 0.1.x binds against **S140 7.0.1**. Adafruit `restore-and-flash` ships **S140 6.1.1**. With 6.1.1, `Softdevice::enable()` and advertising fail silently — the app (color picker) can still run.

---

## Problem 1: SoftDevice version (root cause)

### Symptoms

- Color picker and NeoPixels **work**
- In BLE scanner: **no** device `ColorHurry-CPB` / `ColHurry`
- nRF Connect shows only foreign entries or **N/A**
- D13 after reset: **4× short** (firmware v7), but **no** long blink (~0.5 s) and **no** regular flash (~3×/s)
- Alternatively: **fast strobe** on D13 → advertising error (SD mismatch)

### Cause

| Source | SoftDevice |
|--------|------------|
| Adafruit restore / factory | **S140 6.1.1** |
| Crate `nrf-softdevice-s140` 0.1.2 | **S140 7.0.1** (hardcoded ABI) |

The Rust bindings call SoftDevice APIs with the **7.0.1 symbol table**. If 6.1.1 is in flash, addresses/structures do not match — init or advertising fails, often **without a visible panic** in the UI.

### Fix

Upgrade SoftDevice once, then flash app v7:

```bash
just setup-ble
```

This runs:

1. `just upgrade-softdevice-v7` — Serial DFU, package **S140 7.0.1 + bootloader**
2. `just flash-v7` — app with feature `sd-v7` @ **0x27000**

Manually:

```bash
just upgrade-softdevice-v7
just flash-v7
```

**How to recognize success (D13, no phone):**

| Pattern | Meaning |
|---------|---------|
| 4× short | Firmware v7 @ 0x27000 |
| 1× long (~0.5 s) | SoftDevice OK |
| Short flash ~3×/s | Advertising running |

Details: [BLE.md](BLE.md#d13-diagnostics-no-phone)

---

## Problem 2: Flash layout / link address

### Symptoms

- After `just restore-and-flash` + `just flash`: color picker OK, BLE dead
- Or: wrong combination of SD version and app offset

### Cause

S140 occupies the lower flash. The **app start address** depends on SoftDevice generation:

| SoftDevice | App start | Cargo feature | Just target |
|------------|-----------|---------------|-------------|
| S140 6.1.1 | **0x26000** | `sd-v6` (default without v7) | `just flash` |
| S140 7.0.1 | **0x27000** | `sd-v7` (repo default) | `just flash-v7` |

`build.rs` selects `memory-s140-v6.x` or `memory-s140-v7.x` and sets `cpb_memory_v6` / `cpb_memory_v7` for D13 diagnostics.

### Fix

For BLE always use **both** together:

- Chip: **S140 7.0.1**
- Build/flash: **`just flash-v7`** (feature `sd-v7`, link @ 0x27000)

Without BLE (color picker only): `just restore-and-flash` + `just flash` (6.1.1 / 0x26000) remains valid.

---

## Problem 3: DFU package type for SoftDevice upgrade

### Symptoms

- `adafruit-nrfutil dfu serial` fails or aborts
- A Nordic SoftDevice HEX alone is not enough

### Cause

The **Adafruit serial bootloader** expects DFU packages of type **`softdevice_bootloader`** (SoftDevice + bootloader in one package) — as with `just restore-and-flash`. An isolated SoftDevice HEX without a matching bootloader is rejected.

### Fix

`scripts/upgrade-softdevice-v7.sh`:

1. Extract bootloader from Adafruit zip `circuitplayground_nrf52840_bootloader-0.11.0_s140_6.1.1.zip` (@ 0xF4000)
2. Load Nordic HEX **S140 7.0.1**
3. With `adafruit-nrfutil dfu genpkg --bootloader … --softdevice …` produce a **`softdevice_bootloader`** ZIP
4. Serial DFU: hold button A → reset once → release A

On **macOS**, USB often drops with `Device not configured` — the script retries **3×** and treats a successful reconnect as OK.

---

## Problem 4: Executor blocked (software)

### Symptoms

- SoftDevice theoretically OK, advertising unreliable or dead
- Color-picker loop with long **blocking** waits

### Cause

`delay_ms()` in `time.rs` spins on TIMER2 and **does not yield the Embassy executor**. The SoftDevice stack (`sd.run()`) and `advertise_loop` need regular async time slices.

### Fix

- Color picker: `time::delay_ms_async(TICK_MS).await` (50 ms tick)
- `delay_ms_async`: `embassy_futures::yield_now().await` in the wait loop
- `delay_ms` only for **sync** code (e.g. D13 diagnostics at start in `debug_led.rs`)

Comment in `time.rs`:

```rust
/// **Do not** use in Embassy tasks — blocks SoftDevice and BLE.
pub fn delay_ms(ms: u32) { … }
```

---

## Problem 5: Embassy task split

### Symptoms

- SoftDevice init OK, but no stable advertising

### Cause

`nrf-softdevice` expects a dedicated task that runs **`sd.run().await`**. Advertising runs in a **second** task. Both share the single-threaded Embassy executor.

### Fix (`main.rs`)

```text
main
 ├── color_picker_task()     ← UI first (runs even if BLE fails)
 └── ble_system()
      ├── enable()           ← Softdevice::enable
      ├── spawn softdevice_task(sd)   ← sd.run().await (forever)
      └── advertise_loop(sd)          ← non-connectable scannable advertising
```

In `ble_system`: brief `yield_now()` before `enable()`, so the color picker can initialize HFCLK/timer first.

Relevant modules:

| File | Role |
|------|------|
| `src/ble_broadcast.rs` | Config, manufacturer data, `advertise_loop` |
| `src/main.rs` | Task spawn |
| `src/color_picker.rs` | `sync_ble()` → `ble_broadcast::set_color` |

---

## Problem 6: Testing & visible protocol

### Scanner

- **iPhone/iPad + nRF Connect** — reliable
- **macOS Bluetooth** — often shows only the MacBook; CPB missing or as N/A

### Expected advertising data

- Name: **`ColorHurry-CPB`** (scan response) / short name **`ColHurry`**
- Manufacturer ID: **`0x239A`** (Adafruit)
- Payload v2: **`53 02 03 RR GG BB`** (magic, version, CMD_COLOR, RGB)

Example red: `53 02 03 FF 00 00`. When skipping in the color picker, **RR GG BB** change within ~50 ms.

Old firmware showed only **`53 01 01`** (CMD_PING v1) — then flash with **`just flash-v7`**.

---

## Checklist: get BLE running

```bash
# 1. Once (SD 7.0.1 + app v7)
just setup-ble

# 2. After code changes
just flash-v7

# 3. Check without phone: D13 after reset
#    4× short → 1× long → regular flash

# 4. Check with phone: nRF Connect → ColorHurry-CPB → Manufacturer 0x239A
```

If step 1 fails (macOS USB):

```bash
# Hold button A → reset once → release A
DFU_MANUAL=1 just upgrade-softdevice-v7
just flash-v7
```

---

## What we did **not** need

- No BLE connect / GATT — **non-connectable scannable advertising** is enough
- No SWD debugger for the normal case — serial DFU is enough
- No switch to S140 6.1.1-compatible Rust crates — instead raised SD to 7.0.1

---

## Relevant files

| File | Content |
|------|---------|
| [BLE.md](BLE.md) | Protocol, D13 codes, nRF Connect |
| [scripts/upgrade-softdevice-v7.sh](../scripts/upgrade-softdevice-v7.sh) | SD 6.1.1 → 7.0.1 |
| [memory-s140-v7.x](../memory-s140-v7.x) | App @ 0x27000 |
| [memory-s140-v6.x](../memory-s140-v6.x) | App @ 0x26000 (without BLE) |
| [Justfile](../Justfile) | `setup-ble`, `flash-v7`, `upgrade-softdevice-v7` |
