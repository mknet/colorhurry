//! Button A: nächste Farbe (10 LEDs). Button B: übernehmen → alle LEDs + BLE.

use crate::buttons::{Buttons, Event};
use crate::neopixel_pwm::{Rgb, NUM_LEDS};
use crate::palette;
use crate::time;

const TICK_MS: u32 = 50;
const BLINK_TICKS: u32 = 6;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Selecting,
    Applied,
}

pub struct ColorPicker {
    phase: Phase,
    selected: usize,
    applied: Rgb,
    blink_on: bool,
    blink_counter: u32,
    buttons: Buttons,
}

impl ColorPicker {
    pub fn new(channel: usize) -> Self {
        Self {
            phase: Phase::Selecting,
            selected: channel.min(NUM_LEDS - 1),
            applied: Rgb::OFF,
            blink_on: true,
            blink_counter: 0,
            buttons: Buttons::new(),
        }
    }

    pub async fn run(mut self) -> ! {
        Buttons::init();
        crate::speaker::init();
        self.sync_ble();
        self.render();
        loop {
            self.update_blink();

            match self.buttons.poll() {
                Event::Left => {
                    if self.on_left() {
                        crate::speaker::play_skip().await;
                    }
                }
                Event::Right => {
                    if self.on_right() {
                        crate::speaker::play_apply().await;
                    }
                }
                Event::None => {}
            }

            self.sync_ble();
            self.render();
            time::delay_ms_async(TICK_MS).await;
        }
    }

    fn blinking_led_color(&self) -> Rgb {
        match self.phase {
            Phase::Selecting => palette::spectrum_color(self.selected),
            Phase::Applied => self.applied,
        }
    }

    fn sync_ble(&self) {
        crate::ble_broadcast::set_color(self.blinking_led_color());
    }

    fn on_left(&mut self) -> bool {
        match self.phase {
            Phase::Selecting => {
                self.selected = (self.selected + 1) % NUM_LEDS;
                self.blink_on = false;
                self.blink_counter = 0;
                true
            }
            Phase::Applied => {
                self.phase = Phase::Selecting;
                false
            }
        }
    }

    fn on_right(&mut self) -> bool {
        match self.phase {
            Phase::Selecting => {
                self.applied = palette::spectrum_color(self.selected);
                self.phase = Phase::Applied;
                true
            }
            Phase::Applied => {
                self.phase = Phase::Selecting;
                false
            }
        }
    }

    fn update_blink(&mut self) {
        if self.phase != Phase::Selecting {
            return;
        }
        self.blink_counter += 1;
        if self.blink_counter >= BLINK_TICKS {
            self.blink_counter = 0;
            self.blink_on = !self.blink_on;
        }
    }

    fn render(&self) {
        match self.phase {
            Phase::Selecting => {
                let mut colors = palette::spectrum_array();
                if !self.blink_on {
                    colors[self.selected] = Rgb::OFF;
                }
                crate::neopixel_pwm::show_pixels(&colors);
            }
            Phase::Applied => {
                crate::neopixel_pwm::show_pixels(&[self.applied; NUM_LEDS]);
            }
        }
    }
}

pub async fn run(channel: usize) -> ! {
    ColorPicker::new(channel).run().await
}
