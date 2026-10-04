//! Lautsprecher über PWM1 @ P0.26 (D12/AUDIO), Verstärker-Enable P1.04 (D11).
//!
//! UI-Feedback: kurze Töne ohne Blockieren der Button-Schleife (`tone_service`).

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

/// Bestätigungston beim Übernehmen (Button B, alle LEDs) — höher, Enter-artig.
const APPLY_HZ: u32 = 784;
const APPLY_MS: u32 = 130;
const APPLY_DUTY_PCT: u32 = 34;

static mut TONE_SEQ: [u16; 2] = [500, 500];
static mut READY: bool = false;
static mut TONE_ACTIVE: bool = false;
/// Failsafe: `tone_service`-Aufrufe bis zum erzwungenen Stopp.
static mut TONE_MAX_SERVICES: u8 = 0;
static mut TONE_SERVICE_COUNT: u8 = 0;

pub fn init() {
    enable_amp();
    init_pwm1();
    unsafe {
        READY = true;
        TONE_ACTIVE = false;
        TONE_MAX_SERVICES = 0;
        TONE_SERVICE_COUNT = 0;
    }
    stop_tone();
}

fn ensure_init() {
    if !unsafe { READY } {
        init();
    }
}

/// Dezenter Ton beim Wechsel zur nächsten Farbe — blockiert nicht.
pub fn play_skip() {
    play_ui_tone(SKIP_HZ, SKIP_MS, Some(SKIP_DUTY_PCT));
}

/// Bestätigungston beim Übernehmen — blockiert nicht.
pub fn play_apply() {
    play_ui_tone(APPLY_HZ, APPLY_MS, Some(APPLY_DUTY_PCT));
}

fn play_ui_tone(freq_hz: u32, duration_ms: u32, duty_pct: Option<u32>) {
    ensure_init();
    stop_tone();
    start_tone(freq_hz, duration_ms, duty_pct, true);
}

/// In jeder UI-Schleife (vor und nach `render`) aufrufen.
pub fn tone_service() {
    if !unsafe { TONE_ACTIVE } {
        return;
    }

    unsafe {
        TONE_SERVICE_COUNT = TONE_SERVICE_COUNT.saturating_add(1);
    }

    let pwm = nrf_pac::PWM1;
    if pwm.events_seqend(0).read() != 0 {
        pwm.events_seqend(0).write_value(0);
        stop_tone();
        return;
    }

    unsafe {
        if TONE_MAX_SERVICES > 0 && TONE_SERVICE_COUNT >= TONE_MAX_SERVICES {
            stop_tone();
        }
    }
}

/// Absteigender „Explosion“-Sweep (nach Countdown ohne Treffer).
pub async fn play_explosion() {
    const STEPS: u32 = 10;
    const START_HZ: u32 = 180;
    const END_HZ: u32 = 45;
    const STEP_MS: u32 = 100;

    for i in 0..STEPS {
        let freq = START_HZ - (START_HZ - END_HZ) * i / (STEPS - 1).max(1);
        play_tone(freq, STEP_MS, None).await;
    }
}

/// Einzelton für Countdown/Demo: volle `duration_ms` (wie früher per Timer, ohne UI-Failsafe).
pub async fn play_tone(freq_hz: u32, duration_ms: u32, duty_pct: Option<u32>) {
    ensure_init();
    stop_tone();
    start_tone(freq_hz, duration_ms, duty_pct, false);
    time::delay_ms_async(duration_ms).await;
    stop_tone();
}

fn service_budget(duration_ms: u32) -> u8 {
    // UI-Tick ~50 ms, `tone_service` typisch 2× pro Tick → ~25 ms pro Zähler
    ((duration_ms + 20) / 25).clamp(2, 12) as u8
}

fn loops_for_duration(freq_hz: u32, duration_ms: u32) -> u16 {
    let period_us = 1_000_000 / u64::from(freq_hz.max(1));
    let total_us = u64::from(duration_ms.saturating_mul(1000));
    ((total_us / period_us).max(1).min(0xffff)) as u16
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

fn start_tone(freq_hz: u32, duration_ms: u32, duty_pct: Option<u32>, ui_failsafe: bool) {
    if freq_hz < 35 || freq_hz > 6000 {
        stop_tone();
        return;
    }

    let period = PWM_CLK_HZ / freq_hz;
    let top = (period - 1).clamp(1, 32767) as u16;
    let duty = duty_pct.unwrap_or_else(|| tone_duty_pct(freq_hz));
    let high = (period * duty / 100).clamp(1, u32::from(top) - 1) as u16;
    let low = u32::from(top) + 1 - u32::from(high);
    let loops = loops_for_duration(freq_hz, duration_ms);

    unsafe {
        TONE_SEQ[0] = high;
        TONE_SEQ[1] = low.clamp(1, 32767) as u16;
    }

    let pwm = nrf_pac::PWM1;

    pwm.tasks_stop().write_value(1);
    pwm.enable().write(|w| w.set_enable(false));
    pwm.countertop().write(|w| w.set_countertop(top));
    // LOOP=0 wäre Endlosschleife — Dauer über berechnete Wiederholungen.
    pwm.loop_().write(|w| w.set_cnt(LoopCnt::from_bits(loops)));
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

    pwm.events_seqend(0).write_value(0);
    pwm.enable().write(|w| w.set_enable(true));
    compiler_fence(Ordering::SeqCst);
    pwm.tasks_dma().seq(0).start().write_value(1);
    unsafe {
        TONE_ACTIVE = true;
        TONE_MAX_SERVICES = if ui_failsafe {
            service_budget(duration_ms)
        } else {
            0
        };
        TONE_SERVICE_COUNT = 0;
    }
}

/// Lautsprecher sofort abschalten (z. B. nach Moduswahl-Bestätigungston).
pub fn stop_tone() {
    let pwm = nrf_pac::PWM1;
    pwm.tasks_stop().write_value(1);
    pwm.enable().write(|w| w.set_enable(false));
    pwm.events_seqend(0).write_value(0);
    unsafe {
        TONE_ACTIVE = false;
        TONE_MAX_SERVICES = 0;
        TONE_SERVICE_COUNT = 0;
    }
}
