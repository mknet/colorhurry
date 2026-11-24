//! [`Board`] for the Circuit Playground Bluefruit or the Seeed XIAO nRF52840 (feature
//! `xiao`), over the drivers of this crate.

use color_hurry_core::{Color, Rgb, NUM_LEDS};
use color_hurry_shell::{leds, Board};

#[cfg(not(feature = "xiao"))]
use crate::speaker;
use crate::{ble_broadcast, buttons, neopixel_pwm, time};

pub struct CpbBoard;

impl CpbBoard {
    pub fn new() -> Self {
        buttons::init();
        Self
    }
}

impl Board for CpbBoard {
    fn buttons(&mut self) -> (bool, bool) {
        buttons::raw()
    }

    fn show(&mut self, leds: &[Rgb; NUM_LEDS]) {
        let dimmed = leds::dim_all(leds).map(|c| neopixel_pwm::Rgb {
            r: c.r,
            g: c.g,
            b: c.b,
        });
        neopixel_pwm::show_pixels(&dimmed);
    }

    async fn delay_ms(&mut self, ms: u32) {
        time::delay_ms_async(ms).await;
    }

    #[cfg(not(feature = "xiao"))]
    async fn play_tone(&mut self, hz: u32, ms: u32) {
        let duty = speaker::duty_for_core_tone(hz, ms);
        speaker::play_tone(hz, ms, duty).await;
    }

    /// The XIAO has no speaker: wait as long as the tone would last, so the timing stays.
    #[cfg(feature = "xiao")]
    async fn play_tone(&mut self, _hz: u32, ms: u32) {
        time::delay_ms_async(ms).await;
    }

    #[cfg(not(feature = "xiao"))]
    fn tone_service(&mut self) {
        speaker::tone_service();
    }

    #[cfg(not(feature = "xiao"))]
    fn stop_tone(&mut self) {
        speaker::stop_tone();
    }

    fn radio_off(&mut self) {
        ble_broadcast::set_radio_off();
    }

    fn radio_picker(&mut self, channel: u8) {
        ble_broadcast::set_radio_picker(channel as usize);
    }

    fn broadcast(&mut self, color: Option<Color>) {
        ble_broadcast::set_picker_broadcast(color);
    }

    fn radio_caller(&mut self, channel: u8) {
        ble_broadcast::set_radio_caller(channel as usize);
    }

    fn take_received(&mut self) -> Option<(u8, Color)> {
        ble_broadcast::take_color_received()
    }
}
