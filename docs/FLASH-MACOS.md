# Flashen auf macOS ohne CPLAYBTBOOT

Wenn der Mac das UF2-Laufwerk **nicht mehr mountet** (`Permission denied`, Finder zeigt nichts):

## 1. Mount zurücksetzen

```bash
just mac-reset-boot
```

USB ab, 10 s warten, Reset 2×, `just boot-check`.

---

## 2. Empfohlen: Serial-Flash (kein Laufwerk nötig)

```bash
pip3 install adafruit-nrfutil
just flash-serial
```

Das Board kann **CircuitPython** oder normal laufen — das Skript weckt den Bootloader per 1200-Baud-Touch und flasht dann **ohne** `--touch` im selben Aufruf (stabiler auf macOS).

### „Device not configured“ / Flash bricht ab

USB trennt sich oft mitten in der DFU. Dann **manuell in DFU** gehen und erneut flashen:

1. **Button A (links)** gedrückt halten  
2. **Reset 1×** (Mitte)  
3. **A loslassen**  
4. `just flash-serial-dfu`

Port manuell wählen:

```bash
ls /dev/cu.usb*
SERIAL_PORT=/dev/cu.usbmodem14101 just flash-serial
```

---

## 3. Finder (wenn Laufwerk wieder da ist)

```bash
just flash-finder
```

`.uf2` auf **CPLAYBTBOOT** ziehen — Laufwerk muss **verschwinden**.

---

## 4. Vollzugriff Festplatten (optional)

Systemeinstellungen → Datenschutz & Sicherheit → **Vollzugriff auf Festplatten** → Terminal/iTerm aktivieren → Mac neu starten.

Hilft manchmal bei `cp` auf CPLAYBTBOOT — oft unzuverlässig.

---

## 5. SWD-Debugger

Mit J-Link / DAPLink an den SWD-Pins:

```bash
just flash-probe
```

---

## Erfolg erkennen

Nach Reset **1×**: D13 blinkt **3× kurz**, dann **langsam** — nicht dauerhaft grün (Bootloader), nicht CircuitPython-REPL.
