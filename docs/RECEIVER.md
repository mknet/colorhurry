# Empfänger-Modus

## Setup

Position **5–9** am Ring (rechte Hälfte), Button **B** → Empfänger mit Kanal 0–4.

Die **Kanalfarbe** dient nur der Identität im Setup — sie bestimmt **nicht** die Spiel-Farbe.

## Laufzeit

1. **Zufallsfarbe** aus dem gleichen 10er-Spektrum wie die Auswahl (`palette::random_spectrum_color()`).
2. Kurz alle LEDs in dieser Farbe.
3. **10-Sekunden-Countdown** (`countdown::run_once`): pro Sekunde eine LED CCW aus + absteigender Ton (680 → 200 Hz).
4. Schleife: neue Zufallsfarbe.

## BLE (geplant)

| Heute | Geplant |
|-------|---------|
| Picker **advertised** `CMD_COLOR` | Empfänger **scannt** Picker-Broadcast |
| Empfänger ignoriert Setup-Kanal für Farbe | `ble_broadcast::receiver_wait_color()` (Stub) |

Protokoll-Erweiterung v3: Kanal-ID optional — siehe `protocol::CMD_COLOR_CHANNEL` in `ble_broadcast.rs`.

## Dateien

| Datei | Rolle |
|-------|--------|
| `src/receiver.rs` | Zufallsfarbe + Schleife |
| `src/countdown.rs` | LED + Ton synchron |
