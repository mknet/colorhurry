//! Lautsprecher über PWM1 @ P0.26 (D12/AUDIO), Verstärker-Enable P1.04 (D11).

use core::sync::atomic::{compiler_fence, Ordering};

use nrf_pac::gpio::vals::{Dir, Input, Pull};
use nrf_pac::pwm::vals::{self, CntCnt, LoopCnt, RefreshCnt};
use nrf_pac::shared::vals::Connect;

use crate::time;

const SPEAKER_PIN: u8 = 26;
const AMP_ENABLE_PIN: usize = 4; // P1.04 — LOW = aus, HIGH / Pull-up = an
const PWM_CLK_HZ: u32 = 1_000_000; // 16 MHz / 16

/// Genau ein Ereignis pro Sekunde.
const BEAT_MS: u32 = 1000;
const TONE_MS: u32 = 200;

/// Gitarre: g–h–e (G3, H3, E4) — aufwärts wie Anschlag der oberen drei Saiten.
const RESET_STRUM: [(u32, u32); 3] = [
    (196, 55), // g (G3)
    (247, 55), // h (H3 / B3)
    (330, 85), // e (E4), höchste Saite klingt etwas nach
];
const RESET_GAP_MS: u32 = 12;

const DESCENDING_STEPS: usize = 10;
/// Abgestufte Töne nach dem Reset — Ende ~200 Hz.
const DESCENDING_HZ: [u32; DESCENDING_STEPS] =
    [680, 620, 560, 500, 450, 400, 350, 300, 250, 200];

const EXPLOSION_START_HZ: u32 = 180;
const EXPLOSION_END_HZ: u32 = 45;
const EXPLOSION_STEPS: u32 = 10;

static mut TONE_SEQ: [u16; 2] = [500, 500];

static mut READY: bool = false;

/// Verstärker einschalten und PWM1 vorbereiten.
pub fn init() {
    enable_amp();
    init_pwm1();
    unsafe {
        READY = true;
    }
}

/// Endlosschleife: S1 Reset → S2–11 absteigend → S12 Explosion (volle Sekunde).
pub async fn run_demo() -> ! {
    init();
    loop {
        play_reset().await;

        for i in 0..DESCENDING_STEPS {
            play_beat(descending_freq(i)).await;
        }

        play_explosion().await;
    }
}

/// Sekunde 1: kurzer Gitarren-Anschlag (g–h–e), dann Pause bis 1 s.
async fn play_reset() {
    if !unsafe { READY } {
        init();
    }
    let mut elapsed = 0u32;
    for (i, &(freq, note_ms)) in RESET_STRUM.iter().enumerate() {
        start_tone(freq);
        time::delay_ms_async(note_ms).await;
        stop_tone();
        elapsed += note_ms;
        if i + 1 < RESET_STRUM.len() {
            time::delay_ms_async(RESET_GAP_MS).await;
            elapsed += RESET_GAP_MS;
        }
    }
    time::delay_ms_async(BEAT_MS.saturating_sub(elapsed)).await;
}

/// Kurzer Piep (~200 ms), dann Pause — zusammen 1 s.
async fn play_beat(freq_hz: u32) {
    if !unsafe { READY } {
        init();
    }
    start_tone(freq_hz);
    time::delay_ms_async(TONE_MS).await;
    stop_tone();
    time::delay_ms_async(BEAT_MS.saturating_sub(TONE_MS)).await;
}

/// Sekunde 12: durchgehender Tiefton mit Abwärts-Sweep (~Explosion).
async fn play_explosion() {
    if !unsafe { READY } {
        init();
    }
    let step_ms = BEAT_MS / EXPLOSION_STEPS;
    for i in 0..EXPLOSION_STEPS {
        let freq = explosion_freq(i);
        start_tone(freq);
        time::delay_ms_async(step_ms).await;
    }
    stop_tone();
}

fn explosion_freq(step: u32) -> u32 {
    if EXPLOSION_STEPS <= 1 {
        return EXPLOSION_END_HZ;
    }
    let span = EXPLOSION_START_HZ - EXPLOSION_END_HZ;
    EXPLOSION_START_HZ - span * step / (EXPLOSION_STEPS - 1)
}

fn descending_freq(step: usize) -> u32 {
    DESCENDING_HZ[step.min(DESCENDING_STEPS - 1)]
}

/// Kürzere Einschaltdauer bei hohen Tönen → weniger Obertöne, weicherer Klang.
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

fn start_tone(freq_hz: u32) {
    if freq_hz < 35 || freq_hz > 6000 {
        stop_tone();
        return;
    }

    let period = PWM_CLK_HZ / freq_hz;
    let top = (period - 1).clamp(1, 32767) as u16;
    let duty = tone_duty_pct(freq_hz);
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
