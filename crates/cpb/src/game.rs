//! Spielablauf: Modus wählen → BLE starten → Core-Shell (Picker/Empfänger).

use embassy_executor::Spawner;

use crate::ble_broadcast;
use crate::shell::Shell;
use crate::speaker;

pub async fn run(spawner: Spawner) -> ! {
    let mut shell = Shell::new();
    shell.run_mode_select().await;

    speaker::stop_tone();

    spawner.spawn(crate::ble_system(spawner)).unwrap();
    ble_broadcast::wait_until_ready().await;
    for _ in 0..16 {
        embassy_futures::yield_now().await;
    }

    shell.mark_ble_ready();
    shell.run_game().await
}
