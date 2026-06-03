//! CCW-Ausschalt-Demo — für später (`demo_chase::run()`).

#![allow(dead_code)]

use crate::neopixel_pwm::{Rgb, NUM_LEDS};

use crate::time;

/// CPB: Pixel 0 links neben USB, Indizes steigen gegen den Uhrzeigersinn.
const CCW_OFF_ORDER: [usize; NUM_LEDS] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

fn show_with_n_off(num_off: usize) {
    let mut colors = [Rgb::ORANGE; NUM_LEDS];
    for i in 0..num_off.min(NUM_LEDS) {
        colors[CCW_OFF_ORDER[i]] = Rgb::OFF;
    }
    crate::neopixel_pwm::show_pixels(&colors);
}

/// Alle an, dann pro Sekunde eine LED aus (CCW), Endlosschleife.
pub fn run() -> ! {
    loop {
        show_with_n_off(0);
        time::delay_ms(1000);

        for off in 1..=NUM_LEDS {
            show_with_n_off(off);
            time::delay_ms(1000);
        }
    }
}
