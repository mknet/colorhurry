//! Color Hurry shared core: Crux `App`, no_std + alloc.
//!
//! Shells call `Core::process_event` / `resolve` and draw [`ViewModel`].

#![no_std]

extern crate alloc;

mod color;
mod model;
mod palette;
mod protocol;
mod tones;
mod view;

pub use color::{Color, Rgb};
pub use model::{
    Model, PickerPhase, Role, Round, Screen, BEAT_PERIOD_MS, BLINK_PERIOD_MS, BLINK_TICKS,
    COUNTDOWN_STEPS, NUM_LEDS,
};
pub use palette::{color_from_entropy, CHANNELS, SPECTRUM_LEN};
pub use protocol::{
    decode_color, encode_color, COMPANY_ID, CMD_COLOR, MAGIC, PAYLOAD_LEN, VERSION,
};
pub use tones::{ToneSequence, ToneStep};
pub use view::ViewModel;

use crux_core::{
    App, Command,
    macros::{Operation, effect},
    render::{RenderOperation, render},
};
use facet::Facet;
use serde::{Deserialize, Serialize};
// The typegen code `#[effect]` emits calls `to_string`, which no_std has no prelude for.
#[cfg(feature = "facet_typegen")]
use alloc::string::ToString;
use model::{PickerPhase as Phase, Role as RoleKind, Round as RoundState, Screen as Scr};
use palette as pal;
use tones as tone;

/// Domain events. Shells map hardware / DOM onto these.
#[derive(Facet, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum Event {
    /// Shell finished constructing the core; start blink (and initial render).
    Boot,
    Skip,
    ActivateColor,
    ColorReceived { channel: u8, color: Color },
    RandomByte(u8),
    /// Blink delay finished for generation `id`.
    BlinkDone(u32),
    /// Countdown beat delay finished for generation `id`.
    BeatDone(u32),
}

#[derive(Operation, Facet, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[operation(request, output = ())]
pub struct Delay {
    pub millis: u32,
}

#[derive(Operation, Facet, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[operation(request, output = u8)]
pub struct Random;

#[derive(Operation, Facet, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[operation(notify)]
pub struct PlayTone {
    pub sequence: ToneSequence,
}

#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum BleOp {
    Off,
    Picker { channel: u8 },
    Receiver { channel: u8, color: Color },
    Broadcast { color: Option<Color> },
}

#[derive(Operation, Facet, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[operation(notify)]
pub struct Ble {
    pub op: BleOp,
}

#[effect(facet_typegen)]
#[derive(Debug)]
pub enum Effect {
    Render(RenderOperation),
    Delay(Delay),
    Random(Random),
    PlayTone(PlayTone),
    Ble(Ble),
}

#[derive(Default)]
pub struct ColorHurry;

impl App for ColorHurry {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;

    fn update(&self, event: Event, model: &mut Model) -> Command<Effect, Event> {
        match model.screen {
            Scr::ModeSelect => update_mode_select(event, model),
            Scr::Picker => update_picker(event, model),
            Scr::Receiver => update_receiver(event, model),
        }
    }

    fn view(&self, model: &Model) -> ViewModel {
        view::view_model(model)
    }
}

fn play_tone(seq: ToneSequence) -> Command<Effect, Event> {
    Command::notify_shell(PlayTone { sequence: seq }).into()
}

fn ble(op: BleOp) -> Command<Effect, Event> {
    Command::notify_shell(Ble { op }).into()
}

fn schedule_blink(model: &mut Model) -> Command<Effect, Event> {
    model.blink_gen = model.blink_gen.wrapping_add(1);
    let id = model.blink_gen;
    Command::new(move |ctx| async move {
        ctx.request_from_shell(Delay {
            millis: BLINK_PERIOD_MS,
        })
        .await;
        ctx.send_event(Event::BlinkDone(id));
    })
}

fn schedule_beat_ms(model: &mut Model, millis: u32) -> Command<Effect, Event> {
    model.beat_gen = model.beat_gen.wrapping_add(1);
    let id = model.beat_gen;
    Command::new(move |ctx| async move {
        ctx.request_from_shell(Delay { millis }).await;
        ctx.send_event(Event::BeatDone(id));
    })
}

fn request_random() -> Command<Effect, Event> {
    Command::request_from_shell(Random).then_send(Event::RandomByte)
}

fn advance_blink(model: &mut Model) {
    model.blink_counter += 1;
    if model.blink_counter >= BLINK_TICKS {
        model.blink_counter = 0;
        model.blink_on = !model.blink_on;
    }
}

fn update_mode_select(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::Boot => render().and(schedule_blink(model)),
        Event::BlinkDone(id) if id == model.blink_gen => {
            advance_blink(model);
            render().and(schedule_blink(model))
        }
        Event::BlinkDone(_) => Command::done(),
        Event::Skip => {
            model.mode_step = (model.mode_step + 1) % NUM_LEDS as u8;
            model.blink_on = false;
            model.blink_counter = 0;
            play_tone(tone::skip())
                .and(render())
                .and(schedule_blink(model))
        }
        Event::ActivateColor => {
            let step = model.mode_step as usize;
            let ch = pal::ring_step_channel(step) as u8;
            let role = if pal::ring_step_is_picker(step) {
                RoleKind::Picker { channel: ch }
            } else {
                RoleKind::Receiver { channel: ch }
            };
            model.role = Some(role);
            model.blink_on = true;
            model.blink_counter = 0;
            model.blink_gen = model.blink_gen.wrapping_add(1);

            match role {
                RoleKind::Picker { channel } => {
                    model.screen = Scr::Picker;
                    model.picker_selected = channel.min(NUM_LEDS as u8 - 1);
                    model.picker_phase = Phase::Selecting;
                    play_tone(tone::apply())
                        .and(ble(BleOp::Picker { channel }))
                        .and(ble(BleOp::Broadcast { color: None }))
                        .and(render())
                        .and(schedule_blink(model))
                }
                RoleKind::Receiver { channel: _ } => {
                    model.screen = Scr::Receiver;
                    model.round = RoundState::Waiting;
                    play_tone(tone::apply())
                        .and(ble(BleOp::Off))
                        .and(render())
                        .and(request_random())
                }
            }
        }
        Event::ColorReceived { .. } | Event::RandomByte(_) | Event::BeatDone(_) => {
            // First blink kick: shells may send BlinkDone only after we schedule.
            // Also allow an initial Skip/Activate without prior blink.
            Command::done()
        }
    }
}

fn update_picker(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::Boot => Command::done(),
        Event::BlinkDone(id) if id == model.blink_gen => {
            if model.picker_phase == Phase::Selecting {
                advance_blink(model);
                render().and(schedule_blink(model))
            } else {
                Command::done()
            }
        }
        Event::BlinkDone(_) => Command::done(),
        Event::Skip => match model.picker_phase {
            Phase::Selecting => {
                model.picker_selected = (model.picker_selected + 1) % NUM_LEDS as u8;
                model.blink_on = false;
                model.blink_counter = 0;
                play_tone(tone::skip())
                    .and(render())
                    .and(schedule_blink(model))
            }
            Phase::Applied => {
                model.picker_phase = Phase::Selecting;
                ble(BleOp::Broadcast { color: None })
                    .and(render())
                    .and(schedule_blink(model))
            }
        },
        Event::ActivateColor => match model.picker_phase {
            Phase::Selecting => {
                model.picker_applied = pal::spectrum_color_at(model.picker_selected as usize);
                model.picker_phase = Phase::Applied;
                model.blink_gen = model.blink_gen.wrapping_add(1);
                play_tone(tone::apply())
                    .and(ble(BleOp::Broadcast {
                        color: Some(model.picker_applied),
                    }))
                    .and(render())
            }
            Phase::Applied => {
                model.picker_phase = Phase::Selecting;
                ble(BleOp::Broadcast { color: None })
                    .and(render())
                    .and(schedule_blink(model))
            }
        },
        Event::ColorReceived { .. } | Event::RandomByte(_) | Event::BeatDone(_) => Command::done(),
    }
}

fn update_receiver(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::Boot => Command::done(),
        Event::RandomByte(byte) => {
            if !matches!(model.round, RoundState::Waiting) {
                return Command::done();
            }
            start_round(model, pal::color_from_entropy(byte))
        }
        Event::BeatDone(id) if id == model.beat_gen => countdown_beat(model),
        Event::BeatDone(_) => Command::done(),
        Event::Skip => {
            if matches!(model.round, RoundState::Running { .. }) {
                model.round = RoundState::Waiting;
                model.beat_gen = model.beat_gen.wrapping_add(1);
                ble(BleOp::Off)
                    .and(render())
                    .and(request_random())
            } else {
                Command::done()
            }
        }
        Event::ActivateColor => Command::done(),
        Event::ColorReceived { channel, color } => {
            let RoundState::Running { target, .. } = model.round else {
                return Command::done();
            };
            if channel != model.channel() || color != target {
                return Command::done();
            }
            model.round = RoundState::Waiting;
            model.beat_gen = model.beat_gen.wrapping_add(1);
            ble(BleOp::Off)
                .and(render())
                .and(request_random())
        }
        Event::BlinkDone(_) => Command::done(),
    }
}

fn start_round(model: &mut Model, color: Color) -> Command<Effect, Event> {
    let channel = model.channel();
    // step = how many LEDs are already off / next tone index.
    // Show the full ring first; the first BeatDone (0 ms) plays tone 0 and
    // extinguishes LED 0, same cadence as the old CPB Tick loop.
    model.round = RoundState::Running {
        target: color,
        step: 0,
    };
    ble(BleOp::Receiver { channel, color })
        .and(render())
        .and(schedule_beat_ms(model, 0))
}

/// One countdown beat: play tone for `step`, then extinguish that LED.
fn countdown_beat(model: &mut Model) -> Command<Effect, Event> {
    let RoundState::Running { target, step } = model.round else {
        return Command::done();
    };
    let tone_cmd = play_tone(tone::countdown_step(step));
    let next = step.saturating_add(1);
    if next >= COUNTDOWN_STEPS {
        model.round = RoundState::Waiting;
        model.beat_gen = model.beat_gen.wrapping_add(1);
        tone_cmd
            .and(play_tone(tone::explosion()))
            .and(ble(BleOp::Off))
            .and(render())
            .and(request_random())
    } else {
        model.round = RoundState::Running {
            target,
            step: next,
        };
        tone_cmd
            .and(render())
            .and(schedule_beat_ms(model, BEAT_PERIOD_MS))
    }
}

#[cfg(test)]
mod tests;
