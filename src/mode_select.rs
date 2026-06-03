//! Setup: 5 Kanalfarben links (blink), rechts Radio-LED; getrennt vom 10er-Farbspektrum.

use crate::buttons::{Buttons, Event};
use crate::neopixel_pwm::{Rgb, NUM_LEDS};
use crate::palette::{self, CHANNELS};
use crate::time;

const TICK_MS: u32 = 50;
const BLINK_TICKS: u32 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Picker { channel: usize },
    Receiver { channel: usize },
}

pub struct ModeSelect {
    step: usize,
    blink_on: bool,
    blink_counter: u32,
    buttons: Buttons,
}

impl ModeSelect {
    pub fn new() -> Self {
        Self {
            step: 0,
            blink_on: true,
            blink_counter: 0,
            buttons: Buttons::new(),
        }
    }

    pub async fn run(mut self) -> Role {
        Buttons::init();
        crate::speaker::init();
        self.render();

        loop {
            self.update_blink();

            match self.buttons.poll() {
                Event::Left => {
                    self.step = (self.step + 1) % NUM_LEDS;
                    self.blink_on = false;
                    self.blink_counter = 0;
                    crate::speaker::play_skip().await;
                }
                Event::Right => {
                    crate::speaker::play_apply().await;
                    return self.role_for_step();
                }
                Event::None => {}
            }

            self.render();
            time::delay_ms_async(TICK_MS).await;
        }
    }

    fn role_for_step(&self) -> Role {
        let ch = palette::ring_step_channel(self.step);
        if palette::ring_step_is_picker(self.step) {
            Role::Picker { channel: ch }
        } else {
            Role::Receiver { channel: ch }
        }
    }

    fn update_blink(&mut self) {
        self.blink_counter += 1;
        if self.blink_counter >= BLINK_TICKS {
            self.blink_counter = 0;
            self.blink_on = !self.blink_on;
        }
    }

    fn render(&self) {
        let mut colors = [Rgb::OFF; NUM_LEDS];
        let led = palette::ring_step_led(self.step);
        let ch = palette::ring_step_channel(self.step);

        if palette::ring_step_is_picker(self.step) {
            for i in 0..CHANNELS {
                colors[i] = palette::channel_color(i);
            }
            if !self.blink_on {
                colors[led] = Rgb::OFF;
            }
        } else {
            for i in 0..CHANNELS {
                colors[i] = palette::channel_color(i);
            }
            if self.blink_on {
                colors[led] = palette::channel_color(ch);
            }
        }

        crate::neopixel_pwm::show_pixels(&colors);
    }
}

pub async fn run() -> Role {
    ModeSelect::new().run().await
}
