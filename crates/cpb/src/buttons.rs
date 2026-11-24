//! Button A (left) / B (right), pins in [`crate::pins`].
//!
//! CPB: the buttons connect to 3.3 V when pressed (active high, pull-down).
//! XIAO with the Grove Dual Button: a pressed button pulls its line to GND (active low,
//! pull-up). The edge detection lives in the shared shell (`color_hurry_shell::Edges`).

use nrf_pac::gpio::vals::{Dir, Input, Pull};

use crate::pins::{self, Pin};

fn init_pin(pin: Pin) {
    pin.gpio().pin_cnf(pin.pin).write(|w| {
        w.set_dir(Dir::Input);
        w.set_input(Input::Connect);
        w.set_pull(if pins::BUTTONS_ACTIVE_LOW {
            Pull::Pullup
        } else {
            Pull::Pulldown
        });
    });
}

pub fn init() {
    init_pin(pins::BUTTON_A);
    init_pin(pins::BUTTON_B);
}

fn pressed(pin: Pin) -> bool {
    pin.is_high() != pins::BUTTONS_ACTIVE_LOW
}

/// Raw state of A and B: `true` while pressed.
pub fn raw() -> (bool, bool) {
    (pressed(pins::BUTTON_A), pressed(pins::BUTTON_B))
}
