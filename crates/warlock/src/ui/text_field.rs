//! The game's one text field (the Name step's, and the dialogue screen's Say field): the fixed
//! text keys and `TextBack`'s edit it (accept, erase, back; Random is the caller's), and every
//! other key types its text, literally, as long as the font draws it and the field has room.
//! Held keys type and erase again.

use super::menu::{Edit, Press};
use super::{TEXT, TEXT_DIM};
use crate::render::atlas;
use crate::render::console::Console;

/// A line of typed text of at most `max` characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextField {
    text: String,
    max: usize,
}

impl TextField {
    /// A field holding `text`, taking up to `max` characters.
    pub fn new(text: impl Into<String>, max: usize) -> Self {
        Self {
            text: text.into(),
            max,
        }
    }

    /// What is typed (possibly nothing).
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Puts `text` in the field (a random name).
    pub fn set(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Whether the field takes `ch`: a printable character the font draws, while there is room,
    /// and no space to begin with.
    pub fn takes(&self, ch: char) -> bool {
        !ch.is_control()
            && atlas::draws(ch)
            && self.text.chars().count() < self.max
            && !(ch == ' ' && self.text.is_empty())
    }

    /// Applies a key press: erases, or types what it typed if the field takes it. Returns the
    /// edit the caller acts on (accept, back, random), if the press was one.
    pub fn handle(&mut self, press: &Press) -> Option<Edit> {
        match press.edit {
            Some(Edit::Erase) => {
                self.text.pop();
                None
            }
            Some(edit) => Some(edit),
            None => {
                if let Some(ch) = press.text.filter(|&ch| self.takes(ch)) {
                    self.text.push(ch);
                }
                None
            }
        }
    }

    /// Draws the field at (`x`, `y`): `[`, the text (or, while empty, `placeholder` dimmed), the
    /// cursor, and `]` where the room ends.
    pub fn draw(&self, console: &mut Console, x: i32, y: i32, placeholder: &str) {
        console.print(x, y, "[", TEXT);
        let (shown, color) = if self.text.is_empty() {
            (placeholder, TEXT_DIM)
        } else {
            (self.text.as_str(), TEXT)
        };
        let printed = console.print(x + 1, y, shown, color);
        console.print(x + 1 + printed, y, "_", TEXT);
        console.print(x + 1 + self.max as i32 + 1, y, "]", TEXT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::menu::Nav;

    #[test]
    fn it_types_erases_and_hands_back_the_rest() {
        let mut field = TextField::new("", 5);
        for ch in " ab c,d!".chars() {
            assert_eq!(field.handle(&Press::typed(ch)), None);
        }
        assert_eq!(field.text(), "ab c,", "no leading space, five at most");
        field.handle(&Press::edit(Edit::Erase));
        assert_eq!(field.text(), "ab c");
        for edit in [Edit::Confirm, Edit::Back, Edit::Random] {
            assert_eq!(field.handle(&Press::edit(edit)), Some(edit));
        }
        assert_eq!(field.handle(&Press::nav(Nav::Down)), None);
        assert_eq!(field.text(), "ab c");
        assert!(!field.takes('\u{7}'));
    }
}
