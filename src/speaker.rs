//! Lautsprecher über PWM1 @ P0.26 (D12/AUDIO), Verstärker-Enable P1.04 (D11).
//!
//! UI-Feedback für den Color Picker; die 12-Sekunden-Demo liegt in `speaker_demo.rs`.

use core::sync::atomic::{compiler_fence, Ordering};

use nrf_pac::gpio::vals::{Dir, Input, Pull};
use nrf_pac::pwm::vals::{self, CntCnt, LoopCnt, RefreshCnt};
use nrf_pac::shared::vals::Connect;

use crate::time;

const SPEAKER_PIN: u8 = 26;
const AMP_ENABLE_PIN: usize = 4;
const PWM_CLK_HZ: u32 = 1_000_000;

/// Farbe wechseln (Button A / Skip) — dezent, kurz, eher tief.
const SKIP_HZ: u32 = 380;
const SKIP_MS: u32 = 55;
const SKIP_DUTY_PCT: u32 = 22;

/// Farbe übernehmen (Button B, alle LEDs) — höher, Enter-artig.
const APPLY_HZ: u32 = 784;
const APPLY_MS: u32 = 130;
const APPLY_DUTY_PCT: u32 = 34;

static mut TONE_SEQ: [u16; 2] = [500, 500];
static mut READY: bool = false;

pub fn init() {
    enable_amp();
    init_pwm1();
    unsafe {
        READY = true;
    }
}

fn ensure_init() {
    if !unsafe { READY } {
        init();
    }
}

/// Dezenter Ton beim Wechsel zur nächsten Farbe (Auswahl-Phase).
pub async fn play_skip() {
    play_tone(SKIP_HZ, SKIP_MS, Some(SKIP_DUTY_PCT)).await;
}

/// Bestätigungston beim Übernehmen (alle NeoPixels leuchten).
pub async fn play_apply() {
    play_tone(APPLY_HZ, APPLY_MS, Some(APPLY_DUTY_PCT)).await;
}

/// Einzelton; `duty_pct`: None = automatisch nach Frequenz.
pub async fn play_tone(freq_hz: u32, duration_ms: u32, duty_pct: Option<u32>) {
    ensure_init();
    start_tone(freq_hz, duty_pct);
    time::delay_ms_async(duration_ms).await;
    stop_tone();
}

fn tone_duty_pct(freq_hz: u32) -> u32 {
    match freq_hz {
        f if f >= 650 => 26,
        f if f >= 500 => 32,
        f if f >= 350 => 38,
        _ => 42,
    }
}

fn enable_amp() {
    nrf_pac::P1.pin_cnf(AMP_ENABLE_PIN).write(|w| {
        w.set_dir(Dir::OUTPUT);
        w.set_input(Input::DISCONNECT);
        w.set_pull(Pull::DISABLED);
    });
    nrf_pac::P1
        .outset()
        .write_value(nrf_pac::gpio::regs::Outset(1 << AMP_ENABLE_PIN));
}

fn init_pwm1() {
    let pwm = nrf_pac::PWM1;

    pwm.tasks_stop().write_value(1);
    pwm.enable().write(|w| w.set_enable(false));
    pwm.intenclr().write(|w| w.0 = 0xffff_ffff);
    pwm.shorts().write(|_| ());
    pwm.events_stopped().write_value(0);
    pwm.events_seqend(0).write_value(0);
    pwm.events_seqend(1).write_value(0);

    pwm.psel().out(0).write(|w| {
        w.set_pin(SPEAKER_PIN);
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
        .write(|w| w.set_prescaler(vals::Prescaler::DIV_16));
}

fn start_tone(freq_hz: u32, duty_pct: Option<u32>) {
    if freq_hz < 35 || freq_hz > 6000 {
        stop_tone();
        return;
    }

    let period = PWM_CLK_HZ / freq_hz;
    let top = (period - 1).clamp(1, 32767) as u16;
    let duty = duty_pct.unwrap_or_else(|| tone_duty_pct(freq_hz));
    let high = (period * duty / 100).clamp(1, u32::from(top) - 1) as u16;
    let low = u32::from(top) + 1 - u32::from(high);

    unsafe {
        TONE_SEQ[0] = high;
        TONE_SEQ[1] = low.clamp(1, 32767) as u16;
    }

    let pwm = nrf_pac::PWM1;

    pwm.tasks_stop().write_value(1);
    pwm.enable().write(|w| w.set_enable(false));
    pwm.countertop().write(|w| w.set_countertop(top));
    pwm.loop_().write(|w| w.set_cnt(LoopCnt::from_bits(0)));
    pwm.dma()
        .seq(0)
        .refresh()
        .write(|w| w.set_cnt(RefreshCnt::from_bits(0)));
    pwm.dma().seq(0).enddelay().write(|w| w.set_cnt(0));
    pwm.dma()
        .seq(0)
        .ptr()
        .write_value(unsafe { TONE_SEQ.as_ptr() as u32 });
    pwm.dma()
        .seq(0)
        .maxcnt()
        .write(|w| w.set_cnt(CntCnt::from_bits(2)));

    pwm.enable().write(|w| w.set_enable(true));
    compiler_fence(Ordering::SeqCst);
    pwm.tasks_dma().seq(0).start().write_value(1);
}

fn stop_tone() {
    let pwm = nrf_pac::PWM1;
    pwm.tasks_stop().write_value(1);
    pwm.enable().write(|w| w.set_enable(false));
}
