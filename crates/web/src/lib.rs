//! Browser shell: two boards (sim radio) + experimental live BLE receive for Receiver.

mod ble;

use std::cell::RefCell;
use std::rc::Rc;

use color_hurry_core::{
    update, view as core_view, Effect as CoreEffect, Event, Model, Rgb, Screen, ToneKind,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, HtmlButtonElement, HtmlElement};

struct BoardState {
    model: Model,
    broadcast: Option<(u8, Rgb)>,
    entropy: u32,
    meta: Element,
    leds: Vec<Element>,
}

impl BoardState {
    fn new(doc: &Document, parent: &Element, title: &str, entropy: u32) -> Result<Self, JsValue> {
        let root = doc.create_element("div")?;
        root.set_class_name("board");

        let h2 = doc.create_element("h2")?;
        h2.set_text_content(Some(title));
        root.append_child(&h2)?;

        let meta = doc.create_element("div")?;
        meta.set_class_name("meta");
        root.append_child(&meta)?;

        let ring = doc.create_element("div")?;
        ring.set_class_name("ring");
        let mut leds = Vec::with_capacity(10);
        for _ in 0..10 {
            let led = doc.create_element("div")?;
            led.set_class_name("led");
            ring.append_child(&led)?;
            leds.push(led);
        }
        root.append_child(&ring)?;

        let actions = doc.create_element("div")?;
        actions.set_class_name("actions");

        let id_a = format!("{title}-a");
        let id_b = format!("{title}-b");
        let btn_a = doc
            .create_element("button")?
            .dyn_into::<HtmlButtonElement>()?;
        btn_a.set_id(&id_a);
        btn_a.set_class_name("secondary");
        btn_a.set_inner_text("Button A (skip)");
        let btn_b = doc
            .create_element("button")?
            .dyn_into::<HtmlButtonElement>()?;
        btn_b.set_id(&id_b);
        btn_b.set_inner_text("Button B (apply)");
        actions.append_child(&btn_a)?;
        actions.append_child(&btn_b)?;
        root.append_child(&actions)?;
        parent.append_child(&root)?;

        Ok(Self {
            model: Model::new(),
            broadcast: None,
            entropy,
            meta,
            leds,
        })
    }

    fn next_entropy(&mut self) -> u8 {
        self.entropy = self
            .entropy
            .wrapping_mul(1_103_515_245)
            .wrapping_add(12_345);
        (self.entropy >> 16) as u8
    }

    fn paint(&self) {
        let vm = core_view(&self.model);
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
}

struct SimBus {
    left: BoardState,
    right: BoardState,
}

impl SimBus {
    fn dispatch_left(&mut self, event: Event) {
        Self::dispatch(&mut self.left, &mut self.right, event);
    }

    fn dispatch_right(&mut self, event: Event) {
        Self::dispatch(&mut self.right, &mut self.left, event);
    }

    fn dispatch(me: &mut BoardState, peer: &mut BoardState, event: Event) {
        let effects = update(event, &mut me.model);
        for effect in effects.iter() {
            apply_effect(me, peer, effect);
        }
        if me.model.awaiting_entropy && me.model.screen == Screen::Receiver {
            let byte = me.next_entropy();
            Self::dispatch(me, peer, Event::Entropy(byte));
        }
        me.paint();
        peer.paint();
    }

    fn ingest_live_ble(&mut self, channel: u8, color: Rgb) {
        let event = Event::BleColorReceived { channel, color };
        {
            let SimBus { left, right } = self;
            Self::dispatch(left, right, event);
        }
        {
            let SimBus { left, right } = self;
            Self::dispatch(right, left, event);
        }
    }

    fn ui_ticks(&mut self) {
        if matches!(self.left.model.screen, Screen::ModeSelect | Screen::Picker) {
            let SimBus { left, right } = self;
            Self::dispatch(left, right, Event::Tick);
        }
        if matches!(self.right.model.screen, Screen::ModeSelect | Screen::Picker) {
            let SimBus { left, right } = self;
            Self::dispatch(right, left, Event::Tick);
        }
        self.left.paint();
        self.right.paint();
    }

    fn countdown_ticks(&mut self) {
        if self.left.model.screen == Screen::Receiver && self.left.model.countdown_active {
            let SimBus { left, right } = self;
            Self::dispatch(left, right, Event::Tick);
        }
        if self.right.model.screen == Screen::Receiver && self.right.model.countdown_active {
            let SimBus { left, right } = self;
            Self::dispatch(right, left, Event::Tick);
        }
        self.left.paint();
        self.right.paint();
    }
}

fn apply_effect(me: &mut BoardState, peer: &mut BoardState, effect: CoreEffect) {
    match effect {
        CoreEffect::Render => {}
        CoreEffect::PlayTone(kind) => play_tone(kind),
        CoreEffect::SetBlePicker { .. } => {
            me.broadcast = None;
        }
        CoreEffect::SetPickerBroadcast { color } => {
            let ch = me.model.channel();
            me.broadcast = color.map(|c| (ch, c));
            if let Some((ch, c)) = me.broadcast {
                let effects = update(
                    Event::BleColorReceived {
                        channel: ch,
                        color: c,
                    },
                    &mut peer.model,
                );
                for e in effects.iter() {
                    apply_effect(peer, me, e);
                }
            }
        }
        CoreEffect::SetBleReceiver { .. } => {}
        CoreEffect::ClearBle => {
            me.broadcast = None;
        }
    }
}

fn play_tone(kind: ToneKind) {
    let (freq, ms) = match kind {
        ToneKind::Skip => (380.0, 55.0),
        ToneKind::Apply => (784.0, 130.0),
        ToneKind::CountdownStep(step) => {
            let table = [
                680.0, 620.0, 560.0, 500.0, 450.0, 400.0, 350.0, 300.0, 250.0, 200.0,
            ];
            (table[step.min(9) as usize], 200.0)
        }
        ToneKind::Explosion => (120.0, 400.0),
    };
    let _ = (|| -> Result<(), JsValue> {
        let ctx = web_sys::AudioContext::new()?;
        let osc = ctx.create_oscillator()?;
        let gain = ctx.create_gain()?;
        osc.frequency().set_value(freq as f32);
        gain.gain().set_value(0.08);
        osc.connect_with_audio_node(&gain)?;
        gain.connect_with_audio_node(&ctx.destination())?;
        osc.start()?;
        osc.stop_with_when(ctx.current_time() + ms / 1000.0)?;
        Ok(())
    })();
}

struct LiveBleUi {
    scan: Option<ble::LiveScan>,
    /// Keep closures alive while listening.
    _listeners: Vec<Closure<dyn FnMut(web_sys::Event)>>,
    status: Element,
    button: HtmlButtonElement,
}

impl LiveBleUi {
    fn set_status(&self, text: &str) {
        self.status.set_text_content(Some(text));
    }

    fn set_idle(&mut self) {
        if let Some(scan) = self.scan.take() {
            scan.stop();
        }
        self._listeners.clear();
        self.button.set_inner_text("Live BLE scan");
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
         <p>Two boards share a simulated radio. Optional: listen for real CPB picker ads (Receiver).</p>",
    );
    body.append_child(&header)?;

    let ble_bar = doc.create_element("div")?;
    ble_bar.set_class_name("ble-bar");
    let ble_btn = doc
        .create_element("button")?
        .dyn_into::<HtmlButtonElement>()?;
    ble_btn.set_id("ble-scan");
    ble_btn.set_inner_text("Live BLE scan");
    let ble_status = doc.create_element("div")?;
    ble_status.set_class_name("ble-status");
    ble_status.set_text_content(Some(
        "Chrome/Edge + chrome://flags/#enable-experimental-web-platform-features · Receiver only",
    ));
    ble_bar.append_child(&ble_btn)?;
    ble_bar.append_child(&ble_status)?;
    body.append_child(&ble_bar)?;

    let boards = doc.create_element("div")?;
    boards.set_class_name("boards");
    body.append_child(&boards)?;

    let left = BoardState::new(&doc, &boards, "Board A", 0x1111_2222)?;
    let right = BoardState::new(&doc, &boards, "Board B", 0x3333_4444)?;
    let bus = Rc::new(RefCell::new(SimBus { left, right }));
    let live = Rc::new(RefCell::new(LiveBleUi {
        scan: None,
        _listeners: Vec::new(),
        status: ble_status,
        button: ble_btn.clone(),
    }));

    {
        let mut b = bus.borrow_mut();
        {
            let SimBus { left, right } = &mut *b;
            SimBus::dispatch(left, right, Event::Tick);
        }
        {
            let SimBus { left, right } = &mut *b;
            SimBus::dispatch(right, left, Event::Tick);
        }
        b.left.paint();
        b.right.paint();
    }

    bind_button(&doc, "Board A-a", {
        let bus = bus.clone();
        move || bus.borrow_mut().dispatch_left(Event::ButtonLeft)
    })?;
    bind_button(&doc, "Board A-b", {
        let bus = bus.clone();
        move || bus.borrow_mut().dispatch_left(Event::ButtonRight)
    })?;
    bind_button(&doc, "Board B-a", {
        let bus = bus.clone();
        move || bus.borrow_mut().dispatch_right(Event::ButtonLeft)
    })?;
    bind_button(&doc, "Board B-b", {
        let bus = bus.clone();
        move || bus.borrow_mut().dispatch_right(Event::ButtonRight)
    })?;

    {
        let bus = bus.clone();
        let live = live.clone();
        let closure = Closure::wrap(Box::new(move || {
            let already = live.borrow().scan.is_some();
            if already {
                live.borrow_mut().set_idle();
                live.borrow().set_status("Scan stopped.");
                return;
            }
            live.borrow().button.set_disabled(true);
            live.borrow().set_status("Requesting Bluetooth permission…");
            let bus = bus.clone();
            let live = live.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match ble::start_listening().await {
                    Ok(scan) => {
                        let label = scan.mode_label();
                        let targets = scan.event_targets();
                        let mut listeners = Vec::new();
                        for target in targets {
                            let bus = bus.clone();
                            let live_cb = live.clone();
                            match ble::attach_advertisement_listener(&target, move |adv| {
                                if let Some((ch, color)) = ble::color_from_advertisement(&adv) {
                                    live_cb.borrow().set_status(&format!(
                                        "Live: ch{} rgb({},{},{}) via {}",
                                        ch, color.r, color.g, color.b, label
                                    ));
                                    bus.borrow_mut().ingest_live_ble(ch, color);
                                }
                            }) {
                                Ok(c) => listeners.push(c),
                                Err(e) => {
                                    live.borrow().set_status(&format!("Listener error: {e:?}"));
                                }
                            }
                        }
                        {
                            let mut ui = live.borrow_mut();
                            ui.scan = Some(scan);
                            ui._listeners = listeners;
                            ui.button.set_inner_text("Stop BLE scan");
                            ui.button.set_disabled(false);
                            ui.set_status(&format!(
                                "Listening ({label}). Put a board in Receiver mode; CPB picker must broadcast."
                            ));
                        }
                    }
                    Err(e) => {
                        let mut ui = live.borrow_mut();
                        ui.set_idle();
                        ui.set_status(&format!(
                            "BLE start failed: {e:?}. Flag + user gesture required."
                        ));
                    }
                }
            });
        }) as Box<dyn FnMut()>);
        ble_btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    let bus_ui = bus.clone();
    let ui_cb = Closure::wrap(Box::new(move || {
        bus_ui.borrow_mut().ui_ticks();
    }) as Box<dyn FnMut()>);
    window.set_interval_with_callback_and_timeout_and_arguments_0(
        ui_cb.as_ref().unchecked_ref(),
        50,
    )?;
    ui_cb.forget();

    let bus_cd = bus;
    let cd_cb = Closure::wrap(Box::new(move || {
        bus_cd.borrow_mut().countdown_ticks();
    }) as Box<dyn FnMut()>);
    window.set_interval_with_callback_and_timeout_and_arguments_0(
        cd_cb.as_ref().unchecked_ref(),
        1000,
    )?;
    cd_cb.forget();

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
