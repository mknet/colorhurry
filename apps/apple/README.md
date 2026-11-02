# Color Hurry (iOS)

Native SwiftUI app + CoreBluetooth central (scan manufacturer ads), driven by the Rust core over FFI.

```bash
just ffi-ios               # from repo root: BoltFFI package + generated Swift types
cd apps/apple
xcodegen generate
open ColorHurry.xcodeproj
```

Or from repo root: `just ios`
