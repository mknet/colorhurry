//! App time base: TIMER4 @ 1 MHz.
//!
//! MPSL and the SoftDevice Controller use TIMER0 and RTC0, and `embassy-time` uses RTC1.
//! TIMER4 is free for the app.
//!
//! MPSL owns the CLOCK peripheral, so this module does not start HFCLK. At boot, `main`
//! asks MPSL to keep the 16 MHz crystal (HFXO) on for the life of the firmware, so the
//! crystal clocks TIMER4 at 1 MHz. If MPSL does not start, there is no crystal request.
//! TIMER4 then counts at 1 MHz from the internal oscillator (HFINT), which is less accurate.

use nrf_pac::timer::vals::{Bitmode, Mode};

static mut READY: bool = false;

fn init() {
    unsafe {
        if READY {
            return;
        }
        READY = true;
    }

    let t = nrf_pac::TIMER4;
    t.tasks_stop().write_value(1);
    t.mode().write(|w| w.set_mode(Mode::Timer));
    t.bitmode().write(|w| w.set_bitmode(Bitmode::_32bit));
    t.prescaler().write(|w| w.set_prescaler(4)); // 16 MHz → 1 MHz
}

/// Blocks for `ms` milliseconds (hardware-timed).
///
/// This function blocks the executor, so the radio tasks do not run while it waits. Use it
/// only where no other task must run.
#[allow(dead_code)]
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

/// Async wait for UI/game/countdown (TIMER4).
///
/// TIMER4 has one compare channel for all callers, so only one task at a time can wait.
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
