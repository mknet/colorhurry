use crate::color::Rgb;

/// Side-effects the shell must perform. Fire-and-forget in the MVP (no resolve).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Re-read [`crate::view`] and refresh the UI.
    Render,
    PlayTone(ToneKind),
    SetBlePicker { channel: u8 },
    SetBleReceiver { channel: u8, color: Rgb },
    SetPickerBroadcast { color: Option<Rgb> },
    ClearBle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToneKind {
    Skip,
    Apply,
    CountdownStep(u8),
    Explosion,
}

const MAX_EFFECTS: usize = 8;

/// Fixed-capacity list of effects (no heap).
#[derive(Clone, Debug)]
pub struct Effects {
    items: [Option<Effect>; MAX_EFFECTS],
    len: usize,
}

impl Effects {
    pub const fn new() -> Self {
        Self {
            items: [None; MAX_EFFECTS],
            len: 0,
        }
    }

    pub fn push(&mut self, effect: Effect) {
        if self.len < MAX_EFFECTS {
            self.items[self.len] = Some(effect);
            self.len += 1;
        }
    }

    pub fn extend_from(&mut self, other: &Effects) {
        for e in other.iter() {
            self.push(e);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Effect> + '_ {
        self.items[..self.len].iter().filter_map(|e| *e)
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl Default for Effects {
    fn default() -> Self {
        Self::new()
    }
}

pub fn only(effect: Effect) -> Effects {
    let mut e = Effects::new();
    e.push(effect);
    e
}

pub fn render_and(effect: Effect) -> Effects {
    let mut e = Effects::new();
    e.push(effect);
    e.push(Effect::Render);
    e
}
