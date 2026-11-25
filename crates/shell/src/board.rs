//! What the shell needs from a board.
//!
//! The firmware implements [`Board`] with the nRF52840 drivers, the simulator with a window
//! and UDP. The shell reaches the hardware only through this trait.

use color_hurry_core::{Color, Rgb, NUM_LEDS};

/// One Circuit Playground Bluefruit, real or simulated.
// The shell runs on one thread (firmware executor or simulator thread), so the futures
// need no `Send` bound.
#[allow(async_fn_in_trait)]
pub trait Board {
    /// Raw state of button A (left) and button B (right): `true` while pressed.
    fn buttons(&mut self) -> (bool, bool);

    /// Show the ten LEDs at full brightness. A board may dim them, see [`crate::leds`].
    fn show(&mut self, leds: &[Rgb; NUM_LEDS]);

    /// Wait for `ms` milliseconds.
    async fn delay_ms(&mut self, ms: u32);

    /// Play one tone and return when it ends.
    async fn play_tone(&mut self, hz: u32, ms: u32);

    /// Called once per shell loop, e.g. to stop a tone that ran too long.
    fn tone_service(&mut self) {}

    /// Stop any tone that still plays.
    fn stop_tone(&mut self) {}

    /// Radio off: no advertising, no scanning.
    fn radio_off(&mut self);

    /// Become Picker on `channel`. Nothing is sent until [`Board::broadcast`].
    fn radio_picker(&mut self, channel: u8);

    /// Send `color` as Picker, or stop sending with `None`.
    fn broadcast(&mut self, color: Option<Color>);

    /// Become Caller: listen for colors. The core decides whether a color matches.
    fn radio_caller(&mut self, channel: u8);

    /// The last color that the radio received since the last call, as `(channel, color)`.
    fn take_received(&mut self) -> Option<(u8, Color)>;
}
