//! Spielablauf: Modus wählen → Picker oder Empfänger.

use crate::color_picker;
use crate::mode_select::{self, Role};
use crate::receiver;

pub async fn run() -> ! {
    match mode_select::run().await {
        Role::Picker { channel } => color_picker::run(channel).await,
        Role::Receiver { channel } => receiver::run(channel).await,
    }
}
