//! BLE: Picker advertised `CMD_COLOR` nach Bestätigung; Empfänger scannt passende Farbe.

use core::cell::RefCell;
use core::mem;
use core::sync::atomic::{AtomicBool, Ordering};

use critical_section::Mutex;
use nrf_softdevice::ble::advertisement_builder::{
    AdvertisementDataType, Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload,
};
use nrf_softdevice::ble::central::{self, ScanConfig, ScanError};
use nrf_softdevice::ble::peripheral::{self, AdvertiseError};
use nrf_softdevice::{raw, Softdevice};

use crate::neopixel_pwm::Rgb;

/// Sichtbarer BLE-Gerätename (Scan Response + GAP).
pub const DEVICE_NAME: &str = "ColorHurry-CPB";

static BLE_STACK_READY: AtomicBool = AtomicBool::new(false);

/// SoftDevice + `sd.run()`-Task laufen — Picker/Empfänger erst danach starten.
pub fn mark_stack_ready() {
    BLE_STACK_READY.store(true, Ordering::Release);
}

pub async fn wait_until_ready() {
    while !BLE_STACK_READY.load(Ordering::Acquire) {
        embassy_futures::yield_now().await;
    }
}

/// Adafruit Company ID (Bluetooth SIG).
pub const COMPANY_ID: u16 = 0x239A;

const AD_TYPE_MANUFACTURER: u8 = 0xFF;

/// Broadcast-Protokoll (Manufacturer-Data nach Company ID).
pub mod protocol {
    pub const MAGIC: u8 = b'S';
    /// Payload: `[opcode, r, g, b]`
    pub const VERSION: u8 = 2;

    #[allow(dead_code)]
    pub const CMD_PING: u8 = 0x01;
    #[allow(dead_code)]
    pub const CMD_READY: u8 = 0x02;
    /// Aktuelle Farbe (Palette-Werte 0–255, vor NeoPixel-Dimming).
    pub const CMD_COLOR: u8 = 0x03;
}

/// Manufacturer-Payload: Magic, Version, Opcode, R, G, B.
static mut CMD_PAYLOAD: [u8; 6] = [
    protocol::MAGIC,
    protocol::VERSION,
    protocol::CMD_COLOR,
    255,
    0,
    0,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum RadioMode {
    Off,
    Picker,
    Receiver,
}

struct BleShared {
    mode: RadioMode,
    /// `Some` = Picker sendet diese Farbe; `None` = nicht senden (Auswahl).
    picker_color: Option<Rgb>,
    receiver_target: Option<Rgb>,
    matched: bool,
}

static BLE: Mutex<RefCell<BleShared>> = Mutex::new(RefCell::new(BleShared {
    mode: RadioMode::Off,
    picker_color: None,
    receiver_target: None,
    matched: false,
}));

static RECEIVER_ARMED: AtomicBool = AtomicBool::new(false);

fn with_ble<R>(f: impl FnOnce(&mut BleShared) -> R) -> R {
    critical_section::with(|cs| f(&mut *BLE.borrow(cs).borrow_mut()))
}

/// Setup / Moduswahl: kein Senden und kein Scan.
pub fn set_radio_off() {
    with_ble(|s| {
        s.mode = RadioMode::Off;
        s.picker_color = None;
        s.receiver_target = None;
        s.matched = false;
    });
    RECEIVER_ARMED.store(false, Ordering::Release);
}

/// Picker: Advertising nur wenn `set_picker_broadcast(Some(..))`.
pub fn set_radio_picker() {
    with_ble(|s| {
        s.mode = RadioMode::Picker;
        s.receiver_target = None;
        s.matched = false;
    });
    RECEIVER_ARMED.store(false, Ordering::Release);
}

pub fn set_picker_broadcast(color: Option<Rgb>) {
    if let Some(c) = color {
        set_color(c);
    }
    with_ble(|s| s.picker_color = color);
}

/// Empfänger: scannt nach `CMD_COLOR` == Ziel-Farbe.
pub fn set_radio_receiver(target: Rgb) {
    with_ble(|s| {
        s.mode = RadioMode::Receiver;
        s.receiver_target = Some(target);
        s.picker_color = None;
        s.matched = false;
    });
    RECEIVER_ARMED.store(true, Ordering::Release);
}

pub fn take_color_match() -> bool {
    with_ble(|s| {
        if s.matched {
            s.matched = false;
            true
        } else {
            false
        }
    })
}

/// Aktuelle Farbe für den nächsten Advertising-Zyklus (volle Palette-Werte).
pub fn set_color(color: Rgb) {
    unsafe {
        CMD_PAYLOAD[2] = protocol::CMD_COLOR;
        CMD_PAYLOAD[3] = color.r;
        CMD_PAYLOAD[4] = color.g;
        CMD_PAYLOAD[5] = color.b;
    }
}

fn manufacturer_bytes() -> [u8; 8] {
    let mut mfg = [0u8; 8];
    mfg[0] = (COMPANY_ID & 0xFF) as u8;
    mfg[1] = (COMPANY_ID >> 8) as u8;
    let payload = unsafe { CMD_PAYLOAD };
    mfg[2..8].copy_from_slice(&payload);
    mfg
}

fn adv_data() -> LegacyAdvertisementPayload {
    let mfg = manufacturer_bytes();
    LegacyAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        .short_name("ColHurry")
        .raw(AdvertisementDataType::MANUFACTURER_SPECIFIC_DATA, &mfg)
        .build()
}

fn scan_data() -> LegacyAdvertisementPayload {
    LegacyAdvertisementBuilder::new()
        .full_name(DEVICE_NAME)
        .build()
}

/// Parst `CMD_COLOR` aus Advertising- oder Scan-Response-Daten.
pub fn parse_color_from_adv(data: &[u8]) -> Option<Rgb> {
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
        if typ == AD_TYPE_MANUFACTURER && len >= 8 {
            let ad = &data[i + 2..i + 1 + len];
            if ad.len() >= 8 && u16::from_le_bytes([ad[0], ad[1]]) == COMPANY_ID {
                let payload = &ad[2..];
                if payload.len() >= 6
                    && payload[0] == protocol::MAGIC
                    && payload[1] == protocol::VERSION
                    && payload[2] == protocol::CMD_COLOR
                {
                    return Some(Rgb {
                        r: payload[3],
                        g: payload[4],
                        b: payload[5],
                    });
                }
            }
        }
        i += len + 1;
    }
    None
}

fn on_adv_report(report: &raw::ble_gap_evt_adv_report_t) -> Option<()> {
    if !RECEIVER_ARMED.load(Ordering::Acquire) {
        return None;
    }

    let target = with_ble(|s| {
        if s.mode == RadioMode::Receiver {
            s.receiver_target
        } else {
            None
        }
    })?;

    let len = report.data.len as usize;
    if len == 0 {
        return None;
    }
    let data = unsafe { core::slice::from_raw_parts(report.data.p_data, len) };
    let received = parse_color_from_adv(data)?;

    if received.r == target.r && received.g == target.g && received.b == target.b {
        with_ble(|s| s.matched = true);
        Some(())
    } else {
        None
    }
}

/// SoftDevice aktivieren (S140 7.x, minimal für Peripheral-Advertising).
pub fn enable() -> &'static Softdevice {
    static GAP_NAME: &[u8] = b"ColorHurry-CPB";

    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 1,
            event_length: 24,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 1,
            central_role_count: 1,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: GAP_NAME.as_ptr() as _,
            current_len: GAP_NAME.len() as u16,
            max_len: GAP_NAME.len() as u16,
            write_perm: unsafe { mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    };

    Softdevice::enable(&config)
}

async fn advertise_once(sd: &Softdevice) {
    let mut config = peripheral::Config::default();
    config.interval = 48;
    config.timeout = Some(5);

    let adv = adv_data();
    let scan = scan_data();
    let packet = peripheral::NonconnectableAdvertisement::ScannableUndirected {
        adv_data: &adv,
        scan_data: &scan,
    };

    match peripheral::advertise(sd, packet, &config).await {
        Err(AdvertiseError::Timeout) => {}
        Err(_) => crate::debug_led::signal_advertising_error(),
        Ok(()) => {}
    }
}

async fn scan_once(sd: &Softdevice) {
    let config = ScanConfig {
        extended: false,
        active: true,
        interval: 160,
        window: 160,
        timeout: 100,
        ..ScanConfig::default()
    };

    match central::scan(sd, &config, on_adv_report).await {
        Err(ScanError::Timeout) => {}
        Err(ScanError::Raw(_)) => crate::debug_led::signal_advertising_error(),
        Ok(()) => {}
    }
}

fn current_mode() -> RadioMode {
    with_ble(|s| s.mode)
}

fn picker_should_advertise() -> bool {
    with_ble(|s| s.mode == RadioMode::Picker && s.picker_color.is_some())
}

/// Picker: senden wenn Farbe bestätigt; Empfänger: scannt; sonst idle.
pub async fn radio_loop(sd: &'static Softdevice) -> ! {
    loop {
        match current_mode() {
            RadioMode::Off => {
                embassy_futures::yield_now().await;
            }
            RadioMode::Picker => {
                if picker_should_advertise() {
                    advertise_once(sd).await;
                } else {
                    embassy_futures::yield_now().await;
                }
            }
            RadioMode::Receiver => scan_once(sd).await,
        }
    }
}
