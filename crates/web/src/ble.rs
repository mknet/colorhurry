//! Local browser↔board link via Web Bluetooth (no server).
//!
//! CPB advertises non-connectable packets, so there is no GATT connect.
//! Chrome “connection” = pick the device once, then `watchAdvertisements`.

use color_hurry_core::{Rgb, CHANNELS};
use js_sys::{Function, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Bluetooth, BluetoothAdvertisingEvent, BluetoothDevice, BluetoothLeScanFilterInit, EventTarget,
    RequestDeviceOptions,
};

/// Adafruit Company ID (same as CPB firmware).
pub const COMPANY_ID: u16 = 0x239A;

pub mod protocol {
    pub const MAGIC: u8 = b'S';
    pub const VERSION: u8 = 3;
    pub const CMD_COLOR: u8 = 0x03;
}

/// Parse Color Hurry manufacturer payload (bytes after company ID).
pub fn parse_color_payload(payload: &[u8]) -> Option<(u8, Rgb)> {
    if payload.len() < 7 {
        return None;
    }
    if payload[0] != protocol::MAGIC
        || payload[1] != protocol::VERSION
        || payload[2] != protocol::CMD_COLOR
    {
        return None;
    }
    let channel = payload[3];
    if channel as usize >= CHANNELS {
        return None;
    }
    Some((
        channel,
        Rgb {
            r: payload[4],
            g: payload[5],
            b: payload[6],
        },
    ))
}

pub fn color_from_advertisement(ev: &BluetoothAdvertisingEvent) -> Option<(u8, Rgb)> {
    let map = ev.manufacturer_data();
    let view = map.get(COMPANY_ID)?;
    let mut bytes = vec![0u8; view.byte_length() as usize];
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = view.get_uint8(i);
    }
    parse_color_payload(&bytes)
}

/// Debug helper: any manufacturer blob for company id (hex).
pub fn manufacturer_hex(ev: &BluetoothAdvertisingEvent) -> Option<String> {
    let view = ev.manufacturer_data().get(COMPANY_ID)?;
    let mut out = String::new();
    for i in 0..view.byte_length() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&format!("{:02x}", view.get_uint8(i)));
    }
    Some(out)
}

fn discovery_filters() -> Result<js_sys::Array, JsValue> {
    let filters = js_sys::Array::new();

    let mfg = js_sys::Object::new();
    Reflect::set(
        &mfg,
        &JsValue::from_str("companyIdentifier"),
        &JsValue::from(COMPANY_ID),
    )?;
    let mfg_list = js_sys::Array::new();
    mfg_list.push(&mfg);
    let by_company = BluetoothLeScanFilterInit::new();
    by_company.set_manufacturer_data(mfg_list.as_ref());
    filters.push(by_company.as_ref());

    for name in ["ColorHurry", "ColHurry"] {
        let by_name = BluetoothLeScanFilterInit::new();
        by_name.set_name_prefix(name);
        filters.push(by_name.as_ref());
    }

    Ok(filters)
}

pub fn bluetooth() -> Result<Bluetooth, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    window.navigator().bluetooth().ok_or_else(|| {
        JsValue::from_str(
            "Web Bluetooth unavailable (Chrome/Edge + HTTPS/localhost)",
        )
    })
}

/// Pick the CPB in Chrome’s chooser (local permission — no server).
pub async fn request_board() -> Result<(Bluetooth, BluetoothDevice), JsValue> {
    let bt = bluetooth()?;
    let opts = RequestDeviceOptions::new();
    let filters = discovery_filters()?;
    Reflect::set(opts.as_ref(), &JsValue::from_str("filters"), filters.as_ref())?;
    // Required so advertisement events include our manufacturer payload.
    Reflect::set(
        opts.as_ref(),
        &JsValue::from_str("optionalManufacturerData"),
        &js_sys::Array::of1(&JsValue::from(COMPANY_ID)),
    )?;

    let device = wasm_bindgen_futures::JsFuture::from(bt.request_device(&opts))
        .await?
        .dyn_into::<BluetoothDevice>()?;
    Ok((bt, device))
}

/// Start receiving advertisements from an already-chosen device.
/// Call only after listeners are attached (avoids missing the first packets).
pub async fn watch_board(device: &BluetoothDevice) -> Result<(), JsValue> {
    // Prefer options bag so Chrome keeps delivering manufacturer data.
    let options = js_sys::Object::new();
    Reflect::set(
        &options,
        &JsValue::from_str("optionalManufacturerData"),
        &js_sys::Array::of1(&JsValue::from(COMPANY_ID)),
    )?;

    if Reflect::has(device.as_ref(), &JsValue::from_str("watchAdvertisements"))? {
        let f = Reflect::get(device.as_ref(), &JsValue::from_str("watchAdvertisements"))?
            .dyn_into::<Function>()?;
        let result = f.call1(device.as_ref(), &options).or_else(|_| f.call0(device.as_ref()))?;
        let _ = wasm_bindgen_futures::JsFuture::from(result.dyn_into::<js_sys::Promise>()?).await?;
        return Ok(());
    }
    Err(JsValue::from_str(
        "watchAdvertisements missing — enable chrome://flags/#enable-experimental-web-platform-features",
    ))
}

pub struct BoardLink {
    pub bt: Bluetooth,
    pub device: BluetoothDevice,
}

impl BoardLink {
    pub fn device_label(&self) -> String {
        self.device
            .name()
            .unwrap_or_else(|| "CPB".to_string())
    }

    pub fn event_targets(&self) -> Vec<EventTarget> {
        vec![
            self.bt.clone().unchecked_into(),
            self.device.clone().unchecked_into(),
        ]
    }
}

pub fn attach_advertisement_listener<F>(
    target: &EventTarget,
    mut on_event: F,
) -> Result<Closure<dyn FnMut(web_sys::Event)>, JsValue>
where
    F: FnMut(BluetoothAdvertisingEvent) + 'static,
{
    let closure = Closure::wrap(Box::new(move |ev: web_sys::Event| {
        if let Ok(adv) = ev.dyn_into::<BluetoothAdvertisingEvent>() {
            on_event(adv);
        }
    }) as Box<dyn FnMut(web_sys::Event)>);
    target
        .add_event_listener_with_callback("advertisementreceived", closure.as_ref().unchecked_ref())?;
    Ok(closure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cmd_color_payload() {
        let (ch, c) = parse_color_payload(&[b'S', 3, 0x03, 2, 10, 20, 30]).unwrap();
        assert_eq!(ch, 2);
        assert_eq!(c, Rgb { r: 10, g: 20, b: 30 });
    }

    #[test]
    fn rejects_wrong_magic() {
        assert!(parse_color_payload(&[b'X', 3, 0x03, 0, 1, 2, 3]).is_none());
    }
}
