//! Color Hurry core — platform-free game logic with a Crux-like API.
//!
//! Shells drive the loop: `Event` in → `update` → `Effect`s out → `view` for display.

#![no_std]

mod color;
mod effects;
mod model;
mod palette;
mod update;
mod view;

pub use color::Rgb;
pub use effects::{Effect, Effects, ToneKind};
pub use model::{Event, Model, Role, Screen, NUM_LEDS};
pub use palette::{CHANNELS, SPECTRUM_LEN};
pub use update::update;
pub use view::{view, ViewModel};

#[cfg(test)]
mod tests;
