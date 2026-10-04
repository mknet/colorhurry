//! App-Zeitbasis: TIMER4 @ 1 MHz.
//!
//! Mit S140 sind TIMER2/TIMER3 vom SoftDevice belegt — TIMER4 ist für die App frei.

use nrf_pac::timer::vals::{Bitmode, Mode};

static mut READY: bool = false;

fn init() {
    unsafe {
        if READY {
            return;
        }
        READY = true;
    }

    nrf_pac::CLOCK.tasks_hfclkstart().write_value(1);
    while nrf_pac::CLOCK.events_hfclkstarted().read() == 0 {
        core::hint::spin_loop();
    }

    let t = nrf_pac::TIMER4;
    t.tasks_stop().write_value(1);
    t.mode().write(|w| w.set_mode(Mode::TIMER));
    t.bitmode().write(|w| w.set_bitmode(Bitmode::_32BIT));
    t.prescaler().write(|w| w.set_prescaler(4)); // 16 MHz → 1 MHz
}

/// Blockiert für `ms` Millisekunden (hardware-getaktet).
///
/// **Nur vor SoftDevice-Start** oder aus ISR-freien Kontexten ohne laufenden SD-Stack.
pub fn delay_ms(ms: u32) {
    if ms == 0 {
        return;
    }
    init();

    let t = nrf_pac::TIMER4;
    t.tasks_clear().write_value(1);
    t.events_compare(0).write_value(0);
    t.cc(0).write_value(u32::from(ms.saturating_mul(1000)));
    t.tasks_start().write_value(1);
    while t.events_compare(0).read() == 0 {
        core::hint::spin_loop();
    }
    t.tasks_stop().write_value(1);
}

/// Async-Wartezeit für UI/Spiel/Countdown (TIMER4 — SD-kompatibel).
pub async fn delay_ms_async(ms: u32) {
    if ms == 0 {
        return;
    }
    init();

    let t = nrf_pac::TIMER4;
    t.tasks_clear().write_value(1);
    t.events_compare(0).write_value(0);
    t.cc(0).write_value(u32::from(ms.saturating_mul(1000)));
    t.tasks_start().write_value(1);
    while t.events_compare(0).read() == 0 {
        embassy_futures::yield_now().await;
    }
    t.tasks_stop().write_value(1);
}
