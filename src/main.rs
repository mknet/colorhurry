//! Color Hurry CPB — Color Picker + BLE-Broadcast.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use panic_halt as _;

mod ble_broadcast;
mod buttons;
mod color_picker;
mod debug_led;
mod demo_chase;
mod neopixel_pwm;
mod time;

use nrf_softdevice::Softdevice;

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[embassy_executor::task]
async fn color_picker_task() -> ! {
    color_picker::run().await
}

/// SoftDevice + Advertising (Color Picker läuft bereits parallel).
#[embassy_executor::task]
async fn ble_system(spawner: Spawner) -> ! {
    // Color Picker soll HFCLK/Timer zuerst nutzen können.
    embassy_futures::yield_now().await;
    embassy_futures::yield_now().await;

    let sd = ble_broadcast::enable();
    debug_led::signal_softdevice_ok();

    spawner.spawn(softdevice_task(sd)).unwrap();
    ble_broadcast::advertise_loop(sd).await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    use nrf_softdevice as _;

    neopixel_pwm::power_on();
    neopixel_pwm::init();
    debug_led::init();
    debug_led::show_flash_layout();

    // UI zuerst — läuft auch wenn BLE scheitert (Panic in enable ausgenommen).
    spawner.spawn(color_picker_task()).unwrap();
    spawner.spawn(ble_system(spawner)).unwrap();
}
