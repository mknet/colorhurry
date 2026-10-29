//! Browser shell: Crux `Core<ColorHurry>` + Web Bluetooth receive.

mod ble;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use color_hurry_core::{
    BleOp, Color, ColorHurry, Effect, Event, NUM_LEDS,
};
use crux_core::{Core, Request};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlButtonElement, HtmlElement};

struct PendingDelay {
    due_ms: f64,
    request: Request<color_hurry_core::Delay>,
}

struct Shell {
    core: Core<ColorHurry>,
    entropy: u32,
    meta: Element,
    leds: Vec<Element>,
    delays: VecDeque<PendingDelay>,
}

impl Shell {
    fn new(doc: &Document, parent: &Element) -> Result<Self, JsValue> {
        let root = doc.create_element("div")?;
        root.set_class_name("board");

        let h2 = doc.create_element("h2")?;
        h2.set_text_content(Some("Receiver"));
        root.append_child(&h2)?;

        let meta = doc.create_element("div")?;
        meta.set_class_name("meta");
        root.append_child(&meta)?;

        let face = doc.create_element("div")?;
        face.set_class_name("board-face");
        let ring = doc.create_element("div")?;
        ring.set_class_name("ring");
        let mut leds = Vec::with_capacity(NUM_LEDS);
        for i in 0..NUM_LEDS {
            let led = doc.create_element("div")?;
            led.set_class_name("led");
            let angle = -90.0 + (i as f64) * 36.0;
            if let Ok(html) = led.clone().dyn_into::<HtmlElement>() {
                let _ = html.style().set_property(
                    "transform",
                    &format!("rotate({angle}deg) translateY(var(--led-radius))"),
                );
            }
            ring.append_child(&led)?;
            leds.push(led);
        }
        face.append_child(&ring)?;
        root.append_child(&face)?;

        let actions = doc.create_element("div")?;
        actions.set_class_name("actions");

        let btn_a = doc
            .create_element("button")?
            .dyn_into::<HtmlButtonElement>()?;
        btn_a.set_id("btn-a");
        btn_a.set_class_name("secondary");
        btn_a.set_inner_text("Button A (skip)");
        let btn_b = doc
            .create_element("button")?
            .dyn_into::<HtmlButtonElement>()?;
        btn_b.set_id("btn-b");
        btn_b.set_inner_text("Button B (apply)");
        actions.append_child(&btn_a)?;
        actions.append_child(&btn_b)?;
        root.append_child(&actions)?;
        parent.append_child(&root)?;

        Ok(Self {
            core: Core::new(),
            entropy: 0xC0FF_EE42,
            meta,
            leds,
            delays: VecDeque::new(),
        })
    }

    fn next_entropy(&mut self) -> u8 {
        self.entropy = self
            .entropy
            .wrapping_mul(1_103_515_245)
            .wrapping_add(12_345);
        (self.entropy >> 16) as u8
    }

    fn now_ms() -> f64 {
        js_sys::Date::now()
    }

    fn paint(&self) {
        let vm = self.core.view();
        self.meta.set_text_content(Some(&format!(
            "Screen: {:?} · channel {}",
            vm.screen, vm.channel
        )));
        for (el, c) in self.leds.iter().zip(vm.leds.iter()) {
            if let Ok(html) = el.clone().dyn_into::<HtmlElement>() {
                let _ = html
                    .style()
                    .set_property("background", &format!("rgb({},{},{})", c.r, c.g, c.b));
            }
        }
    }

    fn process(&mut self, event: Event) {
        let effects = self.core.process_event(event);
        self.handle(effects);
    }

    fn handle(&mut self, effects: Vec<Effect>) {
        let mut queue: VecDeque<Effect> = effects.into();
        let mut tones = Vec::new();
        // Start Delay timers before WebAudio work so beat period is wall-clock.
        while let Some(effect) = queue.pop_front() {
            match effect {
                Effect::Render(_) => self.paint(),
                Effect::Delay(request) => {
                    let due = Self::now_ms() + f64::from(request.operation.millis);
                    self.delays.push_back(PendingDelay { due_ms: due, request });
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
                    // Web demo is Receiver-only; inbound ads use watchAdvertisements.
                    let _ = request.operation.op;
                    let _ = BleOp::Off;
                }
            }
        }
        for seq in tones {
            play_sequence(seq.steps());
        }
    }

    fn resolve_due(&mut self) {
        let now = Self::now_ms();
        let mut i = 0;
        while i < self.delays.len() {
            if self.delays[i].due_ms <= now {
                if let Some(PendingDelay { mut request, .. }) = self.delays.remove(i) {
                    let more = self.core.resolve(&mut request, ()).unwrap_or_default();
                    self.handle(more);
                }
            } else {
                i += 1;
            }
        }
    }

    fn ingest_live_ble(&mut self, channel: u8, color: Color) {
        self.process(Event::ColorReceived { channel, color });
    }
}

fn play_sequence(steps: &[color_hurry_core::ToneStep]) {
    let _ = (|| -> Result<(), JsValue> {
        let ctx = web_sys::AudioContext::new()?;
        let mut t = ctx.current_time();
        for step in steps {
            let osc = ctx.create_oscillator()?;
            let gain = ctx.create_gain()?;
            osc.frequency().set_value(step.hz as f32);
            gain.gain().set_value(0.08);
            osc.connect_with_audio_node(&gain)?;
            gain.connect_with_audio_node(&ctx.destination())?;
            osc.start_with_when(t)?;
            let end = t + f64::from(step.ms) / 1000.0;
            osc.stop_with_when(end)?;
            t = end;
        }
        Ok(())
    })();
}

struct LiveBleUi {
    link: Option<ble::BoardLink>,
    _listeners: Vec<Closure<dyn FnMut(web_sys::Event)>>,
    status: Element,
    button: HtmlButtonElement,
}

impl LiveBleUi {
    fn set_status(&self, text: &str) {
        self.status.set_text_content(Some(text));
    }

    fn set_idle(&mut self) {
        self.link = None;
        self._listeners.clear();
        self.button.set_inner_text("Connect board");
        self.button.set_disabled(false);
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let doc = window.document().unwrap();
    let body = doc.body().unwrap();

    let header = doc.create_element("header")?;
    header.set_inner_html(
        "<h1>Color Hurry</h1>\
         <p>Local link (no server): choose the CPB in Chrome, then this tab watches its color ads as Receiver.</p>",
    );
    body.append_child(&header)?;

    let ble_bar = doc.create_element("div")?;
    ble_bar.set_class_name("ble-bar");
    let ble_btn = doc
        .create_element("button")?
        .dyn_into::<HtmlButtonElement>()?;
    ble_btn.set_id("ble-scan");
    ble_btn.set_inner_text("Connect board");
    let ble_status = doc.create_element("div")?;
    ble_status.set_class_name("ble-status");
    ble_status.set_text_content(Some(
        "1) CPB: Picker + apply color  2) Connect board  3) This page: Receiver, same channel  · Flag: experimental Web Platform features",
    ));
    ble_bar.append_child(&ble_btn)?;
    ble_bar.append_child(&ble_status)?;
    body.append_child(&ble_bar)?;

    let stage = doc.create_element("div")?;
    stage.set_class_name("stage");
    body.append_child(&stage)?;

    let shell = Rc::new(RefCell::new(Shell::new(&doc, &stage)?));
    let live = Rc::new(RefCell::new(LiveBleUi {
        link: None,
        _listeners: Vec::new(),
        status: ble_status,
        button: ble_btn.clone(),
    }));

    shell.borrow_mut().process(Event::Boot);

    bind_button(&doc, "btn-a", {
        let shell = shell.clone();
        move || shell.borrow_mut().process(Event::Skip)
    })?;
    bind_button(&doc, "btn-b", {
        let shell = shell.clone();
        move || shell.borrow_mut().process(Event::ActivateColor)
    })?;

    {
        let shell = shell.clone();
        let live = live.clone();
        let closure = Closure::wrap(Box::new(move || {
            let already = live.borrow().link.is_some();
            if already {
                live.borrow_mut().set_idle();
                live.borrow().set_status("Disconnected.");
                return;
            }
            live.borrow().button.set_disabled(true);
            live.borrow()
                .set_status("Choose ColorHurry-CPB in the Chrome dialog…");
            let shell = shell.clone();
            let live = live.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match ble::request_board().await {
                    Ok((bt, device)) => {
                        let link = ble::BoardLink { bt, device };
                        let name = link.device_label();
                        let targets = link.event_targets();
                        let mut listeners = Vec::new();
                        for target in targets {
                            let shell = shell.clone();
                            let live_cb = live.clone();
                            let name_cb = name.clone();
                            match ble::attach_advertisement_listener(&target, move |adv| {
                                if let Some((ch, color)) = ble::color_from_advertisement(&adv) {
                                    live_cb.borrow().set_status(&format!(
                                        "Linked to {name_cb} · ch{} color {:?}",
                                        ch, color
                                    ));
                                    shell.borrow_mut().ingest_live_ble(ch, color);
                                } else if let Some(hex) = ble::manufacturer_hex(&adv) {
                                    live_cb.borrow().set_status(&format!(
                                        "Linked to {name_cb} · ad seen, not CMD_COLOR yet ({hex})"
                                    ));
                                } else {
                                    live_cb.borrow().set_status(&format!(
                                        "Linked to {name_cb} · advertisement (no mfg 0x239A payload)"
                                    ));
                                }
                            }) {
                                Ok(c) => listeners.push(c),
                                Err(e) => {
                                    live.borrow().set_status(&format!("Listener error: {e:?}"));
                                }
                            }
                        }

                        let watch = ble::watch_board(&link.device).await;
                        match watch {
                            Ok(()) => {
                                let mut ui = live.borrow_mut();
                                ui.link = Some(link);
                                ui._listeners = listeners;
                                ui.button.set_inner_text("Disconnect");
                                ui.button.set_disabled(false);
                                ui.set_status(&format!(
                                    "Linked to {name} (ads only, no GATT). Waiting for picker color broadcasts…"
                                ));
                            }
                            Err(e) => {
                                let mut ui = live.borrow_mut();
                                ui.set_idle();
                                ui.set_status(&format!(
                                    "Board chosen but watchAdvertisements failed: {e:?}"
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        let mut ui = live.borrow_mut();
                        ui.set_idle();
                        ui.set_status(&format!("Connect cancelled/failed: {e:?}"));
                    }
                }
            });
        }) as Box<dyn FnMut()>);
        ble_btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    // Resolve Delay effects only (blink + beat); no model peeks.
    let shell_tick = shell;
    let tick_cb = Closure::wrap(Box::new(move || {
        shell_tick.borrow_mut().resolve_due();
    }) as Box<dyn FnMut()>);
    window.set_interval_with_callback_and_timeout_and_arguments_0(
        tick_cb.as_ref().unchecked_ref(),
        10,
    )?;
    tick_cb.forget();

    Ok(())
}

fn bind_button<F>(doc: &Document, id: &str, mut f: F) -> Result<(), JsValue>
where
    F: FnMut() + 'static,
{
    let el = doc
        .get_element_by_id(id)
        .ok_or_else(|| JsValue::from_str("missing button"))?;
    let btn: HtmlButtonElement = el.dyn_into()?;
    let closure = Closure::wrap(Box::new(move || f()) as Box<dyn FnMut()>);
    btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
    Ok(())
}
