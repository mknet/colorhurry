//! Drive [`color_hurry_core`] from CPB hardware (buttons, LEDs, speaker, BLE).

use color_hurry_core::{
    update, view, Effect, Event, Model, Rgb as CoreRgb, Screen, ToneKind, NUM_LEDS,
};

use crate::ble_broadcast;
use crate::buttons::{self, Buttons};
use crate::neopixel_pwm::{self, Rgb};
use crate::speaker;
use crate::time;

const UI_TICK_MS: u32 = 50;

const DESCENDING_HZ: [u32; NUM_LEDS] =
    [680, 620, 560, 500, 450, 400, 350, 300, 250, 200];

pub struct Shell {
    model: Model,
    buttons: Buttons,
    entropy: u32,
    /// SoftDevice ready — BLE effects are applied.
    ble_ready: bool,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            model: Model::new(),
            buttons: Buttons::new(),
            entropy: 0xA5A5_1234,
            ble_ready: false,
        }
    }

    pub fn mark_ble_ready(&mut self) {
        self.ble_ready = true;
    }

    /// Mode-select only (no SoftDevice yet).
    pub async fn run_mode_select(&mut self) {
        Buttons::init();
        self.dispatch(Event::Tick).await;
        while view(&self.model).screen == Screen::ModeSelect {
            self.poll_buttons().await;
            self.dispatch(Event::Tick).await;
            speaker::tone_service();
            time::delay_ms_async(UI_TICK_MS).await;
        }
    }

    /// Apply radio setup after SoftDevice is up, then run picker/receiver forever.
    pub async fn run_game(&mut self) -> ! {
        self.bootstrap_radio().await;
        loop {
            match view(&self.model).screen {
                Screen::ModeSelect => {
                    // Should not happen after mode select finished.
                    time::delay_ms_async(UI_TICK_MS).await;
                }
                Screen::Picker => {
                    self.poll_buttons().await;
                    self.dispatch(Event::Tick).await;
                    speaker::tone_service();
                    time::delay_ms_async(UI_TICK_MS).await;
                }
                Screen::Receiver => {
                    if self.model.awaiting_entropy {
                        let b = self.next_entropy();
                        self.dispatch(Event::Entropy(b)).await;
                        continue;
                    }
                    if self.model.countdown_active {
                        // Beat: tone+render via Tick, then poll remainder for match/abort.
                        self.dispatch(Event::Tick).await;
                        if !self.model.countdown_active {
                            continue;
                        }
                        let _ = self.poll_match_or_abort(800).await;
                    } else {
                        time::delay_ms_async(UI_TICK_MS).await;
                    }
                }
            }
        }
    }

    async fn bootstrap_radio(&mut self) {
        match view(&self.model).screen {
            Screen::Picker => {
                let ch = self.model.channel();
                ble_broadcast::set_radio_picker(ch as usize);
                ble_broadcast::set_picker_broadcast(None);
            }
            Screen::Receiver => {
                // Round starts via Entropy in the loop.
            }
            Screen::ModeSelect => {}
        }
        self.dispatch(Event::Tick).await;
    }

    async fn poll_buttons(&mut self) {
        match self.buttons.poll() {
            buttons::Event::Left => self.dispatch(Event::ButtonLeft).await,
            buttons::Event::Right => self.dispatch(Event::ButtonRight).await,
            buttons::Event::None => {}
        }
    }

    async fn poll_match_or_abort(&mut self, duration_ms: u32) -> bool {
        const SLICE_MS: u32 = 50;
        let slices = duration_ms / SLICE_MS;
        for _ in 0..slices {
            if self.buttons.poll() == buttons::Event::Left {
                self.dispatch(Event::ButtonLeft).await;
                return true;
            }
            if ble_broadcast::take_color_match() {
                let target = self.model.receiver_target;
                let ch = self.model.channel();
                self.dispatch(Event::BleColorReceived {
                    channel: ch,
                    color: target,
                })
                .await;
                return true;
            }
            speaker::tone_service();
            time::delay_ms_async(SLICE_MS).await;
        }
        false
    }

    async fn dispatch(&mut self, event: Event) {
        let effects = update(event, &mut self.model);
        for effect in effects.iter() {
            self.apply_effect(effect).await;
        }
        let vm = view(&self.model);
        let leds = vm.leds.map(to_hw_rgb);
        neopixel_pwm::show_pixels(&leds);
    }

    async fn apply_effect(&mut self, effect: Effect) {
        match effect {
            Effect::Render => {}
            Effect::PlayTone(kind) => match kind {
                ToneKind::Skip => speaker::play_skip(),
                ToneKind::Apply => speaker::play_apply(),
                ToneKind::CountdownStep(step) => {
                    let hz = DESCENDING_HZ[(step as usize).min(NUM_LEDS - 1)];
                    speaker::play_tone(hz, 200, None).await;
                }
                ToneKind::Explosion => speaker::play_explosion().await,
            },
            Effect::SetBlePicker { channel } => {
                if self.ble_ready {
                    ble_broadcast::set_radio_picker(channel as usize);
                }
            }
            Effect::SetBleReceiver { channel, color } => {
                if self.ble_ready {
                    ble_broadcast::set_radio_receiver(to_hw_rgb(color), channel as usize);
                }
            }
            Effect::SetPickerBroadcast { color } => {
                if self.ble_ready {
                    ble_broadcast::set_picker_broadcast(color.map(to_hw_rgb));
                }
            }
            Effect::ClearBle => {
                if self.ble_ready {
                    ble_broadcast::set_radio_off();
                }
            }
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

fn to_hw_rgb(c: CoreRgb) -> Rgb {
    Rgb {
        r: c.r,
        g: c.g,
        b: c.b,
    }
}
