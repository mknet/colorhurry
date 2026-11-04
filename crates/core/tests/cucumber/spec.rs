//! Reads the OpenSpec specs as Gherkin: the spec scenarios are the tests.
//!
//! Capability becomes Feature, Requirement becomes Rule (its SHALL text is the
//! description), Scenario stays Scenario, `- **WHEN** …` becomes a step. Every
//! spec line keeps its line number, so a failing step points into `spec.md`.
//! A scenario with a `Manual:` line needs hardware and is tagged `@allow.skipped`.

use std::path::{Path, PathBuf};
use std::{fs, vec};

use ::cucumber::{cli, gherkin, parser};
use futures::stream;

pub struct SpecParser;

impl<I: AsRef<Path>> parser::Parser<I> for SpecParser {
    type Cli = cli::Empty;
    type Output = stream::Iter<vec::IntoIter<parser::Result<gherkin::Feature>>>;

    fn parse(self, specs: I, _: cli::Empty) -> Self::Output {
        let mut paths: Vec<PathBuf> = fs::read_dir(specs.as_ref())
            .map(|dir| dir.filter_map(Result::ok).map(|e| e.path().join("spec.md")).collect())
            .unwrap_or_default();
        paths.retain(|p| p.is_file());
        paths.sort();
        let features: Vec<_> = paths.iter().map(|p| read(p).map_err(Into::into)).collect();
        stream::iter(features)
    }
}

fn read(path: &Path) -> Result<gherkin::Feature, gherkin::ParseFileError> {
    let markdown = fs::read_to_string(path).map_err(|source| gherkin::ParseFileError::Reading {
        path: path.to_path_buf(),
        source,
    })?;
    let capability = path.parent().and_then(Path::file_name).unwrap_or_default();
    let mut feature = parse(&capability.to_string_lossy(), &markdown).map_err(|source| {
        gherkin::ParseFileError::Parsing { path: path.to_path_buf(), error: None, source }
    })?;
    feature.path = Some(path.to_path_buf());
    Ok(feature)
}

/// Turns one `spec.md` into a Gherkin feature named after the capability.
pub fn parse(capability: &str, markdown: &str) -> Result<gherkin::Feature, gherkin::ParseError> {
    let mut lines = vec![String::new(); markdown.lines().count()];
    let mut rules = Vec::new();
    let (mut step, mut scenario, mut in_rule) = (None, None, false);
    for (n, line) in markdown.lines().enumerate() {
        let line = line.trim();
        let bullet = line.strip_prefix("- **").and_then(|l| l.split_once("**"));
        if n == 0 {
            lines[0] = format!("Feature: {capability}");
        } else if let Some(name) = line.strip_prefix("### Requirement:") {
            lines[n] = format!("  Rule:{name}");
            rules.push(String::new());
            (step, scenario, in_rule) = (None, None, true);
        } else if let Some(name) = line.strip_prefix("#### Scenario:") {
            lines[n] = format!("    Scenario:{name}");
            (step, scenario, in_rule) = (None, Some(n), false);
        } else if let Some((keyword, text)) = bullet.filter(|_| scenario.is_some()) {
            lines[n] = format!("      {} {}", title_case(keyword), text.trim().replace("**", ""));
            step = Some(n);
        } else if line.starts_with("Manual:") && scenario.is_some() {
            lines[scenario.unwrap() - 1] = "    @allow.skipped".into();
        } else if line.starts_with('#') || line.is_empty() {
            step = None;
            in_rule &= !line.starts_with('#');
        } else if let Some(s) = step {
            lines[s] = format!("{} {}", lines[s], line.replace("**", ""));
        } else if in_rule {
            let description = rules.last_mut().unwrap();
            description.push_str(if description.is_empty() { "" } else { " " });
            description.push_str(line);
        }
    }
    let mut feature = gherkin::Feature::parse(lines.join("\n"), Default::default())?;
    for (rule, description) in feature.rules.iter_mut().zip(rules) {
        rule.description = Some(description);
    }
    Ok(feature)
}

fn title_case(keyword: &str) -> String {
    let lower = keyword.to_lowercase();
    lower[..1].to_uppercase() + &lower[1..]
}
