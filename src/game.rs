//! Spielablauf: Modus wählen → BLE starten → Picker oder Empfänger.

use embassy_executor::Spawner;

use crate::ble_broadcast;
use crate::color_picker;
use crate::mode_select::{self, Role};
use crate::receiver;
use crate::speaker;

pub async fn run(spawner: Spawner) -> ! {
    let role = mode_select::run().await;

    speaker::stop_tone();

    spawner.spawn(crate::ble_system(spawner)).unwrap();
    ble_broadcast::wait_until_ready().await;
    for _ in 0..16 {
        embassy_futures::yield_now().await;
    }

    match role {
        Role::Picker { channel } => {
            ble_broadcast::set_radio_picker();
            color_picker::run(channel).await
        }
        Role::Receiver { channel } => receiver::run(channel).await,
    }
}
