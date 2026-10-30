# Color Hurry CPB: Build & Flash
#
# Default on macOS: serial DFU (no CPLAYBTBOOT needed)
#   just flash       → v6 @ 0x26000
#   just flash-v7    → v7 @ 0x27000

target  := "thumbv7em-none-eabihf"
profile := "release"
elf     := "target" / target / profile / "color-hurry-cpb"
uf2     := elf + ".uf2"
board   := "circuit_playground_bluefruit"

default: flash-v7

# Browser shell (WASM): http://127.0.0.1:8080
# Needs --cfg=web_sys_unstable_apis for Web Bluetooth (also in crates/web/.cargo/config.toml).
web-serve:
    cd crates/web && RUSTFLAGS='--cfg=web_sys_unstable_apis' NO_COLOR=false trunk serve --open

web-build:
    cd crates/web && RUSTFLAGS='--cfg=web_sys_unstable_apis' NO_COLOR=false trunk build --release

# Rust core for the native apps: BoltFFI packages + generated Swift/Kotlin types
# Needs `cargo install boltffi_cli --version '=0.30.1' --locked`
ffi: ffi-ios ffi-android

ffi-ios:
    cd crates/ffi && boltffi pack apple
    cargo run -p color-hurry-ffi --features codegen --bin codegen -- --language swift --output-dir apps/ios/generated

# Uses ANDROID_NDK_HOME, else the newest NDK under ~/Library/Android/sdk/ndk
ffi-android:
    #!/usr/bin/env bash
    set -euo pipefail
    export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$(ls -d "$HOME"/Library/Android/sdk/ndk/* | sort -V | tail -1)}"
    (cd crates/ffi && boltffi pack android)
    cargo run -p color-hurry-ffi --features codegen --bin codegen -- --language kotlin --output-dir apps/android/generated

# Native iOS (SwiftUI): regenerate Xcode project then open
ios: ffi-ios
    cd apps/ios && xcodegen generate && open ColorHurry.xcodeproj

ios-build: ffi-ios
    cd apps/ios && xcodegen generate && xcodebuild -scheme ColorHurry -destination 'generic/platform=iOS Simulator' -configuration Debug CODE_SIGNING_ALLOWED=NO build

# Native Android (Kotlin/Compose): needs JDK 17/21
android: ffi-android
    #!/usr/bin/env bash
    set -euo pipefail
    export JAVA_HOME="${JAVA_HOME:-$(/usr/libexec/java_home -v 21 2>/dev/null || /usr/libexec/java_home -v 17)}"
    cd apps/android && ./gradlew :app:installDebug

android-build: ffi-android
    #!/usr/bin/env bash
    set -euo pipefail
    export JAVA_HOME="${JAVA_HOME:-$(/usr/libexec/java_home -v 21 2>/dev/null || /usr/libexec/java_home -v 17)}"
    cd apps/android && ./gradlew :app:assembleDebug

# Core unit tests + Gherkin feature tests + FFI payload helpers
test:
    cargo test -p color-hurry-core -p color-hurry-ffi

# Serial-DFU v7 (S140 7.x @ 0x27000): for BLE / nrf-softdevice
flash-v7:
    DFU_MANUAL=1 CPB_MEMORY=v7 ./scripts/flash-serial.sh

# Serial-DFU v6 (S140 6.1.1 @ 0x26000): Color Picker only, without BLE
flash:
    DFU_MANUAL=1 ./scripts/flash-serial.sh

# Automatic (touch), in case manual DFU is annoying
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

# --- UF2 (only if CPLAYBTBOOT mounts again) ---

flash-uf2: rebuild
    @echo "→ NeoPixels GREEN? Finder: drag .uf2 onto CPLAYBTBOOT (see docs/RECOVERY.md)"
    @echo "→ After flash: unplug USB for 5 s, plug in, Reset 1× (not 2×), no buttons"
    elf2flash convert --board {{board}} {{elf}} {{uf2}}
    ./scripts/copy-uf2.sh {{uf2}}

# Check whether the BOOT drive / USB is visible (macOS)
boot-check:
    @echo "=== Mounted BOOT drives ==="
    @ls -d /Volumes/*BOOT 2>/dev/null || echo "(none)"
    @test -d /Volumes/CPLAYBTBOOT && echo "OK: CPLAYBTBOOT" || echo "MISSING: CPLAYBTBOOT"
    @echo ""
    @echo "=== USB (Adafruit 0x239A) ==="
    @system_profiler SPUSBDataType 2>/dev/null | grep -E "239A|Circuit|Playground|Bluefruit|CPLAY" -A3 || echo "(nothing found)"

# Build only
build:
    cargo build --{{profile}} -p color-hurry-cpb --target {{target}} --features sd-v7

# ELF → UF2 (without flashing)
uf2: build
    elf2flash convert --board {{board}} {{elf}} {{uf2}}
    @echo "→ {{uf2}}"

# Already built (no rebuild)
flash-bin:
    elf2flash convert --board {{board}} {{elf}} {{uf2}}
    ./scripts/copy-uf2.sh {{uf2}}

# Bootloader already active (NeoPixels green): just copy
flash-now: uf2
    ./scripts/copy-uf2.sh {{uf2}}

# macOS: Finder opens .uf2 + CPLAYBTBOOT for manual dragging
flash-finder: uf2
    @open -R {{uf2}}
    @open /Volumes/CPLAYBTBOOT 2>/dev/null || echo "→ Reset 2× first (NeoPixels green)"
    @echo "→ drag color-hurry-cpb.uf2 onto CPLAYBTBOOT until the drive disappears"

# Experimental: direct USB deploy (usually BlockDeviceOpenFail on macOS)
flash-deploy: build
    @echo "→ Board in UF2 mode? (Reset 2×, USB data cable) …"
    elf2flash deploy --board {{board}} {{elf}}

clean:
    cargo clean

install-tools:
    cargo install elf2flash --locked

# macOS: eject a hung CPLAYBTBOOT
mac-reset-boot:
    ./scripts/mac-reset-boot.sh

# Aliases (same as flash / flash-v7)
flash-serial-dfu: flash
flash-serial-dfu-v7: flash-v7

# Repair bootloader + SoftDevice (serial DFU, ~30 s, do not unplug USB!)
restore-bootloader:
    chmod +x ./scripts/restore-bootloader-serial.sh
    ./scripts/restore-bootloader-serial.sh

# Repair, then flash app v6
restore-and-flash: restore-bootloader
    just flash

# SoftDevice 6.1.1 → 7.0.1 (for BLE / nrf-softdevice), then app v7
upgrade-softdevice-v7:
    chmod +x ./scripts/upgrade-softdevice-v7.sh
    ./scripts/upgrade-softdevice-v7.sh

# One-time: SoftDevice v7 + app v7 (BLE)
setup-ble: upgrade-softdevice-v7
    just flash-v7

# Aliases
flash-ble: flash-v7

# Debug run: host log + diagnostic firmware (pre_init strobe on D13)
debug-flash:
    ./scripts/debug-host-log.sh pre-build
    DFU_MANUAL=1 ./scripts/flash-serial.sh
    @echo "→ After Reset 1×: 10× fast D13 strobe, then 3× slow, then orange NeoPixels"

debug-flash-v7:
    CPB_MEMORY=v7 ./scripts/debug-host-log.sh pre-build
    DFU_MANUAL=1 CPB_MEMORY=v7 ./scripts/flash-serial.sh
    @echo "→ After Reset 1×: 10× fast D13 strobe, then 3× slow, then orange NeoPixels"

# UF2 instead of serial DFU (hypothesis H11: direct flash works)
debug-uf2-v7:
    CPB_MEMORY=v7 ./scripts/debug-host-log.sh pre-build
    cargo build --release -p color-hurry-cpb --target {{target}} --no-default-features --features sd-v7
    CPB_MEMORY=v7 ./scripts/debug-host-log.sh pre-uf2
    elf2flash convert --board circuit_playground_bluefruit target/thumbv7em-none-eabihf/release/color-hurry-cpb target/thumbv7em-none-eabihf/release/color-hurry-cpb.uf2
    @open -R target/thumbv7em-none-eabihf/release/color-hurry-cpb.uf2
    @open /Volumes/CPLAYBTBOOT 2>/dev/null || echo "→ Reset 2× (NeoPixels green), then drag .uf2 onto CPLAYBTBOOT"
    @echo "→ The drive must disappear. Then unplug USB, Reset 1×."

# Endless-blink UF2: strongest test of whether the binary starts (H12)
debug-blink-uf2-v7:
    chmod +x ./scripts/debug-blink-uf2.sh
    CPB_MEMORY=v7 ./scripts/debug-blink-uf2.sh

debug-blink-uf2-v6:
    chmod +x ./scripts/debug-blink-uf2.sh
    CPB_MEMORY=v6 ./scripts/debug-blink-uf2.sh

debug-observe obs:
    ./scripts/debug-observe.sh "{{obs}}"

# Show USB serial ports
ports:
    @ls /dev/cu.usb* /dev/cu.usbmodem* 2>/dev/null || ls /dev/cu.* 2>/dev/null | grep -v Bluetooth || true

# Only with an external SWD debugger (J-Link / DAPLink on SWD pins)
flash-probe: build
    probe-rs download --verify --chip nRF52840_xxAA {{elf}}
    probe-rs reset --chip nRF52840_xxAA
