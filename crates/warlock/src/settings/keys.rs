//! Rebindable commands and their text form in the settings file (spec §6).
//!
//! A key is written as the character it produces (`"k"`, `">"`) or as a physical key name
//! (`"ArrowUp"`, `"Numpad8"`, `"F5"`). Commands have stable ids (`"move_n"`, `"wait"`).

use crate::input::{Bindings, KeyBinding, KeyCommand};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use std::collections::BTreeMap;
use warlock_core::Dir;

/// The commands the player may rebind, in the order the controls screen lists them. `Cancel`
/// (Esc) is fixed so a rebinding can always be cancelled, and the dev snapshot keys are not
/// player commands.
pub const REBINDABLE: [KeyCommand; 17] = [
    KeyCommand::Move(Dir::N),
    KeyCommand::Move(Dir::NE),
    KeyCommand::Move(Dir::E),
    KeyCommand::Move(Dir::SE),
    KeyCommand::Move(Dir::S),
    KeyCommand::Move(Dir::SW),
    KeyCommand::Move(Dir::W),
    KeyCommand::Move(Dir::NW),
    KeyCommand::Wait,
    KeyCommand::Open,
    KeyCommand::Close,
    KeyCommand::Descend,
    KeyCommand::Ascend,
    KeyCommand::Lantern,
    KeyCommand::Look,
    KeyCommand::Help,
    KeyCommand::Quit,
];

/// Physical keys a command may be bound to.
const CODES: &[KeyCode] = &[
    KeyCode::ArrowUp,
    KeyCode::ArrowDown,
    KeyCode::ArrowLeft,
    KeyCode::ArrowRight,
    KeyCode::Numpad0,
    KeyCode::Numpad1,
    KeyCode::Numpad2,
    KeyCode::Numpad3,
    KeyCode::Numpad4,
    KeyCode::Numpad5,
    KeyCode::Numpad6,
    KeyCode::Numpad7,
    KeyCode::Numpad8,
    KeyCode::Numpad9,
    KeyCode::NumpadAdd,
    KeyCode::NumpadSubtract,
    KeyCode::NumpadMultiply,
    KeyCode::NumpadDivide,
    KeyCode::NumpadDecimal,
    KeyCode::NumpadEnter,
    KeyCode::F1,
    KeyCode::F2,
    KeyCode::F3,
    KeyCode::F4,
    KeyCode::F5,
    KeyCode::F6,
    KeyCode::F7,
    KeyCode::F8,
    KeyCode::F9,
    KeyCode::F10,
    KeyCode::F11,
    KeyCode::F12,
    KeyCode::Space,
    KeyCode::Tab,
    KeyCode::Insert,
    KeyCode::Home,
    KeyCode::End,
    KeyCode::PageUp,
    KeyCode::PageDown,
];

impl KeyCommand {
    /// Stable name in the settings file; `None` for commands that cannot be rebound.
    pub fn id(self) -> Option<&'static str> {
        use KeyCommand::*;
        Some(match self {
            Move(Dir::N) => "move_n",
            Move(Dir::NE) => "move_ne",
            Move(Dir::E) => "move_e",
            Move(Dir::SE) => "move_se",
            Move(Dir::S) => "move_s",
            Move(Dir::SW) => "move_sw",
            Move(Dir::W) => "move_w",
            Move(Dir::NW) => "move_nw",
            Wait => "wait",
            Open => "open",
            Close => "close",
            Descend => "descend",
            Ascend => "ascend",
            Lantern => "lantern",
            Look => "look",
            Help => "help",
            Quit => "quit",
            Cancel | Snapshot | LoadSnapshot | CycleVision | RevealMap | LightValues => {
                return None;
            }
        })
    }

    pub fn from_id(id: &str) -> Option<Self> {
        REBINDABLE.into_iter().find(|c| c.id() == Some(id))
    }

    /// Name for the controls screen; unlike [`KeyCommand::label`] it tells directions apart.
    pub fn name(self) -> String {
        match self {
            KeyCommand::Move(dir) => format!("Move {}", direction_name(dir)),
            other => other.label().to_string(),
        }
    }
}

fn direction_name(dir: Dir) -> &'static str {
    match dir {
        Dir::N => "north",
        Dir::NE => "northeast",
        Dir::E => "east",
        Dir::SE => "southeast",
        Dir::S => "south",
        Dir::SW => "southwest",
        Dir::W => "west",
        Dir::NW => "northwest",
    }
}

impl KeyBinding {
    /// The settings-file spelling.
    pub fn to_text(&self) -> String {
        match self {
            KeyBinding::Char(s) => s.clone(),
            KeyBinding::Code(code) => format!("{code:?}"),
        }
    }

    /// Parses the settings-file spelling. Esc is reserved, so it cannot be bound.
    pub fn from_text(text: &str) -> Option<Self> {
        let mut chars = text.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => {
                (!c.is_control() && !c.is_whitespace()).then(|| KeyBinding::Char(c.to_string()))
            }
            (Some(_), Some(_)) => CODES
                .iter()
                .find(|code| format!("{code:?}") == text)
                .map(|code| KeyBinding::Code(*code)),
            (None, _) => None,
        }
    }

    /// The binding a key press stands for, if it can be bound. Printable keys bind by the
    /// character they produce; numpad keys (which also produce digits), arrows, and function
    /// keys bind by physical key.
    pub fn from_input(input: &KeyboardInput) -> Option<Self> {
        let numpad = format!("{:?}", input.key_code).starts_with("Numpad");
        if !numpad && let Key::Character(c) = &input.logical_key {
            return KeyBinding::from_text(c.as_str());
        }
        CODES
            .contains(&input.key_code)
            .then_some(KeyBinding::Code(input.key_code))
    }
}

/// Why a key could not be bound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub key: KeyBinding,
    /// The command that already uses the key.
    pub owner: KeyCommand,
}

impl Bindings {
    /// The first command that uses `key`.
    pub fn owner(&self, key: &KeyBinding) -> Option<KeyCommand> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, c)| *c)
    }

    /// Keys of `command`, primary first.
    pub fn keys_of(&self, command: KeyCommand) -> Vec<&KeyBinding> {
        self.0
            .iter()
            .filter(|(_, c)| *c == command)
            .map(|(k, _)| k)
            .collect()
    }

    /// Every key bound to two different commands: `(key, first, second)`.
    pub fn conflicts(&self) -> Vec<(KeyBinding, KeyCommand, KeyCommand)> {
        let mut found = Vec::new();
        for (i, (key, command)) in self.0.iter().enumerate() {
            if let Some((_, first)) = self.0[..i].iter().find(|(k, c)| k == key && c != command) {
                found.push((key.clone(), *first, *command));
            }
        }
        found
    }

    /// Makes `key` the primary (first) key of `command`. Refuses a key another command uses.
    pub fn bind_primary(&mut self, command: KeyCommand, key: KeyBinding) -> Result<(), Conflict> {
        if let Some(owner) = self.owner(&key)
            && owner != command
        {
            return Err(Conflict { key, owner });
        }
        let first = self.0.iter().position(|(_, c)| *c == command);
        let same = self.0.iter().position(|(k, c)| *c == command && *k == key);
        match (first, same) {
            (Some(first), Some(same)) => {
                let old = std::mem::replace(&mut self.0[first].0, key);
                self.0[same].0 = old;
            }
            (Some(first), None) => self.0[first].0 = key,
            (None, _) => self.0.push((key, command)),
        }
        Ok(())
    }

    /// Replaces all keys of `command` with `keys`, at the place of its first entry.
    fn set_keys(&mut self, command: KeyCommand, keys: Vec<KeyBinding>) {
        let at = self
            .0
            .iter()
            .position(|(_, c)| *c == command)
            .unwrap_or(self.0.len());
        self.0.retain(|(_, c)| *c != command);
        let entries = keys.into_iter().map(|k| (k, command));
        self.0.splice(at..at, entries);
    }

    /// Restores the default keys of `command`, except keys another command now uses.
    pub fn reset_command(&mut self, command: KeyCommand) {
        let defaults: Vec<KeyBinding> = Bindings::default()
            .0
            .into_iter()
            .filter(|(k, c)| *c == command && self.owner(k).is_none_or(|o| o == command))
            .map(|(k, _)| k)
            .collect();
        self.set_keys(command, defaults);
    }

    /// Restores the default keys of every rebindable command.
    pub fn reset_all(&mut self) {
        // Twice, so keys that swapped between commands end up home.
        for _ in 0..2 {
            for command in REBINDABLE {
                self.reset_command(command);
            }
        }
    }

    /// The settings-file form: command id to key spellings.
    pub fn to_map(&self) -> BTreeMap<String, Vec<String>> {
        REBINDABLE
            .into_iter()
            .filter_map(|c| {
                let id = c.id()?;
                let keys = self.keys_of(c).iter().map(|k| k.to_text()).collect();
                Some((id.to_string(), keys))
            })
            .collect()
    }

    /// Starts from the defaults and applies the `[bindings]` table of a settings file. Anything
    /// invalid is skipped with a warning, and a command never ends up without a key.
    pub fn from_table(table: &toml::Table, warnings: &mut Vec<String>) -> Self {
        let mut bindings = Bindings::default();
        for id in table.keys() {
            if KeyCommand::from_id(id).is_none() {
                warnings.push(format!("bindings: unknown command `{id}` ignored"));
            }
        }
        for command in REBINDABLE {
            let Some(id) = command.id() else { continue };
            let Some(value) = table.get(id) else { continue };
            let Some(items) = value.as_array() else {
                warnings.push(format!("bindings.{id}: expected a list of keys"));
                continue;
            };
            let mut keys: Vec<KeyBinding> = Vec::new();
            for item in items {
                match item.as_str().and_then(KeyBinding::from_text) {
                    Some(key) if keys.contains(&key) => {}
                    Some(key) => keys.push(key),
                    None => warnings.push(format!("bindings.{id}: invalid key {item}")),
                }
            }
            if keys.is_empty() {
                warnings.push(format!("bindings.{id}: no usable keys; using the defaults"));
            } else {
                bindings.set_keys(command, keys);
            }
        }
        // A key may belong to one command only: later users lose it.
        for (key, first, second) in bindings.conflicts() {
            warnings.push(format!(
                "bindings: `{}` is used by both `{}` and `{}`; kept for the first",
                key.to_text(),
                first.name(),
                second.name()
            ));
        }
        let mut seen: Vec<KeyBinding> = Vec::new();
        bindings.0.retain(|(k, _)| {
            let fresh = !seen.contains(k);
            seen.push(k.clone());
            fresh
        });
        for command in REBINDABLE {
            if bindings.keys_of(command).is_empty() {
                bindings.reset_command(command);
            }
        }
        bindings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(s: &str) -> KeyBinding {
        KeyBinding::Char(s.into())
    }

    #[test]
    fn every_rebindable_command_has_a_unique_id_that_round_trips() {
        for c in REBINDABLE {
            let id = c.id().expect("rebindable commands have ids");
            assert_eq!(KeyCommand::from_id(id), Some(c));
        }
        assert_eq!(KeyCommand::Cancel.id(), None);
        let defaults = Bindings::default();
        for c in REBINDABLE {
            assert!(!defaults.keys_of(c).is_empty(), "{c:?} has a default key");
        }
    }

    #[test]
    fn key_text_round_trips() {
        for key in [
            ch("k"),
            ch(">"),
            ch("Q"),
            KeyBinding::Code(KeyCode::ArrowUp),
            KeyBinding::Code(KeyCode::Numpad8),
            KeyBinding::Code(KeyCode::F5),
        ] {
            assert_eq!(KeyBinding::from_text(&key.to_text()), Some(key));
        }
        // Esc is reserved; unknown names and blanks are not keys.
        assert_eq!(KeyBinding::from_text("Escape"), None);
        assert_eq!(KeyBinding::from_text("Banana"), None);
        assert_eq!(KeyBinding::from_text(""), None);
        assert_eq!(KeyBinding::from_text(" "), None);
    }

    #[test]
    fn default_bindings_have_no_conflicts() {
        assert!(Bindings::default().conflicts().is_empty());
    }

    #[test]
    fn binding_a_used_key_is_a_conflict_and_changes_nothing() {
        let mut b = Bindings::default();
        let before = b.0.clone();
        let err = b.bind_primary(KeyCommand::Wait, ch("k")).unwrap_err();
        assert_eq!(err.owner, KeyCommand::Move(Dir::N));
        assert_eq!(b.0, before);
        // Esc belongs to Cancel as well.
        let esc = KeyBinding::Code(KeyCode::Escape);
        assert!(b.bind_primary(KeyCommand::Wait, esc).is_err());
    }

    #[test]
    fn binding_replaces_the_primary_key_and_keeps_alternates() {
        let mut b = Bindings::default();
        b.bind_primary(KeyCommand::Move(Dir::N), ch("w")).unwrap();
        let keys: Vec<String> = b
            .keys_of(KeyCommand::Move(Dir::N))
            .iter()
            .map(|k| k.to_text())
            .collect();
        assert_eq!(keys, ["w", "Numpad8", "ArrowUp"]);
        assert!(b.conflicts().is_empty());
        // Choosing an alternate promotes it to primary.
        b.bind_primary(KeyCommand::Move(Dir::N), KeyBinding::Code(KeyCode::ArrowUp))
            .unwrap();
        assert_eq!(
            b.keys_of(KeyCommand::Move(Dir::N))[0],
            &KeyBinding::Code(KeyCode::ArrowUp)
        );
        assert_eq!(b.keys_of(KeyCommand::Move(Dir::N)).len(), 3);
    }

    #[test]
    fn reset_restores_defaults_in_place() {
        let mut b = Bindings::default();
        let before = b.to_map();
        b.bind_primary(KeyCommand::Wait, ch("w")).unwrap();
        b.bind_primary(KeyCommand::Open, ch("O")).unwrap();
        b.reset_command(KeyCommand::Wait);
        assert_eq!(b.keys_of(KeyCommand::Wait)[0], &ch("."));
        b.reset_all();
        assert_eq!(b.to_map(), before);
    }

    #[test]
    fn table_applies_overrides_and_survives_garbage() {
        let table: toml::Table = toml::from_str(
            r#"
            wait = ["w", "Numpad5"]
            open = ["k"]
            close = ["Banana", 7]
            ghost = ["x"]
            help = "?"
            "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let b = Bindings::from_table(&table, &mut warnings);
        assert_eq!(b.keys_of(KeyCommand::Wait)[0], &ch("w"));
        // `k` stays with Move north; Open falls back to its default.
        assert_eq!(b.owner(&ch("k")), Some(KeyCommand::Move(Dir::N)));
        assert_eq!(b.keys_of(KeyCommand::Open), vec![&ch("o")]);
        assert_eq!(b.keys_of(KeyCommand::Close), vec![&ch("c")]);
        assert_eq!(b.keys_of(KeyCommand::Help), vec![&ch("?")]);
        assert!(b.conflicts().is_empty());
        for needle in ["ghost", "used by both", "bindings.close", "bindings.help"] {
            assert!(warnings.iter().any(|w| w.contains(needle)), "{needle}");
        }
    }

    #[test]
    fn map_round_trips_through_the_table() {
        let mut b = Bindings::default();
        b.bind_primary(KeyCommand::Descend, ch("d")).unwrap();
        let text = toml::to_string(&b.to_map()).unwrap();
        let table: toml::Table = toml::from_str(&text).unwrap();
        let mut warnings = Vec::new();
        let back = Bindings::from_table(&table, &mut warnings);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(back.to_map(), b.to_map());
    }
}
