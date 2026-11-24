//! LED brightness. The NeoPixels are very bright, so the board caps each channel.

use color_hurry_core::{Rgb, NUM_LEDS};

/// Max value per RGB channel after scaling (8 ≈ 3 % of 255).
pub const MAX_CHANNEL: u8 = 8;

/// Scale one channel from 0..=255 to 0..=[`MAX_CHANNEL`].
pub fn scale_channel(v: u8) -> u8 {
    ((v as u32 * MAX_CHANNEL as u32) / 255) as u8
}

/// One color, dimmed.
pub fn dim(c: Rgb) -> Rgb {
    Rgb {
        r: scale_channel(c.r),
        g: scale_channel(c.g),
        b: scale_channel(c.b),
    }
}

/// All ten LEDs, dimmed.
pub fn dim_all(leds: &[Rgb; NUM_LEDS]) -> [Rgb; NUM_LEDS] {
    leds.map(dim)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_and_off() {
        assert_eq!(scale_channel(255), MAX_CHANNEL);
        assert_eq!(scale_channel(0), 0);
    }

    #[test]
    fn dims_every_led() {
        let leds = [Rgb::from_rgb(255, 128, 0); NUM_LEDS];
        assert!(dim_all(&leds).iter().all(|c| *c == Rgb::from_rgb(8, 4, 0)));
    }
}
