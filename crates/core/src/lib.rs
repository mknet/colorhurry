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
pub use view::{ModeChoice, ModeRole, ViewModel};

/// The time capability, so that the Rust shells name the same `crux_time`
/// operations as the core.
pub use crux_time as time;

use core::time::Duration;

use crux_core::{
    App, Command,
    macros::{Operation, effect},
    render::{RenderOperation, render},
};
use crux_time::{clock::Time, TimerHandle, TimerOutcome};
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
    /// The app went to the background. The core stops its timers and the radio.
    Pause,
    /// The app came back from the background. The core continues where it stopped.
    Resume,
    /// The player goes back to mode select. The core stops the round and the
    /// radio, and mode select starts at the role and channel that the player
    /// entered last.
    Reset,
    /// The shell cannot broadcast a color, so it cannot be Picker. The shell
    /// sends this event once, before `Boot`. After it, mode select offers only
    /// Caller choices. No event turns it off again.
    CallerOnly,

    // Only the core sends the timer events, so they are not serialized. They
    // stay at the end of `Event`, so that the bincode index of each shell
    // event is the same in Rust and in the generated types.
    /// The blink timer fired or was cleared.
    #[serde(skip)]
    #[facet(skip)]
    BlinkDone(#[facet(opaque)] TimerOutcome),
    /// The countdown beat timer fired or was cleared.
    #[serde(skip)]
    #[facet(skip)]
    BeatDone(#[facet(opaque)] TimerOutcome),
    /// The timer for the last beat tone and the explosion fired or was cleared.
    #[serde(skip)]
    #[facet(skip)]
    ExplosionDone(#[facet(opaque)] TimerOutcome),
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
    Caller { channel: u8, color: Color },
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
    TimeNotifyAfter(time::operation::NotifyAfter),
    TimeClear(time::operation::ClearTimer),
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
        match event {
            Event::Pause => return pause(model),
            Event::Resume => return resume(model),
            // A shell capability, not a game action, so the core accepts it
            // also while paused.
            Event::CallerOnly => return caller_only(model),
            // The core cleared this timer because it no longer wants it.
            Event::BlinkDone(TimerOutcome::Cleared)
            | Event::BeatDone(TimerOutcome::Cleared)
            | Event::ExplosionDone(TimerOutcome::Cleared) => return Command::done(),
            _ => {}
        }
        // While paused, the core ignores every event except `Resume` and
        // `CallerOnly`. A `RandomByte` from a request that was in flight is
        // dropped too, because `Resume` requests a new byte.
        if model.paused {
            return Command::done();
        }
        if event == Event::Reset {
            return reset(model);
        }
        match model.screen {
            Scr::ModeSelect => update_mode_select(event, model),
            Scr::Picker => update_picker(event, model),
            Scr::Caller => update_caller(event, model),
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

/// Starts a timer of `millis` in `slot`, and clears the timer that was there.
///
/// The command sends `done` with the outcome of the timer.
fn start_timer(
    slot: &mut Option<TimerHandle>,
    millis: u32,
    done: fn(TimerOutcome) -> Event,
) -> Command<Effect, Event> {
    clear_timer(slot);
    let (notify, handle) = Time::notify_after(Duration::from_millis(u64::from(millis)));
    *slot = Some(handle);
    notify.then_send(done)
}

/// Clears the timer in `slot`, if there is one, and leaves the slot empty.
///
/// The shell frees the timer, and the timer event comes with
/// `TimerOutcome::Cleared`, which `update` ignores.
fn clear_timer(slot: &mut Option<TimerHandle>) {
    if let Some(handle) = slot.take() {
        handle.clear();
    }
}

fn schedule_blink(model: &mut Model) -> Command<Effect, Event> {
    start_timer(&mut model.blink_timer, BLINK_PERIOD_MS, Event::BlinkDone)
}

fn schedule_beat(model: &mut Model, millis: u32) -> Command<Effect, Event> {
    start_timer(&mut model.round_timer, millis, Event::BeatDone)
}

/// Wait for the queued tones to finish, then send `ExplosionDone`.
fn schedule_explosion_done(model: &mut Model, millis: u32) -> Command<Effect, Event> {
    start_timer(&mut model.round_timer, millis, Event::ExplosionDone)
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

/// Stops every timer and the radio, and remembers that the game is paused.
///
/// The core clears the blink timer and the round timer, so the shell frees
/// them.
fn pause(model: &mut Model) -> Command<Effect, Event> {
    if model.paused {
        return Command::done();
    }
    model.paused = true;
    clear_timer(&mut model.blink_timer);
    clear_timer(&mut model.round_timer);
    ble(BleOp::Off)
}

/// Sends the radio op of the current screen again and restarts its timers.
///
/// A running countdown continues at the same step, and its next beat comes one
/// full `BEAT_PERIOD_MS` later. A Caller with no running round requests a new
/// random byte. This starts the next round after an explosion (with no second
/// explosion) and after a random byte that the pause dropped.
fn resume(model: &mut Model) -> Command<Effect, Event> {
    if !model.paused {
        return Command::done();
    }
    model.paused = false;
    match model.screen {
        Scr::ModeSelect => render().and(schedule_blink(model)),
        Scr::Picker => {
            let channel = model.channel();
            let color = match model.picker_phase {
                Phase::Selecting => None,
                Phase::Applied => Some(model.picker_applied),
            };
            let cmd = ble(BleOp::Picker { channel })
                .and(ble(BleOp::Broadcast { color }))
                .and(render());
            match model.picker_phase {
                Phase::Selecting => cmd.and(schedule_blink(model)),
                Phase::Applied => cmd,
            }
        }
        Scr::Caller => match model.round {
            RoundState::Running { target, .. } => {
                let channel = model.channel();
                ble(BleOp::Caller {
                    channel,
                    color: target,
                })
                .and(render())
                .and(schedule_beat(model, BEAT_PERIOD_MS))
            }
            RoundState::Waiting => render().and(request_random()),
        },
    }
}

/// Stops every timer, the round and the radio, and goes back to mode select.
///
/// `ActivateColor` does not change `mode_step`, so mode select starts at the
/// role and channel that the player entered last. The core clears the round
/// timer, and `schedule_blink` clears the blink timer before it starts a new
/// one.
fn reset(model: &mut Model) -> Command<Effect, Event> {
    model.screen = Scr::ModeSelect;
    model.role = None;
    model.picker_selected = 0;
    model.picker_phase = Phase::Selecting;
    model.picker_applied = Color::White;
    model.round = RoundState::Waiting;
    clear_timer(&mut model.round_timer);
    model.blink_on = true;
    model.blink_counter = 0;
    ble(BleOp::Off).and(render()).and(schedule_blink(model))
}

/// Limits mode select to the Caller choices.
///
/// While `caller_only` is set, `mode_step` is always a Caller step. Skip
/// keeps this rule, so `ActivateColor`, the view and `reset` need no change.
/// If the current step is a Picker step, mode select moves to Caller on
/// channel 0, because the board Picker starts on channel 0. The current screen
/// does not change.
fn caller_only(model: &mut Model) -> Command<Effect, Event> {
    model.caller_only = true;
    if !pal::ring_step_is_picker(model.mode_step as usize) {
        return Command::done();
    }
    model.mode_step = pal::CALLER_ONLY_START_STEP;
    if model.screen == Scr::ModeSelect && !model.paused {
        render()
    } else {
        Command::done()
    }
}

fn update_mode_select(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::Boot => render().and(schedule_blink(model)),
        Event::BlinkDone(_) => {
            model.blink_timer = None;
            advance_blink(model);
            render().and(schedule_blink(model))
        }
        Event::Skip => {
            let mut next = (model.mode_step + 1) % NUM_LEDS as u8;
            if model.caller_only && pal::ring_step_is_picker(next as usize) {
                next = pal::FIRST_CALLER_STEP;
            }
            model.mode_step = next;
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
                RoleKind::Caller { channel: ch }
            };
            model.role = Some(role);
            model.blink_on = true;
            model.blink_counter = 0;
            clear_timer(&mut model.blink_timer);

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
                RoleKind::Caller { channel: _ } => {
                    model.screen = Scr::Caller;
                    model.round = RoundState::Waiting;
                    play_tone(tone::apply())
                        .and(ble(BleOp::Off))
                        .and(render())
                        .and(request_random())
                }
            }
        }
        Event::ColorReceived { .. }
        | Event::RandomByte(_)
        | Event::BeatDone(_)
        | Event::ExplosionDone(_) => {
            // First blink kick: shells may send BlinkDone only after we schedule.
            // Also allow an initial Skip/Activate without prior blink.
            Command::done()
        }
        // `update` handles these before it dispatches to a screen.
        Event::Pause | Event::Resume | Event::Reset | Event::CallerOnly => Command::done(),
    }
}

fn update_picker(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::Boot => Command::done(),
        Event::BlinkDone(_) => {
            model.blink_timer = None;
            if model.picker_phase == Phase::Selecting {
                advance_blink(model);
                render().and(schedule_blink(model))
            } else {
                Command::done()
            }
        }
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
                clear_timer(&mut model.blink_timer);
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
        Event::ColorReceived { .. }
        | Event::RandomByte(_)
        | Event::BeatDone(_)
        | Event::ExplosionDone(_) => Command::done(),
        // `update` handles these before it dispatches to a screen.
        Event::Pause | Event::Resume | Event::Reset | Event::CallerOnly => Command::done(),
    }
}

fn update_caller(event: Event, model: &mut Model) -> Command<Effect, Event> {
    match event {
        Event::Boot => Command::done(),
        Event::RandomByte(byte) => {
            if !matches!(model.round, RoundState::Waiting) {
                return Command::done();
            }
            start_round(model, pal::color_from_entropy(byte))
        }
        Event::BeatDone(_) => {
            model.round_timer = None;
            countdown_beat(model)
        }
        // Only the countdown that expired can start the next round. If the
        // round state changes during the explosion, the core clears this
        // timer.
        Event::ExplosionDone(_) => {
            model.round_timer = None;
            request_random()
        }
        Event::Skip => {
            if matches!(model.round, RoundState::Running { .. }) {
                model.round = RoundState::Waiting;
                clear_timer(&mut model.round_timer);
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
            clear_timer(&mut model.round_timer);
            ble(BleOp::Off)
                .and(render())
                .and(request_random())
        }
        Event::BlinkDone(_) => Command::done(),
        // `update` handles these before it dispatches to a screen.
        Event::Pause | Event::Resume | Event::Reset | Event::CallerOnly => Command::done(),
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
    ble(BleOp::Caller { channel, color })
        .and(render())
        .and(schedule_beat(model, 0))
}

/// One countdown beat: play tone for `step`, then extinguish that LED.
fn countdown_beat(model: &mut Model) -> Command<Effect, Event> {
    let RoundState::Running { target, step } = model.round else {
        return Command::done();
    };
    let last_tone = tone::countdown_step(step);
    let explosion = tone::explosion();
    let tone_cmd = play_tone(last_tone);
    let next = step.saturating_add(1);
    if next >= COUNTDOWN_STEPS {
        model.round = RoundState::Waiting;
        // The shell queues tones, so the next round must wait for the last
        // beat tone and the explosion to finish playing.
        let wait_ms = last_tone.total_ms() + explosion.total_ms();
        tone_cmd
            .and(play_tone(explosion))
            .and(ble(BleOp::Off))
            .and(render())
            .and(schedule_explosion_done(model, wait_ms))
    } else {
        model.round = RoundState::Running {
            target,
            step: next,
        };
        tone_cmd
            .and(render())
            .and(schedule_beat(model, BEAT_PERIOD_MS))
    }
}

#[cfg(test)]
mod tests;
