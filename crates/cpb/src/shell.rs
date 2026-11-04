//! Drive [`color_hurry_core::ColorHurry`] from CPB hardware.

use alloc::vec::Vec;

use color_hurry_core::{
    BleOp, ColorHurry, Effect, Event, Screen, ToneStep,
};
use crux_core::{Core, Request};

use crate::ble_broadcast;
use crate::buttons::{self, Buttons};
use crate::neopixel_pwm::{self, Rgb};
use crate::speaker;
use crate::time;

const SLICE_MS: u32 = 10;

pub struct Shell {
    core: Core<ColorHurry>,
    buttons: Buttons,
    entropy: u32,
    ble_ready: bool,
    delays: Vec<(u32, Request<color_hurry_core::Delay>)>,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            core: Core::new(),
            buttons: Buttons::new(),
            entropy: 0xA5A5_1234,
            ble_ready: false,
            delays: Vec::new(),
        }
    }

    pub fn mark_ble_ready(&mut self) {
        self.ble_ready = true;
    }

    pub async fn run_mode_select(&mut self) {
        Buttons::init();
        let effects = self.core.process_event(Event::Boot);
        self.handle(effects).await;
        while self.core.view().screen == Screen::ModeSelect {
            self.poll_inputs().await;
            self.tick_delays(SLICE_MS).await;
            speaker::tone_service();
            time::delay_ms_async(SLICE_MS).await;
        }
    }

    pub async fn run_game(&mut self) -> ! {
        self.bootstrap_radio();
        loop {
            self.poll_inputs().await;
            self.tick_delays(SLICE_MS).await;
            speaker::tone_service();
            time::delay_ms_async(SLICE_MS).await;
        }
    }

    fn bootstrap_radio(&mut self) {
        match self.core.view().screen {
            Screen::Picker => {
                let ch = self.core.view().channel;
                ble_broadcast::set_radio_picker(ch as usize);
            }
            Screen::Receiver | Screen::ModeSelect => {}
        }
    }

    async fn poll_inputs(&mut self) {
        match self.buttons.poll() {
            buttons::Event::Left => {
                let effects = self.core.process_event(Event::Skip);
                self.handle(effects).await;
            }
            buttons::Event::Right => {
                let effects = self.core.process_event(Event::ActivateColor);
                self.handle(effects).await;
            }
            buttons::Event::None => {}
        }
        if let Some((channel, color)) = ble_broadcast::take_color_received() {
            let effects = self
                .core
                .process_event(Event::ColorReceived { channel, color });
            self.handle(effects).await;
        }
    }

    async fn tick_delays(&mut self, elapsed_ms: u32) {
        let mut i = 0;
        while i < self.delays.len() {
            if self.delays[i].0 <= elapsed_ms {
                let (_, mut request) = self.delays.swap_remove(i);
                let effects = self.core.resolve(&mut request, ()).unwrap_or_default();
                self.handle(effects).await;
            } else {
                self.delays[i].0 -= elapsed_ms;
                i += 1;
            }
        }
    }

    async fn handle(&mut self, effects: Vec<Effect>) {
        use alloc::collections::VecDeque;
        let mut queue: VecDeque<Effect> = effects.into();
        let mut tones: Vec<color_hurry_core::ToneSequence> = Vec::new();
        let mut need_render = false;
        // 1) Delays/Ble/Random first (beat clock starts immediately)
        // 2) Tones next (old shell played audio before refreshing LEDs)
        // 3) Render last so the ring updates as the beep ends
        while let Some(effect) = queue.pop_front() {
            match effect {
                Effect::Render(_) => need_render = true,
                Effect::Delay(request) => {
                    let ms = request.operation.millis;
                    self.delays.push((ms, request));
                }
                Effect::Random(mut request) => {
                    let byte = self.next_entropy();
                    let more = self.core.resolve(&mut request, byte).unwrap_or_default();
                    for e in more {
                        queue.push_back(e);
                    }
                }
                Effect::PlayTone(request) => {
                    tones.push(request.operation.sequence);
                }
                Effect::Ble(request) => {
                    self.apply_ble(request.operation.op);
                }
            }
        }
        for seq in tones {
            self.play_sequence(seq.steps()).await;
        }
        if need_render {
            let vm = self.core.view();
            let leds = vm.leds.map(to_hw_rgb);
            neopixel_pwm::show_pixels(&leds);
        }
    }

    fn apply_ble(&mut self, op: BleOp) {
        // Always update intended radio state; advertise/scan runs only when stack is ready.
        match op {
            BleOp::Off => ble_broadcast::set_radio_off(),
            BleOp::Picker { channel } => ble_broadcast::set_radio_picker(channel as usize),
            BleOp::Receiver { channel, color: _ } => {
                ble_broadcast::set_radio_receiver(channel as usize);
            }
            BleOp::Broadcast { color } => ble_broadcast::set_picker_broadcast(color),
        }
        let _ = self.ble_ready;
    }

    async fn play_sequence(&mut self, steps: &[ToneStep]) {
        // Always wait the full duration. The non-blocking UI failsafe path was
        // chopping Apply/Skip after the Crux shell switched to a 10 ms poll.
        for step in steps {
            let duty = speaker::duty_for_core_tone(step.hz, step.ms);
            speaker::play_tone(step.hz, step.ms, duty).await;
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

fn to_hw_rgb(c: color_hurry_core::Rgb) -> Rgb {
    Rgb {
        r: c.r,
        g: c.g,
        b: c.b,
    }
}
