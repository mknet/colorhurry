//! Color Hurry board shell: drives [`color_hurry_core::ColorHurry`] from a [`Board`].
//!
//! The CPB firmware and the desktop simulator run this same code. They differ only in
//! their [`Board`] implementation.

#![no_std]

extern crate alloc;

pub mod board;
pub mod buttons;
pub mod leds;
pub mod radio;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use color_hurry_core::{
    time::operation::NotifyAfter, BleOp, ColorHurry, Effect, Event, Screen, ToneSequence,
    ViewModel,
};
use crux_core::{Core, Request};

pub use board::Board;
pub use buttons::{ButtonEvent, Edges};

/// Length of one shell loop: poll the inputs, advance the timers, wait.
pub const SLICE_MS: u32 = 10;

pub struct Shell<B: Board> {
    core: Core<ColorHurry>,
    board: B,
    edges: Edges,
    entropy: u32,
    /// The pending timers, each with the milliseconds that remain. This list
    /// is the only timer state, so removing a timer frees it.
    timers: Vec<(u32, Request<NotifyAfter>)>,
}

impl<B: Board> Shell<B> {
    pub fn new(board: B) -> Self {
        Self {
            core: Core::new(),
            board,
            edges: Edges::new(),
            entropy: 0xA5A5_1234,
            timers: Vec::new(),
        }
    }

    pub fn board(&self) -> &B {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut B {
        &mut self.board
    }

    pub fn view(&self) -> ViewModel {
        self.core.view()
    }

    /// Start the core: the first render and the blink of mode select.
    pub async fn boot(&mut self) {
        self.dispatch(Event::Boot).await;
    }

    /// Boot, then run until the player has chosen Picker or Caller.
    pub async fn run_mode_select(&mut self) {
        self.boot().await;
        while self.core.view().screen == Screen::ModeSelect {
            self.step().await;
        }
    }

    /// Run the chosen role forever.
    pub async fn run_game(&mut self) -> ! {
        if self.core.view().screen == Screen::Picker {
            let channel = self.core.view().channel;
            self.board.radio_picker(channel);
        }
        loop {
            self.step().await;
        }
    }

    /// One loop of [`SLICE_MS`]: inputs, timers, tone service, wait.
    pub async fn step(&mut self) {
        self.poll_inputs().await;
        self.tick_timers(SLICE_MS).await;
        self.board.tone_service();
        self.board.delay_ms(SLICE_MS).await;
    }

    /// Send one event to the core and perform the effects it returns.
    pub async fn dispatch(&mut self, event: Event) {
        let effects = self.core.process_event(event);
        self.handle(effects).await;
    }

    async fn poll_inputs(&mut self) {
        let (left, right) = self.board.buttons();
        match self.edges.poll(left, right) {
            ButtonEvent::Left => self.dispatch(Event::Skip).await,
            ButtonEvent::Right => self.dispatch(Event::ActivateColor).await,
            ButtonEvent::None => {}
        }
        if let Some((channel, color)) = self.board.take_received() {
            self.dispatch(Event::ColorReceived { channel, color }).await;
        }
    }

    async fn tick_timers(&mut self, elapsed_ms: u32) {
        let mut i = 0;
        while i < self.timers.len() {
            if self.timers[i].0 <= elapsed_ms {
                let (_, mut request) = self.timers.swap_remove(i);
                let id = request.operation.id;
                let effects = self.core.resolve(&mut request, id).unwrap_or_default();
                self.handle(effects).await;
            } else {
                self.timers[i].0 -= elapsed_ms;
                i += 1;
            }
        }
    }

    async fn handle(&mut self, effects: Vec<Effect>) {
        let mut queue: VecDeque<Effect> = effects.into();
        let mut tones: Vec<ToneSequence> = Vec::new();
        let mut need_render = false;
        // 1) Timers/Ble/Random first (beat clock starts immediately)
        // 2) Tones next (old shell played audio before refreshing LEDs)
        // 3) Render last so the ring updates as the beep ends
        while let Some(effect) = queue.pop_front() {
            match effect {
                Effect::Render(_) => need_render = true,
                Effect::TimeNotifyAfter(request) => {
                    let duration = core::time::Duration::from(request.operation.duration);
                    let ms = u32::try_from(duration.as_millis()).unwrap_or(u32::MAX);
                    self.timers.push((ms, request));
                }
                Effect::TimeClear(mut request) => {
                    // Free the timer. It never fires, because the core no
                    // longer listens for it.
                    let id = request.operation.id;
                    self.timers.retain(|(_, timer)| timer.operation.id != id);
                    let more = self.core.resolve(&mut request, id).unwrap_or_default();
                    queue.extend(more);
                }
                Effect::Random(mut request) => {
                    let byte = self.next_entropy();
                    let more = self.core.resolve(&mut request, byte).unwrap_or_default();
                    queue.extend(more);
                }
                Effect::PlayTone(request) => tones.push(request.operation.sequence),
                Effect::Ble(request) => self.apply_ble(request.operation.op),
            }
        }
        for seq in tones {
            // Always wait the full duration. The non-blocking UI failsafe path was
            // chopping Apply/Skip after the Crux shell switched to a 10 ms poll.
            for step in seq.steps() {
                self.board.play_tone(step.hz, step.ms).await;
            }
        }
        if need_render {
            let leds = self.core.view().leds;
            self.board.show(&leds);
        }
    }

    fn apply_ble(&mut self, op: BleOp) {
        match op {
            BleOp::Off => self.board.radio_off(),
            BleOp::Picker { channel } => self.board.radio_picker(channel),
            BleOp::Caller { channel, color: _ } => self.board.radio_caller(channel),
            BleOp::Broadcast { color } => self.board.broadcast(color),
        }
    }

    fn next_entropy(&mut self) -> u8 {
        self.entropy = self
            .entropy
            .wrapping_mul(1_103_515_245)
            .wrapping_add(12_345);
        (self.entropy >> 16) as u8
    }
}

#[cfg(test)]
mod tests;
