# Color Hurry (Android)

Native Kotlin + Jetpack Compose app, driven by the Rust core over FFI. Scans and advertises BLE manufacturer data (protocol v4).

```bash
just ffi-android                                   # from repo root: jniLibs + generated Kotlin types
export JAVA_HOME=$(/usr/libexec/java_home -v 21)   # 17 also fine
./gradlew :app:assembleDebug
./gradlew :app:installDebug   # with a device/emulator
```

Or from repo root: `just android`
