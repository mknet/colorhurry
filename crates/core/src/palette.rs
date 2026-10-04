use crate::color::Rgb;
use crate::model::NUM_LEDS;

pub const CHANNELS: usize = 5;
pub const SPECTRUM_LEN: usize = NUM_LEDS;

const CHANNEL_COLORS: [Rgb; CHANNELS] = [
    Rgb::from_rgb(255, 0, 0),
    Rgb::from_rgb(255, 120, 0),
    Rgb::from_rgb(255, 220, 0),
    Rgb::from_rgb(0, 255, 255),
    Rgb::from_rgb(200, 0, 255),
];

const SPECTRUM: [Rgb; NUM_LEDS] = [
    Rgb::from_rgb(255, 255, 255),
    Rgb::from_rgb(255, 0, 0),
    Rgb::from_rgb(255, 120, 0),
    Rgb::from_rgb(255, 255, 0),
    Rgb::from_rgb(140, 255, 100),
    Rgb::from_rgb(0, 100, 0),
    Rgb::from_rgb(120, 200, 255),
    Rgb::from_rgb(0, 0, 160),
    Rgb::from_rgb(180, 0, 255),
    Rgb::from_rgb(255, 100, 180),
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

/// Map entropy byte to a spectrum color (shell supplies entropy).
pub fn color_from_entropy(byte: u8) -> Rgb {
    spectrum_color(byte as usize % NUM_LEDS)
}
