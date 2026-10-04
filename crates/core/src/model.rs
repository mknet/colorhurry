use crate::color::Rgb;

pub const NUM_LEDS: usize = 10;
pub const BLINK_TICKS: u32 = 6;
/// Countdown steps (= LEDs). Shell sends one `Tick` per second-ish beat.
pub const COUNTDOWN_STEPS: u8 = NUM_LEDS as u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Picker { channel: u8 },
    Receiver { channel: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
pub enum Event {
    /// UI / blink cadence (~50 ms on CPB).
    Tick,
    ButtonLeft,
    ButtonRight,
    /// Matching (or any) color advert received; core checks channel + RGB.
    BleColorReceived { channel: u8, color: Rgb },
    /// Fresh entropy for picking a random receiver target color.
    Entropy(u8),
}

#[derive(Clone, Debug)]
pub struct Model {
    pub screen: Screen,
    pub role: Option<Role>,
    /// Mode-select ring step 0..9
    pub mode_step: u8,
    pub blink_on: bool,
    pub blink_counter: u32,
    // Picker
    pub picker_selected: u8,
    pub picker_phase: PickerPhase,
    pub picker_applied: Rgb,
    // Receiver
    pub receiver_target: Rgb,
    pub countdown_step: u8,
    pub countdown_active: bool,
    pub awaiting_entropy: bool,
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
            picker_selected: 0,
            picker_phase: PickerPhase::Selecting,
            picker_applied: Rgb::OFF,
            receiver_target: Rgb::OFF,
            countdown_step: 0,
            countdown_active: false,
            awaiting_entropy: false,
        }
    }

    pub fn channel(&self) -> u8 {
        match self.role {
            Some(Role::Picker { channel }) | Some(Role::Receiver { channel }) => channel,
            None => 0,
        }
    }
}
