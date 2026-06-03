# Erkenntnisse: Keine LED-Ausgabe → NeoPixels + D13 blinken

Dokumentation der Debug-Session für das **Circuit Playground Bluefruit (CPB)** in diesem Repo.
Stand: nach Color-Picker, Button-Fix und Helligkeits-Anpassung.

---

## Kurzfassung

Es waren **drei getrennte Probleme**, nicht eines:

| # | Problem | Symptom | Fix |
|---|---------|---------|-----|
| 1 | SoftDevice/MBR kaputt | Flash OK, aber App startet nicht | `just restore-and-flash` |
| 2 | WS2812-Bitbang falsch getaktet | NeoPixels **weiß** statt orange | PWM-Treiber `neopixel_pwm.rs` |
| 3 | Keine Blink-Logik für NeoPixels | Orange **dauerhaft**, nur D13 blinkt | `show_orange()` / `show_off()` in der Schleife |
| 4 | NeoPixels zu hell | Farben korrekt, aber unangenehm grell | Harte Kanal-Obergrenze `MAX_CHANNEL = 8` |

**Entscheidend war Schritt 1.** Ohne funktionierenden SoftDevice startet keine Rust-App — egal welche Firmware geflasht wird.

---

## Problem 1: App startet nicht (SoftDevice beschädigt)

### Symptome

- **Kein** D13-Blink nach Reset 1×
- NeoPixels **nur kurz** beim USB-Einstecken (Bootloader/Power-on) — **kein** dauerhaftes Orange
- Serial-DFU meldet `Device programmed`, Board verlässt den DFU-Modus
- Host-Logs: ELF korrekt @ `0x26000` bzw. `0x27000`, Stack-Pointer OK — trotzdem keine sichtbare App

### Ursache

Der **SoftDevice-/MBR-Bereich** auf dem nRF52840 war beschädigt, vermutlich durch frühere Flashes an **Adresse 0x0** (statt App-Basis `0x26000` / `0x27000`).

- Der **UF2-/Serial-Bootloader** lief weiter (kurzes NeoPixel-Aufblitzen beim Einstecken)
- Der **Sprung zur Anwendung** bei der korrekten App-Adresse funktionierte nicht
- Firmware-Updates allein konnten das nicht beheben

### Fix

Bootloader + SoftDevice per **Serial-DFU** wiederherstellen (ohne CPLAYBTBOOT):

```bash
just restore-and-flash
```

Das Skript `scripts/restore-bootloader-serial.sh` flasht:

- `circuitplayground_nrf52840_bootloader-0.11.0_s140_6.1.1.zip`
- SoftDevice **S140 6.1.1** → App gehört danach nach **`0x26000`** (Feature `sd-v6`)

**DFU-Modus:** Button A halten → Reset 1× → A loslassen. Während Restore (~30 s) USB **nicht** abziehen.

### Erfolg erkennen

- D13 blinkt (langsam)
- NeoPixels reagieren (zunächst evtl. noch weiß — siehe Problem 2)

---

## Problem 2: NeoPixels weiß statt orange (WS2812-Timing)

### Symptome

- App läuft (D13 blinkt)
- NeoPixels leuchten **extrem hell weiß**

### Ursache

**CPU-Bitbang** für WS2812 mit `delay_cycles()` und Annahme ~64 MHz CPU-Takt.

Mit laufendem **SoftDevice** stimmt die effektive Taktung für Software-Delays nicht. Die „0“-Pulse werden zu lang → der WS2812-Chip liest fast alle Bits als „1“ → **weiß**.

### Fix

Hardware-**PWM @ 16 MHz** (unabhängig von CPU-Takt), analog zu `src/neopixel.rs` / Embassy:

- Treiber: `src/neopixel_pwm.rs`
- PWM0 an **P0.13** (D8), Stromversorgung **P0.06** (D35, LOW = an)

### Erfolg erkennen

- NeoPixels leuchten **orange** (nicht weiß)

---

## Problem 3: NeoPixels blinken nicht (fehlende Logik)

### Symptome

- NeoPixels **dauerhaft orange**
- Nur D13 blinkt

### Ursache

Kein Bug — `show_orange()` wurde **nur einmal in `main()` beim Start** aufgerufen. Die Hauptschleife schaltete nur D13.

### Fix

In der Blink-Schleife abwechselnd:

```rust
neopixel_pwm::show_orange();
// … D13 an, warten …
neopixel_pwm::show_off();
// … D13 aus, warten …
```

### Erfolg erkennen

- NeoPixels und D13 blinken **synchron** (400 ms)

---

## Problem 4: NeoPixels zu hell (Helligkeit)

### Symptome

- WS2812-Timing und Farben stimmen (kein Weiß mehr)
- Farbauswahl (Color Picker) funktioniert
- LEDs wirken **deutlich zu hell**, auch nach mehreren Reduktionsversuchen

### Was vorher versucht wurde (nicht ausreichend)

Mehrere Stufen **prozentualer Skalierung** in `show_pixels()`:

| Variante | Formel | Max. Kanalwert bei Palette 255 |
|--------|--------|--------------------------------|
| ~60 % | `153/256` | ~153 |
| ~40 % | `102/256` | ~102 |
| ~20 % | `51/256` | ~51 |

Die Palette in `color_picker.rs` blieb dabei absichtlich bei vollen RGB-Werten (0–255), damit die zehn Spektrumfarben unterscheidbar bleiben. Die Abschwächung erfolgte erst beim Senden über `Rgb::dimmed()` in `show_pixels()`.

**Warum das subjektiv nicht reichte:** Bei 20 % liegt der maximale Kanalwert noch bei ~51 — das sind immer noch ~20 % der WS2812-Auflösung. Zehn NeoPixels im Ring addieren sich visuell; in einem dunklen Raum wirkt das weiterhin grell. Prozent-Skalierung ohne harte Obergrenze erlaubt hohe Einzelwerte.

### Fix (letzter Schritt)

Statt nur eines Prozent-Faktors gibt es jetzt eine **absolute Obergrenze pro RGB-Kanal**:

```rust
// src/neopixel_pwm.rs
pub const MAX_CHANNEL: u8 = 8;  // ≈ 3 % von 255

pub fn scale_channel(v: u8) -> u8 {
    if v == 0 {
        return 0;
    }
    ((v as u32 * MAX_CHANNEL as u32) / 255) as u8
}
```

**Verhalten:**

- Jeder Kanal wird **linear** skaliert: volles Rot (255) → **8**, halbes Grün (128) → **4**, Schwarz (0) → **0**
- Kein Kanal kann **8** überschreiten — unabhängig von der Eingabe-Palette
- `show_pixels()` wendet `dimmed()` auf alle Pixel an, bevor die PWM-Sequenz gebaut wird
- Farbverhältnisse (z. B. Orange vs. Gelb) bleiben erhalten, nur die Gesamthelligkeit sinkt

**Anpassen:** `MAX_CHANNEL` in `src/neopixel_pwm.rs` erhöhen (z. B. `12` oder `16`), wenn es zu dunkel wird; senken für noch gedämpfteres Licht.

### Erfolg erkennen

- Farben weiterhin unterscheidbar, aber **deutlich gedämpft** und angenehm im Dunkeln

---

## Was *nicht* das Hauptproblem war

| Vermutung | Ergebnis |
|-----------|----------|
| Flashen schlägt fehl | **Widerlegt** — DFU meldet Erfolg, Board verlässt Flash-Modus |
| Falsche Link-Adresse (v6 vs. v7) | **Allein nicht ausreichend** — korrekt gelinkte ELF halfen erst nach SoftDevice-Restore |
| CPLAYBTBOOT / UF2 auf macOS | Oft `Permission denied` / Laufwerk mountet nicht — **Serial-DFU** zuverlässiger |
| Falscher D13-Pin | **Unwahrscheinlich** — P1.14, active high; D13 blinkte sobald App lief |
| defmt / panic-probe | Vereinfachung sinnvoll, aber **nicht** Ursache für fehlenden App-Start |

---

## Hardware (CPB, verifiziert)

| Funktion | Pin |
|----------|-----|
| NeoPixels (10×) | P0.13 (D8) |
| NeoPixel-/Sensor-Strom (D35) | P0.06, **LOW = an** |
| Rote LED D13 | P1.14, active high |
| Button A / B | P1.02 / P1.15 |

---

## Flash-Workflow (macOS, empfohlen)

Standard ohne UF2-Laufwerk:

```bash
just flash          # v6 @ 0x26000 (nach S140 6.1.1 Restore)
just flash-v7       # nur wenn Board S140 7.x / App @ 0x27000 hat
```

Bei kaputtem SoftDevice:

```bash
just restore-and-flash
```

UF2 (nur wenn CPLAYBTBOOT mountet): `just flash-uf2`

Siehe auch: [RECOVERY.md](RECOVERY.md), [FLASH-MACOS.md](FLASH-MACOS.md)

---

## Firmware-Evolution in diesem Repo

| Phase | Code | Ergebnis |
|-------|------|----------|
| Anfang | Embassy, defmt, NeoPixel-Bitbang, Diagnose-`pre_init` | App startete nicht (SoftDevice kaputt) |
| Serial-DFU + minimal | D13-only, `panic-halt` | Immer noch kein Start ohne Restore |
| Nach `restore-and-flash` | Bitbang-Orange | App läuft, NeoPixels **weiß** |
| PWM-Treiber | `neopixel_pwm.rs` | NeoPixels **orange** (statisch) |
| Blink-Schleife | `show_orange` / `show_off` | NeoPixels + D13 **synchron** |
| Color Picker | `color_picker.rs`, Buttons active-high | Farbauswahl, Skip mit Button A |
| Helligkeit | `MAX_CHANNEL = 8` in `neopixel_pwm.rs` | Angenehm gedämpft (~3 % max.) |

---

## Unterscheidung: Bootloader vs. unsere App

| Beobachtung nach Reset 1× | Bedeutung |
|---------------------------|-----------|
| NeoPixels **kurz** beim Einstecken, sonst nichts | Bootloader/Power-on, **App läuft nicht** |
| D13 blinkt, NeoPixels reagieren | **Unsere App läuft** |
| NeoPixels **grün** dauerhaft | UF2-Bootloader-Modus (Reset 2×) |
| Nur 2× kurzes D13 | Bootloader-Puls, nicht unsere Blink-Schleife |

---

## Relevante Dateien

| Datei | Rolle |
|-------|--------|
| `src/main.rs` | D13 + NeoPixel-Blink-Schleife |
| `src/neopixel_pwm.rs` | WS2812 über nRF-PWM (16 MHz), globale Helligkeit (`MAX_CHANNEL`) |
| `src/color_picker.rs` | Farbauswahl (volle Palette, dimmt in `show_pixels`) |
| `src/neopixel.rs` | Embassy-PWM-Variante (für später) |
| `memory-s140-v6.x` | App @ `0x26000`, S140 6.1.1 |
| `memory-s140-v7.x` | App @ `0x27000`, S140 7.x |
| `scripts/restore-bootloader-serial.sh` | SoftDevice + Bootloader-Reparatur |
| `scripts/flash-serial.sh` | App-Flash per adafruit-nrfutil |

---

## Nächste Schritte (Entwicklung)

- Embassy wieder aktivieren (`neopixel.rs` oder PWM-Treiber portieren)
- Color-Hurry-Spiel-Logik (Buttons, BLE, Speaker)
- Optional: alte Debug-Skripte (`debug-host-log.sh`, …) entfernen oder in `scripts/debug/` verschieben
