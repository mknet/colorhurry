use crate::color::Rgb;
use crate::effects::{self, Effect, Effects, ToneKind};
use crate::model::{
    Event, Model, PickerPhase, Role, Screen, BLINK_TICKS, COUNTDOWN_STEPS, NUM_LEDS,
};
use crate::palette;

pub fn update(event: Event, model: &mut Model) -> Effects {
    match model.screen {
        Screen::ModeSelect => update_mode_select(event, model),
        Screen::Picker => update_picker(event, model),
        Screen::Receiver => update_receiver(event, model),
    }
}

fn update_mode_select(event: Event, model: &mut Model) -> Effects {
    match event {
        Event::Tick => {
            advance_blink(model);
            effects::only(Effect::Render)
        }
        Event::ButtonLeft => {
            model.mode_step = (model.mode_step + 1) % NUM_LEDS as u8;
            model.blink_on = false;
            model.blink_counter = 0;
            effects::render_and(Effect::PlayTone(ToneKind::Skip))
        }
        Event::ButtonRight => {
            let step = model.mode_step as usize;
            let ch = palette::ring_step_channel(step) as u8;
            let role = if palette::ring_step_is_picker(step) {
                Role::Picker { channel: ch }
            } else {
                Role::Receiver { channel: ch }
            };
            model.role = Some(role);
            model.blink_on = true;
            model.blink_counter = 0;

            let mut out = Effects::new();
            out.push(Effect::PlayTone(ToneKind::Apply));

            match role {
                Role::Picker { channel } => {
                    model.screen = Screen::Picker;
                    model.picker_selected = channel.min(NUM_LEDS as u8 - 1);
                    model.picker_phase = PickerPhase::Selecting;
                    out.push(Effect::SetBlePicker { channel });
                    out.push(Effect::SetPickerBroadcast { color: None });
                }
                Role::Receiver { channel: _ } => {
                    model.screen = Screen::Receiver;
                    model.awaiting_entropy = true;
                    model.countdown_active = false;
                    model.countdown_step = 0;
                    out.push(Effect::ClearBle);
                    // Shell should follow with Entropy
                }
            }
            out.push(Effect::Render);
            out
        }
        Event::BleColorReceived { .. } | Event::Entropy(_) => Effects::new(),
    }
}

fn update_picker(event: Event, model: &mut Model) -> Effects {
    match event {
        Event::Tick => {
            if model.picker_phase == PickerPhase::Selecting {
                advance_blink(model);
            }
            effects::only(Effect::Render)
        }
        Event::ButtonLeft => match model.picker_phase {
            PickerPhase::Selecting => {
                model.picker_selected = (model.picker_selected + 1) % NUM_LEDS as u8;
                model.blink_on = false;
                model.blink_counter = 0;
                effects::render_and(Effect::PlayTone(ToneKind::Skip))
            }
            PickerPhase::Applied => {
                model.picker_phase = PickerPhase::Selecting;
                let mut out = Effects::new();
                out.push(Effect::SetPickerBroadcast { color: None });
                out.push(Effect::Render);
                out
            }
        },
        Event::ButtonRight => match model.picker_phase {
            PickerPhase::Selecting => {
                model.picker_applied = palette::spectrum_color(model.picker_selected as usize);
                model.picker_phase = PickerPhase::Applied;
                let mut out = Effects::new();
                out.push(Effect::PlayTone(ToneKind::Apply));
                out.push(Effect::SetPickerBroadcast {
                    color: Some(model.picker_applied),
                });
                out.push(Effect::Render);
                out
            }
            PickerPhase::Applied => {
                model.picker_phase = PickerPhase::Selecting;
                let mut out = Effects::new();
                out.push(Effect::SetPickerBroadcast { color: None });
                out.push(Effect::Render);
                out
            }
        },
        Event::BleColorReceived { .. } | Event::Entropy(_) => Effects::new(),
    }
}

fn update_receiver(event: Event, model: &mut Model) -> Effects {
    match event {
        Event::Entropy(byte) => {
            if !model.awaiting_entropy && model.countdown_active {
                return Effects::new();
            }
            start_round(model, palette::color_from_entropy(byte))
        }
        Event::Tick => {
            if model.awaiting_entropy {
                return Effects::new();
            }
            if !model.countdown_active {
                return Effects::new();
            }
            // Shell sends Tick once per countdown beat (not UI blink).
            let step = model.countdown_step;
            let mut out = Effects::new();
            out.push(Effect::PlayTone(ToneKind::CountdownStep(step)));
            model.countdown_step = step.saturating_add(1);
            if model.countdown_step > COUNTDOWN_STEPS {
                // Missed — explosion, then ask for new entropy
                model.countdown_active = false;
                model.awaiting_entropy = true;
                out.push(Effect::PlayTone(ToneKind::Explosion));
                out.push(Effect::ClearBle);
            }
            out.push(Effect::Render);
            out
        }
        Event::ButtonLeft => {
            // Abort countdown like a match (no explosion)
            if model.countdown_active {
                model.countdown_active = false;
                model.awaiting_entropy = true;
                let mut out = Effects::new();
                out.push(Effect::ClearBle);
                out.push(Effect::Render);
                out
            } else {
                Effects::new()
            }
        }
        Event::ButtonRight => Effects::new(),
        Event::BleColorReceived { channel, color } => {
            if !model.countdown_active {
                return Effects::new();
            }
            if channel != model.channel() {
                return Effects::new();
            }
            if color != model.receiver_target {
                return Effects::new();
            }
            // Match — next round, no explosion
            model.countdown_active = false;
            model.awaiting_entropy = true;
            let mut out = Effects::new();
            out.push(Effect::ClearBle);
            out.push(Effect::Render);
            out
        }
    }
}

fn start_round(model: &mut Model, color: Rgb) -> Effects {
    model.receiver_target = color;
    model.countdown_step = 0;
    model.countdown_active = true;
    model.awaiting_entropy = false;
    let channel = model.channel();
    let mut out = Effects::new();
    out.push(Effect::SetBleReceiver { channel, color });
    out.push(Effect::Render);
    out
}

fn advance_blink(model: &mut Model) {
    model.blink_counter += 1;
    if model.blink_counter >= BLINK_TICKS {
        model.blink_counter = 0;
        model.blink_on = !model.blink_on;
    }
}
