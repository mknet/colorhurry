# Color Hurry — Circuit Playground Bluefruit (Rust)

Kooperatives 2-Spieler-Spiel für zwei **Adafruit Circuit Playground Bluefruit** (nRF52840) mit Embassy, NeoPixels, BLE-Broadcast und PWM-Audio.

## Voraussetzungen

- [Rust](https://rustup.rs/) (stable) mit Target `thumbv7em-none-eabihf`
- [just](https://github.com/casey/just) (`cargo install just`)
- [elf2flash](https://crates.io/crates/elf2flash) für Flashen per USB/UF2 (`just install-tools`)

```bash
rustup target add thumbv7em-none-eabihf
just install-tools   # einmalig
```

Optional (nur mit externem SWD-Debugger): [probe-rs](https://probe.rs/) — `just flash-probe`

## BLE (SoftDevice S140 7.0.1)

Standard-Build: Feature **`sd-v7`**, App @ **0x27000**. Adafruit Restore liefert **S140 6.1.1** — damit sendet BLE **nicht**. Einmalig:

```bash
just setup-ble    # SD 6.1.1 → 7.0.1 + App flashen
just flash-v7     # danach nur noch App
```

Ausführlich: [docs/BLE.md](docs/BLE.md), Troubleshooting: [docs/ERKENNTNISSE-BLE-FIX.md](docs/ERKENNTNISSE-BLE-FIX.md)

Ohne BLE (nur Color Picker): `just restore-and-flash` + `just flash` (6.1.1 @ 0x26000).

**Dauerhaft grün** = Bootloader, App läuft nicht → [docs/RECOVERY.md](docs/RECOVERY.md)

**macOS: kein Zugriff auf CPLAYBTBOOT** → [docs/FLASH-MACOS.md](docs/FLASH-MACOS.md) (`just flash-serial`)

## Bauen & Flashen (USB)

Board mit **USB-Datenkabel** verbinden (reines Ladekabel reicht nicht).

**Bootloader:** nicht Button A/B — den kleinen **Reset in der Mitte** **2× schnell** drücken, **oder** **Button A (links) halten** und **Reset 1×**. Im Finder erscheint **CPLAYBTBOOT** (NeoPixels: rot → grün). Ausführlich: [docs/BOOTLOADER.md](docs/BOOTLOADER.md).

Dann:

```bash
just          # baut (release), erzeugt .uf2, kopiert auf CPLAYBTBOOT
# oder:
just flash
just build    # nur kompilieren
```

`just` wartet bis zu 60 s auf das BOOT-Laufwerk (Reset kann auch nach Start erfolgen).

Alternativ: `cargo run --release` (gleicher Ablauf über `scripts/cargo-uf2-runner.sh`).

Ohne Rebuild: `just flash-bin`.

Experimentell (oft auf macOS kaputt): `just flash-deploy` (`elf2flash deploy` → `BlockDeviceOpenFail`).

**Hinweis:** Verschwindet `CPLAYBTBOOT` während des Kopierens mit `Input/output error`, war der Flash oft trotzdem erfolgreich (Board startet sofort neu). Das Skript erkennt das jetzt als Erfolg.

## defmt-Logs

RTT funktioniert nur mit einem **SWD-Debugger** (probe-rs), nicht über den UF2-USB-Bootloader allein:

```bash
probe-rs attach --chip nRF52840_xxAA
```

## Hardware (Bluefruit)

| Funktion   | Pin    |
|-----------|--------|
| NeoPixel  | P0.08  |
| Button A  | P1.02  |
| Button B  | P0.07  |
| Speaker   | P0.26  |
| Spk. En.  | P1.04  |

Pinout: [Circuit Playground Bluefruit](https://learn.adafruit.com/adafruit-circuit-playground-bluefruit/pinouts)

## Projektphasen

1. **Skelett** — Embassy + defmt
2. **NeoPixel-Treiber** (aktuell) — 10× WS2812 an P0.13, Blinktest mit D13
3. Buttons mit Debouncing
4. Speaker (Destroy / Success)
5. BLE Advertising + Scan (`nrf-softdevice`)
6. Spiel-State-Machine

## Lizenz

MIT OR Apache-2.0
