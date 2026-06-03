//! Adafruit-nRF52-Bootloader: UF2-Modus per GPREGRET (0x57)
//!
//! Bootloader manuell: Reset 2× (Mitte) oder Button A halten + Reset 1×.
//! Nicht beim normalen App-Start aufrufen — GPREGRET bleibt sonst gesetzt.

/// Nächster Reset startet den UF2-Bootloader (CPLAYBTBOOT).
#[allow(dead_code)]
pub fn enter_uf2() -> ! {
    nrf_pac::POWER.gpregret().write(|w| w.set_gpregret(0x57));
    cortex_m::peripheral::SCB::sys_reset();
}
