//! WS2812 NeoPixels (10×) on P0.13, Circuit Playground Bluefruit

use embassy_nrf::gpio::{OutputDrive, Pin as GpioPin};
use embassy_nrf::peripherals::PWM0;
use embassy_nrf::pwm::{
    Config, Prescaler, SequenceConfig, SequenceLoad, SequencePwm, SingleSequenceMode, SingleSequencer,
};
use embassy_nrf::Peri;
use embassy_time::Timer;

/// Number of NeoPixels around the board edge
pub const NUM_LEDS: usize = 10;

/// RGB color (sent as GRB to WS2812)
#[derive(Clone, Copy, Debug, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const OFF: Self = Self { r: 0, g: 0, b: 0 };
    pub const CYAN: Self = Self {
        r: 0,
        g: 48,
        b: 48,
    };
}

const SEQ_LEN: usize = NUM_LEDS * 24 + 1;

// WS2812: inverted polarity (high bit set), 1.25 µs bit time @ 16 MHz / 20 ticks
const T1H: u16 = 0x8000 | 13;
const T0H: u16 = 0x8000 | 7;
const RES: u16 = 0x8000;

/// NeoPixel driver via PWM sequencer
pub struct NeoPixel<'d> {
    pwm: SequencePwm<'d>,
    seq_words: [u16; SEQ_LEN],
    seq_config: SequenceConfig,
}

impl<'d> NeoPixel<'d> {
    /// NeoPixel data line: P0.13 (D8)
    pub fn new(pwm0: Peri<'d, PWM0>, pin: Peri<'d, impl GpioPin>) -> Self {
        let mut config = Config::default();
        config.sequence_load = SequenceLoad::Common;
        config.prescaler = Prescaler::Div1;
        config.max_duty = 20;
        config.ch0_drive = OutputDrive::HighDrive;
        let pwm = SequencePwm::new_1ch(pwm0, pin, config).unwrap();

        let mut seq_config = SequenceConfig::default();
        // ~50 µs reset after the sequence (as in the Embassy example)
        seq_config.end_delay = 799;

        let mut this = Self {
            pwm,
            seq_words: [RES; SEQ_LEN],
            seq_config,
        };
        this.set_colors(&[Rgb::OFF; NUM_LEDS]);
        this
    }

    /// Set all 10 LEDs (do not send yet; call `show`)
    pub fn set_colors(&mut self, colors: &[Rgb; NUM_LEDS]) {
        let mut i = 0;
        for c in colors {
            push_color(&mut self.seq_words, &mut i, *c);
        }
        self.seq_words[i] = RES;
    }

    /// One color on all LEDs
    pub fn fill(&mut self, color: Rgb) {
        self.set_colors(&[color; NUM_LEDS]);
    }

    pub fn clear(&mut self) {
        self.fill(Rgb::OFF);
    }

    /// Send color data to the LEDs
    pub async fn show(&mut self) {
        let seq = SingleSequencer::new(&mut self.pwm, &self.seq_words, self.seq_config.clone());
        seq.start(SingleSequenceMode::Times(1)).unwrap();
        // WS2812 needs time for the full sequence + reset (~400 µs for 10 LEDs)
        Timer::after_millis(2).await;
    }
}

fn push_color(seq: &mut [u16], idx: &mut usize, c: Rgb) {
    push_byte(seq, idx, c.g);
    push_byte(seq, idx, c.r);
    push_byte(seq, idx, c.b);
}

fn push_byte(seq: &mut [u16], idx: &mut usize, byte: u8) {
    for bit in (0..8).rev() {
        seq[*idx] = if byte & (1 << bit) != 0 { T1H } else { T0H };
        *idx += 1;
    }
}
