//! Edge detection for buttons A and B: one event per press.

/// What a poll of the two buttons found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonEvent {
    None,
    /// Button A (left) went down.
    Left,
    /// Button B (right) went down.
    Right,
}

/// Remembers the last button state, so that a held button fires only once.
#[derive(Default)]
pub struct Edges {
    left_down: bool,
    right_down: bool,
}

impl Edges {
    pub const fn new() -> Self {
        Self {
            left_down: false,
            right_down: false,
        }
    }

    /// Feed the raw state, get the new press. If both go down at once, A wins and B is
    /// lost, as on the board.
    pub fn poll(&mut self, left: bool, right: bool) -> ButtonEvent {
        let event = if left && !self.left_down {
            ButtonEvent::Left
        } else if right && !self.right_down {
            ButtonEvent::Right
        } else {
            ButtonEvent::None
        };
        self.left_down = left;
        self.right_down = right;
        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_fires_once() {
        let mut e = Edges::new();
        assert_eq!(e.poll(true, false), ButtonEvent::Left);
        assert_eq!(e.poll(true, false), ButtonEvent::None);
        assert_eq!(e.poll(false, false), ButtonEvent::None);
        assert_eq!(e.poll(true, false), ButtonEvent::Left);
    }

    #[test]
    fn right_button() {
        let mut e = Edges::new();
        assert_eq!(e.poll(false, true), ButtonEvent::Right);
        assert_eq!(e.poll(false, true), ButtonEvent::None);
    }

    #[test]
    fn left_wins_when_both_go_down() {
        let mut e = Edges::new();
        assert_eq!(e.poll(true, true), ButtonEvent::Left);
        assert_eq!(e.poll(true, true), ButtonEvent::None);
    }
}
