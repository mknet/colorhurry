//! Color Hurry: CPB shell (hardware + Bluetooth) over `color-hurry-core`.
//!
//! Bluetooth runs on Nordic's SoftDevice Controller (`nrf-sdc` with MPSL), which is linked
//! into this image. The resident SoftDevice on the board stays disabled, so the same code
//! runs on boards with S140 6.x and 7.x. Only the memory layout differs (see `build.rs`).

#![no_std]
#![no_main]

extern crate alloc;

use core::mem::MaybeUninit;

use embassy_executor::Spawner;
use embassy_nrf::interrupt::Priority;
use embassy_nrf::mode::Async;
use embassy_nrf::peripherals::RNG;
use embassy_nrf::{bind_interrupts, rng};
use embedded_alloc::LlffHeap as Heap;
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc, mpsl, SoftdeviceController};
use panic_halt as _;
use static_cell::StaticCell;

#[global_allocator]
static HEAP: Heap = Heap::empty();

const HEAP_SIZE: usize = 32 * 1024;

mod ble_broadcast;
mod buttons;
mod debug_led;
mod game;
mod neopixel_pwm;
mod shell;
mod speaker;
#[cfg(feature = "speaker-demo")]
mod speaker_demo;
mod time;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<RNG>;
    EGU0_SWI0 => mpsl::LowPrioInterruptHandler;
    CLOCK_POWER => mpsl::ClockInterruptHandler;
    RADIO => mpsl::HighPrioInterruptHandler;
    TIMER0 => mpsl::HighPrioInterruptHandler;
    RTC0 => mpsl::HighPrioInterruptHandler;
});

/// Memory for the SoftDevice Controller.
///
/// `sdc::Builder::build` refuses a buffer that is too small, and accepts a bigger one. The
/// `sdc.h` formulas give about 1.4 KB for this configuration (one advertising set of 31
/// bytes, a legacy scanner with 3 report buffers, and the filter accept list). That sum does
/// not include the base memory of the library, which `Builder::required_memory()` returns
/// only on the board. 8 KB is the size that the scan probe used on the board. It costs
/// 8 KB of about 232 KB of RAM.
const SDC_MEM: usize = 8192;

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}

#[embassy_executor::task]
async fn game_task() -> ! {
    game::run().await
}

#[cfg(feature = "speaker-demo")]
#[embassy_executor::task]
async fn speaker_demo_task() -> ! {
    speaker_demo::run().await
}

/// Runs the Bluetooth host and the radio. `None` if the radio set-up failed.
#[embassy_executor::task]
async fn ble_system(controller: Option<SoftdeviceController<'static>>) -> ! {
    ble_broadcast::run(controller).await
}

/// Start MPSL and build the SoftDevice Controller with an advertiser and a scanner.
///
/// Returns `None` if MPSL or the controller refuses the configuration.
async fn start_radio(
    spawner: Spawner,
    p: embassy_nrf::Peripherals,
) -> Option<SoftdeviceController<'static>> {
    static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
    static RNG_CELL: StaticCell<rng::Rng<'static, Async>> = StaticCell::new();
    static MEM: StaticCell<sdc::Mem<SDC_MEM>> = StaticCell::new();

    let mpsl_p =
        mpsl::Peripherals::new(p.RTC0, p.TIMER0, p.TEMP, p.PPI_CH19, p.PPI_CH30, p.PPI_CH31);
    // The CPB has no 32 kHz crystal, so LFCLK runs from the RC oscillator with calibration.
    let lfclk_cfg = mpsl::raw::mpsl_clock_lfclk_cfg_t {
        source: mpsl::raw::MPSL_CLOCK_LF_SRC_RC as u8,
        rc_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_CTIV as u8,
        rc_temp_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
        accuracy_ppm: mpsl::raw::MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
        skip_wait_lfclk_started: mpsl::raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
    };
    let mpsl = MPSL.init(MultiprotocolServiceLayer::new(mpsl_p, Irqs, lfclk_cfg).ok()?);
    spawner.spawn(mpsl_task(mpsl).ok()?);

    // Keep the 16 MHz crystal (HFXO) on for the life of the firmware, as the old firmware
    // did. TIMER4 (delays, blinks and countdown beats) and the PWM tones get their clock
    // from HFCLK. Without this request, MPSL runs the crystal only for radio events, and the
    // less accurate internal oscillator (HFINT) clocks them at other times. `forget` keeps
    // the guard, so MPSL never releases the crystal. If the request fails, the game runs on
    // HFINT.
    if let Ok(hfclk) = mpsl.request_hfclk().await {
        let _ = mpsl::Hfclk::wait().await;
        core::mem::forget(hfclk);
    }

    let sdc_p = sdc::Peripherals::new(
        p.PPI_CH17, p.PPI_CH18, p.PPI_CH20, p.PPI_CH21, p.PPI_CH22, p.PPI_CH23, p.PPI_CH24,
        p.PPI_CH25, p.PPI_CH26, p.PPI_CH27, p.PPI_CH28, p.PPI_CH29,
    );
    let rng = RNG_CELL.init(rng::Rng::new(p.RNG, Irqs));
    let mem = MEM.init(sdc::Mem::new());

    sdc::Builder::new()
        .ok()?
        .support_adv()
        .support_scan()
        // No connections: no links of either role.
        .central_count(0)
        .ok()?
        .peripheral_count(0)
        .ok()?
        .adv_count(1)
        .ok()?
        .build(sdc_p, rng, mpsl, mem)
        .ok()
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    {
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        #[allow(static_mut_refs)]
        unsafe {
            HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE);
        }
    }

    // MPSL runs RADIO, TIMER0 and RTC0 at priority 0. The app interrupts (the RTC1 time
    // driver and GPIOTE) run at a lower priority, so they cannot delay the radio.
    let mut config = embassy_nrf::config::Config::default();
    config.time_interrupt_priority = Priority::P2;
    config.gpiote_interrupt_priority = Priority::P2;
    let p = embassy_nrf::init(config);

    // MPSL starts and requests the crystal before the app uses TIMER4 or the PWMs. If MPSL
    // does not start, there is no crystal request, and TIMER4 and the PWMs run on HFINT.
    let controller = start_radio(spawner, p).await;

    neopixel_pwm::power_on();
    neopixel_pwm::init();
    debug_led::init();
    debug_led::show_flash_layout().await;
    speaker::init();

    spawner.spawn(ble_system(controller).unwrap());
    spawner.spawn(game_task().unwrap());
    #[cfg(feature = "speaker-demo")]
    spawner.spawn(speaker_demo_task().unwrap());
}
