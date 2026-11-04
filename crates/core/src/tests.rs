use crux_core::App as _;

use crate::{
    color_from_entropy, decode_color, encode_color, tones, BleOp, Color, ColorHurry,
    EffectTestExt as _, Event, Model, Round, Screen, BEAT_PERIOD_MS, COUNTDOWN_STEPS, NUM_LEDS,
};

fn enter_picker(app: &ColorHurry, model: &mut Model) {
    let _ = app.update(Event::ActivateColor, model);
    assert_eq!(model.screen, Screen::Picker);
}

fn enter_caller(app: &ColorHurry, model: &mut Model) {
    for _ in 0..5 {
        let _ = app.update(Event::Skip, model);
    }
    let _ = app.update(Event::ActivateColor, model);
    assert_eq!(model.screen, Screen::Caller);
    assert!(matches!(model.round, Round::Waiting));
}

fn start_round_with(app: &ColorHurry, model: &mut Model, byte: u8) -> Color {
    let mut cmd = app.update(Event::RandomByte(byte), model);
    // Full ring still lit; first BeatDone will play tone 0 / extinguish LED 0.
    let target = match model.round {
        Round::Running { target, step: 0 } => target,
        other => panic!("expected running round at step 0, got {other:?}"),
    };
    let caller = BleOp::Caller {
        channel: model.channel(),
        color: target,
    };
    cmd.expect_ble_with(|op| assert_eq!(op.op, caller))
        .expect_render()
        .expect_only_delay();
    target
}

#[test]
fn mode_select_activate_enters_picker() {
    let app = ColorHurry;
    let mut model = Model::new();
    let mut cmd = app.update(Event::ActivateColor, &mut model);
    assert_eq!(model.screen, Screen::Picker);
    cmd.expect_play_tone()
        .expect_ble_with(|op| assert_eq!(op.op, BleOp::Picker { channel: 0 }))
        .expect_ble_with(|op| assert_eq!(op.op, BleOp::Broadcast { color: None }))
        .expect_render()
        .expect_only_delay();
}

#[test]
fn mode_select_left_then_activate_enters_caller() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_caller(&app, &mut model);
    let mut cmd = app.update(Event::RandomByte(3), &mut model);
    assert!(matches!(model.round, Round::Running { .. }));
    let caller = BleOp::Caller {
        channel: model.channel(),
        color: color_from_entropy(3),
    };
    cmd.expect_ble_with(|op| assert_eq!(op.op, caller))
        .expect_render()
        .expect_only_delay();
}

#[test]
fn match_only_through_core_no_explosion() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_caller(&app, &mut model);
    let target = start_round_with(&app, &mut model, 1);
    let ch = model.channel();
    let mut cmd = app.update(
        Event::ColorReceived {
            channel: ch,
            color: target,
        },
        &mut model,
    );
    assert!(matches!(model.round, Round::Waiting));
    cmd.expect_ble_with(|op| assert_eq!(op.op, BleOp::Off))
        .expect_render()
        .expect_only_random();
}

#[test]
fn wrong_channel_ignored() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_caller(&app, &mut model);
    let target = start_round_with(&app, &mut model, 2);
    let ch = model.channel();
    app.update(
        Event::ColorReceived {
            channel: (ch + 1) % 5,
            color: target,
        },
        &mut model,
    )
    .expect_done();
    assert!(matches!(model.round, Round::Running { .. }));
}

#[test]
fn wrong_color_ignored() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_caller(&app, &mut model);
    let target = start_round_with(&app, &mut model, 2);
    let other = Color::from_index((target.index() + 1) % NUM_LEDS as u8).unwrap();
    app.update(
        Event::ColorReceived {
            channel: model.channel(),
            color: other,
        },
        &mut model,
    )
    .expect_done();
    assert!(matches!(model.round, Round::Running { .. }));
}

#[test]
fn after_beats_explosion_then_random() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_caller(&app, &mut model);
    let _ = start_round_with(&app, &mut model, 0);
    let mut last_cmd = None;
    for _ in 0..=COUNTDOWN_STEPS {
        let id = match model.round {
            Round::Running { .. } => model.beat_gen,
            Round::Waiting => break,
        };
        last_cmd = Some(app.update(Event::BeatDone(id), &mut model));
    }
    assert!(matches!(model.round, Round::Waiting));
    // The next round waits for the last beat tone and the explosion.
    let last_tone = tones::countdown_step(COUNTDOWN_STEPS - 1);
    let explosion = tones::explosion();
    let wait_ms = last_tone.total_ms() + explosion.total_ms();
    last_cmd
        .expect("at least one beat")
        .expect_play_tone_with(|op| assert_eq!(op.sequence, last_tone))
        .expect_play_tone_with(|op| assert_eq!(op.sequence, explosion))
        .expect_ble_with(|op| assert_eq!(op.op, BleOp::Off))
        .expect_render()
        .expect_only_delay_with(|op| assert_eq!(op.millis, wait_ms));

    let id = model.beat_gen;
    app.update(Event::ExplosionDone(id.wrapping_add(1)), &mut model)
        .expect_done();
    app.update(Event::ExplosionDone(id), &mut model)
        .expect_only_random();
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
    let broadcast = BleOp::Broadcast {
        color: Some(Color::White),
    };
    app.update(Event::ActivateColor, &mut model)
        .expect_play_tone()
        .expect_ble_with(|op| assert_eq!(op.op, broadcast))
        .expect_only_render();
}

#[test]
fn beat_delay_period_is_core_constant() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_caller(&app, &mut model);
    // Round start kicks the first beat immediately (0 ms), then BEAT_PERIOD_MS.
    app.update(Event::RandomByte(0), &mut model)
        .expect_ble()
        .expect_render()
        .expect_only_delay_with(|op| assert_eq!(op.millis, 0));

    let id = model.beat_gen;
    app.update(Event::BeatDone(id), &mut model)
        .expect_play_tone()
        .expect_render()
        .expect_only_delay_with(|op| assert_eq!(op.millis, BEAT_PERIOD_MS));
}
