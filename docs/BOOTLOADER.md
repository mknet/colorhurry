# Bootloader / Flashen (CPLAYBTBOOT)

## Wichtig: welcher Taster?

| Taster | Position | Funktion |
|--------|----------|----------|
| **Reset** | **Mitte** (klein) | 1× = Neustart, **2× schnell** = UF2-Bootloader |
| **Button A** | links | Im Spiel / **halten beim Reset** → Bootloader |
| **Button B** | rechts | Im Spiel |

**Nicht** Button A oder B zweimal drücken — das startet **keinen** Bootloader.

## Bootloader starten (3 Wege)

### 1. Doppel-Reset (Standard)

1. USB-**Daten**kabel (kein reines Ladekabel)
2. **Reset in der Mitte** zweimal **schnell** hintereinander (< 500 ms)
3. NeoPixels: kurz **rot**, dann **grün**
4. Im Finder: Laufwerk **CPLAYBTBOOT**

### 2. Button A + Reset (zuverlässiger)

1. **Button A (links) gedrückt halten**
2. Währenddessen **Reset (Mitte) einmal** tippen (oder USB kurz trennen und wieder verbinden, A weiter halten)
3. **A loslassen**, wenn die NeoPixels grün werden
4. **CPLAYBTBOOT** sollte erscheinen

*(Mit aktueller Firmware: **Button A halten**, USB einstecken oder **Reset 1×** → startet direkt in den Bootloader.)*

### 3. Firmware schon drauf, Bootloader per Code

Mit geflaschter Firmware: **Button A halten**, **Reset 1×** → `GPREGRET` → UF2.

## Prüfen am Mac

```bash
just boot-check
```

### `Permission denied` in `/Volumes/CPLAYBTBOOT`

Kommt auf dem Mac oft vor — der Bootloader ist trotzdem aktiv.

- **Nicht** mit `cd` ins Laufwerk wechseln und `ls` machen
- Stattdessen von außen kopieren:

```bash
cp /pfad/zur/datei.uf2 /Volumes/CPLAYBTBOOT/
# oder im Projekt:
just flash
```

Oder im **Finder**: `.uf2` auf das Icon **CPLAYBTBOOT** ziehen (ohne den Ordner zu öffnen).

Falls auch `cp` scheitert: Laufwerk auswerfen, Reset 2×, erneut versuchen.  
Optional: **Systemeinstellungen → Datenschutz → Vollzugriff auf Festplatten** → Terminal/iTerm aktivieren.

USB-Gerät (Vendor 0x239A):

```bash
system_profiler SPUSBDataType | grep -A6 -i "239A\|Circuit"
```

## Alle NeoPixels bleiben grün (nach erfolgreichem `just`)

Das Board ist noch im **UF2-Bootloader**, die App läuft nicht. Häufige Ursachen:

1. **Falsche Flash-Adresse** — App muss ab `0x27000` gelinkt sein (`memory.x`). Nach Änderung: `just flash-clean`
2. **GPREGRET / Button A** — ältere Firmware sprang bei gedrücktem A sofort in den Bootloader. Aktuelle Firmware setzt GPREGRET beim Start auf 0.
3. **Finger weg von Button A** beim Reset nach dem Flash.

Erwartung nach OK: **cyan blinkende** Rand-LEDs, **rote** D13 blinkt mit — **nicht** dauerhaft grün.

## Wenn kein Laufwerk kommt

- Anderes USB-Kabel / direkt am Mac (ohne Hub)
- Anderen USB-Port
- NeoPixels bleiben **rot** → oft Kabel/Port (kein Datenkontakt)
- Doppel-Reset-Rhythmus variieren (schneller/langsamer)
- **Button A + Reset** probieren

## Flashen

```bash
just
```

Wartet bis zu 60 s auf **CPLAYBTBOOT** (Reset kann nach `just`-Start erfolgen).
