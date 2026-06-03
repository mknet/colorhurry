//! Empfänger: Zufallsfarbe aus dem 10er-Spektrum, Countdown (LED + Ton).

use crate::countdown;
use crate::neopixel_pwm::NUM_LEDS;
use crate::palette;
use crate::time;

pub async fn run(_channel: usize) -> ! {
    loop {
        let color = palette::random_spectrum_color();
        crate::neopixel_pwm::show_pixels(&[color; NUM_LEDS]);
        time::delay_ms_async(400).await;
        countdown::run_once(color).await;
    }
}
