//! Adafruit nRF52 bootloader: UF2 mode via GPREGRET (0x57)
//!
//! Manual bootloader: reset 2× (center) or hold button A + reset once.
//! Do not call on normal app start; GPREGRET would stay set.

/// Next reset starts the UF2 bootloader (CPLAYBTBOOT).
#[allow(dead_code)]
pub fn enter_uf2() -> ! {
    nrf_pac::POWER.gpregret().write(|w| w.set_gpregret(0x57));
    cortex_m::peripheral::SCB::sys_reset();
}
