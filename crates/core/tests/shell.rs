//! Integration test binary: runs the OpenSpec spec scenarios against the core.
//!
//! `openspec/specs/*/spec.md` is read as Gherkin, the steps map the scenario
//! wording onto domain [`color_hurry_core::Event`]s. The harness only answers
//! the timers and Random, and records effects.

mod cucumber;

#[tokio::main]
async fn main() {
    cucumber::run().await;
}
