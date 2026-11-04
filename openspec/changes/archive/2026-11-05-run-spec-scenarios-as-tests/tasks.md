# Tasks

## 1. Parser

- [x] 1.1 Add `crates/core/tests/cucumber/spec.rs`: `SpecParser` reads `openspec/specs/*/spec.md` as Gherkin features with requirement rules, keeps line numbers and tags `Manual:` scenarios `@allow.skipped`; verify the unit test in `tests/spec_parser.rs`
- [x] 1.2 Run the specs with `.with_parser(SpecParser)` and `.fail_on_skipped()`; update the doc comment in `tests/shell.rs`

## 2. Specs and steps

- [x] 2.1 Rewrite the spec scenarios as GIVEN/WHEN/THEN with recurring phrases, mark scenarios that need hardware `Manual:`
- [x] 2.2 Turn the imperative steps into helpers on `ColorHurryWorld` (`enter`, `select`, `beats`, `settle`, `byte_for`) and write declarative steps on top; verify `just test`
- [x] 2.3 Map each of the 27 feature scenarios: covered by the spec, added to the spec, or a unit test in `crates/core/src/tests.rs` (blink rescheduling, all ten ring steps, spectrum order)

## 3. Cleanup

- [x] 3.1 Delete `crates/core/tests/features`, remove the `Acceptance:` lines, update `README.md` and `justfile`; verify changing one word in a spec scenario fails `just test`
- [x] 3.2 Verify `openspec validate --all --strict`
