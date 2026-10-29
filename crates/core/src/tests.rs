extern crate alloc;

use alloc::vec::Vec;
use crux_core::{App as _, Command};

use crate::{
    color_from_entropy, decode_color, encode_color, BleOp, Color, ColorHurry, Effect, Event, Model,
    Round, Screen, BEAT_PERIOD_MS, COUNTDOWN_STEPS, NUM_LEDS,
};

fn effects(cmd: &mut Command<Effect, Event>) -> Vec<Effect> {
    cmd.effects().filter(|e| !e.is_render()).collect()
}

fn enter_picker(app: &ColorHurry, model: &mut Model) {
    let mut cmd = app.update(Event::ActivateColor, model);
    let _ = effects(&mut cmd);
    assert_eq!(model.screen, Screen::Picker);
}

fn enter_receiver(app: &ColorHurry, model: &mut Model) {
    for _ in 0..5 {
        let mut cmd = app.update(Event::Skip, model);
        let _ = effects(&mut cmd);
    }
    let mut cmd = app.update(Event::ActivateColor, model);
    let _ = effects(&mut cmd);
    assert_eq!(model.screen, Screen::Receiver);
    assert!(matches!(model.round, Round::Waiting));
}

fn start_round_with(app: &ColorHurry, model: &mut Model, byte: u8) -> Color {
    let mut cmd = app.update(Event::RandomByte(byte), model);
    let effs = effects(&mut cmd);
    assert!(effs.iter().any(|e| matches!(e, Effect::Ble(_))));
    assert!(effs.iter().any(|e| matches!(e, Effect::Delay(_))));
    // Full ring still lit; first BeatDone will play tone 0 / extinguish LED 0.
    match model.round {
        Round::Running { target, step: 0 } => target,
        other => panic!("expected running round at step 0, got {other:?}"),
    }
}

#[test]
fn mode_select_activate_enters_picker() {
    let app = ColorHurry;
    let mut model = Model::new();
    let mut cmd = app.update(Event::ActivateColor, &mut model);
    let effs = effects(&mut cmd);
    assert_eq!(model.screen, Screen::Picker);
    assert!(effs.iter().any(|e| matches!(
        e,
        Effect::Ble(r) if matches!(r.operation.op, BleOp::Picker { channel: 0 })
    )));
}

#[test]
fn mode_select_left_then_activate_enters_receiver() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_receiver(&app, &mut model);
    let mut cmd = app.update(Event::RandomByte(3), &mut model);
    let effs = effects(&mut cmd);
    assert!(matches!(model.round, Round::Running { .. }));
    assert!(effs.iter().any(|e| matches!(
        e,
        Effect::Ble(r) if matches!(r.operation.op, BleOp::Receiver { .. })
    )));
}

#[test]
fn match_only_through_core_no_explosion() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_receiver(&app, &mut model);
    let target = start_round_with(&app, &mut model, 1);
    let ch = model.channel();
    let mut cmd = app.update(
        Event::ColorReceived {
            channel: ch,
            color: target,
        },
        &mut model,
    );
    let effs = effects(&mut cmd);
    assert!(matches!(model.round, Round::Waiting));
    assert!(effs.iter().any(|e| matches!(e, Effect::Random(_))));
    assert!(effs.iter().any(|e| matches!(
        e,
        Effect::Ble(r) if matches!(r.operation.op, BleOp::Off)
    )));
}

#[test]
fn wrong_channel_ignored() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_receiver(&app, &mut model);
    let target = start_round_with(&app, &mut model, 2);
    let ch = model.channel();
    let mut cmd = app.update(
        Event::ColorReceived {
            channel: (ch + 1) % 5,
            color: target,
        },
        &mut model,
    );
    assert!(effects(&mut cmd).is_empty());
    assert!(matches!(model.round, Round::Running { .. }));
}

#[test]
fn wrong_color_ignored() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_receiver(&app, &mut model);
    let target = start_round_with(&app, &mut model, 2);
    let other = Color::from_index((target.index() + 1) % NUM_LEDS as u8).unwrap();
    let mut cmd = app.update(
        Event::ColorReceived {
            channel: model.channel(),
            color: other,
        },
        &mut model,
    );
    assert!(effects(&mut cmd).is_empty());
    assert!(matches!(model.round, Round::Running { .. }));
}

#[test]
fn after_beats_explosion_then_random() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_receiver(&app, &mut model);
    let _ = start_round_with(&app, &mut model, 0);
    let mut last_effs = Vec::new();
    for _ in 0..=COUNTDOWN_STEPS {
        let id = match model.round {
            Round::Running { .. } => model.beat_gen,
            Round::Waiting => break,
        };
        let mut cmd = app.update(Event::BeatDone(id), &mut model);
        last_effs = effects(&mut cmd);
    }
    assert!(matches!(model.round, Round::Waiting));
    assert!(last_effs.iter().any(|e| matches!(e, Effect::Random(_))));
    assert!(last_effs.iter().any(|e| matches!(e, Effect::PlayTone(_))));
}

#[test]
fn palette_identity_entropy_matches_index() {
    for i in 0..NUM_LEDS {
        let c = color_from_entropy(i as u8);
        assert_eq!(c.index() as usize, i % NUM_LEDS);
        assert_eq!(color_from_entropy(i as u8 + 10 * NUM_LEDS as u8), c);
    }
}

#[test]
fn protocol_roundtrip_and_rejects() {
    let bytes = encode_color(2, Color::Blue).unwrap();
    assert_eq!(decode_color(&bytes), Some((2, Color::Blue)));
    assert!(decode_color(&[b'X', 4, 0x03, 0, 1]).is_none());
    assert!(encode_color(9, Color::Red).is_none());
}

#[test]
fn picker_apply_broadcasts_named_color() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_picker(&app, &mut model);
    let mut cmd = app.update(Event::ActivateColor, &mut model);
    let effs = effects(&mut cmd);
    assert!(effs.iter().any(|e| matches!(
        e,
        Effect::Ble(r) if matches!(
            r.operation.op,
            BleOp::Broadcast {
                color: Some(Color::White)
            }
        )
    )));
}

#[test]
fn beat_delay_period_is_core_constant() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_receiver(&app, &mut model);
    let mut cmd = app.update(Event::RandomByte(0), &mut model);
    let first = effects(&mut cmd)
        .iter()
        .find_map(|e| match e {
            Effect::Delay(d) => Some(d.operation.millis),
            _ => None,
        })
        .expect("first delay");
    // Round start kicks the first beat immediately (0 ms), then BEAT_PERIOD_MS.
    assert_eq!(first, 0);

    let id = model.beat_gen;
    let mut cmd = app.update(Event::BeatDone(id), &mut model);
    let next = effects(&mut cmd)
        .iter()
        .find_map(|e| match e {
            Effect::Delay(d) => Some(d.operation.millis),
            _ => None,
        })
        .expect("follow-up delay");
    assert_eq!(next, BEAT_PERIOD_MS);
}
