//! BLE: Picker advertises `CMD_COLOR` (protocol v4); caller scans and forwards parses.
//!
//! The stack is the `trouble-host` host on Nordic's SoftDevice Controller (`nrf-sdc`). The
//! board uses no connections: the Picker sends non-connectable advertisements, and the
//! Caller runs a passive scan. Because there are no connections, there is no pairing.

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};

use critical_section::Mutex;
use embassy_futures::join::join;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use nrf_sdc::SoftdeviceController;
use trouble_host::prelude::*;
use trouble_host::scan::LeAdvReportsIter;

use color_hurry_core::{decode_color, encode_color, Color, CHANNELS, COMPANY_ID, PAYLOAD_LEN};

/// Visible BLE device name (scan response).
pub const DEVICE_NAME: &str = "ColorHurry-CPB";

/// Short name in the advertisement.
const SHORT_NAME: &[u8] = b"ColHurry";

static BLE_STACK_READY: AtomicBool = AtomicBool::new(false);

pub fn mark_stack_ready() {
    BLE_STACK_READY.store(true, Ordering::Release);
}

pub async fn wait_until_ready() {
    while !BLE_STACK_READY.load(Ordering::Acquire) {
        embassy_futures::yield_now().await;
    }
}

const AD_TYPE_MANUFACTURER: u8 = 0xFF;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RadioMode {
    Off,
    Picker,
    Caller,
}

struct BleShared {
    mode: RadioMode,
    picker_color: Option<Color>,
    picker_channel: usize,
    /// The v4 payload that the Picker advertises.
    picker_payload: [u8; PAYLOAD_LEN],
    /// Last parsed advertisement for the core (unfiltered).
    received: Option<(u8, Color)>,
}

static BLE: Mutex<RefCell<BleShared>> = Mutex::new(RefCell::new(BleShared {
    mode: RadioMode::Off,
    picker_color: None,
    picker_channel: 0,
    picker_payload: [0; PAYLOAD_LEN],
    received: None,
}));

static CALLER_ARMED: AtomicBool = AtomicBool::new(false);

/// Each setter raises this signal. `radio_loop` then compares the new state with what the
/// radio does now.
static RADIO_CHANGED: Signal<CriticalSectionRawMutex, ()> = Signal::new();

fn with_ble<R>(f: impl FnOnce(&mut BleShared) -> R) -> R {
    critical_section::with(|cs| f(&mut *BLE.borrow(cs).borrow_mut()))
}

pub fn set_radio_off() {
    with_ble(|s| {
        s.mode = RadioMode::Off;
        s.picker_color = None;
        s.received = None;
    });
    CALLER_ARMED.store(false, Ordering::Release);
    RADIO_CHANGED.signal(());
}

pub fn set_radio_picker(channel: usize) {
    let ch = channel.min(CHANNELS - 1);
    with_ble(|s| {
        s.mode = RadioMode::Picker;
        s.picker_channel = ch;
        s.received = None;
    });
    CALLER_ARMED.store(false, Ordering::Release);
    RADIO_CHANGED.signal(());
}

pub fn set_picker_broadcast(color: Option<Color>) {
    with_ble(|s| {
        if let Some(c) = color {
            if let Some(bytes) = encode_color(s.picker_channel as u8, c) {
                s.picker_payload = bytes;
            }
        }
        s.picker_color = color;
    });
    RADIO_CHANGED.signal(());
}

/// Arm scan; core decides matches. `channel` kept for future directed scan filters.
pub fn set_radio_caller(_channel: usize) {
    with_ble(|s| {
        s.mode = RadioMode::Caller;
        s.picker_color = None;
        s.received = None;
    });
    CALLER_ARMED.store(true, Ordering::Release);
    RADIO_CHANGED.signal(());
}

pub fn take_color_received() -> Option<(u8, Color)> {
    with_ble(|s| s.received.take())
}

pub fn parse_color_from_adv(data: &[u8]) -> Option<(u8, Color)> {
    let mut i = 0;
    while i < data.len() {
        let len = data[i] as usize;
        if len == 0 {
            break;
        }
        if i + 1 + len > data.len() {
            break;
        }
        let typ = data[i + 1];
        if typ == AD_TYPE_MANUFACTURER && len >= 2 + PAYLOAD_LEN {
            let ad = &data[i + 2..i + 1 + len];
            if ad.len() >= 2 + PAYLOAD_LEN
                && u16::from_le_bytes([ad[0], ad[1]]) == COMPANY_ID
            {
                return decode_color(&ad[2..]);
            }
        }
        i += len + 1;
    }
    None
}

/// Receives the advertising reports from the host runner.
///
/// The runner calls this handler synchronously, so the handler only stores the result.
struct Reports;

impl EventHandler for Reports {
    fn on_adv_reports(&self, reports: LeAdvReportsIter) {
        if !CALLER_ARMED.load(Ordering::Acquire) {
            return;
        }
        // `flatten` skips a report that does not parse. The iterator then stops.
        for report in reports.flatten() {
            if let Some(parsed) = parse_color_from_adv(report.data) {
                with_ble(|s| s.received = Some(parsed));
            }
        }
    }
}

/// What the radio must do for the current state.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Radio {
    Off,
    Advertise([u8; PAYLOAD_LEN]),
    Scan,
}

fn wanted() -> Radio {
    with_ble(|s| match s.mode {
        RadioMode::Picker if s.picker_color.is_some() => Radio::Advertise(s.picker_payload),
        RadioMode::Caller => Radio::Scan,
        RadioMode::Off | RadioMode::Picker => Radio::Off,
    })
}

/// Waits until the state needs something other than `current`.
async fn wait_for_change(current: Radio) {
    loop {
        RADIO_CHANGED.wait().await;
        if wanted() != current {
            return;
        }
    }
}

/// The static random address of the board, from the factory address in FICR.
///
/// This is the factory address, so each board keeps its own address.
fn device_address() -> Address {
    let low = nrf_pac::FICR.deviceaddr(0).read().to_le_bytes();
    let high = nrf_pac::FICR.deviceaddr(1).read().to_le_bytes();
    // A static random address has the two most significant bits set.
    Address::random([low[0], low[1], low[2], low[3], high[0], high[1] | 0xC0])
}

/// Write the advertisement and the scan response for `payload`. Returns their lengths.
fn encode_packets(
    payload: &[u8; PAYLOAD_LEN],
    adv: &mut [u8; 31],
    scan: &mut [u8; 31],
) -> Result<(usize, usize), trouble_host::Error> {
    let adv_len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ShortenedLocalName(SHORT_NAME),
            AdStructure::ManufacturerSpecificData {
                company_identifier: COMPANY_ID,
                payload,
            },
        ],
        &mut adv[..],
    )?;
    let scan_len = AdStructure::encode_slice(
        &[AdStructure::CompleteLocalName(DEVICE_NAME.as_bytes())],
        &mut scan[..],
    )?;
    Ok((adv_len, scan_len))
}

/// Start the host on `controller`, and run the radio forever.
///
/// If `controller` is `None`, the radio set-up failed in `main`. The game then runs
/// without Bluetooth.
pub async fn run(controller: Option<SoftdeviceController<'static>>) -> ! {
    let Some(controller) = controller else {
        crate::debug_led::signal_radio_error().await;
        mark_stack_ready();
        core::future::pending::<()>().await;
        unreachable!()
    };

    // No connections and no L2CAP channels.
    let mut resources: HostResources<DefaultPacketPool, 0, 0> = HostResources::new();
    let stack = trouble_host::new(controller, &mut resources)
        .set_random_address(device_address())
        .build();
    let mut runner = stack.runner();

    crate::debug_led::signal_radio_ok().await;
    // The host waits for the runner to initialize the controller before it sends a command.
    mark_stack_ready();

    join(
        async {
            // The runner stops only if the controller fails.
            let _ = runner.run_with_handler(&Reports).await;
            loop {
                crate::debug_led::signal_radio_error().await;
                Timer::after_secs(2).await;
            }
        },
        radio_loop(&stack),
    )
    .await;
    unreachable!()
}

async fn radio_loop(stack: &Stack<'_, SoftdeviceController<'static>, DefaultPacketPool>) -> ! {
    let mut peripheral = stack.peripheral();
    let mut central = stack.central();
    let mut scanner = Scanner::new(&mut central);

    // 48 × 0.625 ms, as before.
    let adv_params = AdvertisementParameters {
        interval_min: Duration::from_millis(30),
        interval_max: Duration::from_millis(30),
        ..AdvertisementParameters::default()
    };
    // Passive scan. The color is in the advertisement, so the scan response is not needed.
    let scan_config = ScanConfig {
        active: false,
        interval: Duration::from_millis(100),
        window: Duration::from_millis(100),
        ..ScanConfig::default()
    };

    loop {
        let current = wanted();
        match current {
            Radio::Off => RADIO_CHANGED.wait().await,
            Radio::Advertise(payload) => {
                let mut adv = [0; 31];
                let mut scan = [0; 31];
                let Ok((adv_len, scan_len)) = encode_packets(&payload, &mut adv, &mut scan) else {
                    radio_error().await;
                    continue;
                };
                let advertisement = Advertisement::NonconnectableScannableUndirected {
                    adv_data: &adv[..adv_len],
                    scan_data: &scan[..scan_len],
                };
                match peripheral.advertise(&adv_params, advertisement).await {
                    // The advertisement stays on until the advertiser is dropped.
                    Ok(advertiser) => {
                        wait_for_change(current).await;
                        drop(advertiser);
                    }
                    Err(_) => radio_error().await,
                }
            }
            Radio::Scan => match scanner.scan(&scan_config).await {
                // The scan stays on until the session is dropped.
                Ok(session) => {
                    wait_for_change(current).await;
                    drop(session);
                }
                Err(_) => radio_error().await,
            },
        }
    }
}

/// Show the error, then wait before the next attempt.
async fn radio_error() {
    crate::debug_led::signal_radio_error().await;
    Timer::after_secs(1).await;
}
