//! Advertising data: build and parse the Color Hurry advertisement.
//!
//! The board gets the data from the BLE scanner, the simulator from UDP. Both parse it
//! with [`parse_color_from_adv`].

use color_hurry_core::{decode_color, encode_color, Color, COMPANY_ID, PAYLOAD_LEN};

/// AD type "Flags".
pub const AD_TYPE_FLAGS: u8 = 0x01;
/// AD type "Shortened Local Name".
pub const AD_TYPE_SHORT_NAME: u8 = 0x08;
/// AD type "Manufacturer Specific Data".
pub const AD_TYPE_MANUFACTURER: u8 = 0xFF;

/// Short name in the advertisement.
pub const SHORT_NAME: &[u8] = b"ColHurry";

/// Max length of legacy advertising data.
pub const ADV_MAX: usize = 31;

/// Find the Color Hurry color in raw advertising data (a list of AD structures:
/// `[len, type, data...]`, where `len` counts the type and the data).
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
            if ad.len() >= 2 + PAYLOAD_LEN && u16::from_le_bytes([ad[0], ad[1]]) == COMPANY_ID {
                return decode_color(&ad[2..]);
            }
        }
        i += len + 1;
    }
    None
}

/// Build the advertising data that a Picker sends: flags, short name and the color.
/// Returns the buffer and the used length, or `None` for an invalid channel.
pub fn build_adv(channel: u8, color: Color) -> Option<([u8; ADV_MAX], usize)> {
    let payload = encode_color(channel, color)?;
    let mut buf = [0u8; ADV_MAX];
    let mut i = 0;
    let mut push = |bytes: &[u8]| {
        buf[i..i + bytes.len()].copy_from_slice(bytes);
        i += bytes.len();
    };
    // LE General Discoverable, BR/EDR not supported.
    push(&[2, AD_TYPE_FLAGS, 0x06]);
    push(&[1 + SHORT_NAME.len() as u8, AD_TYPE_SHORT_NAME]);
    push(SHORT_NAME);
    let company = COMPANY_ID.to_le_bytes();
    push(&[1 + 2 + PAYLOAD_LEN as u8, AD_TYPE_MANUFACTURER, company[0], company[1]]);
    push(&payload);
    Some((buf, i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let (buf, len) = build_adv(3, Color::Purple).unwrap();
        assert_eq!(parse_color_from_adv(&buf[..len]), Some((3, Color::Purple)));
    }

    #[test]
    fn ignores_other_company() {
        let (mut buf, len) = build_adv(0, Color::Red).unwrap();
        let at = len - PAYLOAD_LEN - 2;
        buf[at] = 0x4C; // Apple
        buf[at + 1] = 0x00;
        assert_eq!(parse_color_from_adv(&buf[..len]), None);
    }

    #[test]
    fn stops_at_truncated_structure() {
        let (buf, len) = build_adv(1, Color::Green).unwrap();
        assert_eq!(parse_color_from_adv(&buf[..len - 1]), None);
    }

    #[test]
    fn empty_and_zero_length() {
        assert_eq!(parse_color_from_adv(&[]), None);
        assert_eq!(parse_color_from_adv(&[0, 0xFF, 1]), None);
    }

    #[test]
    fn rejects_invalid_channel() {
        assert!(build_adv(200, Color::Red).is_none());
    }
}
