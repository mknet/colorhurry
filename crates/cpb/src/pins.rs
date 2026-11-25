//! The pins of the board: Circuit Playground Bluefruit (default) or Seeed XIAO nRF52840
//! (feature `xiao`).
//!
//! Both boards have the nRF52840, so only the pins differ. The radio, the time base and the
//! memory layout are the same.

use nrf_pac::gpio::Gpio;

/// One GPIO pin: port P0 or P1, and the pin number on that port.
#[derive(Clone, Copy)]
pub struct Pin {
    pub port1: bool,
    pub pin: usize,
}

impl Pin {
    const fn p0(pin: usize) -> Self {
        Self { port1: false, pin }
    }

    #[allow(dead_code)]
    const fn p1(pin: usize) -> Self {
        Self { port1: true, pin }
    }

    pub fn gpio(self) -> Gpio {
        if self.port1 {
            nrf_pac::P1
        } else {
            nrf_pac::P0
        }
    }

    /// Drive the pin high.
    pub fn set_high(self) {
        self.gpio().outset().write_value(nrf_pac::gpio::regs::Outset(1 << self.pin));
    }

    /// Drive the pin low.
    pub fn set_low(self) {
        self.gpio().outclr().write_value(nrf_pac::gpio::regs::Outclr(1 << self.pin));
    }

    /// Read the input level.
    pub fn is_high(self) -> bool {
        self.gpio().in_().read().pin(self.pin)
    }
}

#[cfg(not(feature = "xiao"))]
mod board_pins {
    use super::Pin;

    /// The ten NeoPixels of the ring (D8).
    pub const NEOPIXEL: Pin = Pin::p0(13);
    /// NeoPixel power (D35), low = on.
    pub const NEOPIXEL_POWER: Option<Pin> = Some(Pin::p0(6));
    /// Button A (left, D4) and button B (right, D5).
    pub const BUTTON_A: Pin = Pin::p1(2);
    pub const BUTTON_B: Pin = Pin::p1(15);
    /// The buttons connect to 3.3 V when pressed: active high, with pull-down.
    pub const BUTTONS_ACTIVE_LOW: bool = false;
    /// The red LED next to USB (D13), high = on.
    pub const DEBUG_LED: Pin = Pin::p1(14);
    pub const DEBUG_LED_ACTIVE_LOW: bool = false;
}

#[cfg(feature = "xiao")]
mod board_pins {
    use super::Pin;

    /// Grove RGB LED Stick (10 × WS2813) on D0.
    pub const NEOPIXEL: Pin = Pin::p0(2);
    /// The stick takes its power from the Grove port, there is no switch.
    pub const NEOPIXEL_POWER: Option<Pin> = None;
    /// Grove Dual Button: yellow wire (button 1 → A) on D1, white wire (button 2 → B) on D2.
    pub const BUTTON_A: Pin = Pin::p0(3);
    pub const BUTTON_B: Pin = Pin::p0(28);
    /// A pressed button pulls its line to GND: active low, with pull-up.
    pub const BUTTONS_ACTIVE_LOW: bool = true;
    /// The red part of the on-board RGB LED, low = on.
    pub const DEBUG_LED: Pin = Pin::p0(26);
    pub const DEBUG_LED_ACTIVE_LOW: bool = true;
}

pub use board_pins::*;
