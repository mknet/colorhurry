//! Unit test of the spec parser. The `shell` test runs without a test harness,
//! so the parser's own test lives in this separate target.

#[allow(dead_code)]
#[path = "cucumber/spec.rs"]
mod spec;

use spec::parse;

    const SPEC: &str = "# demo Specification

## Purpose
Only for the reader.

## Requirements

### Requirement: Matching color completes the task
A matching color SHALL complete the task.
Only the core SHALL decide.

#### Scenario: Picker sends the target in time
- **GIVEN** the Caller on channel 0
  with target Red
- **WHEN** Red arrives on channel 0
- **THEN** the countdown stops
- **AND** no explosion plays

#### Scenario: Board is visible in a scanner
Manual: needs a phone.
- **WHEN** a phone scans
- **THEN** it lists the board
";

    #[test]
    fn spec_becomes_feature_rule_scenario_and_steps() {
        let feature = parse("demo", SPEC).expect("valid gherkin");
        assert_eq!(feature.name, "demo");
        assert_eq!(feature.description, None);
        assert_eq!(feature.rules.len(), 1);

        let rule = &feature.rules[0];
        assert_eq!(rule.name, "Matching color completes the task");
        assert_eq!(
            rule.description.as_deref(),
            Some("A matching color SHALL complete the task. Only the core SHALL decide.")
        );
        assert_eq!(rule.scenarios.len(), 2);

        let scenario = &rule.scenarios[0];
        assert_eq!(scenario.name, "Picker sends the target in time");
        assert_eq!(scenario.position.line, 12);
        assert!(scenario.tags.is_empty());
        let steps: Vec<_> = scenario.steps.iter().map(|s| (s.keyword.trim(), s.value.as_str())).collect();
        assert_eq!(
            steps,
            [
                ("Given", "the Caller on channel 0 with target Red"),
                ("When", "Red arrives on channel 0"),
                ("Then", "the countdown stops"),
                ("And", "no explosion plays"),
            ]
        );

        assert_eq!(rule.scenarios[1].tags, ["allow.skipped"]);
    }
