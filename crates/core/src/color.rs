//! Display RGB and named game colors (identity on the wire).

use facet::Facet;
use serde::{Deserialize, Serialize};

/// Platform-free RGB (full palette 0-255; shells may dim for hardware).
#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const OFF: Self = Self { r: 0, g: 0, b: 0 };

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Named spectrum color. Index 0..[`crate::SPECTRUM_LEN`] is the BLE identity (protocol v4).
#[derive(Facet, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Red = 1,
    Orange = 2,
    Yellow = 3,
    LightGreen = 4,
    Green = 5,
    LightBlue = 6,
    Blue = 7,
    Purple = 8,
    Pink = 9,
}

impl Color {
    pub const ALL: [Color; 10] = [
        Color::White,
        Color::Red,
        Color::Orange,
        Color::Yellow,
        Color::LightGreen,
        Color::Green,
        Color::LightBlue,
        Color::Blue,
        Color::Purple,
        Color::Pink,
    ];

    pub fn from_index(index: u8) -> Option<Self> {
        Self::ALL.get(index as usize).copied()
    }

    pub const fn index(self) -> u8 {
        self as u8
    }
}
