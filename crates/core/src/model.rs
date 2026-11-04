use crux_time::TimerHandle;
use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::color::Color;

pub const NUM_LEDS: usize = 10;
pub const BLINK_TICKS: u32 = 6;
/// UI blink period, as a `crux_time` timer.
pub const BLINK_PERIOD_MS: u32 = 50;
/// Countdown beat period, as a `crux_time` timer.
pub const BEAT_PERIOD_MS: u32 = 1000;
/// Countdown steps (= LEDs).
pub const COUNTDOWN_STEPS: u8 = NUM_LEDS as u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Picker { channel: u8 },
    Caller { channel: u8 },
}

#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Screen {
    ModeSelect,
    Picker,
    Caller,
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

/// The game state. It is not `Clone`, because a `TimerHandle` is not.
#[derive(Debug)]
pub struct Model {
    pub(crate) screen: Screen,
    pub(crate) role: Option<Role>,
    pub(crate) mode_step: u8,
    pub(crate) blink_on: bool,
    pub(crate) blink_counter: u32,
    /// The pending blink tick, which the core clears when the blink stops.
    pub(crate) blink_timer: Option<TimerHandle>,
    pub(crate) picker_selected: u8,
    pub(crate) picker_phase: PickerPhase,
    pub(crate) picker_applied: Color,
    pub(crate) round: Round,
    /// The pending countdown beat or explosion wait. The two are never
    /// pending at the same time.
    pub(crate) round_timer: Option<TimerHandle>,
    /// True between `Pause` and `Resume`.
    pub(crate) paused: bool,
    /// True after `CallerOnly`. Mode select then offers only Caller choices.
    pub(crate) caller_only: bool,
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
            blink_timer: None,
            picker_selected: 0,
            picker_phase: PickerPhase::Selecting,
            picker_applied: Color::White,
            round: Round::Waiting,
            round_timer: None,
            paused: false,
            caller_only: false,
        }
    }

    pub(crate) fn channel(&self) -> u8 {
        match self.role {
            Some(Role::Picker { channel }) | Some(Role::Caller { channel }) => channel,
            None => 0,
        }
    }
}
