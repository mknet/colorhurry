//! Effect runtime for the spec scenarios: virtual time, scripted entropy, logs.
//!
//! Domain steps call [`Harness::send`] with [`Event`]s derived from the scenario
//! text. This type is not a product shell. It answers the timers and `Random`,
//! and it records the other effects for assertions. Timers fire only when
//! virtual time advances, so a scenario sees the state right after an event.
//! A `Random` request waits until a step answers it if no byte is queued.

use std::collections::VecDeque;
use std::fmt;

use color_hurry_core::{
    time::{operation::NotifyAfter, TimerId},
    BleOp, Color, ColorHurry, Effect, Event, Random, ToneSequence, ViewModel, NUM_LEDS, Rgb,
};
use crux_core::{Core, Request};

struct PendingTimer {
    due_ms: u64,
    request: Request<NotifyAfter>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedTone {
    pub at_ms: u64,
    pub steps: Vec<(u32, u32)>,
}

impl ObservedTone {
    fn from_sequence(at_ms: u64, seq: &ToneSequence) -> Self {
        Self {
            at_ms,
            steps: seq.steps().iter().map(|s| (s.hz, s.ms)).collect(),
        }
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
    /// `Random` requests that came while no byte was queued, oldest first.
    held_random: VecDeque<Request<Random>>,
    random_requests: usize,
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
            .field("random_requests", &self.random_requests)
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
            held_random: VecDeque::new(),
            random_requests: 0,
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

    /// Random requests since the last [`Harness::clear_logs`].
    pub fn random_requests(&self) -> usize {
        self.random_requests
    }

    pub fn clear_logs(&mut self) {
        self.random_requests = 0;
        self.ble_log.clear();
        self.tone_log.clear();
        self.timer_log.clear();
        self.clear_log.clear();
    }

    pub fn push_random_many(&mut self, bytes: impl IntoIterator<Item = u8>) {
        self.random_queue.extend(bytes);
    }

    /// Answer the oldest open `Random` request with `byte`.
    pub fn answer_random(&mut self, byte: u8) {
        let mut request = self
            .held_random
            .pop_front()
            .expect("the core has not requested a random byte");
        let more = self.core.resolve(&mut request, byte).unwrap_or_default();
        self.handle(more);
    }

    /// Deliver a domain event and drain the resulting effects.
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

    /// Jump to the next pending timer and fire everything due then.
    pub fn advance_to_next_timer(&mut self) {
        let due = self
            .timers
            .iter()
            .map(|d| d.due_ms)
            .min()
            .expect("no pending timer");
        self.now_ms = self.now_ms.max(due);
        self.resolve_due();
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
                Effect::Random(mut request) => {
                    self.random_requests += 1;
                    match self.random_queue.pop_front() {
                        Some(byte) => {
                            let more = self.core.resolve(&mut request, byte).unwrap_or_default();
                            queue.extend(more);
                        }
                        None => self.held_random.push_back(request),
                    }
                }
                Effect::PlayTone(request) => {
                    self.tone_log.push(ObservedTone::from_sequence(
                        self.now_ms,
                        &request.operation.sequence,
                    ));
                }
                Effect::Ble(request) => {
                    self.ble_log.push(request.operation.op);
                }
            }
        }
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

pub fn spectrum_rgb(color: Color) -> Rgb {
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
