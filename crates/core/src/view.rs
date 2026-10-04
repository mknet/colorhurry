use crate::color::Rgb;
use crate::model::{Model, PickerPhase, Screen, NUM_LEDS};
use crate::palette::{self, CHANNELS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewModel {
    pub screen: Screen,
    pub leds: [Rgb; NUM_LEDS],
    pub channel: u8,
    pub countdown_step: u8,
    pub countdown_active: bool,
}

pub fn view(model: &Model) -> ViewModel {
    let leds = match model.screen {
        Screen::ModeSelect => mode_select_leds(model),
        Screen::Picker => picker_leds(model),
        Screen::Receiver => receiver_leds(model),
    };

    ViewModel {
        screen: model.screen,
        leds,
        channel: model.channel(),
        countdown_step: model.countdown_step,
        countdown_active: model.countdown_active,
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
        PickerPhase::Applied => [model.picker_applied; NUM_LEDS],
    }
}

fn receiver_leds(model: &Model) -> [Rgb; NUM_LEDS] {
    let mut colors = [model.receiver_target; NUM_LEDS];
    let off = model.countdown_step as usize;
    for i in 0..off.min(NUM_LEDS) {
        colors[i] = Rgb::OFF;
    }
    colors
}
