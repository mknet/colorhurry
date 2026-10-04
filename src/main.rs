//! Color Hurry — Moduswahl, Color Picker oder Empfänger-Countdown + BLE.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use panic_halt as _;

mod ble_broadcast;
mod buttons;
mod color_picker;
mod countdown;
mod debug_led;
mod game;
mod mode_select;
mod neopixel_pwm;
mod palette;
mod receiver;
mod speaker;
#[cfg(feature = "speaker-demo")]
mod speaker_demo;
mod time;

use nrf_softdevice::Softdevice;

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    embassy_futures::yield_now().await;
    ble_broadcast::mark_stack_ready();
    sd.run().await
}

#[embassy_executor::task]
async fn game_task(spawner: Spawner) -> ! {
    game::run(spawner).await
}

#[cfg(feature = "speaker-demo")]
#[embassy_executor::task]
async fn speaker_demo_task() -> ! {
    speaker_demo::run().await
}

#[embassy_executor::task]
async fn ble_system(spawner: Spawner) -> ! {
    embassy_futures::yield_now().await;
    embassy_futures::yield_now().await;

    let sd = ble_broadcast::enable();
    debug_led::signal_softdevice_ok_async().await;

    spawner.spawn(softdevice_task(sd)).unwrap();
    ble_broadcast::radio_loop(sd).await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    use nrf_softdevice as _;

    neopixel_pwm::power_on();
    neopixel_pwm::init();
    debug_led::init();
    debug_led::show_flash_layout();
    speaker::init();

    spawner.spawn(game_task(spawner)).unwrap();
    #[cfg(feature = "speaker-demo")]
    spawner.spawn(speaker_demo_task()).unwrap();
}
