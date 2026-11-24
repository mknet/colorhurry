//! Button A (left) / B (right), CPB: P1.02 / P1.15.
//!
//! The buttons connect to 3.3 V when pressed: **active high**, with pull-**down**.
//! The edge detection lives in the shared shell (`color_hurry_shell::Edges`).

use nrf_pac::gpio::vals::{Dir, Input, Pull};

const PIN_LEFT: usize = 2; // Button A = D4
const PIN_RIGHT: usize = 15; // Button B = D5

fn init_pin(pin: usize) {
    nrf_pac::P1.pin_cnf(pin).write(|w| {
        w.set_dir(Dir::Input);
        w.set_input(Input::Connect);
        w.set_pull(Pull::Pulldown);
    });
}

pub fn init() {
    init_pin(PIN_LEFT);
    init_pin(PIN_RIGHT);
}

/// Raw state of A and B: `true` while pressed.
pub fn raw() -> (bool, bool) {
    let port = nrf_pac::P1.in_().read();
    (port.pin(PIN_LEFT), port.pin(PIN_RIGHT))
}
