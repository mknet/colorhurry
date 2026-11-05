//! Effect runtime for feature tests: virtual time, scripted entropy, logs.
//!
//! Domain steps call [`Harness::send`] with [`Event`]s derived from the feature
//! text. This type is not a product shell. It answers the timers and `Random`,
//! and it records the other effects for assertions.

use std::collections::VecDeque;
use std::fmt;

use color_hurry_core::{
    time::{operation::NotifyAfter, TimerId},
    BleOp, Color, ColorHurry, Effect, Event, Random, ToneSequence, ViewModel,
    BEAT_PERIOD_MS, NUM_LEDS, Rgb,
};
use crux_core::{Core, Request};

struct PendingTimer {
    due_ms: u64,
    request: Request<NotifyAfter>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedTone {
    pub steps: Vec<(u32, u32)>,
}

impl ObservedTone {
    fn from_sequence(seq: &ToneSequence) -> Self {
        Self {
            steps: seq.steps().iter().map(|s| (s.hz, s.ms)).collect(),
        }
    }

    pub fn is_skip(&self) -> bool {
        self.steps == [(380, 55)]
    }

    pub fn is_apply(&self) -> bool {
        self.steps == [(784, 130)]
    }

    pub fn is_explosion(&self) -> bool {
        self.steps.len() == 10 && self.steps.first().map(|s| s.0) == Some(180)
    }

    pub fn is_countdown_step(&self) -> bool {
        self.steps.len() == 1 && self.steps[0].1 == 200
    }
}

pub struct Harness {
    core: Core<ColorHurry>,
    now_ms: u64,
    timers: VecDeque<PendingTimer>,
    random_queue: VecDeque<u8>,
    /// When true, `Random` requests wait in `held_random` until a step answers them.
    hold_random: bool,
    held_random: VecDeque<Request<Random>>,
    ble_log: Vec<BleOp>,
    tone_log: Vec<ObservedTone>,
    /// The duration in milliseconds of each timer that the core requested.
    timer_log: Vec<u64>,
    /// The id of each timer that the core cleared.
    clear_log: Vec<TimerId>,
}

impl fmt::Debug for Harness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Harness")
            .field("now_ms", &self.now_ms)
            .field("view", &self.view())
            .field("pending_timers", &self.timers.len())
            .field("random_queue", &self.random_queue)
            .field("held_random", &self.held_random.len())
            .field("ble_log", &self.ble_log)
            .field("tone_log", &self.tone_log)
            .field("timer_log", &self.timer_log)
            .field("clear_log", &self.clear_log)
            .finish()
    }
}

impl Default for Harness {
    fn default() -> Self {
        Self::new()
    }
}

impl Harness {
    pub fn new() -> Self {
        Self {
            core: Core::new(),
            now_ms: 0,
            timers: VecDeque::new(),
            random_queue: VecDeque::new(),
            hold_random: false,
            held_random: VecDeque::new(),
            ble_log: Vec::new(),
            tone_log: Vec::new(),
            timer_log: Vec::new(),
            clear_log: Vec::new(),
        }
    }

    pub fn view(&self) -> ViewModel {
        self.core.view()
    }

    pub fn ble_log(&self) -> &[BleOp] {
        &self.ble_log
    }

    pub fn tone_log(&self) -> &[ObservedTone] {
        &self.tone_log
    }

    pub fn timer_log(&self) -> &[u64] {
        &self.timer_log
    }

    pub fn clear_log(&self) -> &[TimerId] {
        &self.clear_log
    }

    pub fn last_tone(&self) -> Option<&ObservedTone> {
        self.tone_log.last()
    }

    pub fn clear_logs(&mut self) {
        self.ble_log.clear();
        self.tone_log.clear();
        self.timer_log.clear();
        self.clear_log.clear();
    }

    pub fn push_random_many(&mut self, bytes: impl IntoIterator<Item = u8>) {
        self.random_queue.extend(bytes);
    }

    /// Keep later `Random` requests in flight until [`Self::answer_held_random`].
    pub fn hold_random(&mut self, hold: bool) {
        self.hold_random = hold;
    }

    /// Answer the oldest held `Random` request with `byte`.
    pub fn answer_held_random(&mut self, byte: u8) {
        let mut request = self
            .held_random
            .pop_front()
            .expect("no Random request is held");
        let more = self.core.resolve(&mut request, byte).unwrap_or_default();
        self.handle(more);
    }

    /// Deliver a domain event and drain resulting effects (including 0 ms timers).
    pub fn send(&mut self, event: Event) {
        let effects = self.core.process_event(event);
        self.handle(effects);
    }

    pub fn advance_ms(&mut self, ms: u64) {
        let target = self.now_ms.saturating_add(ms);
        while self.now_ms < target {
            let next_due = self
                .timers
                .iter()
                .map(|d| d.due_ms)
                .filter(|d| *d <= target)
                .min();
            match next_due {
                Some(due) if due > self.now_ms => {
                    self.now_ms = due;
                    self.resolve_due();
                }
                Some(_) => self.resolve_due(),
                None => break,
            }
        }
        self.now_ms = target;
        self.resolve_due();
    }

    pub fn run_full_countdown(&mut self) {
        for _ in 0..NUM_LEDS + 2 {
            if !self.view().countdown_active && self.timers.is_empty() {
                break;
            }
            if let Some(due) = self.timers.iter().map(|d| d.due_ms).min() {
                if due > self.now_ms {
                    self.now_ms = due;
                }
            } else if self.view().countdown_active {
                self.now_ms = self.now_ms.saturating_add(u64::from(BEAT_PERIOD_MS));
            } else {
                break;
            }
            self.resolve_due();
        }
    }

    pub fn led_is_off(&self, index: usize) -> bool {
        self.view().leds[index % NUM_LEDS] == Rgb::OFF
    }

    pub fn all_leds_equal(&self, color: Color) -> bool {
        let rgb = spectrum_rgb(color);
        self.view().leds.iter().all(|c| *c == rgb)
    }

    fn handle(&mut self, effects: Vec<Effect>) {
        let mut queue: VecDeque<Effect> = effects.into();
        while let Some(effect) = queue.pop_front() {
            match effect {
                Effect::Render(_) => {}
                Effect::TimeNotifyAfter(request) => {
                    let duration = core::time::Duration::from(request.operation.duration);
                    let millis = u64::try_from(duration.as_millis()).unwrap_or(u64::MAX);
                    self.timer_log.push(millis);
                    let due = self.now_ms.saturating_add(millis);
                    self.timers.push_back(PendingTimer { due_ms: due, request });
                }
                Effect::TimeClear(mut request) => {
                    // A cleared timer never fires.
                    let id = request.operation.id;
                    self.clear_log.push(id);
                    self.timers.retain(|timer| timer.request.operation.id != id);
                    let more = self.core.resolve(&mut request, id).unwrap_or_default();
                    for e in more {
                        queue.push_back(e);
                    }
                }
                Effect::Random(request) if self.hold_random => {
                    self.held_random.push_back(request);
                }
                Effect::Random(mut request) => {
                    let byte = self.random_queue.pop_front().unwrap_or_else(|| {
                        panic!(
                            "core requested Random but the entropy queue is empty; \
                             queue bytes before the event that needs them"
                        )
                    });
                    let more = self.core.resolve(&mut request, byte).unwrap_or_default();
                    for e in more {
                        queue.push_back(e);
                    }
                }
                Effect::PlayTone(request) => {
                    self.tone_log
                        .push(ObservedTone::from_sequence(&request.operation.sequence));
                }
                Effect::Ble(request) => {
                    self.ble_log.push(request.operation.op);
                }
            }
        }
        self.resolve_due();
    }

    fn resolve_due(&mut self) {
        loop {
            let Some(idx) = self
                .timers
                .iter()
                .position(|d| d.due_ms <= self.now_ms)
            else {
                break;
            };
            let Some(PendingTimer { mut request, .. }) = self.timers.remove(idx) else {
                break;
            };
            let id = request.operation.id;
            let more = self.core.resolve(&mut request, id).unwrap_or_default();
            self.handle(more);
        }
    }
}

fn spectrum_rgb(color: Color) -> Rgb {
    // Matches crates/core/src/palette.rs (palette module is crate-private).
    const SPECTRUM: [Rgb; 10] = [
        Rgb::from_rgb(255, 255, 255),
        Rgb::from_rgb(255, 0, 0),
        Rgb::from_rgb(255, 120, 0),
        Rgb::from_rgb(255, 255, 0),
        Rgb::from_rgb(140, 255, 100),
        Rgb::from_rgb(0, 100, 0),
        Rgb::from_rgb(120, 200, 255),
        Rgb::from_rgb(0, 0, 160),
        Rgb::from_rgb(180, 0, 255),
        Rgb::from_rgb(255, 100, 180),
    ];
    SPECTRUM[color.index() as usize % NUM_LEDS]
}
