//! Experimental Web Bluetooth advertising receive (Chrome flag required).

use color_hurry_core::{Rgb, CHANNELS};
use js_sys::{Function, Reflect, Uint8Array};
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

fn manufacturer_filter_array() -> Result<js_sys::Array, JsValue> {
    let mfg = js_sys::Object::new();
    Reflect::set(
        &mfg,
        &JsValue::from_str("companyIdentifier"),
        &JsValue::from(COMPANY_ID),
    )?;
    let prefix = Uint8Array::new_with_length(3);
    prefix.set_index(0, protocol::MAGIC);
    prefix.set_index(1, protocol::VERSION);
    prefix.set_index(2, protocol::CMD_COLOR);
    Reflect::set(&mfg, &JsValue::from_str("dataPrefix"), &prefix)?;

    let arr = js_sys::Array::new();
    arr.push(&mfg);
    Ok(arr)
}

fn scan_filter() -> Result<BluetoothLeScanFilterInit, JsValue> {
    let filter = BluetoothLeScanFilterInit::new();
    filter.set_manufacturer_data(manufacturer_filter_array()?.as_ref());
    Ok(filter)
}

pub fn bluetooth() -> Result<Bluetooth, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    window
        .navigator()
        .bluetooth()
        .ok_or_else(|| {
            JsValue::from_str(
                "Web Bluetooth unavailable (Chrome/Edge + HTTPS/localhost; enable experimental flag)",
            )
        })
}

/// Prefer `requestLEScan` (room scan). Falls back to `requestDevice` + `watchAdvertisements`.
pub async fn start_listening() -> Result<LiveScan, JsValue> {
    let bt = bluetooth()?;
    if Reflect::has(bt.as_ref(), &JsValue::from_str("requestLEScan"))? {
        match start_le_scan(&bt).await {
            Ok(scan) => return Ok(LiveScan::LeScan { bt, scan }),
            Err(e) => {
                web_sys::console::warn_2(
                    &JsValue::from_str("requestLEScan failed, trying watchAdvertisements:"),
                    &e,
                );
            }
        }
    }
    let device = start_watch(&bt).await?;
    Ok(LiveScan::Watch { bt, device })
}

async fn start_le_scan(bt: &Bluetooth) -> Result<JsValue, JsValue> {
    let options = js_sys::Object::new();
    let filters = js_sys::Array::new();
    filters.push(scan_filter()?.as_ref());
    Reflect::set(&options, &JsValue::from_str("filters"), &filters)?;
    Reflect::set(
        &options,
        &JsValue::from_str("keepRepeatedDevices"),
        &JsValue::TRUE,
    )?;

    let f = Reflect::get(bt.as_ref(), &JsValue::from_str("requestLEScan"))?
        .dyn_into::<Function>()?;
    let promise = f
        .call1(bt.as_ref(), &options)?
        .dyn_into::<js_sys::Promise>()?;
    wasm_bindgen_futures::JsFuture::from(promise).await
}

async fn start_watch(bt: &Bluetooth) -> Result<BluetoothDevice, JsValue> {
    let opts = RequestDeviceOptions::new();
    let filter = scan_filter()?;
    opts.set_filters(std::slice::from_ref(&filter));
    Reflect::set(
        opts.as_ref(),
        &JsValue::from_str("optionalManufacturerData"),
        &js_sys::Array::of1(&JsValue::from(COMPANY_ID)),
    )?;

    let device = wasm_bindgen_futures::JsFuture::from(bt.request_device(&opts))
        .await?
        .dyn_into::<BluetoothDevice>()?;

    wasm_bindgen_futures::JsFuture::from(device.watch_advertisements()).await?;
    Ok(device)
}

pub enum LiveScan {
    LeScan { bt: Bluetooth, scan: JsValue },
    Watch { bt: Bluetooth, device: BluetoothDevice },
}

impl LiveScan {
    pub fn stop(&self) {
        if let LiveScan::LeScan { scan, .. } = self {
            if let Ok(f) = Reflect::get(scan, &JsValue::from_str("stop")).and_then(|v| v.dyn_into::<Function>()) {
                let _ = f.call0(scan);
            }
        }
    }

    pub fn mode_label(&self) -> &'static str {
        match self {
            LiveScan::LeScan { .. } => "requestLEScan",
            LiveScan::Watch { .. } => "watchAdvertisements",
        }
    }

    pub fn event_targets(&self) -> Vec<EventTarget> {
        match self {
            LiveScan::LeScan { bt, .. } => vec![bt.clone().unchecked_into()],
            LiveScan::Watch { bt, device } => {
                vec![bt.clone().unchecked_into(), device.clone().unchecked_into()]
            }
        }
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
