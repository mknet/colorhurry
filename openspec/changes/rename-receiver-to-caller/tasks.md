# Tasks

## 1. Core

- [ ] 1.1 Rename `Role::Receiver` and `Screen::Receiver` to `Caller`, `update_receiver` to `update_caller`, and the view helper, keep `BleOp::Receiver`; verify `cargo test -p color-hurry-core`
- [ ] 1.2 Rename `receiver.feature` to `caller.feature` and the steps "I enter caller on channel …", update `mode_select.feature`; verify all Gherkin scenarios pass with `just test`

## 2. Shells

- [ ] 2.1 Board and web shell: screen matches and UI texts say Caller; verify `just build` and `just web-build`
- [ ] 2.2 iOS and Android: regenerate the types with `just ffi`, rename screen cases and UI texts; verify `just ios-build` and `just android-build`

## 3. Docs

- [ ] 3.1 Move `docs/RECEIVER.md` to `docs/CALLER.md`, update README files, other docs and the OpenSpec project context; verify no role mention of Receiver remains outside the radio code, `docs/CORE.md` and the archive
