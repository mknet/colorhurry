//! Tone meaning and parameters (same everywhere; shell only plays).

use facet::Facet;
use serde::{Deserialize, Serialize};

/// One frequency/duration step for the shell PWM / WebAudio output.
#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToneStep {
    pub hz: u32,
    pub ms: u32,
}

pub const MAX_TONE_STEPS: usize = 12;

/// Fixed-capacity tone sequence carried in a [`crate::PlayTone`] effect.
#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToneSequence {
    steps: [ToneStep; MAX_TONE_STEPS],
    len: u8,
}

impl ToneSequence {
    pub const fn empty() -> Self {
        Self {
            steps: [ToneStep { hz: 0, ms: 0 }; MAX_TONE_STEPS],
            len: 0,
        }
    }

    pub const fn single(hz: u32, ms: u32) -> Self {
        let mut s = Self::empty();
        s.steps[0] = ToneStep { hz, ms };
        s.len = 1;
        s
    }

    pub fn from_slice(steps: &[ToneStep]) -> Self {
        let mut s = Self::empty();
        let n = steps.len().min(MAX_TONE_STEPS);
        s.steps[..n].copy_from_slice(&steps[..n]);
        s.len = n as u8;
        s
    }

    pub fn steps(&self) -> &[ToneStep] {
        &self.steps[..self.len as usize]
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

pub fn skip() -> ToneSequence {
    ToneSequence::single(380, 55)
}

pub fn apply() -> ToneSequence {
    ToneSequence::single(784, 130)
}

const COUNTDOWN_HZ: [u32; 10] = [680, 620, 560, 500, 450, 400, 350, 300, 250, 200];

pub fn countdown_step(step: u8) -> ToneSequence {
    let hz = COUNTDOWN_HZ[(step as usize).min(COUNTDOWN_HZ.len() - 1)];
    ToneSequence::single(hz, 200)
}

/// Same sweep the CPB used (~1 s): 180 Hz down to 45 Hz in 10 steps.
pub fn explosion() -> ToneSequence {
    const START_HZ: u32 = 180;
    const END_HZ: u32 = 45;
    const STEPS: u32 = 10;
    const STEP_MS: u32 = 100;
    let mut steps = [ToneStep { hz: 0, ms: 0 }; MAX_TONE_STEPS];
    let mut i = 0u32;
    while i < STEPS {
        let hz = START_HZ - (START_HZ - END_HZ) * i / (STEPS - 1);
        steps[i as usize] = ToneStep { hz, ms: STEP_MS };
        i += 1;
    }
    ToneSequence::from_slice(&steps[..STEPS as usize])
}
