//! Feature World: Gherkin steps map onto domain [`Event`]s.

mod harness;
mod parse;

use color_hurry_core::{BleOp, Event, Screen};
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

fn receiver_mode_step(channel: u8) -> u8 {
    9 - channel
}

// ----- Given -----

#[given("a fresh game")]
async fn fresh(world: &mut ColorHurryWorld) {
    *world = ColorHurryWorld::default();
}

#[given("the game has been booted")]
async fn booted(world: &mut ColorHurryWorld) {
    world.harness.send(Event::Boot);
}

#[given(regex = r"^the next random bytes? are? (.+)$")]
async fn next_random(world: &mut ColorHurryWorld, list: String) {
    world.harness.push_random_many(parse_byte_list(&list));
}

// ----- When -----

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

#[when(regex = r"^I enter receiver on channel (\d+)$")]
async fn enter_receiver(world: &mut ColorHurryWorld, channel: u8) {
    assert!(channel < 5, "channel must be 0..4");
    for _ in 0..receiver_mode_step(channel) {
        world.harness.send(Event::Skip);
    }
    world.harness.send(Event::ActivateColor);
    assert_eq!(world.harness.view().screen, Screen::Receiver);
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

#[then(regex = r"^BLE should have received a Receiver op for channel (\d+) and (.+)$")]
async fn ble_receiver(world: &mut ColorHurryWorld, channel: u8, color: String) {
    let color = parse_color(&color).expect("color");
    assert!(
        world.harness.ble_log().iter().any(|op| matches!(
            op,
            BleOp::Receiver { channel: c, color: col } if *c == channel && *col == color
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

#[then(regex = r"^a delay of (\d+) ms should have been requested$")]
async fn delay_requested(world: &mut ColorHurryWorld, ms: u32) {
    assert!(
        world.harness.delay_log().contains(&ms),
        "delays: {:?}",
        world.harness.delay_log()
    );
}

#[then(regex = r"^a delay of (\d+) ms should have been requested at least (\d+) times$")]
async fn delay_requested_n(world: &mut ColorHurryWorld, ms: u32, n: usize) {
    let count = world
        .harness
        .delay_log()
        .iter()
        .filter(|d| **d == ms)
        .count();
    assert!(count >= n, "expected >= {n} delays of {ms}, got {count}");
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

pub async fn run() {
    ColorHurryWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("tests/features")
        .await;
}
