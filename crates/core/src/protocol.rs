//! BLE manufacturer payload (after company ID). Pure encode/decode for both shells.

use crate::color::Color;
use crate::palette::CHANNELS;

pub const COMPANY_ID: u16 = 0x239A;

pub const MAGIC: u8 = b'S';
/// Named color index on the wire (not RGB).
pub const VERSION: u8 = 4;
pub const CMD_COLOR: u8 = 0x03;

/// `[magic, version, opcode, channel, color_index]`
pub const PAYLOAD_LEN: usize = 5;

pub fn encode_color(channel: u8, color: Color) -> Option<[u8; PAYLOAD_LEN]> {
    if channel as usize >= CHANNELS {
        return None;
    }
    Some([MAGIC, VERSION, CMD_COLOR, channel, color.index()])
}

pub fn decode_color(payload: &[u8]) -> Option<(u8, Color)> {
    if payload.len() < PAYLOAD_LEN {
        return None;
    }
    if payload[0] != MAGIC || payload[1] != VERSION || payload[2] != CMD_COLOR {
        return None;
    }
    let channel = payload[3];
    if channel as usize >= CHANNELS {
        return None;
    }
    let color = Color::from_index(payload[4])?;
    Some((channel, color))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let bytes = encode_color(2, Color::Orange).unwrap();
        assert_eq!(decode_color(&bytes), Some((2, Color::Orange)));
    }

    #[test]
    fn rejects_bad_magic() {
        assert!(decode_color(&[b'X', VERSION, CMD_COLOR, 0, 1]).is_none());
    }

    #[test]
    fn rejects_v3_rgb_shaped() {
        // Old v3 was 7 bytes with RGB; must not decode as v4.
        assert!(decode_color(&[MAGIC, 3, CMD_COLOR, 0, 255, 0, 0]).is_none());
    }

    #[test]
    fn rejects_bad_channel() {
        assert!(encode_color(5, Color::Red).is_none());
        assert!(decode_color(&[MAGIC, VERSION, CMD_COLOR, 5, 1]).is_none());
    }

    #[test]
    fn rejects_bad_color_index() {
        assert!(decode_color(&[MAGIC, VERSION, CMD_COLOR, 0, 10]).is_none());
    }

    #[test]
    fn wire_constants() {
        assert_eq!(COMPANY_ID, 0x239A);
        assert_eq!(VERSION, 4);
        assert_eq!(PAYLOAD_LEN, 5);
    }
}
