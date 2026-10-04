//! 10-Sekunden-Countdown: LEDs CCW aus + absteigender Ton pro Sekunde (synchron).

use crate::neopixel_pwm::{Rgb, NUM_LEDS};
use crate::speaker;
use crate::time;

/// CPB: Pixel 0 links neben USB, Indizes steigen gegen den Uhrzeigersinn.
const CCW_OFF_ORDER: [usize; NUM_LEDS] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

const BEAT_MS: u32 = 1000;
const TONE_MS: u32 = 200;

const DESCENDING_HZ: [u32; NUM_LEDS] =
    [680, 620, 560, 500, 450, 400, 350, 300, 250, 200];

fn show_with_n_off(color: Rgb, num_off: usize) {
    let mut colors = [color; NUM_LEDS];
    for i in 0..num_off.min(NUM_LEDS) {
        colors[CCW_OFF_ORDER[i]] = Rgb::OFF;
    }
    crate::neopixel_pwm::show_pixels(&colors);
}

async fn play_beat(freq_hz: u32) {
    speaker::play_tone(freq_hz, TONE_MS, None).await;
    time::delay_ms_async(BEAT_MS.saturating_sub(TONE_MS)).await;
}

/// Alle LEDs in `color`, dann pro Sekunde eine LED aus + passender Ton (10 Schritte).
pub async fn run(color: Rgb) -> ! {
    speaker::init();
    loop {
        let _ = run_once(color, &mut || false).await;
    }
}

/// Countdown mit Abbruch-Prüfung zwischen den Schritten.
///
/// `should_abort` nach jedem LED-Schritt und Beat — z. B. BLE-Farb-Treffer.
/// Gibt `true` zurück, wenn abgebrochen (Treffer während Countdown).
pub async fn run_once<F>(color: Rgb, should_abort: &mut F) -> bool
where
    F: FnMut() -> bool,
{
    speaker::init();

    show_with_n_off(color, 0);
    if should_abort() {
        speaker::stop_tone();
        return true;
    }
    play_beat(DESCENDING_HZ[0]).await;
    if should_abort() {
        speaker::stop_tone();
        return true;
    }

    for off in 1..=NUM_LEDS {
        show_with_n_off(color, off);
        if should_abort() {
            speaker::stop_tone();
            return true;
        }
        play_beat(DESCENDING_HZ[off - 1]).await;
        if should_abort() {
            speaker::stop_tone();
            return true;
        }
    }

    false
}
