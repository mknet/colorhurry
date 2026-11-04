use crux_core::{App as _, Command};
use crux_time::TimerOutcome;

use crate::{
    color_from_entropy, decode_color, encode_color, time, tones, BleOp, Color, ColorHurry, Effect,
    EffectTestExt as _, Event, Model, Role, Round, Screen, BEAT_PERIOD_MS, BLINK_PERIOD_MS,
    COUNTDOWN_STEPS, NUM_LEDS,
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

fn millis(ms: u32) -> time::Duration {
    time::Duration::from_millis(u64::from(ms))
}

/// Fires the next timer of `cmd`, which must be of `ms`, and returns its event.
fn fire_timer(cmd: &mut Command<Effect, Event>, ms: u32) -> Event {
    cmd.resolve_time_notify_after(|op| {
        assert_eq!(op.duration, millis(ms));
        op.id
    });
    cmd.events().next().expect("the timer sends an event")
}

/// Runs a round from `byte` to the end of its countdown, and returns the
/// command of the last beat. Its next effect is the timer of the explosion.
fn run_countdown(app: &ColorHurry, model: &mut Model, byte: u8) -> Command<Effect, Event> {
    let mut cmd = app.update(Event::RandomByte(byte), model);
    cmd.expect_ble().expect_render();
    let mut beat = fire_timer(&mut cmd, 0);
    for _ in 0..COUNTDOWN_STEPS {
        let mut cmd = app.update(beat, model);
        if matches!(model.round, Round::Waiting) {
            return cmd;
        }
        cmd.expect_play_tone().expect_render();
        beat = fire_timer(&mut cmd, BEAT_PERIOD_MS);
    }
    panic!("the countdown did not end");
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
        .expect_only_time_notify_after();
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
        .expect_only_time_notify_after();
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
        .expect_only_time_notify_after();
}

#[test]
fn match_only_through_core_no_explosion() {
    let app = ColorHurry;
    // Given a Caller on channel 4, three beats into the countdown, target Red
    let mut model = Model {
        screen: Screen::Caller,
        role: Some(Role::Caller { channel: 4 }),
        round: Round::Running {
            target: Color::Red,
            step: 3,
        },
        ..Model::new()
    };
    // When the target arrives on the Caller's channel
    let mut cmd = app.update(
        Event::ColorReceived {
            channel: 4,
            color: Color::Red,
        },
        &mut model,
    );
    // Then the round ends, the radio goes off, and the core asks for the next target
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
    let mut cmd = run_countdown(&app, &mut model, 0);
    assert!(matches!(model.round, Round::Waiting));
    // The next round waits for the last beat tone and the explosion.
    let last_tone = tones::countdown_step(COUNTDOWN_STEPS - 1);
    let explosion = tones::explosion();
    let wait_ms = last_tone.total_ms() + explosion.total_ms();
    cmd.expect_play_tone_with(|op| assert_eq!(op.sequence, last_tone))
        .expect_play_tone_with(|op| assert_eq!(op.sequence, explosion))
        .expect_ble_with(|op| assert_eq!(op.op, BleOp::Off))
        .expect_render();
    let done = fire_timer(&mut cmd, wait_ms);

    app.update(done, &mut model).expect_only_random();
}

#[test]
fn reset_during_the_explosion_clears_its_timer() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_caller(&app, &mut model);
    let mut cmd = run_countdown(&app, &mut model, 0);
    cmd.expect_play_tone()
        .expect_play_tone()
        .expect_ble()
        .expect_render()
        .expect_time_notify_after();

    let _ = app.update(Event::Reset, &mut model);
    cmd.resolve_time_clear(|op| op.id);
    let done = cmd
        .events()
        .next()
        .expect("the cleared timer sends an event");
    assert_eq!(done, Event::ExplosionDone(TimerOutcome::Cleared));

    // An old timer does not change the game.
    app.update(done, &mut model).expect_done();
    assert_eq!(model.screen, Screen::ModeSelect);
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
fn beat_timer_period_is_core_constant() {
    let app = ColorHurry;
    let mut model = Model::new();
    enter_caller(&app, &mut model);
    // Round start kicks the first beat immediately (0 ms), then BEAT_PERIOD_MS.
    let mut cmd = app.update(Event::RandomByte(0), &mut model);
    cmd.expect_ble().expect_render();
    let beat = fire_timer(&mut cmd, 0);

    app.update(beat, &mut model)
        .expect_play_tone()
        .expect_render()
        .expect_only_time_notify_after_with(|op| assert_eq!(op.duration, millis(BEAT_PERIOD_MS)));
}

#[test]
fn every_blink_tick_schedules_the_next() {
    let app = ColorHurry;
    let mut model = Model::new();
    let mut cmd = app.update(Event::Boot, &mut model);
    for _ in 0..3 {
        cmd.expect_render();
        let tick = fire_timer(&mut cmd, BLINK_PERIOD_MS);
        cmd = app.update(tick, &mut model);
    }
    cmd.expect_render()
        .expect_only_time_notify_after_with(|op| assert_eq!(op.duration, millis(BLINK_PERIOD_MS)));
}

#[test]
fn every_ring_step_enters_its_role_and_channel() {
    let app = ColorHurry;
    for step in 0..NUM_LEDS {
        let mut model = Model::new();
        for _ in 0..step {
            let _ = app.update(Event::Skip, &mut model);
        }
        let _ = app.update(Event::ActivateColor, &mut model);
        let (screen, channel) = if step < 5 {
            (Screen::Picker, step)
        } else {
            (Screen::Caller, 9 - step)
        };
        assert_eq!((model.screen, model.channel()), (screen, channel as u8), "step {step}");
    }
}

#[test]
fn skips_walk_the_spectrum_in_order() {
    let app = ColorHurry;
    for (skips, color) in Color::ALL.iter().enumerate() {
        let mut model = Model::new();
        enter_picker(&app, &mut model);
        for _ in 0..skips {
            let _ = app.update(Event::Skip, &mut model);
        }
        let mut cmd = app.update(Event::ActivateColor, &mut model);
        let broadcast = BleOp::Broadcast { color: Some(*color) };
        assert!(cmd
            .effects()
            .any(|e| matches!(e, Effect::Ble(r) if r.operation.op == broadcast)));
    }
}
