//! Spec World: the scenarios in `openspec/specs/*/spec.md` are the tests.
//!
//! [`spec::SpecParser`] turns each spec into a Gherkin feature. The steps below
//! use the wording of the scenarios and map it onto domain [`Event`]s through a
//! few helpers on [`ColorHurryWorld`].

mod harness;
mod parse;
pub mod spec;

use color_hurry_core::{
    decode_color, encode_color, BleOp, Color, Event, ModeChoice, ModeRole, Rgb, Screen,
    ViewModel, BLINK_PERIOD_MS, BLINK_TICKS,
};
use cucumber::{given, then, when, World};
use harness::{spectrum_rgb, Harness};
use parse::{parse_color, parse_number, parse_screen};

#[derive(Debug, Default, World)]
pub struct ColorHurryWorld {
    harness: Harness,
    /// The view when the Given part ended, for "nothing changes".
    snapshot: Option<ViewModel>,
    encoded: Option<[u8; 5]>,
    decoded: Option<Option<(u8, Color)>>,
}

/// Spectrum index of `color`: the random byte that makes it the target.
fn byte_for(color: Color) -> u8 {
    color.index()
}

fn color(name: &str) -> Color {
    parse_color(name).unwrap_or_else(|e| panic!("{e}"))
}

fn number(word: &str) -> usize {
    parse_number(word).unwrap_or_else(|e| panic!("{e}"))
}

impl ColorHurryWorld {
    fn boot(&mut self) {
        self.harness.send(Event::Boot);
    }

    fn skip(&mut self, times: usize) {
        for _ in 0..times {
            self.harness.send(Event::Skip);
        }
    }

    fn activate(&mut self) {
        self.harness.send(Event::ActivateColor);
    }

    /// Mode select: steps 0 to 4 are Picker on channel `step`, 5 to 9 Caller on `9 - step`.
    fn enter(&mut self, screen: Screen, channel: u8) {
        assert!(channel < 5, "channel must be 0 to 4");
        let step = if screen == Screen::Picker { channel } else { 9 - channel };
        self.skip(step.into());
        self.activate();
        assert_eq!(self.view().screen, screen);
        assert_eq!(self.view().channel, channel);
    }

    /// Picker: Skip until `color` is selected. Entering on channel `n` selects color `n`.
    fn select(&mut self, color: Color) {
        let from = self.view().channel;
        self.skip(usize::from((color.index() + 10 - from) % 10));
    }

    /// Advance virtual time until `n` more countdown beats have played.
    fn beats(&mut self, n: usize) {
        let target = self.countdown_tones().len() + n;
        while self.countdown_tones().len() < target {
            self.harness.advance_to_next_timer();
        }
    }

    fn countdown_tones(&self) -> Vec<&harness::ObservedTone> {
        self.harness.tone_log().iter().filter(|t| t.is_countdown_step()).collect()
    }

    /// End of the Given part: forget the setup effects, remember the view.
    fn settle(&mut self) {
        self.harness.clear_logs();
        self.snapshot = Some(self.view());
    }

    fn view(&self) -> ViewModel {
        self.harness.view()
    }

    fn last_broadcast(&self) -> Option<Color> {
        self.harness
            .ble_log()
            .iter()
            .rev()
            .find_map(|op| match op {
                BleOp::Broadcast { color } => Some(*color),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no broadcast in {:?}", self.harness.ble_log()))
    }

    /// The LED toggles within one blink phase while all others stay as they are.
    fn assert_blinks(&mut self, led: usize) {
        let before = self.view().leds;
        self.harness.advance_ms(u64::from(BLINK_TICKS * BLINK_PERIOD_MS));
        let after = self.view().leds;
        for i in 0..before.len() {
            assert_eq!(before[i] != after[i], i == led, "LED {i}: {before:?} then {after:?}");
        }
    }
}

// ----- Given -----

impl ColorHurryWorld {
    /// Boot, as a caller-only game if asked, and Skip to the chosen step.
    fn mode_select(&mut self, game: &str, choice: Option<(&str, u8)>) {
        let caller_only = game == "a caller-only game";
        if caller_only {
            self.harness.send(Event::CallerOnly);
        }
        self.boot();
        if let Some((role, channel)) = choice {
            // Caller-only mode select starts at Caller on channel 0 and walks 4, 3, 2, 1.
            let skips = match (caller_only, role) {
                (true, _) => (5 - channel) % 5,
                (false, "Picker") => channel,
                (false, _) => 9 - channel,
            };
            self.skip(skips.into());
        }
        self.settle();
    }
}

#[given(regex = r"^(a caller-only game|the board) in mode select$")]
async fn in_mode_select(world: &mut ColorHurryWorld, game: String) {
    world.mode_select(&game, None);
}

#[given(regex = r"^(a caller-only game|the board) in mode select with (Picker|Caller) on channel (\d) chosen$")]
async fn in_mode_select_at(world: &mut ColorHurryWorld, game: String, role: String, channel: u8) {
    world.mode_select(&game, Some((&role, channel)));
}

#[given("the app is in the background")]
async fn in_background(world: &mut ColorHurryWorld) {
    world.harness.send(Event::Pause);
    world.settle();
}

#[given(regex = r"^(\w+) beats? (?:have|has) passed$")]
async fn beats_have_passed(world: &mut ColorHurryWorld, n: String) {
    world.beats(number(&n));
    world.settle();
}

#[given(regex = r"^the Picker on channel (\d)$")]
async fn picker(world: &mut ColorHurryWorld, channel: u8) {
    world.boot();
    world.enter(Screen::Picker, channel);
    world.settle();
}

#[given(regex = r"^the Picker on channel (\d) with (\w+) (selected|applied)$")]
async fn picker_with(world: &mut ColorHurryWorld, channel: u8, chosen: String, state: String) {
    world.boot();
    world.enter(Screen::Picker, channel);
    world.select(color(&chosen));
    if state == "applied" {
        world.activate();
    }
    world.settle();
}

#[given(regex = r"^the Caller on channel (\d) with target (\w+)$")]
async fn caller(world: &mut ColorHurryWorld, channel: u8, target: String) {
    world.boot();
    world.harness.push_random_many([byte_for(color(&target))]);
    world.enter(Screen::Caller, channel);
    world.settle();
}

// ----- When -----

#[when("the board boots")]
async fn boots(world: &mut ColorHurryWorld) {
    world.boot();
}

#[when("the app goes to the background")]
async fn pause(world: &mut ColorHurryWorld) {
    world.harness.send(Event::Pause);
}

#[when("the app comes back")]
async fn resume(world: &mut ColorHurryWorld) {
    world.harness.send(Event::Resume);
}

#[when("the player sends Reset")]
async fn reset(world: &mut ColorHurryWorld) {
    world.harness.send(Event::Reset);
}

#[when("the shell sends CallerOnly")]
async fn caller_only(world: &mut ColorHurryWorld) {
    world.harness.send(Event::CallerOnly);
}

#[when("the explosion has played")]
async fn explosion_played(world: &mut ColorHurryWorld) {
    world.harness.advance_to_next_timer();
}

#[when(regex = r"^the player presses (Skip|ActivateColor)$")]
async fn presses(world: &mut ColorHurryWorld, button: String) {
    match button.as_str() {
        "Skip" => world.skip(1),
        _ => world.activate(),
    }
}

#[when(regex = r"^the player presses Skip (\w+)(?: times)?(?: without any time passing)?$")]
async fn presses_skip_n(world: &mut ColorHurryWorld, times: String) {
    world.skip(number(&times));
}

#[when(regex = r"^the player enters (Picker|Caller) on channel (\d)$")]
async fn enters(world: &mut ColorHurryWorld, screen: String, channel: u8) {
    world.enter(parse_screen(&screen).unwrap(), channel);
}

#[when(regex = r"^the player applies (\w+)$")]
async fn applies(world: &mut ColorHurryWorld, name: String) {
    world.select(color(&name));
    world.activate();
}

#[when(regex = r"^the shell answers the random request with (\d+)$")]
async fn answers_random(world: &mut ColorHurryWorld, byte: u8) {
    world.harness.answer_random(byte);
}

#[when(regex = r"^(\w+) arrives on channel (\d)$")]
async fn arrives(world: &mut ColorHurryWorld, name: String, channel: u8) {
    world.harness.send(Event::ColorReceived { channel, color: color(&name) });
}

#[when(regex = r"^(\w+) beats? (?:have|has) passed$")]
async fn beats_passed(world: &mut ColorHurryWorld, n: String) {
    world.beats(number(&n));
}

#[given(regex = r"^(\d+) ms ha(?:ve|s) passed$")]
async fn time_has_passed(world: &mut ColorHurryWorld, ms: u64) {
    world.harness.advance_ms(ms);
    world.settle();
}

#[when(regex = r"^(\d+) ms ha(?:ve|s) passed$")]
async fn time_passed(world: &mut ColorHurryWorld, ms: u64) {
    world.harness.advance_ms(ms);
}

#[when(regex = r"^the (?:[\w-]+ )*payload `([0-9A-F ]+)` arrives$")]
async fn payload_arrives(world: &mut ColorHurryWorld, hex: String) {
    let bytes: Vec<u8> = hex.split(' ').map(|b| u8::from_str_radix(b, 16).unwrap()).collect();
    world.decoded = Some(decode_color(&bytes));
}

#[when(regex = r"^channel (\d) and (\w+) are encoded$")]
async fn encoded(world: &mut ColorHurryWorld, channel: u8, name: String) {
    world.encoded = Some(encode_color(channel, color(&name)).expect("encodable"));
}

#[when("the result is decoded again")]
async fn decoded_again(world: &mut ColorHurryWorld) {
    world.decoded = Some(decode_color(&world.encoded.expect("nothing encoded")));
}

// ----- Then: screen and LEDs -----

#[then("mode select is shown")]
async fn mode_select_shown(world: &mut ColorHurryWorld) {
    assert_eq!(world.view().screen, Screen::ModeSelect);
}

#[then(regex = r"^the board is the (Picker|Caller) on channel (\d)$")]
async fn board_is(world: &mut ColorHurryWorld, screen: String, channel: u8) {
    assert_eq!(world.view().screen, parse_screen(&screen).unwrap());
    assert_eq!(world.view().channel, channel);
}

#[then(regex = r"^LED (\d) blinks$")]
async fn led_blinks(world: &mut ColorHurryWorld, led: usize) {
    world.assert_blinks(led);
}

#[then(regex = r"^the (\w+) LED blinks$")]
async fn color_led_blinks(world: &mut ColorHurryWorld, name: String) {
    world.assert_blinks(color(&name).index().into());
}

#[then(regex = r"^LED (\d) is (on|off)$")]
async fn led_is(world: &mut ColorHurryWorld, led: usize, state: String) {
    assert_eq!(world.harness.led_is_off(led), state == "off", "{:?}", world.view().leds);
}

#[then("the ring shows the spectrum")]
async fn ring_shows_spectrum(world: &mut ColorHurryWorld) {
    let leds = world.view().leds;
    let blinking = leds.iter().filter(|rgb| **rgb == Rgb::OFF).count();
    assert!(blinking <= 1, "{leds:?}");
    for (rgb, color) in leds.iter().zip(Color::ALL) {
        assert!(*rgb == spectrum_rgb(color) || *rgb == Rgb::OFF, "{leds:?}");
    }
}

#[then(regex = r"^all LEDs show (\w+)$")]
async fn all_leds_show(world: &mut ColorHurryWorld, name: String) {
    assert!(world.harness.all_leds_equal(color(&name)), "{:?}", world.view().leds);
}

#[then(regex = r"^(\w+) LEDs are off$")]
async fn leds_off(world: &mut ColorHurryWorld, n: String) {
    let off = world.view().leds.iter().filter(|rgb| **rgb == Rgb::OFF).count();
    assert_eq!(off, number(&n));
}

#[then("nothing changes")]
async fn nothing_changes(world: &mut ColorHurryWorld) {
    assert_eq!(Some(world.view()), world.snapshot);
    no_effect(world).await;
}

#[then("no effect is requested")]
async fn no_effect(world: &mut ColorHurryWorld) {
    let h = &world.harness;
    assert!(h.ble_log().is_empty(), "{:?}", h.ble_log());
    assert!(h.tone_log().is_empty(), "{:?}", h.tone_log());
    assert!(h.timer_log().is_empty(), "{:?}", h.timer_log());
    assert!(h.clear_log().is_empty(), "{:?}", h.clear_log());
    assert_eq!(h.random_requests(), 0);
}

// ----- Then: what the view names -----

#[then(regex = r"^the choice is (Picker|Caller) on channel (\d)$")]
async fn choice_is(world: &mut ColorHurryWorld, role: String, channel: u8) {
    let role = if role == "Picker" { ModeRole::Picker } else { ModeRole::Caller };
    assert_eq!(world.view().mode_choice, Some(ModeChoice { role, channel }));
}

#[then("the view names no choice")]
async fn no_choice(world: &mut ColorHurryWorld) {
    assert_eq!(world.view().mode_choice, None);
}

#[then("the view names no Picker color")]
async fn no_picker_color(world: &mut ColorHurryWorld) {
    assert_eq!(world.view().picker_color, None);
    assert!(!world.view().picker_applied);
}

#[then(regex = r"^the view names (\w+) as (selected|applied)$")]
async fn picker_color_is(world: &mut ColorHurryWorld, name: String, state: String) {
    assert_eq!(world.view().picker_color, Some(color(&name)));
    assert_eq!(world.view().picker_applied, state == "applied");
}

// ----- Then: the Caller's round -----

#[then(regex = r"^the target is (\w+)$")]
async fn target_is(world: &mut ColorHurryWorld, name: String) {
    let rgb = spectrum_rgb(color(&name));
    let leds = world.view().leds;
    assert!(world.view().countdown_active);
    assert!(leds.iter().all(|led| *led == rgb || *led == Rgb::OFF), "{leds:?}");
    assert!(leds.contains(&rgb), "{leds:?}");
}

#[then("the countdown begins")]
async fn countdown_begins(world: &mut ColorHurryWorld) {
    assert!(world.view().countdown_active);
    assert_eq!(world.view().countdown_step, 0);
}

#[then("the countdown stops")]
async fn countdown_stops(world: &mut ColorHurryWorld) {
    assert!(!world.view().countdown_active);
}

#[then("the countdown keeps running")]
async fn countdown_keeps_running(world: &mut ColorHurryWorld) {
    assert!(world.view().countdown_active);
    assert_eq!(Some(world.view()), world.snapshot);
    assert_eq!(world.harness.random_requests(), 0);
    assert!(world.harness.ble_log().is_empty(), "{:?}", world.harness.ble_log());
}

#[then("a new random byte is requested")]
async fn random_requested(world: &mut ColorHurryWorld) {
    assert_eq!(world.harness.random_requests(), 1);
}

#[then("no random byte is requested")]
async fn no_random_requested(world: &mut ColorHurryWorld) {
    assert_eq!(world.harness.random_requests(), 0);
}

// ----- Then: tones and delays -----

#[then(regex = r"^(no explosion|the explosion) plays$")]
async fn explosion(world: &mut ColorHurryWorld, which: String) {
    let played = world.harness.tone_log().iter().any(|t| t.is_explosion());
    assert_eq!(played, which == "the explosion", "{:?}", world.harness.tone_log());
}

#[then(regex = r"^the explosion sweeps in (\w+) (\d+) ms steps from (\d+) Hz down to (\d+) Hz$")]
async fn explosion_sweep(world: &mut ColorHurryWorld, n: String, ms: u32, from: u32, to: u32) {
    let tone = world.harness.tone_log().iter().find(|t| t.is_explosion()).expect("explosion");
    assert_eq!(tone.steps.len(), number(&n));
    assert!(tone.steps.iter().all(|s| s.1 == ms), "{tone:?}");
    assert!(tone.steps.windows(2).all(|w| w[1].0 < w[0].0), "{tone:?}");
    assert_eq!((tone.steps[0].0, tone.steps[tone.steps.len() - 1].0), (from, to));
}

#[then(regex = r"^(\w+) descending tones have played$")]
async fn descending_tones(world: &mut ColorHurryWorld, n: String) {
    let hz: Vec<u32> = world.countdown_tones().iter().map(|t| t.steps[0].0).collect();
    assert_eq!(hz.len(), number(&n));
    assert!(hz.windows(2).all(|w| w[1] < w[0]), "{hz:?}");
}

#[then(regex = r"^(\d+) ms tones of (.+) Hz have played one second apart$")]
async fn countdown_table(world: &mut ColorHurryWorld, ms: u32, table: String) {
    let expected: Vec<(u32, u32)> = table
        .split([',', ' '])
        .filter(|w| !w.is_empty() && *w != "and")
        .map(|hz| (hz.parse().unwrap(), ms))
        .collect();
    let beats = world.countdown_tones();
    let played: Vec<(u32, u32)> = beats.iter().map(|t| t.steps[0]).collect();
    assert_eq!(played, expected);
    assert!(beats.windows(2).all(|w| w[1].at_ms - w[0].at_ms == 1000), "{beats:?}");
}

#[then(regex = r"^a (\d+) Hz tone plays for (\d+) ms$")]
async fn tone_plays(world: &mut ColorHurryWorld, hz: u32, ms: u32) {
    let tones = world.harness.tone_log();
    assert_eq!(tones.last().map(|t| t.steps.as_slice()), Some(&[(hz, ms)][..]), "{tones:?}");
}

#[then(regex = r"^(\w+) Skip tones have played$")]
async fn skip_tones(world: &mut ColorHurryWorld, n: String) {
    let skips = world.harness.tone_log().iter().filter(|t| t.steps == [(380, 55)]).count();
    assert_eq!(skips, number(&n));
}

#[then("no tone plays")]
async fn no_tone(world: &mut ColorHurryWorld) {
    assert!(world.harness.tone_log().is_empty(), "{:?}", world.harness.tone_log());
}

#[then(regex = r"^a (\d+) ms timer is requested$")]
async fn timer_requested(world: &mut ColorHurryWorld, ms: u64) {
    assert!(world.harness.timer_log().contains(&ms), "{:?}", world.harness.timer_log());
}

#[then("no timer is requested")]
async fn no_timer(world: &mut ColorHurryWorld) {
    assert!(world.harness.timer_log().is_empty(), "{:?}", world.harness.timer_log());
}

#[then(regex = r"^the core clears the pending (?:blink|beat) timer$")]
async fn timer_cleared(world: &mut ColorHurryWorld) {
    assert!(!world.harness.clear_log().is_empty(), "no timer was cleared");
}

// ----- Then: radio and payload -----

#[then("the radio is off")]
async fn radio_off(world: &mut ColorHurryWorld) {
    assert_eq!(world.harness.ble_log().last(), Some(&BleOp::Off), "{:?}", world.harness.ble_log());
}

#[then("no Color Hurry advertisement is sent")]
async fn no_advertisement(world: &mut ColorHurryWorld) {
    let ops = world.harness.ble_log();
    assert!(ops.iter().all(|op| matches!(op, BleOp::Off)), "{ops:?}");
}

#[then(regex = r"^the board advertises on channel (\d) without a color$")]
async fn advertises_without_color(world: &mut ColorHurryWorld, channel: u8) {
    assert!(world.harness.ble_log().contains(&BleOp::Picker { channel }));
    assert_eq!(world.last_broadcast(), None);
}

#[then(regex = r"^the board scans channel (\d) for (\w+)$")]
async fn scans(world: &mut ColorHurryWorld, channel: u8, name: String) {
    let op = BleOp::Caller { channel, color: color(&name) };
    assert!(world.harness.ble_log().contains(&op), "{:?}", world.harness.ble_log());
}

#[then(regex = r"^the payload is `([0-9A-F ]+)`$")]
async fn payload_is(world: &mut ColorHurryWorld, hex: String) {
    let color = world.last_broadcast().expect("a color is broadcast");
    let bytes = encode_color(world.view().channel, color).expect("encodable");
    let wire: Vec<String> = bytes.iter().map(|b| format!("{b:02X}")).collect();
    assert_eq!(wire.join(" "), hex);
}

#[then(regex = r"^a scanner sees the (\w+) payload for channel (\d)$")]
async fn scanner_sees(world: &mut ColorHurryWorld, name: String, channel: u8) {
    let color_now = world.last_broadcast().expect("a color is broadcast");
    let bytes = encode_color(world.view().channel, color_now).expect("encodable");
    assert_eq!(decode_color(&bytes), Some((channel, color(&name))));
}

#[then("no color payload is advertised any more")]
async fn no_color_payload(world: &mut ColorHurryWorld) {
    assert_eq!(world.last_broadcast(), None);
}

#[then("it is not turned into a ColorReceived")]
async fn not_a_color(world: &mut ColorHurryWorld) {
    assert_eq!(world.decoded, Some(None));
}

#[then(regex = r"^the decoder returns channel (\d) and (\w+)$")]
async fn decoder_returns(world: &mut ColorHurryWorld, channel: u8, name: String) {
    assert_eq!(world.decoded, Some(Some((channel, color(&name)))));
}

/// Runs every scenario in `openspec/specs`. A step without a match fails the
/// run, unless the scenario is marked `Manual:` in the spec.
pub async fn run() {
    let specs = concat!(env!("CARGO_MANIFEST_DIR"), "/../../openspec/specs");
    ColorHurryWorld::cucumber::<&str>()
        .with_parser(spec::SpecParser)
        .fail_on_skipped()
        .run_and_exit(specs)
        .await;
}
