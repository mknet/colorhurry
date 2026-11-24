//! WS2812 via nRF52840 PWM @ 16 MHz (like Embassy neopixel.rs, independent of CPU clock).

use core::sync::atomic::{compiler_fence, Ordering};

use nrf_pac::gpio::vals::{Dir, Drive, Input, Pull};
use nrf_pac::pwm::vals::{self, CntCnt, LoopCnt, RefreshCnt};
use nrf_pac::shared::vals::Connect;

pub const NUM_LEDS: usize = 10;

const SEQ_LEN: usize = NUM_LEDS * 24 + 1;
const T1H: u16 = 0x8000 | 13;
const T0H: u16 = 0x8000 | 7;
const RES: u16 = 0x8000;
const END_DELAY: u32 = 799;

pub use color_hurry_core::Rgb;

fn push_byte(seq: &mut [u16], idx: &mut usize, byte: u8) {
    for bit in (0..8).rev() {
        seq[*idx] = if byte & (1 << bit) != 0 { T1H } else { T0H };
        *idx += 1;
    }
}

fn build_pixels(seq: &mut [u16; SEQ_LEN], colors: &[Rgb; NUM_LEDS]) {
    let mut i = 0;
    for c in colors {
        push_byte(seq, &mut i, c.g);
        push_byte(seq, &mut i, c.r);
        push_byte(seq, &mut i, c.b);
    }
    seq[i] = RES;
}

fn configure_neo_pin() {
    nrf_pac::P0.pin_cnf(13).write(|w| {
        w.set_dir(Dir::Output);
        w.set_input(Input::Disconnect);
        w.set_drive(Drive::H0h1);
        w.set_pull(Pull::Disabled);
    });
}

fn init_pwm0() {
    let pwm = nrf_pac::PWM0;

    pwm.tasks_stop().write_value(1);
    pwm.enable().write(|w| w.set_enable(false));

    pwm.intenclr().write(|w| w.0 = 0xffff_ffff);
    pwm.shorts().write(|_| ());
    pwm.events_stopped().write_value(0);
    pwm.events_seqend(0).write_value(0);
    pwm.events_seqend(1).write_value(0);

    pwm.psel().out(0).write(|w| {
        w.set_pin(13);
        w.set_port(false);
        w.set_connect(Connect::Connected);
    });
    for ch in 1..4 {
        pwm.psel().out(ch).write(|w| w.set_connect(Connect::Disconnected));
    }

    pwm.decoder().write(|w| {
        w.set_load(vals::Load::Common);
        w.set_mode(vals::Mode::RefreshCount);
    });
    pwm.mode().write(|w| w.set_updown(vals::Updown::Up));
    pwm.prescaler()
        .write(|w| w.set_prescaler(vals::Prescaler::Div1));
    pwm.countertop().write(|w| w.set_countertop(20));
}

fn play_sequence(seq: &[u16; SEQ_LEN]) {
    let pwm = nrf_pac::PWM0;
    let len = SEQ_LEN as u16;

    pwm.loop_().write(|w| w.set_cnt(LoopCnt::from_bits(1)));
    pwm.dma().seq(0).refresh().write(|w| w.set_cnt(RefreshCnt::from_bits(0)));
    pwm.dma().seq(0).enddelay().write(|w| w.set_cnt(END_DELAY));
    pwm.dma().seq(0).ptr().write_value(seq.as_ptr() as u32);
    pwm.dma()
        .seq(0)
        .maxcnt()
        .write(|w| w.set_cnt(CntCnt::from_bits(len)));

    pwm.enable().write(|w| w.set_enable(true));
    compiler_fence(Ordering::SeqCst);
    pwm.tasks_dma().seq(0).start().write_value(1);

    while pwm.events_seqend(0).read() == 0 {
        core::hint::spin_loop();
    }
    pwm.events_seqend(0).write_value(0);

    pwm.tasks_stop().write_value(1);
    pwm.enable().write(|w| w.set_enable(false));
}

/// NeoPixel-Stromversorgung einschalten (D35 / P0.06, LOW = an).
pub fn power_on() {
    nrf_pac::P0.pin_cnf(6).write(|w| {
        w.set_dir(Dir::Output);
        w.set_input(Input::Disconnect);
        w.set_drive(Drive::S0s1);
        w.set_pull(Pull::Disabled);
    });
    nrf_pac::P0.outclr().write_value(nrf_pac::gpio::regs::Outclr(1 << 6));
}

/// Initialize PWM + NeoPixel pin once (after `power_on`).
pub fn init() {
    configure_neo_pin();
    init_pwm0();
}

/// Show colors on all 10 NeoPixels (index 0 = left of USB, then CCW), as given.
///
/// The driver does not dim. The board dims in `board.rs` (`color_hurry_shell::leds`).
pub fn show_pixels(colors: &[Rgb; NUM_LEDS]) {
    let mut seq = [RES; SEQ_LEN];
    build_pixels(&mut seq, colors);
    play_sequence(&seq);
}
