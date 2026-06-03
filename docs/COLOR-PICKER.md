# Color Picker (Picker-Modus)

## Start: Moduswahl

Nach Reset erscheint der **Setup-Ring**:

### Modus + Kanal (Setup)

| Anzeige | **5 Kanalfarben** links (blink), rechts **eine** LED an (Radio) |

| Taste | Aktion |
|-------|--------|
| **A** | Nächster Ring-Schritt (CCW, Lila links → Lila rechts) |
| **B** | Start (0–4 = Picker, 5–9 = Empfänger) |

Kanäle: `palette::CHANNEL_COLORS` (5 Farben).

### Farbauswahl (Picker-Modus)

Alle **10** LEDs = `palette::SPECTRUM` (weiß … rosa), gewählte blinkt aus — **getrennt** vom Kanal-Setup.

## Picker-Laufzeit

| Taste | Aktion |
|-------|--------|
| **A** | Nächste Farbe (10 Spektrumfarben am Ring) |
| **B** | Farbe übernehmen → **alle 10** LEDs + BLE `CMD_COLOR` |

Töne: `speaker::play_skip` / `play_apply`.

## BLE

Picker sendet die gewählte Farbe per Manufacturer Data (`ble_broadcast::set_color`). Siehe [BLE.md](BLE.md).

## Dateien

| Datei | Rolle |
|-------|--------|
| `src/mode_select.rs` | Setup-UI |
| `src/color_picker.rs` | Picker |
| `src/game.rs` | Modus-Verzweigung |
