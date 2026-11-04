//! Feature World: Gherkin steps map onto domain [`Event`]s.

mod harness;
mod parse;

use color_hurry_core::{BleOp, Event, ModeChoice, ModeRole, Screen};
use cucumber::{given, then, when, World};
use harness::Harness;
use parse::{parse_color, parse_screen};

#[derive(Debug, Default, World)]
pub struct ColorHurryWorld {
    harness: Harness,
}

fn parse_byte_list(s: &str) -> Vec<u8> {
    s.split(',')
        .map(|p| {
            p.trim()
                .parse::<u8>()
                .unwrap_or_else(|_| panic!("bad random byte '{p}'"))
        })
        .collect()
}

fn picker_mode_step(channel: u8) -> u8 {
    channel
}

fn caller_mode_step(channel: u8) -> u8 {
    9 - channel
}

// ----- Given -----

#[given("a fresh game")]
async fn fresh(world: &mut ColorHurryWorld) {
    *world = ColorHurryWorld::default();
}

#[given("the shell cannot broadcast")]
async fn cannot_broadcast(world: &mut ColorHurryWorld) {
    world.harness.send(Event::CallerOnly);
}

#[when("the shell says that it cannot broadcast")]
async fn says_cannot_broadcast(world: &mut ColorHurryWorld) {
    world.harness.send(Event::CallerOnly);
}

#[given("the game has been booted")]
async fn booted(world: &mut ColorHurryWorld) {
    world.harness.send(Event::Boot);
}

#[given(regex = r"^the next random bytes? are? (.+)$")]
async fn next_random(world: &mut ColorHurryWorld, list: String) {
    world.harness.push_random_many(parse_byte_list(&list));
}

#[when("the shell holds random requests")]
async fn hold_random(world: &mut ColorHurryWorld) {
    world.harness.hold_random(true);
}

#[when("the shell answers random requests at once")]
async fn answer_random(world: &mut ColorHurryWorld) {
    world.harness.hold_random(false);
}

// ----- When -----

#[when("the app goes to the background")]
async fn pause(world: &mut ColorHurryWorld) {
    world.harness.send(Event::Pause);
}

#[when("the app comes back")]
async fn resume(world: &mut ColorHurryWorld) {
    world.harness.send(Event::Resume);
}

#[when("I go back to mode select")]
async fn reset(world: &mut ColorHurryWorld) {
    world.harness.send(Event::Reset);
}

#[when(regex = r"^the shell answers the held random request with (\d+)$")]
async fn answer_held(world: &mut ColorHurryWorld, byte: u8) {
    world.harness.answer_held_random(byte);
}

#[when("I apply")]
async fn apply(world: &mut ColorHurryWorld) {
    world.harness.send(Event::ActivateColor);
}

#[when("I skip")]
async fn skip_once(world: &mut ColorHurryWorld) {
    world.harness.send(Event::Skip);
}

#[when(regex = r"^I skip (\d+) times?$")]
async fn skip_n(world: &mut ColorHurryWorld, n: u32) {
    for _ in 0..n {
        world.harness.send(Event::Skip);
    }
}

#[when(regex = r"^I enter picker on channel (\d+)$")]
async fn enter_picker(world: &mut ColorHurryWorld, channel: u8) {
    assert!(channel < 5, "channel must be 0..4");
    for _ in 0..picker_mode_step(channel) {
        world.harness.send(Event::Skip);
    }
    world.harness.send(Event::ActivateColor);
    assert_eq!(world.harness.view().screen, Screen::Picker);
    assert_eq!(world.harness.view().channel, channel);
}

#[when(regex = r"^I enter caller on channel (\d+)$")]
async fn enter_caller(world: &mut ColorHurryWorld, channel: u8) {
    assert!(channel < 5, "channel must be 0..4");
    for _ in 0..caller_mode_step(channel) {
        world.harness.send(Event::Skip);
    }
    world.harness.send(Event::ActivateColor);
    assert_eq!(world.harness.view().screen, Screen::Caller);
    assert_eq!(world.harness.view().channel, channel);
}

#[when("I clear effect logs")]
async fn clear_logs(world: &mut ColorHurryWorld) {
    world.harness.clear_logs();
}

#[when(regex = r"^a color (.+) is received on channel (\d+)$")]
async fn color_received(world: &mut ColorHurryWorld, color: String, channel: u8) {
    let color = parse_color(&color).expect("color");
    world
        .harness
        .send(Event::ColorReceived { channel, color });
}

#[when(regex = r"^I advance time by (\d+) ms$")]
async fn advance(world: &mut ColorHurryWorld, ms: u64) {
    world.harness.advance_ms(ms);
}

#[when("the full countdown runs")]
async fn full_countdown(world: &mut ColorHurryWorld) {
    world.harness.run_full_countdown();
}

// ----- Then -----

#[then(regex = r"^the screen should be (.+)$")]
async fn screen_is(world: &mut ColorHurryWorld, name: String) {
    let expected = parse_screen(&name).expect("screen");
    assert_eq!(world.harness.view().screen, expected);
}

#[then(regex = r"^the channel should be (\d+)$")]
async fn channel_is(world: &mut ColorHurryWorld, channel: u8) {
    assert_eq!(world.harness.view().channel, channel);
}

#[then(regex = r"^the mode choice should be (Picker|Caller) channel (\d+)$")]
async fn mode_choice_is(world: &mut ColorHurryWorld, role: String, channel: u8) {
    let role = if role == "Picker" {
        ModeRole::Picker
    } else {
        ModeRole::Caller
    };
    assert_eq!(
        world.harness.view().mode_choice,
        Some(ModeChoice { role, channel })
    );
}

#[then("there should be no mode choice")]
async fn no_mode_choice(world: &mut ColorHurryWorld) {
    assert_eq!(world.harness.view().mode_choice, None);
}

#[then(regex = r"^the picker should name (.+) as (selected|applied)$")]
async fn picker_names(world: &mut ColorHurryWorld, color: String, state: String) {
    let view = world.harness.view();
    assert_eq!(view.picker_color, Some(parse_color(&color).expect("color")));
    assert_eq!(view.picker_applied, state == "applied");
}

#[then("there should be no picker color")]
async fn no_picker_color(world: &mut ColorHurryWorld) {
    let view = world.harness.view();
    assert_eq!(view.picker_color, None);
    assert!(!view.picker_applied);
}

#[then("the countdown should be active")]
async fn countdown_active(world: &mut ColorHurryWorld) {
    assert!(world.harness.view().countdown_active);
}

#[then("the countdown should not be active")]
async fn countdown_inactive(world: &mut ColorHurryWorld) {
    assert!(!world.harness.view().countdown_active);
}

#[then(regex = r"^the countdown step should be (\d+)$")]
async fn assert_countdown_step(world: &mut ColorHurryWorld, value: u8) {
    assert_eq!(world.harness.view().countdown_step, value);
}

#[then(regex = r"^BLE should have received a Picker op for channel (\d+)$")]
async fn ble_picker(world: &mut ColorHurryWorld, channel: u8) {
    assert!(
        world
            .harness
            .ble_log()
            .iter()
            .any(|op| matches!(op, BleOp::Picker { channel: c } if *c == channel)),
        "ble log: {:?}",
        world.harness.ble_log()
    );
}

#[then(regex = r"^BLE should have received a Caller op for channel (\d+) and (.+)$")]
async fn ble_caller(world: &mut ColorHurryWorld, channel: u8, color: String) {
    let color = parse_color(&color).expect("color");
    assert!(
        world.harness.ble_log().iter().any(|op| matches!(
            op,
            BleOp::Caller { channel: c, color: col } if *c == channel && *col == color
        )),
        "ble log: {:?}",
        world.harness.ble_log()
    );
}

#[then(regex = r"^BLE should have received a Broadcast of (.+)$")]
async fn ble_broadcast_color(world: &mut ColorHurryWorld, color: String) {
    let color = parse_color(&color).expect("color");
    assert!(
        world.harness.ble_log().iter().any(|op| matches!(
            op,
            BleOp::Broadcast { color: Some(c) } if *c == color
        )),
        "ble log: {:?}",
        world.harness.ble_log()
    );
}

#[then("BLE should have received a Broadcast with no color")]
async fn ble_broadcast_none(world: &mut ColorHurryWorld) {
    assert!(
        world
            .harness
            .ble_log()
            .iter()
            .any(|op| matches!(op, BleOp::Broadcast { color: None })),
        "ble log: {:?}",
        world.harness.ble_log()
    );
}

#[then("BLE should have received Off")]
async fn ble_off(world: &mut ColorHurryWorld) {
    assert!(
        world
            .harness
            .ble_log()
            .iter()
            .any(|op| matches!(op, BleOp::Off)),
        "ble log: {:?}",
        world.harness.ble_log()
    );
}

#[then("BLE should not have received a Caller op")]
async fn ble_no_caller(world: &mut ColorHurryWorld) {
    assert!(
        !world
            .harness
            .ble_log()
            .iter()
            .any(|op| matches!(op, BleOp::Caller { .. })),
        "ble log: {:?}",
        world.harness.ble_log()
    );
}

#[then(regex = r"^BLE should have received (\d+) Caller ops?$")]
async fn ble_caller_count(world: &mut ColorHurryWorld, n: usize) {
    let count = world
        .harness
        .ble_log()
        .iter()
        .filter(|op| matches!(op, BleOp::Caller { .. }))
        .count();
    assert_eq!(count, n, "ble log: {:?}", world.harness.ble_log());
}

#[then("BLE should not have received any new ops")]
async fn ble_empty(world: &mut ColorHurryWorld) {
    assert!(
        world.harness.ble_log().is_empty(),
        "unexpected ble ops: {:?}",
        world.harness.ble_log()
    );
}

#[then("the last tone should be Skip")]
async fn tone_skip(world: &mut ColorHurryWorld) {
    let tone = world.harness.last_tone().expect("tone");
    assert!(tone.is_skip(), "{tone:?}");
}

#[then("the last tone should be Apply")]
async fn tone_apply(world: &mut ColorHurryWorld) {
    let tone = world.harness.last_tone().expect("tone");
    assert!(tone.is_apply(), "{tone:?}");
}

#[then("an explosion tone should have been played")]
async fn tone_explosion(world: &mut ColorHurryWorld) {
    assert!(
        world.harness.tone_log().iter().any(|t| t.is_explosion()),
        "tones: {:?}",
        world.harness.tone_log()
    );
}

#[then("a countdown tone should have been played")]
async fn tone_countdown(world: &mut ColorHurryWorld) {
    assert!(
        world
            .harness
            .tone_log()
            .iter()
            .any(|t| t.is_countdown_step()),
        "tones: {:?}",
        world.harness.tone_log()
    );
}

#[then(regex = r"^a timer of (\d+) ms should have been requested$")]
async fn timer_requested(world: &mut ColorHurryWorld, ms: u64) {
    assert!(
        world.harness.timer_log().contains(&ms),
        "timers: {:?}",
        world.harness.timer_log()
    );
}

#[then(regex = r"^a timer of (\d+) ms should have been requested at least (\d+) times$")]
async fn timer_requested_n(world: &mut ColorHurryWorld, ms: u64, n: usize) {
    let count = world
        .harness
        .timer_log()
        .iter()
        .filter(|d| **d == ms)
        .count();
    assert!(count >= n, "expected >= {n} timers of {ms} ms, got {count}");
}

#[then("a timer should have been cleared")]
async fn timer_cleared(world: &mut ColorHurryWorld) {
    assert!(
        !world.harness.clear_log().is_empty(),
        "no timer was cleared"
    );
}

#[then("no tone should have been played")]
async fn no_tone(world: &mut ColorHurryWorld) {
    assert!(
        world.harness.tone_log().is_empty(),
        "tones: {:?}",
        world.harness.tone_log()
    );
}

#[then("no explosion tone should have been played")]
async fn no_explosion(world: &mut ColorHurryWorld) {
    assert!(
        !world.harness.tone_log().iter().any(|t| t.is_explosion()),
        "tones: {:?}",
        world.harness.tone_log()
    );
}

#[then("no timer should have been requested")]
async fn no_timer(world: &mut ColorHurryWorld) {
    assert!(
        world.harness.timer_log().is_empty(),
        "timers: {:?}",
        world.harness.timer_log()
    );
}

#[then(regex = r"^all LEDs should show (.+)$")]
async fn all_leds(world: &mut ColorHurryWorld, color: String) {
    let color = parse_color(&color).expect("color");
    assert!(
        world.harness.all_leds_equal(color),
        "leds: {:?}",
        world.harness.view().leds
    );
}

#[then(regex = r"^LED (\d+) should be off$")]
async fn led_off(world: &mut ColorHurryWorld, index: usize) {
    assert!(
        world.harness.led_is_off(index),
        "leds: {:?}",
        world.harness.view().leds
    );
}

#[then(regex = r"^LED (\d+) should be lit$")]
async fn led_lit(world: &mut ColorHurryWorld, index: usize) {
    assert!(
        !world.harness.led_is_off(index),
        "leds: {:?}",
        world.harness.view().leds
    );
}

pub async fn run() {
    ColorHurryWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features")
        .await;
}
