//! The shell on a mock board: no hardware, no window.

extern crate std;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use color_hurry_core::{Color, Rgb, Screen, NUM_LEDS};
use embassy_futures::block_on;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
enum RadioCall {
    Off,
    Picker(u8),
    Caller(u8),
    Broadcast(Option<Color>),
}

#[derive(Default)]
struct MockBoard {
    /// Raw button states for the next polls. When empty, no button is down.
    buttons: VecDeque<(bool, bool)>,
    received: VecDeque<(u8, Color)>,
    frames: Vec<[Rgb; NUM_LEDS]>,
    tones: Vec<(u32, u32)>,
    radio: Vec<RadioCall>,
    waited_ms: u64,
}

impl Board for MockBoard {
    fn buttons(&mut self) -> (bool, bool) {
        self.buttons.pop_front().unwrap_or((false, false))
    }
    fn show(&mut self, leds: &[Rgb; NUM_LEDS]) {
        self.frames.push(*leds);
    }
    async fn delay_ms(&mut self, ms: u32) {
        self.waited_ms += u64::from(ms);
    }
    async fn play_tone(&mut self, hz: u32, ms: u32) {
        self.tones.push((hz, ms));
        self.waited_ms += u64::from(ms);
    }
    fn radio_off(&mut self) {
        self.radio.push(RadioCall::Off);
    }
    fn radio_picker(&mut self, channel: u8) {
        self.radio.push(RadioCall::Picker(channel));
    }
    fn broadcast(&mut self, color: Option<Color>) {
        self.radio.push(RadioCall::Broadcast(color));
    }
    fn radio_caller(&mut self, channel: u8) {
        self.radio.push(RadioCall::Caller(channel));
    }
    fn take_received(&mut self) -> Option<(u8, Color)> {
        self.received.pop_front()
    }
}

const UP: (bool, bool) = (false, false);
const A: (bool, bool) = (true, false);
const B: (bool, bool) = (false, true);

/// A booted shell whose board plays `presses`, each followed by a release.
fn shell_with(presses: &[(bool, bool)]) -> Shell<MockBoard> {
    let mut board = MockBoard::default();
    for p in presses {
        board.buttons.push_back(*p);
        board.buttons.push_back(UP);
    }
    let mut shell = Shell::new(board);
    block_on(shell.boot());
    shell
}

fn run_steps(shell: &mut Shell<MockBoard>, n: usize) {
    for _ in 0..n {
        block_on(shell.step());
    }
}

#[test]
fn boot_renders_mode_select() {
    let shell = shell_with(&[]);
    assert_eq!(shell.view().screen, Screen::ModeSelect);
    assert_eq!(shell.board().frames.len(), 1);
}

#[test]
fn held_button_fires_once() {
    let mut board = MockBoard::default();
    // B held for five polls: one ActivateColor, so Picker on channel 0.
    board.buttons.extend([B; 5]);
    let mut shell = Shell::new(board);
    block_on(shell.boot());
    run_steps(&mut shell, 5);
    assert_eq!(shell.view().screen, Screen::Picker);
    assert_eq!(shell.view().channel, 0);
}

#[test]
fn five_skips_enter_caller_on_channel_4() {
    let mut shell = shell_with(&[A, A, A, A, A, B]);
    run_steps(&mut shell, 12);
    assert_eq!(shell.view().screen, Screen::Caller);
    assert_eq!(shell.view().channel, 4);
    assert!(shell.board().radio.contains(&RadioCall::Caller(4)));
}

#[test]
fn picker_broadcasts_the_applied_color() {
    let mut shell = shell_with(&[B, UP, B]);
    run_steps(&mut shell, 6);
    assert_eq!(shell.view().screen, Screen::Picker);
    let color = shell.view().picker_color.expect("a selected color");
    assert!(shell
        .board()
        .radio
        .contains(&RadioCall::Broadcast(Some(color))));
}

#[test]
fn timers_drive_the_blink() {
    let mut shell = shell_with(&[]);
    // One second of mode select: the blink timer fires and the ring is drawn again.
    run_steps(&mut shell, 100);
    assert!(shell.board().frames.len() > 1);
    assert!(shell.board().waited_ms >= 1000);
}

#[test]
fn buttons_play_tones() {
    let mut shell = shell_with(&[A]);
    run_steps(&mut shell, 2);
    assert!(!shell.board().tones.is_empty());
}

#[test]
fn received_color_reaches_the_core() {
    // Caller on channel 4; a received color must not panic and keeps the Caller running.
    let mut shell = shell_with(&[A, A, A, A, A, B]);
    run_steps(&mut shell, 12);
    shell.board_mut().received.push_back((4, Color::Red));
    run_steps(&mut shell, 3);
    assert_eq!(shell.view().screen, Screen::Caller);
    assert!(shell.board().received.is_empty());
}
