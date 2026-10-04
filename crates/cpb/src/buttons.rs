//! Button A (links) / B (rechts) — CPB: P1.02 / P1.15.
//!
//! Adafruit CPB: Tasten liegen bei Druck an 3,3 V → **active high**, Pull-**down**.

use nrf_pac::gpio::vals::{Dir, Input, Pull};

const PIN_LEFT: usize = 2; // Button A = D4
const PIN_RIGHT: usize = 15; // Button B = D5

fn init_pin(pin: usize) {
    nrf_pac::P1.pin_cnf(pin).write(|w| {
        w.set_dir(Dir::INPUT);
        w.set_input(Input::CONNECT);
        w.set_pull(Pull::PULLDOWN);
    });
}

fn raw_pressed(pin: usize) -> bool {
    nrf_pac::P1.in_().read().pin(pin)
}

pub struct Buttons {
    left_down: bool,
    right_down: bool,
}

#[derive(PartialEq, Eq)]
pub enum Event {
    None,
    Left,
    Right,
}

impl Buttons {
    pub fn init() {
        init_pin(PIN_LEFT);
        init_pin(PIN_RIGHT);
    }

    pub fn new() -> Self {
        Self {
            left_down: false,
            right_down: false,
        }
    }

    /// Flanken-Erkennung (ein Event pro Tastendruck).
    pub fn poll(&mut self) -> Event {
        let left = raw_pressed(PIN_LEFT);
        let right = raw_pressed(PIN_RIGHT);

        let ev = if left && !self.left_down {
            Event::Left
        } else if right && !self.right_down {
            Event::Right
        } else {
            Event::None
        };

        self.left_down = left;
        self.right_down = right;
        ev
    }
}
