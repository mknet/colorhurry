//! Node-API shell around [`color_hurry_core::ColorHurry`].
//!
//! Timers and random stay in Rust (Crux request handles never cross into JS).
//! BLE ops, tones and the view are returned for the Node.js shell to perform.

use std::collections::VecDeque;
use std::time::Instant;

use color_hurry_core::{
    time::operation::NotifyAfter, BleOp, Color, ColorHurry, Effect, Event, ModeRole, Screen,
    COMPANY_ID, NUM_LEDS,
};
use color_hurry_shell::radio::{build_adv, parse_color_from_adv};
use crux_core::{Core, Request};
use napi::bindgen_prelude::*;
use napi_derive::napi;

struct PendingTimer {
    due: Instant,
    request: Request<NotifyAfter>,
}

#[napi(object)]
pub struct JsRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[napi(object)]
pub struct JsModeChoice {
    pub role: String,
    pub channel: u8,
}

#[napi(object)]
pub struct JsViewModel {
    pub screen: String,
    pub leds: Vec<JsRgb>,
    pub channel: u8,
    pub countdown_step: u8,
    pub countdown_active: bool,
    pub mode_choice: Option<JsModeChoice>,
    pub picker_color: Option<u8>,
    pub picker_applied: bool,
}

#[napi(object)]
pub struct JsToneStep {
    pub hz: u32,
    pub ms: u32,
}

#[napi(object)]
pub struct JsBleOp {
    /// `"off"` | `"picker"` | `"caller"` | `"broadcast"`
    pub kind: String,
    pub channel: Option<u8>,
    /// Named color index when present.
    pub color: Option<u8>,
}

#[napi(object)]
pub struct ShellUpdate {
    pub view: JsViewModel,
    pub tones: Vec<Vec<JsToneStep>>,
    pub ble: Vec<JsBleOp>,
}

#[napi(object)]
pub struct DecodedColor {
    pub channel: u8,
    pub color: u8,
}

#[napi]
pub struct Shell {
    core: Core<ColorHurry>,
    entropy: u32,
    timers: Vec<PendingTimer>,
}

#[napi]
impl Shell {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            core: Core::new(),
            entropy: 0xC0DE_4A11,
            timers: Vec::new(),
        }
    }

    /// Dispatch a shell event by name.
    ///
    /// Names: `boot`, `skip`, `apply`, `reset`, `pause`, `resume`, `callerOnly`.
    #[napi]
    pub fn dispatch(&mut self, event: String) -> Result<ShellUpdate> {
        let event = match event.as_str() {
            "boot" => Event::Boot,
            "skip" => Event::Skip,
            "apply" => Event::ActivateColor,
            "reset" => Event::Reset,
            "pause" => Event::Pause,
            "resume" => Event::Resume,
            "callerOnly" => Event::CallerOnly,
            other => {
                return Err(Error::from_reason(format!("unknown event: {other}")));
            }
        };
        Ok(self.process(event))
    }

    /// Feed a decoded color advertisement into the core.
    #[napi]
    pub fn ingest_color(&mut self, channel: u8, color_index: u8) -> Result<ShellUpdate> {
        let color = Color::from_index(color_index)
            .ok_or_else(|| Error::from_reason(format!("invalid color index: {color_index}")))?;
        Ok(self.process(Event::ColorReceived { channel, color }))
    }

    /// Resolve timers that are due. Call on a short interval from Node.
    #[napi]
    pub fn poll(&mut self) -> ShellUpdate {
        let now = Instant::now();
        let mut i = 0;
        let mut effects = Vec::new();
        while i < self.timers.len() {
            if self.timers[i].due <= now {
                let PendingTimer { mut request, .. } = self.timers.swap_remove(i);
                let id = request.operation.id;
                effects.extend(self.core.resolve(&mut request, id).unwrap_or_default());
            } else {
                i += 1;
            }
        }
        self.handle(effects)
    }

    #[napi]
    pub fn view(&self) -> JsViewModel {
        to_js_view(&self.core.view())
    }
}

impl Shell {
    fn process(&mut self, event: Event) -> ShellUpdate {
        let effects = self.core.process_event(event);
        self.handle(effects)
    }

    fn handle(&mut self, effects: Vec<Effect>) -> ShellUpdate {
        let mut queue: VecDeque<Effect> = effects.into();
        let mut tones = Vec::new();
        let mut ble = Vec::new();

        while let Some(effect) = queue.pop_front() {
            match effect {
                Effect::Render(_) => {}
                Effect::TimeNotifyAfter(request) => {
                    let duration = std::time::Duration::from(request.operation.duration);
                    self.timers.push(PendingTimer {
                        due: Instant::now() + duration,
                        request,
                    });
                }
                Effect::TimeClear(mut request) => {
                    let id = request.operation.id;
                    self.timers.retain(|timer| timer.request.operation.id != id);
                    let more = self.core.resolve(&mut request, id).unwrap_or_default();
                    queue.extend(more);
                }
                Effect::Random(mut request) => {
                    let byte = self.next_entropy();
                    let more = self.core.resolve(&mut request, byte).unwrap_or_default();
                    queue.extend(more);
                }
                Effect::PlayTone(request) => {
                    tones.push(
                        request
                            .operation
                            .sequence
                            .steps()
                            .iter()
                            .map(|s| JsToneStep {
                                hz: s.hz,
                                ms: s.ms,
                            })
                            .collect(),
                    );
                }
                Effect::Ble(request) => ble.push(to_js_ble(request.operation.op)),
            }
        }

        ShellUpdate {
            view: to_js_view(&self.core.view()),
            tones,
            ble,
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

fn to_js_view(vm: &color_hurry_core::ViewModel) -> JsViewModel {
    let leds = vm
        .leds
        .iter()
        .map(|c| JsRgb {
            r: c.r,
            g: c.g,
            b: c.b,
        })
        .collect();
    let screen = match vm.screen {
        Screen::ModeSelect => "modeSelect",
        Screen::Picker => "picker",
        Screen::Caller => "caller",
    }
    .to_string();
    let mode_choice = vm.mode_choice.map(|c| JsModeChoice {
        role: match c.role {
            ModeRole::Picker => "picker",
            ModeRole::Caller => "caller",
        }
        .to_string(),
        channel: c.channel,
    });
    JsViewModel {
        screen,
        leds,
        channel: vm.channel,
        countdown_step: vm.countdown_step,
        countdown_active: vm.countdown_active,
        mode_choice,
        picker_color: vm.picker_color.map(|c| c.index()),
        picker_applied: vm.picker_applied,
    }
}

fn to_js_ble(op: BleOp) -> JsBleOp {
    match op {
        BleOp::Off => JsBleOp {
            kind: "off".into(),
            channel: None,
            color: None,
        },
        BleOp::Picker { channel } => JsBleOp {
            kind: "picker".into(),
            channel: Some(channel),
            color: None,
        },
        BleOp::Caller { channel, color } => JsBleOp {
            kind: "caller".into(),
            channel: Some(channel),
            color: Some(color.index()),
        },
        BleOp::Broadcast { color } => JsBleOp {
            kind: "broadcast".into(),
            channel: None,
            color: color.map(|c| c.index()),
        },
    }
}

/// Decode Color Hurry manufacturer data: `[company_lo, company_hi, payload…]`.
#[napi]
pub fn decode_manufacturer_data(data: Buffer) -> Option<DecodedColor> {
    if data.len() < 2 {
        return None;
    }
    let company = u16::from_le_bytes([data[0], data[1]]);
    if company != COMPANY_ID {
        return None;
    }
    let (channel, color) = color_hurry_core::decode_color(&data[2..])?;
    Some(DecodedColor {
        channel,
        color: color.index(),
    })
}

/// Parse a full advertising PDU (AD structures) for a Color Hurry color.
#[napi]
pub fn decode_advertisement(data: Buffer) -> Option<DecodedColor> {
    let (channel, color) = parse_color_from_adv(&data)?;
    Some(DecodedColor {
        channel,
        color: color.index(),
    })
}

/// Build legacy advertising bytes (flags, short name, manufacturer payload).
#[napi]
pub fn encode_advertisement(channel: u8, color_index: u8) -> Result<Buffer> {
    let color = Color::from_index(color_index)
        .ok_or_else(|| Error::from_reason(format!("invalid color index: {color_index}")))?;
    let (buf, len) = build_adv(channel, color)
        .ok_or_else(|| Error::from_reason(format!("invalid channel: {channel}")))?;
    Ok(Buffer::from(&buf[..len]))
}

/// Adafruit company ID used in Color Hurry manufacturer data.
#[napi]
pub fn company_id() -> u16 {
    COMPANY_ID
}

/// Number of LEDs on the ring.
#[napi]
pub fn num_leds() -> u32 {
    NUM_LEDS as u32
}
