# Color Hurry CPB: Build & Flash
#
# A board starts the app right after its SoftDevice. S140 6.x needs the v6 layout
# (0x26000) and S140 7.x needs the v7 layout (0x27000). Bluetooth works with both.
#
# UF2 recipes (CPLAYBTBOOT mounted): read the board's SoftDevice and pick the layout.
#   just board       → print the SoftDevice and the matching layout
#   just flash-uf2   → build, convert and copy the UF2 (override: just flash-uf2 v6)
# Serial DFU (no CPLAYBTBOOT needed) cannot read the SoftDevice, so you choose.
#   just flash       → v6 @ 0x26000
#   just flash-v7    → v7 @ 0x27000

target  := "thumbv7em-none-eabihf"
profile := "release"
elf     := "target" / target / profile / "color-hurry-cpb"
uf2_file   := "target/color-hurry-cpb.uf2"
uf2_family := "0xADA52840"
uf2_tools  := "target/uf2-tools"
board   := "circuit_playground_bluefruit"
# The board's bootloader drive. Override with CPB_DRIVE=/path/to/CPLAYBTBOOT.
drive   := env("CPB_DRIVE", if os() == "macos" { "/Volumes/CPLAYBTBOOT" } else { "/media/" + env("USER", "") + "/CPLAYBTBOOT" })

default: flash-v7

# Browser shell (WASM): http://127.0.0.1:8080
# Needs --cfg=web_sys_unstable_apis for Web Bluetooth (also in crates/web/.cargo/config.toml).
web-serve:
    cd crates/web && RUSTFLAGS='--cfg=web_sys_unstable_apis' NO_COLOR=false trunk serve --open

web-build:
    cd crates/web && RUSTFLAGS='--cfg=web_sys_unstable_apis' NO_COLOR=false trunk build --release

# Rust core for the native apps: BoltFFI packages + generated Swift/Kotlin types
# Needs `cargo install boltffi_cli --version '=0.30.1' --locked`
ffi: ffi-apple ffi-android

# Builds the core for iOS, macOS and watchOS. BoltFFI makes the iOS and macOS slices, and
# scripts/xcframework_watchos.rs adds the watchOS slices (needs nightly rust-src).
ffi-apple:
    cd crates/ffi && boltffi pack apple --release
    ./scripts/xcframework_watchos.rs
    cargo run -p color-hurry-ffi --features codegen --bin codegen -- --language swift --output-dir apps/apple/generated

alias ffi-ios := ffi-apple

# Uses ANDROID_NDK_HOME, else the newest NDK under ~/Library/Android/sdk/ndk
ffi-android:
    #!/usr/bin/env bash
    set -euo pipefail
    export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$(ls -d "$HOME"/Library/Android/sdk/ndk/* | sort -V | tail -1)}"
    (cd crates/ffi && boltffi pack android)
    cargo run -p color-hurry-ffi --features codegen --bin codegen -- --language kotlin --output-dir apps/android/generated

# Native iOS, macOS and watchOS (SwiftUI): regenerate the Xcode project, then open it
apple: ffi-apple
    cd apps/apple && xcodegen generate && open ColorHurry.xcodeproj

ios: apple

ios-build: ffi-apple
    cd apps/apple && xcodegen generate && xcodebuild -scheme ColorHurry -destination 'generic/platform=iOS Simulator' -configuration Debug CODE_SIGNING_ALLOWED=NO build

mac-build: ffi-apple
    cd apps/apple && xcodegen generate && xcodebuild -scheme ColorHurryMac -destination 'platform=macOS' -configuration Debug CODE_SIGNING_ALLOWED=NO build

watch-build: ffi-apple
    cd apps/apple && xcodegen generate && xcodebuild -scheme ColorHurryWatch -destination 'generic/platform=watchOS Simulator' -configuration Debug CODE_SIGNING_ALLOWED=NO build

# Run the watch app on a 40 mm watch simulator (the screen size of a Series 6). Creates the simulator the first time.
watch_sim := "ColorHurry Watch 40mm"
watch-sim: ffi-apple
    xcrun simctl list devices | grep -q "{{watch_sim}}" || xcrun simctl create "{{watch_sim}}" com.apple.CoreSimulator.SimDeviceType.Apple-Watch-SE-3-40mm
    xcrun simctl boot "{{watch_sim}}" 2>/dev/null || true
    open "$(xcode-select -p)/../Applications/DeviceHub.app"
    cd apps/apple && xcodegen generate && xcodebuild -scheme ColorHurryWatch -destination 'platform=watchOS Simulator,name={{watch_sim}}' -configuration Debug -derivedDataPath ../../target/xcode CODE_SIGNING_ALLOWED=NO build
    xcrun simctl bootstatus "{{watch_sim}}"
    xcrun simctl install "{{watch_sim}}" target/xcode/Build/Products/Debug-watchsimulator/ColorHurry.app
    xcrun simctl launch --terminate-running-process "{{watch_sim}}" com.colorhurry.app.watch

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

# Core unit tests + spec scenarios run as tests + FFI payload helpers
test:
    cargo test -p color-hurry-core -p color-hurry-ffi

# Serial DFU cannot read the SoftDevice version, so you choose the layout:
# `flash` for v6 and `flash-v7` for v7. The UF2 recipes (`board`, `uf2`, `flash-uf2`)
# detect the layout from the board.

# Serial-DFU v7 (S140 7.x @ 0x27000)
flash-v7:
    DFU_MANUAL=1 CPB_MEMORY=v7 ./scripts/flash-serial.sh

# Serial-DFU v6 (S140 6.1.1 @ 0x26000)
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

# --- UF2 (needs CPLAYBTBOOT mounted: double-press reset) ---

# Print the board's SoftDevice and the matching layout (builds and flashes nothing)
[script('bash')]
board:
    set -euo pipefail
    {{ just_executable() }} _sd-layout auto report > /dev/null

# (internal) print v6 or v7 on stdout. sd is auto (read INFO_UF2.TXT on the board), v6 or v7.
[script('bash')]
_sd-layout sd="auto" report="":
    set -euo pipefail
    sd='{{ sd }}'
    if [ "$sd" = auto ]; then
        info='{{ drive }}/INFO_UF2.TXT'
        if [ ! -f "$info" ]; then
            echo '✗ {{ drive }} is not mounted: double-press the reset button, set CPB_DRIVE, or pass v6 or v7 as the sd argument' >&2
            exit 1
        fi
        line=$(grep -m1 '^SoftDevice:' "$info" || true)
        version=$(printf '%s\n' "$line" | sed -n 's/^SoftDevice: S140 \([0-9][0-9.]*\).*/\1/p')
        case "$version" in
            6.*) sd=v6 ;;
            7.*) sd=v7 ;;
            *)
                echo "✗ unsupported SoftDevice in $info: '${line:-no SoftDevice line}' (expected S140 6.x or 7.x). Pass v6 or v7 as the sd argument to choose." >&2
                exit 1
                ;;
        esac
        if [ -n '{{ report }}' ]; then
            if [ "$sd" = v6 ]; then origin=0x26000; else origin=0x27000; fi
            echo "SoftDevice S140 $version: layout $sd (flash origin $origin)" >&2
        fi
    elif [ "$sd" != v6 ] && [ "$sd" != v7 ]; then
        echo "✗ unknown sd '$sd': use auto, v6 or v7" >&2
        exit 1
    fi
    echo "$sd"

# (internal) fetch Microsoft's uf2conv.py into target/ (not committed)
[script('bash')]
_uf2-tools:
    set -euo pipefail
    mkdir -p {{ uf2_tools }}
    for file in uf2conv.py uf2families.json; do
        [ -f {{ uf2_tools }}/$file ] || curl -fsSL -o {{ uf2_tools }}/$file https://raw.githubusercontent.com/microsoft/uf2/master/utils/$file
    done

# Make target/color-hurry-cpb.uf2. sd: auto reads the board, or v6 or v7 (just uf2 v6)
[script('bash')]
uf2 sd="auto": _uf2-tools
    set -euo pipefail
    layout=$({{ just_executable() }} _sd-layout {{ sd }} report)
    if [ "$layout" = v6 ]; then
        base=0x26000
        features='--no-default-features --features sd-v6'
    else
        base=0x27000
        features='--no-default-features --features sd-v7'
    fi
    cargo build --{{ profile }} -p color-hurry-cpb --target {{ target }} $features
    rust-objcopy -O binary {{ elf }} target/color-hurry-cpb.bin
    echo "UF2 base $base ($layout)"
    python3 -I {{ uf2_tools }}/uf2conv.py target/color-hurry-cpb.bin -c -b "$base" -f {{ uf2_family }} -o {{ uf2_file }}

# Make the UF2 and copy it to the board (double-press reset first). sd: auto, v6 or v7 (just flash-uf2 v6)
[script('bash')]
flash-uf2 sd="auto": (uf2 sd)
    set -euo pipefail
    if [ ! -d '{{ drive }}' ]; then
        echo '✗ {{ drive }} is not mounted: double-press the reset button, or set CPB_DRIVE'
        exit 1
    fi
    # The board resets as soon as it has the image, so the copy can report an error
    # writing metadata. On macOS, -X skips the extended attributes.
    if [ '{{ os() }}' = macos ]; then cp -X {{ uf2_file }} '{{ drive }}/'; else cp {{ uf2_file }} '{{ drive }}/'; fi
    echo '✓ flashed; the board resets into the app'
    echo '→ If the app does not start: unplug USB for 5 s, plug in, Reset 1× (not 2×), no buttons'

# Check whether the CPLAYBTBOOT drive / USB is visible (macOS)
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

# macOS: Finder opens the UF2 and CPLAYBTBOOT for manual dragging (auto layout, or just flash-finder v6)
flash-finder sd="auto": (uf2 sd)
    @open -R {{uf2_file}}
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

# SoftDevice 6.1.1 → 7.0.1, then app v7
upgrade-softdevice-v7:
    chmod +x ./scripts/upgrade-softdevice-v7.sh
    ./scripts/upgrade-softdevice-v7.sh

# One-time: upgrade the SoftDevice to v7 and flash the v7 app (only if the board has S140 6.x and you want v7)
setup-ble: upgrade-softdevice-v7
    just flash-v7

# Alias of flash-v7 (serial DFU, v7 layout)
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
