# Audio (Lautsprecher CPB)

## Hardware

| Pin | Funktion |
|-----|----------|
| P0.26 (D12) | PWM1 → Lautsprecher |
| P1.04 (D11) | Verstärker Enable (HIGH = an) |

PWM0 ist für NeoPixels reserviert.

## Color Picker — UI-Töne (Standard)

| Aktion | Button | Ton | Charakter |
|--------|--------|-----|-----------|
| **Farbe wechseln** | A (Skip, Auswahl) | 380 Hz, 55 ms, Duty 22 % | dezent, tief |
| **Farbe übernehmen** | B (alle LEDs) | 784 Hz, 130 ms, Duty 34 % | höher, Enter-artig |

Implementierung: `speaker::play_skip()` / `speaker::play_apply()` in `color_picker.rs`.

Konstanten in `src/speaker.rs` anpassen.

## 12-Sekunden-Demo (aufgehoben)

Die Test-Sequenz (Gitarren-Reset → 10 absteigende Töne → Explosion) liegt in **`src/speaker_demo.rs`** und ist standardmäßig **aus**.

Zum Testen bauen mit Feature:

```bash
cargo build --release --features speaker-demo
# oder in Cargo.toml default erweitern; dann flash-v7 wie gewohnt
```

In `main.rs` wird dann `speaker_demo_task` mitgestartet. **Nicht** parallel zur UI-Nutzung gedacht (beide nutzen PWM1).

### Demo-Ablauf

| Sekunde | Inhalt |
|---------|--------|
| 1 | Gitarren-Anschlag g–h–e |
| 2–11 | Absteigende Tonleiter (680 → 200 Hz) |
| 12 | Explosion (Sweep, volle Sekunde) |

## Relevante Dateien

| Datei | Rolle |
|-------|--------|
| `src/speaker.rs` | PWM-Treiber, UI-Töne |
| `src/speaker_demo.rs` | Demo-Sequenz (Feature `speaker-demo`) |
| `src/color_picker.rs` | Ruft UI-Töne bei Buttons auf |
