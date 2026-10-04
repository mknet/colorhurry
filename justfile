# Color Hurry CPB — Build & Flash
#
# Standard auf macOS: Serial-DFU (kein CPLAYBTBOOT nötig)
#   just flash       → v6 @ 0x26000
#   just flash-v7    → v7 @ 0x27000

target  := "thumbv7em-none-eabihf"
profile := "release"
elf     := "target" / target / profile / "color-hurry-cpb"
uf2     := elf + ".uf2"
board   := "circuit_playground_bluefruit"

default: flash-v7


# Serial-DFU v7 (S140 7.x @ 0x27000) — für BLE / nrf-softdevice
flash-v7:
    DFU_MANUAL=1 CPB_MEMORY=v7 ./scripts/flash-serial.sh

# Serial-DFU v6 (S140 6.1.1 @ 0x26000) — nur Color Picker ohne BLE
flash:
    DFU_MANUAL=1 ./scripts/flash-serial.sh

# Automatisch (Touch) — falls manueller DFU nervt
flash-serial:
    ./scripts/flash-serial.sh

flash-serial-v7:
    CPB_MEMORY=v7 ./scripts/flash-serial.sh

flash-clean:
    cargo clean
    just flash

rebuild:
    cargo build --release -p color-hurry-cpb --target {{target}} --features sd-v7
    @arm-none-eabi-objdump -h {{elf}} 2>/dev/null | grep vector || true

# --- UF2 (nur wenn CPLAYBTBOOT wieder mountet) ---

flash-uf2: rebuild
    @echo "→ NeoPixels GRÜN? Finder: .uf2 auf CPLAYBTBOOT ziehen (siehe docs/RECOVERY.md)"
    @echo "→ Nach Flash: USB 5s ab, einstecken, Reset 1× (nicht 2×), keine Buttons"
    elf2flash convert --board {{board}} {{elf}} {{uf2}}
    ./scripts/copy-uf2.sh {{uf2}}

# Prüfen, ob BOOT-Laufwerk / USB sichtbar ist (macOS)
boot-check:
    @echo "=== Gemountete BOOT-Laufwerke ==="
    @ls -d /Volumes/*BOOT 2>/dev/null || echo "(keins)"
    @test -d /Volumes/CPLAYBTBOOT && echo "OK: CPLAYBTBOOT" || echo "— CPLAYBTBOOT fehlt"
    @echo ""
    @echo "=== USB (Adafruit 0x239A) ==="
    @system_profiler SPUSBDataType 2>/dev/null | grep -E "239A|Circuit|Playground|Bluefruit|CPLAY" -A3 || echo "(nichts gefunden)"

# Nur bauen
build:
    cargo build --{{profile}} -p color-hurry-cpb --target {{target}} --features sd-v7

# ELF → UF2 (ohne Flashen)
uf2: build
    elf2flash convert --board {{board}} {{elf}} {{uf2}}
    @echo "→ {{uf2}}"

# Bereits gebaut (ohne Rebuild)
flash-bin:
    elf2flash convert --board {{board}} {{elf}} {{uf2}}
    ./scripts/copy-uf2.sh {{uf2}}

# Bootloader schon aktiv (NeoPixels grün) — nur kopieren
flash-now: uf2
    ./scripts/copy-uf2.sh {{uf2}}

# macOS: Finder öffnet .uf2 + CPLAYBTBOOT zum manuellen Ziehen
flash-finder: uf2
    @open -R {{uf2}}
    @open /Volumes/CPLAYBTBOOT 2>/dev/null || echo "→ Zuerst Reset 2× (NeoPixels grün)"
    @echo "→ color-hurry-cpb.uf2 auf CPLAYBTBOOT ziehen, bis Laufwerk verschwindet"

# Experimentell: direkter USB-Deploy (auf macOS meist BlockDeviceOpenFail)
flash-deploy: build
    @echo "→ Board im UF2-Modus? (Reset 2×, USB-Datenkabel) …"
    elf2flash deploy --board {{board}} {{elf}}

clean:
    cargo clean

install-tools:
    cargo install elf2flash --locked

# macOS: hängendes CPLAYBTBOOT auswerfen
mac-reset-boot:
    ./scripts/mac-reset-boot.sh

# Aliase (gleich wie flash / flash-v7)
flash-serial-dfu: flash
flash-serial-dfu-v7: flash-v7

# Bootloader + SoftDevice reparieren (Serial-DFU, ~30 s, USB nicht abziehen!)
restore-bootloader:
    chmod +x ./scripts/restore-bootloader-serial.sh
    ./scripts/restore-bootloader-serial.sh

# Reparieren, dann App v6 flashen
restore-and-flash: restore-bootloader
    just flash

# SoftDevice 6.1.1 → 7.0.1 (für BLE / nrf-softdevice), dann App v7
upgrade-softdevice-v7:
    chmod +x ./scripts/upgrade-softdevice-v7.sh
    ./scripts/upgrade-softdevice-v7.sh

# Einmalig: SoftDevice v7 + App v7 (BLE)
setup-ble: upgrade-softdevice-v7
    just flash-v7

# Aliase
flash-ble: flash-v7

# Debug-Lauf: Host-Log + Diagnose-Firmware (pre_init-Stroboskop auf D13)
debug-flash:
    ./scripts/debug-host-log.sh pre-build
    DFU_MANUAL=1 ./scripts/flash-serial.sh
    @echo "→ Nach Reset 1×: 10× schnelles D13-Stroboskop, dann 3× langsam, dann orange NeoPixels"

debug-flash-v7:
    CPB_MEMORY=v7 ./scripts/debug-host-log.sh pre-build
    DFU_MANUAL=1 CPB_MEMORY=v7 ./scripts/flash-serial.sh
    @echo "→ Nach Reset 1×: 10× schnelles D13-Stroboskop, dann 3× langsam, dann orange NeoPixels"

# UF2 statt Serial-DFU (Hypothese H11: direkter Flash funktioniert)
debug-uf2-v7:
    CPB_MEMORY=v7 ./scripts/debug-host-log.sh pre-build
    cargo build --release -p color-hurry-cpb --target {{target}} --no-default-features --features sd-v7
    CPB_MEMORY=v7 ./scripts/debug-host-log.sh pre-uf2
    elf2flash convert --board circuit_playground_bluefruit target/thumbv7em-none-eabihf/release/color-hurry-cpb target/thumbv7em-none-eabihf/release/color-hurry-cpb.uf2
    @open -R target/thumbv7em-none-eabihf/release/color-hurry-cpb.uf2
    @open /Volumes/CPLAYBTBOOT 2>/dev/null || echo "→ Reset 2× (NeoPixels grün), dann .uf2 auf CPLAYBTBOOT ziehen"
    @echo "→ Laufwerk muss verschwinden. Dann USB ab, Reset 1×."

# Endlos-Blink-UF2 — stärkster Test ob Binary startet (H12)
debug-blink-uf2-v7:
    chmod +x ./scripts/debug-blink-uf2.sh
    CPB_MEMORY=v7 ./scripts/debug-blink-uf2.sh

debug-blink-uf2-v6:
    chmod +x ./scripts/debug-blink-uf2.sh
    CPB_MEMORY=v6 ./scripts/debug-blink-uf2.sh

debug-observe obs:
    ./scripts/debug-observe.sh "{{obs}}"

# USB-Serial-Ports anzeigen
ports:
    @ls /dev/cu.usb* /dev/cu.usbmodem* 2>/dev/null || ls /dev/cu.* 2>/dev/null | grep -v Bluetooth || true

# Nur mit externem SWD-Debugger (J-Link / DAPLink an SWD-Pins)
flash-probe: build
    probe-rs download --verify --chip nRF52840_xxAA {{elf}}
    probe-rs reset --chip nRF52840_xxAA
