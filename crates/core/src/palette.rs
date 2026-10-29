use crate::color::{Color, Rgb};
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

const SPECTRUM_RGB: [Rgb; NUM_LEDS] = [
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

pub fn spectrum_rgb(color: Color) -> Rgb {
    SPECTRUM_RGB[color.index() as usize % NUM_LEDS]
}

pub fn spectrum_color_at(index: usize) -> Color {
    Color::from_index((index % NUM_LEDS) as u8).unwrap_or(Color::White)
}

pub fn spectrum_array() -> [Rgb; NUM_LEDS] {
    SPECTRUM_RGB
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

/// Map entropy byte to a named spectrum color.
pub fn color_from_entropy(byte: u8) -> Color {
    spectrum_color_at(byte as usize)
}
