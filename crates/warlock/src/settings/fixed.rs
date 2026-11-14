//! The fixed keys (#1533): the keys of menus and screens, text fields, the dialog, the info
//! pages, the rest prompt, and aiming. Only play commands are rebindable (CLAUDE.md, "Hard
//! rules"); these keys are the same for everyone, so every hint line names them as written here.
//! The back-style commands of those places (`Back`, `TextBack`, `PrevPage`, `PrevTarget`) stay
//! rebindable, unbound by default, and may not take a key that is fixed where they are read
//! ([`refusal`]).
//!
//! Each place reads a press through [`read`]: the keys the press may stand for
//! ([`mods::press_candidates`], as bindings are matched), most specific first, each tried
//! against the place's rebindable commands (`own`) and then its fixed keys.

use crate::input::{Context, KeyBinding};
use crate::settings::mods::{self, Mods};
use crate::ui::{Edit, Nav};
use bevy::input::keyboard::{KeyCode, KeyboardInput};
use warlock_core::Rest;
use warlock_core::time::Duration;

/// A fixed key of the dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Yes,
    No,
}

/// A fixed key of the info pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageKey {
    /// Page `n`, 1 to 9.
    To(u8),
    /// The next page (with Shift, the one before).
    Next,
}

/// A fixed key of aiming.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AimKey {
    /// Shoot, throw, or cast at the cursor.
    Loose,
    /// The cursor to the next creature in view (with Shift, the one before).
    NextTarget,
}

/// The menu keys as hint lines name them.
pub const UP: &str = "↑";
pub const DOWN: &str = "↓";
pub const LEFT: &str = "←";
pub const RIGHT: &str = "→";
pub const CONFIRM: &str = "Enter";
pub const ERASE: &str = "Del";
pub const MARK: &str = "Space";
pub const MARK_ALL: &str = ",";
pub const NEXT_LIST_PAGE: &str = ">";
pub const PREV_LIST_PAGE: &str = "<";
/// The text field keys as hint lines name them.
pub const TEXT_CONFIRM: &str = "Enter";
pub const TEXT_ERASE: &str = "Backspace";
pub const TEXT_RANDOM: &str = "Tab";
/// The dialog's keys as hint lines name them.
pub const YES: &str = "y";
pub const NO: &str = "n";
/// Aiming's keys as hint lines name them.
pub const LOOSE: &str = "Enter";
pub const NEXT_TARGET: &str = "Tab";

/// The rest prompt's answers: the key, what it rests for, and how the prompt words it.
pub const REST: [(&str, Rest, &str); 3] = [
    ("h", Rest::For(Duration::hours(1)), "hour"),
    ("d", Rest::Dawn, "until dawn"),
    ("n", Rest::Dusk, "until nightfall"),
];

fn is_char(key: &KeyBinding, s: &str) -> bool {
    matches!(key, KeyBinding::Char(c) if c == s)
}

fn is_code(key: &KeyBinding, code: KeyCode) -> bool {
    matches!(key, KeyBinding::Code(c) if *c == code)
}

fn is_enter(key: &KeyBinding) -> bool {
    is_code(key, KeyCode::Enter) || is_code(key, KeyCode::NumpadEnter)
}

/// A menu key's meaning: the arrows and `hjkl` move, Enter chooses, Delete and Backspace erase,
/// Space and `,` mark, `>` and `<` (PageDown, PageUp) turn an item list's pages. The numpad's
/// arrows are the menu's own (`ui::menu`).
pub fn menu(key: &KeyBinding) -> Option<Nav> {
    let nav = match key {
        KeyBinding::Code(KeyCode::ArrowUp) => Nav::Up,
        KeyBinding::Code(KeyCode::ArrowDown) => Nav::Down,
        KeyBinding::Code(KeyCode::ArrowLeft) => Nav::Left,
        KeyBinding::Code(KeyCode::ArrowRight) => Nav::Right,
        KeyBinding::Code(KeyCode::Enter | KeyCode::NumpadEnter) => Nav::Enter,
        KeyBinding::Code(KeyCode::Delete | KeyCode::Backspace) => Nav::Erase,
        KeyBinding::Code(KeyCode::Space) => Nav::Mark,
        KeyBinding::Code(KeyCode::PageDown) => Nav::NextPage,
        KeyBinding::Code(KeyCode::PageUp) => Nav::PrevPage,
        KeyBinding::Char(c) => match c.as_str() {
            "k" => Nav::Up,
            "j" => Nav::Down,
            "h" => Nav::Left,
            "l" => Nav::Right,
            "," => Nav::MarkAll,
            ">" => Nav::NextPage,
            "<" => Nav::PrevPage,
            _ => return None,
        },
        _ => return None,
    };
    Some(nav)
}

/// A text field key's meaning: Enter accepts, Backspace and Delete erase, Tab puts a random
/// value in.
pub fn text(key: &KeyBinding) -> Option<Edit> {
    match key {
        k if is_enter(k) => Some(Edit::Confirm),
        KeyBinding::Code(KeyCode::Backspace | KeyCode::Delete) => Some(Edit::Erase),
        KeyBinding::Code(KeyCode::Tab) => Some(Edit::Random),
        _ => None,
    }
}

/// The dialog's shortcuts: `y` and `n`.
pub fn dialog(key: &KeyBinding) -> Option<Answer> {
    match key {
        k if is_char(k, YES) => Some(Answer::Yes),
        k if is_char(k, NO) => Some(Answer::No),
        _ => None,
    }
}

/// The info pages' keys: the top-row digits `1` to `9`, and Tab for the next page.
pub fn page(key: &KeyBinding) -> Option<PageKey> {
    match key {
        KeyBinding::Code(KeyCode::Tab) => Some(PageKey::Next),
        KeyBinding::Char(c) => {
            let n = c.parse::<u8>().ok().filter(|n| (1..=9).contains(n))?;
            (c.len() == 1).then_some(PageKey::To(n))
        }
        _ => None,
    }
}

/// Aiming's keys: Enter looses, Tab picks the next target.
pub fn aim(key: &KeyBinding) -> Option<AimKey> {
    match key {
        k if is_enter(k) => Some(AimKey::Loose),
        KeyBinding::Code(KeyCode::Tab) => Some(AimKey::NextTarget),
        _ => None,
    }
}

/// The rest prompt's answer a key gives.
pub fn rest(key: &KeyBinding) -> Option<Rest> {
    (REST.iter())
        .find(|(k, ..)| is_char(key, k))
        .map(|(_, rest, _)| *rest)
}

/// Whether `key` is fixed in `context`, so no command of it may take the key.
pub fn taken(context: Context, key: &KeyBinding) -> bool {
    match context {
        Context::Menu => menu(key).is_some(),
        Context::Text => text(key).is_some(),
        Context::Pages => page(key).is_some(),
        Context::Aim => aim(key).is_some(),
        Context::Play | Context::More | Context::Dev => false,
    }
}

/// Why `key` may not be a key of a command of `context`: it is fixed there.
pub fn refusal(key: &KeyBinding, context: Context) -> Option<String> {
    taken(context, key).then(|| {
        let place = context.title().to_lowercase();
        format!("{} is fixed in {place}", key.display())
    })
}

/// What a press means in one place: for each key the press may stand for, most specific first,
/// the place's rebindable command (`own`), else its fixed key (`fixed`).
pub fn read<T>(
    input: &KeyboardInput,
    mods: Mods,
    own: impl Fn(&KeyBinding) -> Option<T>,
    fixed: impl Fn(&KeyBinding) -> Option<T>,
) -> Option<T> {
    (mods::press_candidates(input, mods, crate::settings::keys::CODES).iter())
        .find_map(|key| own(key).or_else(|| fixed(key)))
}
