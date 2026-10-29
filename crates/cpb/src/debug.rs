//! Debug checkpoints — stage in RAM + defmt (RTT) when a probe is connected.

use core::sync::atomic::{AtomicU32, Ordering};

static STAGE: AtomicU32 = AtomicU32::new(0);

pub fn set_stage_only(stage: u32) {
    STAGE.store(stage, Ordering::Relaxed);
}

/// Checkpoint with defmt (only after RTT init, not in pre_init).
pub fn checkpoint(stage: u32) {
    STAGE.store(stage, Ordering::Relaxed);
    // #region agent log
    defmt::info!(
        "cpb checkpoint stage={} layout={=u32}",
        stage,
        flash_layout_tag()
    );
    // #endregion
}

pub fn stage() -> u32 {
    STAGE.load(Ordering::Relaxed)
}

pub const fn flash_layout_tag() -> u32 {
    #[cfg(cpb_memory_v7)]
    {
        0x27000
    }
    #[cfg(not(cpb_memory_v7))]
    {
        0x26000
    }
}
