use facet::Facet;
use serde::{Deserialize, Serialize};

use crate::color::{Color, Rgb};
use crate::model::{Model, PickerPhase, Round, Screen, NUM_LEDS};
use crate::palette::{self, CHANNELS};

#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewModel {
    pub screen: Screen,
    pub leds: [Rgb; NUM_LEDS],
    pub channel: u8,
    pub countdown_step: u8,
    pub countdown_active: bool,
}

pub fn view_model(model: &Model) -> ViewModel {
    let leds = match model.screen {
        Screen::ModeSelect => mode_select_leds(model),
        Screen::Picker => picker_leds(model),
        Screen::Receiver => receiver_leds(model),
    };

    let (countdown_step, countdown_active) = match model.round {
        Round::Waiting => (0, false),
        Round::Running { step, .. } => (step, true),
    };

    ViewModel {
        screen: model.screen,
        leds,
        channel: model.channel(),
        countdown_step,
        countdown_active,
    }
}

fn mode_select_leds(model: &Model) -> [Rgb; NUM_LEDS] {
    let mut colors = [Rgb::OFF; NUM_LEDS];
    let step = model.mode_step as usize;
    let led = palette::ring_step_led(step);
    let ch = palette::ring_step_channel(step);

    for i in 0..CHANNELS {
        colors[i] = palette::channel_color(i);
    }

    if palette::ring_step_is_picker(step) {
        if !model.blink_on {
            colors[led] = Rgb::OFF;
        }
    } else if model.blink_on {
        colors[led] = palette::channel_color(ch);
    }

    colors
}

fn picker_leds(model: &Model) -> [Rgb; NUM_LEDS] {
    match model.picker_phase {
        PickerPhase::Selecting => {
            let mut colors = palette::spectrum_array();
            if !model.blink_on {
                colors[model.picker_selected as usize % NUM_LEDS] = Rgb::OFF;
            }
            colors
        }
        PickerPhase::Applied => [palette::spectrum_rgb(model.picker_applied); NUM_LEDS],
    }
}

fn receiver_leds(model: &Model) -> [Rgb; NUM_LEDS] {
    let target = match model.round {
        Round::Running { target, .. } => target,
        Round::Waiting => Color::White,
    };
    let rgb = match model.round {
        Round::Waiting => Rgb::OFF,
        Round::Running { .. } => palette::spectrum_rgb(target),
    };
    let mut colors = [rgb; NUM_LEDS];
    let off = match model.round {
        Round::Running { step, .. } => step as usize,
        Round::Waiting => 0,
    };
    for i in 0..off.min(NUM_LEDS) {
        colors[i] = Rgb::OFF;
    }
    colors
}
