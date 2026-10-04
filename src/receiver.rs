//! Empfänger: Zufallsfarbe → sofort Countdown; Treffer währenddessen = nächste Farbe,
//! sonst Explosion → nächste Farbe.

use crate::ble_broadcast;
use crate::buttons::{Buttons, Event};
use crate::countdown;
use crate::neopixel_pwm::NUM_LEDS;
use crate::palette;
use crate::speaker;

pub async fn run(_channel: usize) -> ! {
    speaker::stop_tone();
    Buttons::init();
    let mut buttons = Buttons::new();

    loop {
        let color = palette::random_spectrum_color();
        ble_broadcast::set_radio_receiver(color);
        crate::neopixel_pwm::show_pixels(&[color; NUM_LEDS]);

        let matched_during = countdown::run_once(color, &mut || {
            buttons.poll() == Event::Left || ble_broadcast::take_color_match()
        })
        .await;

        if matched_during {
            speaker::stop_tone();
            continue;
        }

        speaker::play_explosion().await;
    }
}
