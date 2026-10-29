use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::color::Color;

pub const NUM_LEDS: usize = 10;
pub const BLINK_TICKS: u32 = 6;
/// UI blink period requested via [`crate::Delay`].
pub const BLINK_PERIOD_MS: u32 = 50;
/// Countdown beat period requested via [`crate::Delay`].
pub const BEAT_PERIOD_MS: u32 = 1000;
/// Countdown steps (= LEDs).
pub const COUNTDOWN_STEPS: u8 = NUM_LEDS as u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Picker { channel: u8 },
    Receiver { channel: u8 },
}

#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Screen {
    ModeSelect,
    Picker,
    Receiver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerPhase {
    Selecting,
    Applied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Round {
    Waiting,
    Running { target: Color, step: u8 },
}

#[derive(Clone, Debug)]
pub struct Model {
    pub(crate) screen: Screen,
    pub(crate) role: Option<Role>,
    pub(crate) mode_step: u8,
    pub(crate) blink_on: bool,
    pub(crate) blink_counter: u32,
    /// Generation so stale blink delays are ignored.
    pub(crate) blink_gen: u32,
    pub(crate) picker_selected: u8,
    pub(crate) picker_phase: PickerPhase,
    pub(crate) picker_applied: Color,
    pub(crate) round: Round,
    /// Generation so stale countdown delays are ignored.
    pub(crate) beat_gen: u32,
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}

impl Model {
    pub fn new() -> Self {
        Self {
            screen: Screen::ModeSelect,
            role: None,
            mode_step: 0,
            blink_on: true,
            blink_counter: 0,
            blink_gen: 0,
            picker_selected: 0,
            picker_phase: PickerPhase::Selecting,
            picker_applied: Color::White,
            round: Round::Waiting,
            beat_gen: 0,
        }
    }

    pub(crate) fn channel(&self) -> u8 {
        match self.role {
            Some(Role::Picker { channel }) | Some(Role::Receiver { channel }) => channel,
            None => 0,
        }
    }
}
