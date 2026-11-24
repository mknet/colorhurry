# Seeed XIAO nRF52840

The XIAO nRF52840 (and the Sense variant) has the same chip as the Circuit Playground Bluefruit. It runs the same firmware (`crates/cpb`) with the `xiao` feature. The core, the shell and the radio protocol are the same, so a XIAO plays with a CPB, the browser and the apps.

## Parts

The XIAO has one RGB LED and no user buttons or speaker. Two Grove parts give it the ten LEDs and buttons A and B:

| Part | Grove port / XIAO pin | Role |
|------|------------------------|------|
| Grove RGB LED Stick (10 × WS2813) | D0 (P0.02) | the ten LEDs, index 0 next to the plug |
| Grove Dual Button, button 1 (yellow wire) | D1 (P0.03) | button A: Skip |
| Grove Dual Button, button 2 (white wire) | D2 (P0.28) | button B: Apply |
| on-board red LED | P0.26 | diagnostics (layout and radio blinks, as D13 on the CPB) |

The buttons pull their line to GND when pressed (active low). The firmware turns on the internal pull-ups. The pins are in `crates/cpb/src/pins.rs`.

There is no speaker. A tone waits for its length without sound, so the countdown keeps its timing.

## Flash

The XIAO ships with an Adafruit-type UF2 bootloader and SoftDevice S140 7.3.0, so the app sits at 0x27000 (`sd-v7`). Press the reset button next to USB-C **twice quickly** to start the bootloader.

```bash
just xiao-flash          # UF2: reads INFO_UF2.TXT from XIAO-SENSE, copies the UF2
just xiao-flash-serial   # serial DFU, when macOS does not mount the drive
just xiao-build          # build only
```

The UF2 recipes look for the drive at `/Volumes/XIAO-SENSE`. Set `XIAO_DRIVE` for another name. With `just xiao-flash-serial`, press reset twice within the 8 seconds that the script waits.

The game firmware has no USB stack, so a 1200-baud touch cannot start the bootloader once the game runs: use the reset button.
