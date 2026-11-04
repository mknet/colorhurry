//! Map spec scenario wording onto core types (cucumber steps only).

use color_hurry_core::{Color, Screen};

pub fn parse_color(name: &str) -> Result<Color, String> {
    match name.trim().to_ascii_lowercase().as_str() {
        "white" => Ok(Color::White),
        "red" => Ok(Color::Red),
        "orange" => Ok(Color::Orange),
        "yellow" => Ok(Color::Yellow),
        "lightgreen" | "light_green" | "light green" => Ok(Color::LightGreen),
        "green" => Ok(Color::Green),
        "lightblue" | "light_blue" | "light blue" => Ok(Color::LightBlue),
        "blue" => Ok(Color::Blue),
        "purple" => Ok(Color::Purple),
        "pink" => Ok(Color::Pink),
        other => Err(format!("unknown color '{other}'")),
    }
}

pub fn parse_screen(name: &str) -> Result<Screen, String> {
    match name.trim().to_ascii_lowercase().replace(' ', "").as_str() {
        "modeselect" | "mode" => Ok(Screen::ModeSelect),
        "picker" => Ok(Screen::Picker),
        "caller" => Ok(Screen::Caller),
        other => Err(format!("unknown screen '{other}'")),
    }
}

pub fn parse_number(word: &str) -> Result<usize, String> {
    const WORDS: [&str; 11] = [
        "zero", "once", "twice", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    ];
    word.parse().or_else(|_| match word {
        "one" => Ok(1),
        "two" => Ok(2),
        _ => WORDS.iter().position(|w| *w == word).ok_or(format!("unknown number '{word}'")),
    })
}
