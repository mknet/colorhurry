# Standard-Programm / CircuitPython statt Rust

Wenn nach dem Flash weiterhin **CircuitPython** oder das **alte Werkprogramm** läuft (z. B. `CIRCUITPY` im Finder, bunte Demo, kein 2×-Orange-Start):

1. **Bootloader:** Reset **2×** → NeoPixels **grün**
2. **Finder** (auf dem Mac fast immer nötig — Terminal: `Permission denied`):
   ```bash
   just flash-finder
   ```
   Dann `.uf2` auf **CPLAYBTBOOT** ziehen (auf das **Symbol**, nicht in den Ordner)
3. Laufwerk muss **verschwinden** — sonst ist der Flash fehlgeschlagen
4. USB 5 s ab, einstecken, Reset **1×**

**Rust erkannt?** Beim Start **2 Sekunden alle NeoPixels orange**, danach **cyan blinken** + rote D13.

`just` meldet früher fälschlich „OK“, obwohl CPLAYBTBOOT noch da war — jetzt nur Erfolg, wenn das Laufwerk weg ist.

---

# Board bleibt grün (Bootloader) — Recovery

**Grün = UF2-Bootloader.** Deine Rust-App läuft nicht.

## Schnelltest nach dem Flash

1. **USB komplett abziehen**, 5 Sekunden warten, wieder einstecken  
2. **Keine** Buttons berühren  
3. Nur **Reset (Mitte) 1×** — **nicht** 2×  
4. Erwartung: **rote LED D13** blinkt (~500 ms) — **kein** dauerhaft grün

## Wenn weiterhin nur grün

### A) GPREGRET / Bootloader-Modus „klebt“

Frühere Firmware konnte per Button in den Bootloader springen (`GPREGRET = 0x57`). Das überlebt Soft-Resets.

→ **Strom wirklich weg** (USB ab), nicht nur Reset.

### B) SoftDevice durch falschen Flash beschädigt

Wurde früher bei Flash-Adresse **0x0** geflasht, kann der **SoftDevice**-Bereich kaputt sein. Der Bootloader (grün) geht noch, die App bei **0x27000** nicht.

**Reparatur (UF2, NeoPixels grün):**

1. [Circuit Playground Bluefruit Bootloader-Update](https://learn.adafruit.com/adafruit-circuit-playground-bluefruit/update-bootloader-use-uf2)  
2. Datei `update-*circuitplayground*` / `*CPLAYBT*` von [Adafruit_nRF52_Bootloader Releases](https://github.com/adafruit/Adafruit_nRF52_Bootloader/releases)  
3. Per Finder auf **CPLAYBTBOOT** ziehen (nicht Terminal-`cd` ins Laufwerk)

**Oder Test mit CircuitPython:**

- [CircuitPython UF2 für CPB](https://circuitpython.org/board/circuitplayground_bluefruit/)  
- Wenn danach **CIRCUITPY** erscheint: Hardware OK → erneut Rust flashen mit `just flash-now`

### C) Flash hat nicht wirklich geschrieben

`just` meldet „UF2 übernommen“, aber `dd` kann auf dem Mac fehlschlagen.

→ Im **Finder**: `target/thumbv7em-none-eabihf/release/color-hurry-cpb.uf2` auf **CPLAYBTBOOT** ziehen  
→ NeoPixels kurz **rot**, Laufwerk weg, Board neu startet

## Danach Rust flashen

```bash
just flash-clean
```

(oder im Bootloader: `just flash-now`)

## NeoPixels wieder einschalten

Wenn D13 blinkt, in `main.rs` wieder `mod neopixel` und NeoPixel-Code aktivieren (ältere Git-Version).
