//! Zwei Paletten: 5 Kanalfarben (Modus-Setup) + 10 Spektrumfarben (Picker / Zufall).

use crate::neopixel_pwm::{Rgb, NUM_LEDS};

/// Kanäle für Moduswahl (links 0–4, rechts 5–9 am Ring).
pub const CHANNELS: usize = 5;

const CHANNEL_COLORS: [Rgb; CHANNELS] = [
    Rgb::from_rgb(255, 0, 0),
    Rgb::from_rgb(255, 120, 0),
    Rgb::from_rgb(255, 220, 0),
    Rgb::from_rgb(0, 255, 255),
    Rgb::from_rgb(200, 0, 255),
];

/// Farbauswahl Picker + Zufallstopf Empfänger (eine Farbe pro LED).
pub const SPECTRUM: [Rgb; NUM_LEDS] = [
    Rgb::from_rgb(255, 255, 255), // weiß
    Rgb::from_rgb(255, 0, 0),     // rot
    Rgb::from_rgb(255, 120, 0),   // orange
    Rgb::from_rgb(255, 255, 0),   // gelb
    Rgb::from_rgb(140, 255, 100), // hellgrün
    Rgb::from_rgb(0, 100, 0),     // dunkelgrün
    Rgb::from_rgb(120, 200, 255), // hellblau
    Rgb::from_rgb(0, 0, 160),     // dunkelblau
    Rgb::from_rgb(180, 0, 255),   // lila
    Rgb::from_rgb(255, 100, 180), // rosa
];

pub fn channel_color(i: usize) -> Rgb {
    CHANNEL_COLORS[i.min(CHANNELS - 1)]
}

pub fn spectrum_color(index: usize) -> Rgb {
    SPECTRUM[index % NUM_LEDS]
}

pub fn spectrum_array() -> [Rgb; NUM_LEDS] {
    SPECTRUM
}

/// Ring-Schritt 0..9 (CCW): links Kanal 0→4, rechts LED 5→9 mit Kanal 4→0.
pub fn ring_step_led(step: usize) -> usize {
    step % NUM_LEDS
}

pub fn ring_step_channel(step: usize) -> usize {
    let step = step % NUM_LEDS;
    if step < CHANNELS {
        step
    } else {
        NUM_LEDS - 1 - step
    }
}

pub fn ring_step_is_picker(step: usize) -> bool {
    ring_step_led(step) < CHANNELS
}

static mut PRNG_STATE: u32 = 0xA5A5_1234;

fn prng_byte() -> u8 {
    unsafe {
        PRNG_STATE = PRNG_STATE
            .wrapping_mul(1_103_515_245)
            .wrapping_add(12_345);
        (PRNG_STATE >> 16) as u8
    }
}

pub fn random_spectrum_color() -> Rgb {
    let idx = (prng_byte() as usize) % NUM_LEDS;
    spectrum_color(idx)
}
