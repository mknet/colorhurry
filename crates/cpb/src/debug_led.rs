//! D13 (P1.14, active high) — BLE-/Layout-Diagnose ohne Serial.

use nrf_pac::gpio::vals::{Dir, Input, Pull};

pub fn init() {
    nrf_pac::P1.pin_cnf(14).write(|w| {
        w.set_dir(Dir::OUTPUT);
        w.set_input(Input::DISCONNECT);
        w.set_pull(Pull::DISABLED);
    });
    off();
}

fn on() {
    nrf_pac::P1.outset().write_value(nrf_pac::gpio::regs::Outset(1 << 14));
}

fn off() {
    nrf_pac::P1.outclr().write_value(nrf_pac::gpio::regs::Outclr(1 << 14));
}

fn delay_ms(ms: u32) {
    crate::time::delay_ms(ms);
}

/// Kurze Blink-Sequenz (Anzahl = Info-Code).
pub fn blink(count: u32) {
    for _ in 0..count {
        on();
        delay_ms(120);
        off();
        delay_ms(120);
    }
}

/// Beim Start: 2× = Firmware v6 (0x26000), 4× = v7 (0x27000, BLE).
pub fn show_flash_layout() {
    #[cfg(cpb_memory_v7)]
    blink(4);
    #[cfg(not(cpb_memory_v7))]
    blink(2);
    delay_ms(300);
}

/// SoftDevice aktiv — 1 langer Blink (blockiert den Executor — nur vor Task-Start).
pub fn signal_softdevice_ok() {
    on();
    delay_ms(500);
    off();
}

/// Wie oben, ohne andere Embassy-Tasks zu blockieren.
pub async fn signal_softdevice_ok_async() {
    on();
    crate::time::delay_ms_async(500).await;
    off();
}

/// Advertising-Fehler — schnelles Stroboskop.
pub fn signal_advertising_error() {
    for _ in 0..8 {
        on();
        delay_ms(50);
        off();
        delay_ms(50);
    }
}
