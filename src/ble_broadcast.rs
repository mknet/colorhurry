//! BLE-Broadcast: Kommando in Manufacturer-Data (Scan Response + Name im Adv).

use core::mem;

use nrf_softdevice::ble::advertisement_builder::{
    AdvertisementDataType, Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload,
};
use nrf_softdevice::ble::peripheral::{self, AdvertiseError};
use nrf_softdevice::{raw, Softdevice};

/// Sichtbarer BLE-Gerätename (Scan Response + GAP).
pub const DEVICE_NAME: &str = "ColorHurry-CPB";

/// Adafruit Company ID (Bluetooth SIG).
pub const COMPANY_ID: u16 = 0x239A;

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
    /// Geplant v3: Kanal-ID des Pickers (unabhängig von Empfänger-Zufallsfarbe).
    #[allow(dead_code)]
    pub const CMD_COLOR_CHANNEL: u8 = 0x04;
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

/// Kommando-Opcode setzen (Farbe bleibt).
#[allow(dead_code)]
pub fn set_command(cmd: u8) {
    unsafe {
        CMD_PAYLOAD[2] = cmd;
    }
}

/// Aktuelle Farbe für den nächsten Advertising-Zyklus (volle Palette-Werte).
pub fn set_color(color: crate::neopixel_pwm::Rgb) {
    unsafe {
        CMD_PAYLOAD[2] = protocol::CMD_COLOR;
        CMD_PAYLOAD[3] = color.r;
        CMD_PAYLOAD[4] = color.g;
        CMD_PAYLOAD[5] = color.b;
    }
}

/// Aktuelle Payload-Farbe lesen.
#[allow(dead_code)]
pub fn color() -> crate::neopixel_pwm::Rgb {
    unsafe {
        crate::neopixel_pwm::Rgb {
            r: CMD_PAYLOAD[3],
            g: CMD_PAYLOAD[4],
            b: CMD_PAYLOAD[5],
        }
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
        // Farbe im Adv-Paket (nicht nur Scan Response) — schneller sichtbar im Scanner
        .raw(AdvertisementDataType::MANUFACTURER_SPECIFIC_DATA, &mfg)
        .build()
}

fn scan_data() -> LegacyAdvertisementPayload {
    LegacyAdvertisementBuilder::new()
        .full_name(DEVICE_NAME)
        .build()
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

/// Empfänger-Modus: Farbe vom Picker-Broadcast (Central-Scan) — noch nicht implementiert.
///
/// Der Empfänger nutzt aktuell eine **lokale Zufallsfarbe**; Setup-Kanalfarben
/// dienen nur der Modus-/Identitätswahl. Siehe `docs/RECEIVER.md`.
#[allow(dead_code)]
pub async fn receiver_wait_color() -> crate::neopixel_pwm::Rgb {
    crate::neopixel_pwm::Rgb::OFF
}

/// Endlos: scannbares Advertising (~30 ms Intervall, Payload alle ~50 ms neu).
pub async fn advertise_loop(sd: &'static Softdevice) -> ! {
    let mut config = peripheral::Config::default();
    config.interval = 48; // ~30 ms
    // Ohne Timeout läuft ein Adv-Zyklus unbegrenzt — Manufacturer Data bliebe
    // beim Startwert (Rot), auch wenn set_color() später aufgerufen wird.
    config.timeout = Some(5); // 5 × 10 ms = 50 ms, danach neu bauen + starten

    loop {
        let adv = adv_data();
        let scan = scan_data();
        let packet = peripheral::NonconnectableAdvertisement::ScannableUndirected {
            adv_data: &adv,
            scan_data: &scan,
        };

        match peripheral::advertise(sd, packet, &config).await {
            Err(AdvertiseError::Timeout) => {
                // Erwarteter Pfad: Zyklus endet, Payload wird neu gebaut — kein D13-Blink.
            }
            Err(_) => {
                crate::debug_led::signal_advertising_error();
            }
            Ok(()) => {}
        }
    }
}
