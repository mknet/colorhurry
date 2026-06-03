//! Button A (links): nächste Farbe. Button B (rechts): Farbe übernehmen.

use crate::neopixel_pwm::{Rgb, NUM_LEDS};

use crate::buttons::{Buttons, Event};
use crate::time;

const TICK_MS: u32 = 50;
const BLINK_TICKS: u32 = 6; // ~300 ms

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Spektrum sichtbar, `selected` blinkt.
    Selecting,
    /// Alle LEDs zeigen die gewählte Farbe.
    Applied,
}

pub struct ColorPicker {
    phase: Phase,
    spectrum: [Rgb; NUM_LEDS],
    selected: usize,
    applied: Rgb,
    blink_on: bool,
    blink_counter: u32,
    buttons: Buttons,
}

impl ColorPicker {
    pub fn new() -> Self {
        Self {
            phase: Phase::Selecting,
            spectrum: build_spectrum(),
            selected: 0,
            applied: Rgb::OFF,
            blink_on: true,
            blink_counter: 0,
            buttons: Buttons::new(),
        }
    }

    /// Endlosschleife: Farbauswahl + Übernahme (async Task neben BLE).
    pub async fn run(mut self) -> ! {
        Buttons::init();
        self.sync_ble();
        self.render();
        loop {
            self.tick();
            time::delay_ms_async(TICK_MS).await;
        }
    }

    fn tick(&mut self) {
        self.update_blink();

        match self.buttons.poll() {
            Event::Left => self.on_left(),
            Event::Right => self.on_right(),
            Event::None => {}
        }

        self.sync_ble();
        self.render();
    }

    /// Farbe der LED an `selected` — die blinkende Position in der Auswahl.
    /// Immer volle Palette-Werte (nicht „aus“ während der Blink-Pause).
    fn blinking_led_color(&self) -> Rgb {
        match self.phase {
            Phase::Selecting => self.spectrum[self.selected],
            Phase::Applied => self.applied,
        }
    }

    /// Ausgewählte bzw. übernommene Farbe per BLE mitsenden.
    fn sync_ble(&self) {
        crate::ble_broadcast::set_color(self.blinking_led_color());
    }

    fn on_left(&mut self) {
        match self.phase {
            Phase::Selecting => {
                self.selected = (self.selected + 1) % NUM_LEDS;
                // Neue Auswahl sofort sichtbar + BLE = Farbe dieser LED
                self.blink_on = true;
                self.blink_counter = 0;
            }
            Phase::Applied => {
                self.phase = Phase::Selecting;
            }
        }
    }

    fn on_right(&mut self) {
        match self.phase {
            Phase::Selecting => {
                self.applied = self.spectrum[self.selected];
                self.phase = Phase::Applied;
            }
            Phase::Applied => {
                self.phase = Phase::Selecting;
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
                let mut colors = self.spectrum;
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

/// Öffentlicher Einstieg — startet die Farbauswahl als Embassy-Task.
pub async fn run() -> ! {
    ColorPicker::new().run().await
}

/// Zehn klar unterscheidbare Farben (Helligkeit via `show_pixels`, max. Kanal 8).
fn build_spectrum() -> [Rgb; NUM_LEDS] {
    [
        Rgb::from_rgb(255, 0, 0),
        Rgb::from_rgb(255, 120, 0),
        Rgb::from_rgb(255, 220, 0),
        Rgb::from_rgb(160, 255, 0),
        Rgb::from_rgb(0, 255, 60),
        Rgb::from_rgb(0, 255, 255),
        Rgb::from_rgb(0, 100, 255),
        Rgb::from_rgb(120, 0, 255),
        Rgb::from_rgb(255, 0, 200),
        Rgb::from_rgb(255, 80, 120),
    ]
}
