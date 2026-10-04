//! 12-Sekunden-Demo (Reset → Tonleiter → Explosion) — standardmäßig aus.
//!
//! Aktivieren: Feature `speaker-demo` in `Cargo.toml` und `speaker_task` in `main.rs`.

use crate::speaker;
use crate::time;

const BEAT_MS: u32 = 1000;
const TONE_MS: u32 = 200;

const RESET_STRUM: [(u32, u32); 3] = [(196, 55), (247, 55), (330, 85)];
const RESET_GAP_MS: u32 = 12;

const DESCENDING_STEPS: usize = 10;
const DESCENDING_HZ: [u32; DESCENDING_STEPS] =
    [680, 620, 560, 500, 450, 400, 350, 300, 250, 200];

pub async fn run() -> ! {
    speaker::init();
    loop {
        play_reset().await;
        for i in 0..DESCENDING_STEPS {
            play_beat(DESCENDING_HZ[i]).await;
        }
        speaker::play_explosion().await;
    }
}

async fn play_reset() {
    let mut elapsed = 0u32;
    for (i, &(freq, note_ms)) in RESET_STRUM.iter().enumerate() {
        speaker::play_tone(freq, note_ms, None).await;
        elapsed += note_ms;
        if i + 1 < RESET_STRUM.len() {
            time::delay_ms_async(RESET_GAP_MS).await;
            elapsed += RESET_GAP_MS;
        }
    }
    time::delay_ms_async(BEAT_MS.saturating_sub(elapsed)).await;
}

async fn play_beat(freq_hz: u32) {
    speaker::play_tone(freq_hz, TONE_MS, None).await;
    time::delay_ms_async(BEAT_MS.saturating_sub(TONE_MS)).await;
}

