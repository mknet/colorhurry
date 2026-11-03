# Color Hurry (Apple)

Native SwiftUI apps with a CoreBluetooth central (scan manufacturer ads), driven by the Rust core over FFI. One xcodegen project holds three apps:

- `ColorHurry`: iOS.
- `ColorHurryMac`: macOS 14+.
- `ColorHurryWatch`: standalone watchOS 10+ (Apple Watch Series 6 and later).

None of them can be Picker, because CoreBluetooth cannot advertise manufacturer data on these systems. Use a board or an Android phone as Picker.

The `arm64_32` watch build needs nightly Rust with `rust-src`. Install it once:

```bash
rustup component add rust-src --toolchain nightly
```

Then build the core and open the project:

```bash
just ffi-apple             # from repo root: BoltFFI package, watchOS slices, generated Swift types
cd apps/apple
xcodegen generate
open ColorHurry.xcodeproj
```

Or from repo root: `just apple`. To build one app without signing, run `just ios-build`, `just mac-build` or `just watch-build`.

To run an app on a device, or to use Bluetooth on the Mac, set your signing team. Create `apps/apple/Local.xcconfig` with this line:

```
DEVELOPMENT_TEAM = <your team ID>
```

Git ignores `Local.xcconfig` and `ColorHurry.xcodeproj`. `xcodegen generate` makes the project again from `project.yml`, so do not set the team in Xcode. Xcode does not keep it.
