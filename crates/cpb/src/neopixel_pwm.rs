//! WS2812 über nRF52840 PWM @ 16 MHz (wie Embassy neopixel.rs — unabhängig von CPU-Takt).

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

/// Max. Wert pro RGB-Kanal nach Skalierung (8 ≈ 3 % von 255).
pub const MAX_CHANNEL: u8 = 8;

/// Einzelnen Kanal auf `MAX_CHANNEL` begrenzen (linear skaliert).
#[inline(never)]
pub fn scale_channel(v: u8) -> u8 {
    if v == 0 {
        return 0;
    }
    ((v as u32 * MAX_CHANNEL as u32) / 255) as u8
}

/// GRB-Farbe für WS2812
#[derive(Clone, Copy, Debug, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const OFF: Self = Self { r: 0, g: 0, b: 0 };
    pub const ORANGE: Self = Self {
        r: 0x60,
        g: 0x30,
        b: 0x00,
    };

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    #[inline(never)]
    pub fn dimmed(self) -> Self {
        Self {
            r: scale_channel(self.r),
            g: scale_channel(self.g),
            b: scale_channel(self.b),
        }
    }
}

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
        w.set_dir(Dir::OUTPUT);
        w.set_input(Input::DISCONNECT);
        w.set_drive(Drive::H0H1);
        w.set_pull(Pull::DISABLED);
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
        w.set_connect(Connect::CONNECTED);
    });
    for ch in 1..4 {
        pwm.psel().out(ch).write(|w| w.set_connect(Connect::DISCONNECTED));
    }

    pwm.decoder().write(|w| {
        w.set_load(vals::Load::COMMON);
        w.set_mode(vals::Mode::REFRESH_COUNT);
    });
    pwm.mode().write(|w| w.set_updown(vals::Updown::UP));
    pwm.prescaler()
        .write(|w| w.set_prescaler(vals::Prescaler::DIV_1));
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
        w.set_dir(Dir::OUTPUT);
        w.set_input(Input::DISCONNECT);
        w.set_drive(Drive::S0S1);
        w.set_pull(Pull::DISABLED);
    });
    nrf_pac::P0.outclr().write_value(nrf_pac::gpio::regs::Outclr(1 << 6));
}

/// PWM + NeoPixel-Pin einmalig initialisieren (nach `power_on`).
pub fn init() {
    configure_neo_pin();
    init_pwm0();
}

/// Farben für alle 10 NeoPixels anzeigen (Index 0 = links neben USB, dann CCW).
pub fn show_pixels(colors: &[Rgb; NUM_LEDS]) {
    let mut dimmed = [Rgb::OFF; NUM_LEDS];
    for (d, s) in dimmed.iter_mut().zip(colors.iter()) {
        *d = s.dimmed();
    }
    let mut seq = [RES; SEQ_LEN];
    build_pixels(&mut seq, &dimmed);
    play_sequence(&seq);
}

/// Alle NeoPixels orange.
#[allow(dead_code)]
pub fn show_orange() {
    show_pixels(&[Rgb::ORANGE; NUM_LEDS]);
}

/// Alle NeoPixels aus.
#[allow(dead_code)]
pub fn show_off() {
    show_pixels(&[Rgb::OFF; NUM_LEDS]);
}
