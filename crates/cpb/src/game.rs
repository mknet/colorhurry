//! Game flow: choose mode → wait for BLE → core shell (picker/caller).
//!
//! `main` starts the Bluetooth stack at boot. The radio stays off until the core selects a
//! mode.

use crate::ble_broadcast;
use crate::shell::Shell;
use crate::speaker;

pub async fn run() -> ! {
    let mut shell = Shell::new();
    shell.run_mode_select().await;

    speaker::stop_tone();

    ble_broadcast::wait_until_ready().await;
    for _ in 0..16 {
        embassy_futures::yield_now().await;
    }

    shell.mark_ble_ready();
    shell.run_game().await
}
