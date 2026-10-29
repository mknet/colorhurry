//! Integration test binary: runs Gherkin features against the core.
//!
//! Steps map feature wording onto domain [`color_hurry_core::Event`]s.
//! The cucumber harness only completes Delay/Random and records effects.

mod cucumber;

#[tokio::main]
async fn main() {
    cucumber::run().await;
}
