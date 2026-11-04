# Design

## Parser

cucumber-rs accepts its own `cucumber::Parser`. `SpecParser` reads every `openspec/specs/<capability>/spec.md`, writes Gherkin text and parses it with `gherkin::Feature::parse` from the `gherkin` crate that cucumber re-exports. No new dependency.

| spec.md | Gherkin |
| --- | --- |
| `<capability>` (directory) | `Feature: <capability>` |
| `### Requirement: <name>` | `Rule: <name>`, SHALL text as description |
| `#### Scenario: <name>` | `Scenario: <name>` |
| `- **GIVEN/WHEN/THEN/AND** <text>` | `Given/When/Then/And <text>` |
| `Manual: <reason>` | `@allow.skipped` on the scenario |

Every spec line keeps its line number in the generated text, and the feature's `path` is the `spec.md`. A failing or unmatched step therefore points to `openspec/specs/<capability>/spec.md:<line>`. The `## Purpose` text is not part of the feature. Requirement descriptions are set after parsing, because a SHALL sentence may start with a Gherkin keyword ("When the tenth beat …").

## Manual scenarios

Some scenarios describe a board, a browser or a phone. They stay in the spec, marked with a line `Manual: <what it needs>`. The parser tags them `@allow.skipped`, so `fail_on_skipped` still fails every other scenario whose steps do not match.

## Harness

Timers fire only when virtual time advances, so "the countdown begins" sees the full ring before the first beat. A random request without queued entropy stays open until "the shell answers the random request with N", so "a new random byte is requested" is observable.
