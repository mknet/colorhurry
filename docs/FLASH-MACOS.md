# Flashing on macOS without CPLAYBTBOOT

If the Mac **no longer mounts** the UF2 drive (`Permission denied`, Finder shows nothing):

## 1. Reset the mount

```bash
just mac-reset-boot
```

Unplug USB, wait 10 s, Reset 2×, `just boot-check`.

---

## 2. Recommended: serial flash (no drive needed)

```bash
pip3 install adafruit-nrfutil
just flash-serial
```

The board can be running **CircuitPython** or anything else; the script wakes the bootloader with a 1200-baud touch and then flashes **without** `--touch` in the same invocation (more stable on macOS).

### "Device not configured" / flash aborts

USB often disconnects in the middle of DFU. In that case, **enter DFU manually** and flash again:

1. Hold **Button A (left)**  
2. **Reset 1×** (middle)  
3. **Release A**  
4. `just flash-serial-dfu`

Choose the port manually:

```bash
ls /dev/cu.usb*
SERIAL_PORT=/dev/cu.usbmodem14101 just flash-serial
```

---

## 3. Finder (once the drive is back)

```bash
just flash-finder
```

Drag the `.uf2` onto **CPLAYBTBOOT**; the drive must **disappear**.

---

## 4. Full Disk Access (optional)

System Settings → Privacy & Security → **Full Disk Access** → enable Terminal/iTerm → restart the Mac.

Sometimes helps with `cp` to CPLAYBTBOOT, often unreliable.

---

## 5. SWD debugger

With a J-Link / DAPLink on the SWD pins:

```bash
just flash-probe
```

---

## Recognizing success

After Reset **1×**: D13 blinks **3× short**, then **slowly**, not permanently green (bootloader), not the CircuitPython REPL.
