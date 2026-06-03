# Erkenntnisse: BLE sendet nichts → Advertising funktioniert

Dokumentation der Debug-Session für **BLE-Broadcast** auf dem **Circuit Playground Bluefruit (CPB)** in diesem Repo.

---

## Kurzfassung

Es waren **mehrere getrennte Probleme**. Der Color Picker lief schon — **BLE blieb unsichtbar**, weil der Nordic-SoftDevice-Stack auf dem Chip nicht zur Rust-Bibliothek passte.

| # | Problem | Symptom | Fix |
|---|---------|---------|-----|
| 1 | **SoftDevice-Version passt nicht** | Kein `ColorHurry-CPB` im Scanner; D13: nur **4× kurz**, kein langes Blitz, kein ~3×/s Advertising | S140 **6.1.1 → 7.0.1** (`just setup-ble`) |
| 2 | **App an falscher Flash-Adresse** | App startet, BLE-Code läuft nicht am erwarteten Ort | Feature **`sd-v7`**, Link @ **0x27000** (`just flash-v7`) |
| 3 | **Falsches DFU-Paket für SD-Upgrade** | `adafruit-nrfutil` lehnt reines SoftDevice-Paket ab | **`softdevice_bootloader`**-ZIP bauen (`scripts/upgrade-softdevice-v7.sh`) |
| 4 | **Blockierende Delays im Executor** | SoftDevice bekommt keine CPU-Zeit, Advertising hängt | `delay_ms_async` + `yield_now` statt `delay_ms` in Tasks |
| 5 | **Task-Architektur** | Radio-Stack braucht eigenen async Task | `softdevice_task` mit `sd.run()` + separater `advertise_loop` |
| 6 | **Falscher Test-Client** | Am Mac „nichts zu sehen“ | **nRF Connect auf iPhone/iPad**, nicht macOS-Bluetooth |

**Entscheidend war Schritt 1 + 2.** `nrf-softdevice-s140` 0.1.x bindet gegen **S140 7.0.1**. Adafruit `restore-and-flash` liefert **S140 6.1.1**. Mit 6.1.1 schlagen `Softdevice::enable()` und Advertising still fehl — die App (Color Picker) kann trotzdem laufen.

---

## Problem 1: SoftDevice-Version (Hauptursache)

### Symptome

- Color Picker und NeoPixels **funktionieren**
- Im BLE-Scanner: **kein** Gerät `ColorHurry-CPB` / `ColHurry`
- nRF Connect zeigt nur fremde Einträge oder **N/A**
- D13 nach Reset: **4× kurz** (Firmware v7), aber **kein** langes Blitz (~0,5 s) und **kein** regelmäßiges Aufblitzen (~3×/s)
- Alternativ: **schnelles Stroboskop** auf D13 → Advertising-Fehler (SD passt nicht)

### Ursache

| Quelle | SoftDevice |
|--------|------------|
| Adafruit Restore / Werkzustand | **S140 6.1.1** |
| Crate `nrf-softdevice-s140` 0.1.2 | **S140 7.0.1** (hardcoded ABI) |

Die Rust-Bindings rufen SoftDevice-APIs mit der **7.0.1-Symboltabelle** auf. Liegt 6.1.1 im Flash, stimmen Adressen/Strukturen nicht — Init oder Advertising scheitern, oft **ohne sichtbaren Panic** in der UI.

### Fix

Einmalig SoftDevice upgraden, dann App v7 flashen:

```bash
just setup-ble
```

Das führt aus:

1. `just upgrade-softdevice-v7` — Serial-DFU, Paket **S140 7.0.1 + Bootloader**
2. `just flash-v7` — App mit Feature `sd-v7` @ **0x27000**

Manuell:

```bash
just upgrade-softdevice-v7
just flash-v7
```

**Erfolg erkennen (D13, ohne Handy):**

| Muster | Bedeutung |
|--------|-----------|
| 4× kurz | Firmware v7 @ 0x27000 |
| 1× lang (~0,5 s) | SoftDevice OK |
| Kurzes Aufblitzen ~3×/s | Advertising läuft |

Details: [BLE.md](BLE.md#d13-diagnose-ohne-handy)

---

## Problem 2: Flash-Layout / Link-Adresse

### Symptome

- Nach `just restore-and-flash` + `just flash`: Color Picker OK, BLE tot
- Oder: falsche Kombination aus SD-Version und App-Offset

### Ursache

S140 belegt den unteren Flash. Die **App-Startadresse** hängt von der SoftDevice-Generation ab:

| SoftDevice | App-Start | Cargo-Feature | Just-Target |
|------------|-----------|---------------|-------------|
| S140 6.1.1 | **0x26000** | `sd-v6` (default ohne v7) | `just flash` |
| S140 7.0.1 | **0x27000** | `sd-v7` (Repo-Default) | `just flash-v7` |

`build.rs` wählt `memory-s140-v6.x` bzw. `memory-s140-v7.x` und setzt `cpb_memory_v6` / `cpb_memory_v7` für die D13-Diagnose.

### Fix

Für BLE immer **beides** zusammen:

- Chip: **S140 7.0.1**
- Build/Flash: **`just flash-v7`** (Feature `sd-v7`, Link @ 0x27000)

Ohne BLE (nur Color Picker): `just restore-and-flash` + `just flash` (6.1.1 / 0x26000) bleibt gültig.

---

## Problem 3: DFU-Pakettyp beim SoftDevice-Upgrade

### Symptome

- `adafruit-nrfutil dfu serial` schlägt fehl oder bricht ab
- Reines Nordic-SoftDevice-HEX allein reicht nicht

### Ursache

Der **Adafruit Serial-Bootloader** erwartet DFU-Pakete vom Typ **`softdevice_bootloader`** (SoftDevice + Bootloader in einem Paket) — wie bei `just restore-and-flash`. Ein isoliertes SoftDevice-HEX ohne passenden Bootloader wird nicht akzeptiert.

### Fix

`scripts/upgrade-softdevice-v7.sh`:

1. Bootloader aus dem Adafruit-Zip `circuitplayground_nrf52840_bootloader-0.11.0_s140_6.1.1.zip` extrahieren (@ 0xF4000)
2. Nordic-HEX **S140 7.0.1** laden
3. Mit `adafruit-nrfutil dfu genpkg --bootloader … --softdevice …` ein **`softdevice_bootloader`**-ZIP erzeugen
4. Serial-DFU: Button A halten → Reset 1× → A loslassen

Auf **macOS** bricht USB oft mit `Device not configured` ab — das Skript versucht **3×** und behandelt erfolgreichen Reconnect als OK.

---

## Problem 4: Executor blockiert (Software)

### Symptome

- SoftDevice theoretisch OK, Advertising unzuverlässig oder tot
- Color-Picker-Schleife mit langen **blockierenden** Wartezeiten

### Ursache

`delay_ms()` in `time.rs` spinnt auf TIMER2 und **gibt den Embassy-Executor nicht ab**. Der SoftDevice-Stack (`sd.run()`) und `advertise_loop` brauchen aber regelmäßig async-Zeitscheiben.

### Fix

- Color Picker: `time::delay_ms_async(TICK_MS).await` (50 ms Tick)
- `delay_ms_async`: in der Warteschleife `embassy_futures::yield_now().await`
- `delay_ms` nur noch für **sync**-Code (z. B. D13-Diagnose beim Start in `debug_led.rs`)

Kommentar in `time.rs`:

```rust
/// **Nicht** in Embassy-Tasks verwenden — blockiert SoftDevice und BLE.
pub fn delay_ms(ms: u32) { … }
```

---

## Problem 5: Embassy-Task-Aufteilung

### Symptome

- SoftDevice init OK, aber kein stabiles Advertising

### Ursache

`nrf-softdevice` erwartet einen dedizierten Task, der **`sd.run().await`** ausführt. Advertising läuft in einem **zweiten** Task. Beides teilt sich den single-threaded Embassy-Executor.

### Fix (`main.rs`)

```text
main
 ├── color_picker_task()     ← UI zuerst (läuft auch wenn BLE scheitert)
 └── ble_system()
      ├── enable()           ← Softdevice::enable
      ├── spawn softdevice_task(sd)   ← sd.run().await (Endlosschleife)
      └── advertise_loop(sd)          ← non-connectable scannable advertising
```

In `ble_system`: kurz `yield_now()` vor `enable()`, damit der Color Picker HFCLK/Timer zuerst initialisieren kann.

Relevante Module:

| Datei | Rolle |
|-------|--------|
| `src/ble_broadcast.rs` | Config, Manufacturer Data, `advertise_loop` |
| `src/main.rs` | Task-Spawn |
| `src/color_picker.rs` | `sync_ble()` → `ble_broadcast::set_color` |

---

## Problem 6: Test & sichtbares Protokoll

### Scanner

- **iPhone/iPad + nRF Connect** — zuverlässig
- **macOS-Bluetooth** — zeigt oft nur das MacBook, CPB fehlt oder als N/A

### Erwartete Werbedaten

- Name: **`ColorHurry-CPB`** (Scan Response) / Kurzname **`ColHurry`**
- Manufacturer ID: **`0x239A`** (Adafruit)
- Payload v2: **`53 02 03 RR GG BB`** (Magic, Version, CMD_COLOR, RGB)

Beispiel Rot: `53 02 03 FF 00 00`. Beim Skippen im Color Picker ändern sich **RR GG BB** innerhalb ~50 ms.

Alte Firmware zeigte nur **`53 01 01`** (CMD_PING v1) — dann **`just flash-v7`** flashen.

---

## Checkliste: BLE zum Laufen bringen

```bash
# 1. Einmalig (SD 7.0.1 + App v7)
just setup-ble

# 2. Nach Code-Änderungen
just flash-v7

# 3. Prüfen ohne Handy: D13 nach Reset
#    4× kurz → 1× lang → regelmäßiges Aufblitzen

# 4. Prüfen mit Handy: nRF Connect → ColorHurry-CPB → Manufacturer 0x239A
```

Wenn Schritt 1 fehlschlägt (macOS USB):

```bash
# Button A halten → Reset 1× → A loslassen
DFU_MANUAL=1 just upgrade-softdevice-v7
just flash-v7
```

---

## Was wir **nicht** brauchten

- Kein BLE-Connect / GATT — reicht **non-connectable scannable advertising**
- Kein SWD-Debugger für den Normalfall — Serial-DFU reicht
- Kein Wechsel auf S140 6.1.1-kompatible Rust-Crates — stattdessen SD auf 7.0.1 angehoben

---

## Relevante Dateien

| Datei | Inhalt |
|-------|--------|
| [BLE.md](BLE.md) | Protokoll, D13-Codes, nRF Connect |
| [scripts/upgrade-softdevice-v7.sh](../scripts/upgrade-softdevice-v7.sh) | SD 6.1.1 → 7.0.1 |
| [memory-s140-v7.x](../memory-s140-v7.x) | App @ 0x27000 |
| [memory-s140-v6.x](../memory-s140-v6.x) | App @ 0x26000 (ohne BLE) |
| [Justfile](../Justfile) | `setup-ble`, `flash-v7`, `upgrade-softdevice-v7` |
