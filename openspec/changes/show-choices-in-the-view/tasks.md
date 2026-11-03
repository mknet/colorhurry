# Tasks

## 1. Core

- [x] 1.1 Add `mode_choice` to the `ViewModel` and export `ModeChoice` and `ModeRole`, verify with `just test`
- [x] 1.2 Add the Gherkin scenarios to `crates/core/tests/features/mode_select.feature`, verify they pass

- [x] 1.3 Add `picker_color` and `picker_applied`, with scenarios in `crates/core/tests/features/picker.feature`, verify they pass

## 2. Shells

- [x] 2.1 Regenerate the Swift types and show the choice in `modeHint`, verify the iOS, macOS and watchOS builds
- [ ] 2.2 Show the choice in the Android `modeHint`, verify with `./gradlew :app:assembleDebug`
