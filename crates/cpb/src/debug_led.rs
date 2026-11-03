//! D13 (P1.14, active high): BLE/layout diagnostics without serial.
//!
//! The waits use `embassy-time` (RTC1), not the TIMER4 time base in `time.rs`. A blink
//! therefore never disturbs a game delay, and it never blocks the radio tasks.

use embassy_time::Timer;
use nrf_pac::gpio::vals::{Dir, Input, Pull};

pub fn init() {
    nrf_pac::P1.pin_cnf(14).write(|w| {
        w.set_dir(Dir::Output);
        w.set_input(Input::Disconnect);
        w.set_pull(Pull::Disabled);
    });
    off();
}

fn on() {
    nrf_pac::P1.outset().write_value(nrf_pac::gpio::regs::Outset(1 << 14));
}

fn off() {
    nrf_pac::P1.outclr().write_value(nrf_pac::gpio::regs::Outclr(1 << 14));
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
