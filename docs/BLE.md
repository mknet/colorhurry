# BLE-Broadcast (Color Hurry CPB)

Das Board sendet parallel zum **Color Picker** ein BLE-Kommando per **non-connectable Advertising** (Manufacturer Data im Werbepaket). Keine Verbindung nötig.

> **Hintergrund / Troubleshooting:** Warum BLE anfangs unsichtbar blieb und was dafür nötig war (SoftDevice 7.0.1, Flash-Layout, DFU-Paket, async Tasks) → [ERKENNTNISSE-BLE-FIX.md](ERKENNTNISSE-BLE-FIX.md)

## Wichtig: SoftDevice-Version

| Auf dem Chip (Adafruit Restore) | Rust-Bibliothek `nrf-softdevice-s140` |
|----------------------------------|----------------------------------------|
| **S140 6.1.1** (Standard)        | **S140 7.0.1**                         |

**Das ist der Grund, warum du kein `ColorHurry-CPB` siehst**, obwohl der Color Picker läuft: `Softdevice::enable()` / Advertising schlagen mit 6.1.1 still fehl — am Handy erscheint nichts Sinnvolles (nur fremde **N/A**-Einträge).

### Einmalige Einrichtung für BLE

```bash
just setup-ble
```

Das macht:

1. `upgrade-softdevice-v7` — SoftDevice **7.0.1** per Serial-DFU (Button A + Reset)
2. `flash-v7` — App @ **0x27000** (`sd-v7`)

**Manuell:**

```bash
just upgrade-softdevice-v7   # nur SoftDevice tauschen
just flash-v7                # App bauen + flashen
```

Ohne BLE (nur Color Picker, wie früher): weiterhin `just restore-and-flash` + `just flash` (v6 / 6.1.1).

## Architektur

| Task | Modul | Aufgabe |
|------|-------|---------|
| SoftDevice | `main.rs` | `sd.run()` — Radio-Stack |
| BLE | `ble_broadcast.rs` | Advertising mit Kommando-Payload (Picker) |
| UI | `game.rs` | Moduswahl → Picker oder Empfänger |

### Picker vs. Empfänger

- **Picker:** sendet `CMD_COLOR` nur nach **Button B** (alle LEDs in der Farbe), nicht während der Auswahl.
- **Empfänger:** zeigt Zufallsfarbe, **scannt** nach passendem `CMD_COLOR`, bei Treffer Countdown → neue Zufallsfarbe.
- **Kanal** (0–4, Setup-Farbe links): Picker sendet Kanal im Payload; Empfänger ignoriert andere Kanäle.

## Gerätename

**`ColorHurry-CPB`** — im Werbepaket (`full_name`) und im GAP-Stack (`gap_device_name`). In nRF Connect sollte der Name **namentlich** erscheinen (nicht nur N/A).

## Protokoll (Manufacturer Data)

Company ID: **Adafruit `0x239A`**, danach:

| Offset | Feld | Wert |
|--------|------|------|
| 0 | Magic | `'S'` (`0x53`) |
| 1 | Version | `0x03` |
| 2 | Opcode | `0x03` = CMD_COLOR |
| 3 | Kanal | 0–4 (wie `palette::CHANNEL_COLORS` im Setup) |
| 4 | Rot | 0–255 (Palette, ungedimmt) |
| 5 | Grün | 0–255 |
| 6 | Blau | 0–255 |

Beispiel Kanal 2 (grün), Farbe Rot: `53 03 03 02 FF 00 00`. Gesendet wird nur die **bestätigte** Farbe (Picker, Phase „Applied“).

### Firmware

```rust
ble_broadcast::set_picker_broadcast(Some(Rgb { r: 255, g: 120, b: 0 }));
```

**Live-Updates beim Skippen:** Advertising wird alle **~50 ms** neu gestartet (`config.timeout = 5`), damit `set_color()` auch wirklich im Funk ankommt. Ohne Timeout bliebe die Manufacturer Data beim Startwert (Rot).

## D13-Diagnose (ohne Handy)

Nach Reset 1× die **rote D13-LED** (neben USB) beobachten:

| D13-Muster | Bedeutung |
|------------|-----------|
| **4× kurz** beim Start | Firmware **v7** (0x27000) — richtig für BLE |
| **2× kurz** beim Start | Firmware **v6** (0x26000) — **falsch für BLE** → `just flash-v7` |
| **1× lang** (~0,5 s) | SoftDevice OK |
| **Danach ruhig** | Advertising läuft (Payload-Refresh alle ~50 ms, ohne D13-Feedback) |
| **Schnelles Stroboskop** (8×) | Advertising-Fehler — SD-Version passt nicht |
| **Nur 4×, sonst nichts** | SoftDevice-Init scheitert (Panic) — SD-Upgrade nötig |
| **Dauerndes Zucken** | Alte Firmware: D13-Blink pro Adv-Timeout — `just flash-v7` |

NeoPixels (Color Picker) können **gleichzeitig** blinken — das ist normal.

## Test mit nRF Connect

**Wichtig:** Am **iPhone/iPad** mit der App **nRF Connect** scannen — nicht am Mac. macOS zeigt oft nur den MacBook-Namen, CPB-Geräte fehlen oder erscheinen als N/A.

1. `just setup-ble` (einmalig) oder nach SD-Upgrade: `just flash-v7`
2. Scanner → **`ColorHurry-CPB`** oder **`ColHurry`**
3. Manufacturer data: **`0x239A`**, Payload z. B. **`53 03 03 00 FF 00 00`** (Kanal 0, Rot) — ändert sich beim Skippen im Color Picker

## Fehler: „Device not configured“ beim Upgrade

Der Adafruit-Serial-Bootloader braucht ein **`softdevice_bootloader`**-DFU-Paket (SoftDevice + Bootloader zusammen), wie bei `just restore-and-flash` — kein reines SoftDevice-Paket.

Auf **macOS** bricht USB oft kurz ab (`Device not configured`). Das Skript versucht **3×**. Manuell:

1. **Button A halten → Reset 1× → A loslassen** (DFU-Modus)
2. Sofort:

```bash
DFU_MANUAL=1 just upgrade-softdevice-v7
just flash-v7
```

Erfolg: Meldung `Device programmed.` — danach USB ab, 5 s warten, einstecken, Reset 1×.

## Kommando ändern (Firmware)

```rust
ble_broadcast::set_command(ble_broadcast::protocol::CMD_READY);
```

## Relevante Dateien

| Datei | Rolle |
|-------|--------|
| [ERKENNTNISSE-BLE-FIX.md](ERKENNTNISSE-BLE-FIX.md) | Debug-Session: alle Fixes chronologisch |
| `src/ble_broadcast.rs` | SoftDevice, Advertising, Name, Protokoll |
| `src/main.rs` | Embassy-Tasks |
| `memory-s140-v7.x` | App @ `0x27000` (mit S140 7.x) |
| `scripts/upgrade-softdevice-v7.sh` | SD 6.1.1 → 7.0.1 |
