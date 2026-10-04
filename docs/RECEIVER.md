# Empfänger-Modus

## Setup

Position **5–9** am Ring (rechte Hälfte), Button **B** → Empfänger mit Kanal 0–4.

Die **Kanalfarbe** dient nur der Identität im Setup — sie bestimmt **nicht** die Spiel-Farbe.

## Laufzeit

1. **Zufallsfarbe** aus dem 10er-Spektrum, alle LEDs an.
2. Sofort **10-Sekunden-Countdown** (LEDs CCW aus + absteigender Ton).
3. Während des Countdowns: BLE-Scan auf passenden Picker-`CMD_COLOR`.
   - **Treffer:** Countdown bricht ab → **keine** Explosion → neue Zufallsfarbe.
   - **Kein Treffer:** nach Countdown **Explosion** (Sweep 180→45 Hz) → neue Zufallsfarbe.
4. Button **A** bricht den Countdown ab (wie Treffer, ohne Explosion).

## BLE

Empfänger scannt während des Countdowns. Picker sendet `CMD_COLOR` erst nach Farbbestätigung (Button B). **Kanal** im Payload muss zum Setup passen (z. B. beide „blau“ = Kanal 3); sonst wird das Paket ignoriert. RGB muss exakt zur aktuellen Zufallsfarbe passen.

## Dateien

| Datei | Rolle |
|-------|--------|
| `src/receiver.rs` | Schleife + Explosion |
| `src/countdown.rs` | LED + Ton, Abbruch per Callback |
| `src/speaker.rs` | `play_explosion()` |
