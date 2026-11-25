//! Debug LED (CPB: D13, XIAO: red on-board LED, see [`crate::pins`]): BLE/layout
//! diagnostics without serial.
//!
//! The waits use `embassy-time` (RTC1), not the TIMER4 time base in `time.rs`. A blink
//! therefore never disturbs a game delay, and it never blocks the radio tasks.

use embassy_time::Timer;
use nrf_pac::gpio::vals::{Dir, Input, Pull};

use crate::pins;

pub fn init() {
    let pin = pins::DEBUG_LED;
    pin.gpio().pin_cnf(pin.pin).write(|w| {
        w.set_dir(Dir::Output);
        w.set_input(Input::Disconnect);
        w.set_pull(Pull::Disabled);
    });
    off();
}

fn on() {
    if pins::DEBUG_LED_ACTIVE_LOW {
        pins::DEBUG_LED.set_low();
    } else {
        pins::DEBUG_LED.set_high();
    }
}

fn off() {
    if pins::DEBUG_LED_ACTIVE_LOW {
        pins::DEBUG_LED.set_high();
    } else {
        pins::DEBUG_LED.set_low();
    }
}

async fn delay_ms(ms: u64) {
    Timer::after_millis(ms).await;
}

/// Short blink sequence (count = info code).
pub async fn blink(count: u32) {
    for _ in 0..count {
        on();
        delay_ms(120).await;
        off();
        delay_ms(120).await;
    }
}

/// At start: 2× = firmware v6 (0x26000), 4× = v7 (0x27000).
pub async fn show_flash_layout() {
    #[cfg(cpb_memory_v7)]
    blink(4).await;
    #[cfg(not(cpb_memory_v7))]
    blink(2).await;
    delay_ms(300).await;
}

/// Bluetooth stack ready: 1 long blink.
pub async fn signal_radio_ok() {
    on();
    delay_ms(500).await;
    off();
}

/// Bluetooth error (advertising, scanning or set-up): fast strobe.
pub async fn signal_radio_error() {
    for _ in 0..8 {
        on();
        delay_ms(50).await;
        off();
        delay_ms(50).await;
    }
}
