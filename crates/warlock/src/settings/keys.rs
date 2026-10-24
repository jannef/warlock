//! Rebindable commands and their text form in the settings file (spec §6).
//!
//! A key is written as the character it produces (`"k"`, `">"`) or as a physical key name
//! (`"ArrowUp"`, `"Numpad8"`, `"Enter"`, `"F5"`). Commands have stable ids (`"move_n"`,
//! `"wait"`, `"menu_up"`), unique across contexts.
//!
//! No lockout (#159): Esc cannot be bound (it always backs out, cancels, and opens the pause
//! menu), and the [`VITAL`] commands, which reach the controls screen and its Reset all keys,
//! never lose their last key.

use crate::input::{Bindings, Context, KeyBinding, KeyCommand};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use std::collections::BTreeMap;
use warlock_core::Dir;

/// The commands the player may rebind, in the order the controls screen lists them within each
/// context (it groups them by context). `Cancel` (Esc) is fixed so a rebinding can always be
/// cancelled and every screen backed out of.
/// A slice, not a fixed-size array, so features adding commands don't collide on the length.
pub const REBINDABLE: &[KeyCommand] = &[
    KeyCommand::Move(Dir::N),
    KeyCommand::Move(Dir::NE),
    KeyCommand::Move(Dir::E),
    KeyCommand::Move(Dir::SE),
    KeyCommand::Move(Dir::S),
    KeyCommand::Move(Dir::SW),
    KeyCommand::Move(Dir::W),
    KeyCommand::Move(Dir::NW),
    KeyCommand::Wait,
    KeyCommand::Search,
    KeyCommand::SearchLong,
    KeyCommand::Open,
    KeyCommand::Close,
    KeyCommand::Kick,
    KeyCommand::Descend,
    KeyCommand::Ascend,
    KeyCommand::Lantern,
    KeyCommand::PickUp,
    KeyCommand::Drop,
    KeyCommand::Inventory,
    KeyCommand::Equipment,
    KeyCommand::Character,
    KeyCommand::Skills,
    KeyCommand::WeaponSkills,
    KeyCommand::Quests,
    KeyCommand::Talents,
    KeyCommand::Messages,
    KeyCommand::Rest,
    KeyCommand::RestHour,
    KeyCommand::RestDawn,
    KeyCommand::RestDusk,
    KeyCommand::MoreNext,
    KeyCommand::MoreSkip,
    KeyCommand::Eat,
    KeyCommand::Read,
    KeyCommand::Look,
    KeyCommand::Missile,
    KeyCommand::Fire,
    KeyCommand::Throw,
    KeyCommand::MoreAggressive,
    KeyCommand::MoreDefensive,
    KeyCommand::Abilities,
    KeyCommand::RepeatAbility,
    KeyCommand::Bash,
    KeyCommand::ThrowVoice,
    KeyCommand::Disarm,
    KeyCommand::PickLock,
    KeyCommand::Steal,
    KeyCommand::HoodLantern,
    KeyCommand::Pay,
    KeyCommand::NextTarget,
    KeyCommand::PrevTarget,
    KeyCommand::Loose,
    KeyCommand::UseTool,
    KeyCommand::Chat,
    KeyCommand::Help,
    KeyCommand::Quit,
    KeyCommand::MenuUp,
    KeyCommand::MenuDown,
    KeyCommand::MenuLeft,
    KeyCommand::MenuRight,
    KeyCommand::Confirm,
    KeyCommand::Back,
    KeyCommand::Erase,
    KeyCommand::Mark,
    KeyCommand::MarkAll,
    KeyCommand::NextListPage,
    KeyCommand::PrevListPage,
    KeyCommand::TextConfirm,
    KeyCommand::TextErase,
    KeyCommand::TextRandom,
    KeyCommand::TextBack,
    KeyCommand::Yes,
    KeyCommand::No,
    KeyCommand::Page(1),
    KeyCommand::Page(2),
    KeyCommand::Page(3),
    KeyCommand::Page(4),
    KeyCommand::Page(5),
    KeyCommand::Page(6),
    KeyCommand::Page(7),
    KeyCommand::Page(8),
    KeyCommand::Page(9),
    KeyCommand::NextPage,
    KeyCommand::PrevPage,
    KeyCommand::RevealMap,
    KeyCommand::CycleVision,
    KeyCommand::LightValues,
    KeyCommand::Snapshot,
    KeyCommand::SkipHour,
    KeyCommand::NextWeather,
    KeyCommand::IdentifyAll,
    KeyCommand::LoadSnapshot,
];

/// The commands that reach the controls screen (and its Reset all keys) from anywhere, and the
/// one that gets a new character past its name: they never lose their last key, so no binding,
/// settings file, or keymap can lock the player out. Esc, which always backs out, is the rest of
/// the way there.
pub const VITAL: &[KeyCommand] = &[
    KeyCommand::MenuUp,
    KeyCommand::MenuDown,
    KeyCommand::Confirm,
    KeyCommand::TextConfirm,
];

/// The settings ids of [`KeyCommand::Page`] 1 to 9.
const PAGE_IDS: [&str; 9] = [
    "page_1", "page_2", "page_3", "page_4", "page_5", "page_6", "page_7", "page_8", "page_9",
];

/// Rebindable commands with no key by default: abilities (`warlock_core::ability`), which the
/// abilities menu (#156) lists instead of a key each; `f` (fire), which `t` covers; Quit, done
/// from the pause menu; the previous target and the previous info page, which Shift with the
/// next one's key picks; and Back (in menus and in text fields) and skipping the rest at
/// `--more--`, which Esc always does. The player may bind one in Controls.
pub const UNBOUND: &[KeyCommand] = &[
    KeyCommand::Bash,
    KeyCommand::ThrowVoice,
    KeyCommand::Disarm,
    KeyCommand::PickLock,
    KeyCommand::Steal,
    KeyCommand::HoodLantern,
    KeyCommand::Fire,
    KeyCommand::Talents,
    KeyCommand::Quit,
    KeyCommand::PrevTarget,
    KeyCommand::MoreSkip,
    KeyCommand::PrevPage,
    KeyCommand::Back,
    KeyCommand::TextBack,
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
    KeyCode::Enter,
    KeyCode::Backspace,
    KeyCode::Delete,
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
            Search => "search",
            SearchLong => "search_long",
            Open => "open",
            Close => "close",
            Kick => "kick",
            Descend => "descend",
            Ascend => "ascend",
            Lantern => "lantern",
            PickUp => "pick_up",
            Drop => "drop",
            Inventory => "inventory",
            Equipment => "equipment",
            Character => "character",
            Skills => "skills",
            WeaponSkills => "weapon_skills",
            Quests => "quests",
            Talents => "talents",
            Messages => "messages",
            Rest => "rest",
            RestHour => "rest_hour",
            RestDawn => "rest_dawn",
            RestDusk => "rest_dusk",
            MoreNext => "more_next",
            MoreSkip => "more_skip",
            Eat => "eat",
            Read => "read",
            Look => "look",
            Missile => "missile",
            Fire => "fire",
            Throw => "throw",
            MoreAggressive => "more_aggressive",
            MoreDefensive => "more_defensive",
            Abilities => "abilities",
            RepeatAbility => "repeat_ability",
            Bash => "bash",
            ThrowVoice => "throw_voice",
            Disarm => "disarm",
            PickLock => "pick_lock",
            Steal => "steal",
            HoodLantern => "hood_lantern",
            NextTarget => "next_target",
            PrevTarget => "prev_target",
            Loose => "loose",
            UseTool => "use_tool",
            Chat => "chat",
            Pay => "pay",
            MenuUp => "menu_up",
            MenuDown => "menu_down",
            MenuLeft => "menu_left",
            MenuRight => "menu_right",
            Confirm => "confirm",
            Back => "back",
            Erase => "erase",
            Mark => "mark",
            MarkAll => "mark_all",
            NextListPage => "next_list_page",
            PrevListPage => "prev_list_page",
            TextConfirm => "text_confirm",
            TextBack => "text_back",
            TextErase => "text_erase",
            TextRandom => "text_random",
            Yes => "yes",
            No => "no",
            Help => "help",
            Quit => "quit",
            Page(n) => PAGE_IDS.get(usize::from(n).checked_sub(1)?)?,
            NextPage => "next_page",
            PrevPage => "prev_page",
            RevealMap => "dev_reveal_map",
            CycleVision => "dev_next_vision",
            LightValues => "dev_light_values",
            Snapshot => "dev_snapshot",
            SkipHour => "dev_skip_hour",
            NextWeather => "dev_next_weather",
            IdentifyAll => "dev_identify_all",
            LoadSnapshot => "dev_load_snapshot",
            Cancel => return None,
        })
    }

    pub fn from_id(id: &str) -> Option<Self> {
        REBINDABLE.iter().copied().find(|c| c.id() == Some(id))
    }

    /// Name for the controls screen; unlike [`KeyCommand::label`] it tells directions apart.
    pub fn name(self) -> String {
        match self {
            KeyCommand::Move(dir) => format!("Move {}", direction_name(dir)),
            KeyCommand::Page(n) => format!("Info page {n}"),
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

/// Why a key could not be bound: it belongs to a fixed command (one the player cannot rebind,
/// such as Cancel, which opens the pause menu and backs out of screens), or it is the last key of
/// a [`VITAL`] command, which would be locked out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub key: KeyBinding,
    /// The command that uses the key.
    pub owner: KeyCommand,
}

/// What binding a key did besides making it the command's primary key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Bound {
    /// The key was free (or already the command's).
    Free,
    /// Another command of the same context had the key and lost it.
    Moved {
        key: KeyBinding,
        from: KeyCommand,
        /// `from` has no key left.
        keyless: bool,
    },
}

impl Bindings {
    /// The first command that uses `key` in play.
    pub fn owner(&self, key: &KeyBinding) -> Option<KeyCommand> {
        self.owner_in(Context::Play, key)
    }

    /// The first command that uses `key` in `context`.
    pub fn owner_in(&self, context: Context, key: &KeyBinding) -> Option<KeyCommand> {
        (self.0.iter())
            .find(|(k, c)| k == key && c.context() == context)
            .map(|(_, c)| *c)
    }

    /// Keys of `command`, primary first.
    pub fn keys_of(&self, command: KeyCommand) -> Vec<&KeyBinding> {
        self.0
            .iter()
            .filter(|(_, c)| *c == command)
            .map(|(k, _)| k)
            .collect()
    }

    /// The name of `command`'s primary key, as hints and prompts show it; `None` without one.
    pub fn key_name(&self, command: KeyCommand) -> Option<String> {
        self.keys_of(command).first().map(|k| k.display())
    }

    /// Whether `owner` may not give up `key`: a fixed command's, or a vital command's last.
    pub(super) fn keeps(&self, owner: KeyCommand, key: &KeyBinding) -> bool {
        let last = self.keys_of(owner) == [key];
        !REBINDABLE.contains(&owner) || (VITAL.contains(&owner) && last)
    }

    /// Gives every [`VITAL`] command left without a key its default keys back, taken from
    /// whoever holds them unless that is another vital command's last key. The last guard
    /// against a lockout, after a settings file or a keymap is read.
    pub(crate) fn ensure_vital(&mut self) {
        for &command in VITAL {
            if !self.keys_of(command).is_empty() {
                continue;
            }
            let defaults = Bindings::default();
            for key in defaults.keys_of(command) {
                match self.owner_in(command.context(), key) {
                    Some(owner) if self.keeps(owner, key) => continue,
                    Some(owner) => self.0.retain(|(k, c)| !(*c == owner && k == key)),
                    None => {}
                }
                self.0.push((key.clone(), command));
            }
        }
    }

    /// Every key bound to two different commands of one context: `(key, first, second)`.
    pub fn conflicts(&self) -> Vec<(KeyBinding, KeyCommand, KeyCommand)> {
        let mut found = Vec::new();
        for (i, (key, command)) in self.0.iter().enumerate() {
            if let Some((_, first)) = (self.0[..i].iter())
                .find(|(k, c)| k == key && c != command && c.context() == command.context())
            {
                found.push((key.clone(), *first, *command));
            }
        }
        found
    }

    /// Makes `key` the primary (first) key of `command`. A key another command of the same
    /// context uses is taken from it ([`Bound::Moved`]), unless that command is fixed (not in
    /// [`REBINDABLE`]): those are never touched, so the player cannot lock themselves out.
    pub fn bind_primary(
        &mut self,
        command: KeyCommand,
        key: KeyBinding,
    ) -> Result<Bound, Conflict> {
        let mut bound = Bound::Free;
        if let Some(owner) = self.owner_in(command.context(), &key)
            && owner != command
        {
            if self.keeps(owner, &key) {
                return Err(Conflict { key, owner });
            }
            self.0.retain(|(k, c)| !(*c == owner && *k == key));
            bound = Bound::Moved {
                key: key.clone(),
                from: owner,
                keyless: self.keys_of(owner).is_empty(),
            };
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
        Ok(bound)
    }

    /// Replaces all keys of `command` with `keys`, at the place of its first entry.
    pub(super) fn set_keys(&mut self, command: KeyCommand, keys: Vec<KeyBinding>) {
        let at = self
            .0
            .iter()
            .position(|(_, c)| *c == command)
            .unwrap_or(self.0.len());
        self.0.retain(|(_, c)| *c != command);
        let entries = keys.into_iter().map(|k| (k, command));
        self.0.splice(at..at, entries);
    }

    /// Restores the default keys of `command`, except keys another command now uses. A vital
    /// command that would be left without a key keeps the keys it has.
    pub fn reset_command(&mut self, command: KeyCommand) {
        let defaults: Vec<KeyBinding> = Bindings::default()
            .0
            .into_iter()
            .filter(|(k, c)| {
                *c == command
                    && self
                        .owner_in(command.context(), k)
                        .is_none_or(|o| o == command)
            })
            .map(|(k, _)| k)
            .collect();
        if defaults.is_empty() && VITAL.contains(&command) {
            return;
        }
        self.set_keys(command, defaults);
    }

    /// Restores the default keys of every rebindable command.
    pub fn reset_all(&mut self) {
        // Twice, so keys that swapped between commands end up home.
        for _ in 0..2 {
            for &command in REBINDABLE {
                self.reset_command(command);
            }
        }
    }

    /// The settings-file form: command id to key spellings. A command with no key is an empty
    /// list, unless it has none by default ([`UNBOUND`]), when it is left out.
    pub fn to_map(&self) -> BTreeMap<String, Vec<String>> {
        REBINDABLE
            .iter()
            .copied()
            .filter_map(|c| {
                let id = c.id()?;
                let keys: Vec<String> = self.keys_of(c).iter().map(|k| k.to_text()).collect();
                (!keys.is_empty() || !UNBOUND.contains(&c)).then(|| (id.to_string(), keys))
            })
            .collect()
    }

    /// Starts from the defaults and applies the `[bindings]` table of a settings file. Anything
    /// invalid is skipped with a warning. An empty list leaves a command unbound; a command whose
    /// listed keys are all unusable gets its defaults back.
    pub fn from_table(table: &toml::Table, warnings: &mut Vec<String>) -> Self {
        let mut bindings = Bindings::default();
        for id in table.keys() {
            if KeyCommand::from_id(id).is_none() {
                warnings.push(format!("bindings: unknown command `{id}` ignored"));
            }
        }
        let mut emptied: Vec<KeyCommand> = Vec::new();
        for &command in REBINDABLE {
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
                if items.is_empty() && VITAL.contains(&command) {
                    warnings.push(format!(
                        "bindings.{id}: needed to reach the controls; using the defaults"
                    ));
                } else if items.is_empty() {
                    // Left without a key on purpose (its key was moved to another command).
                    bindings.set_keys(command, Vec::new());
                    emptied.push(command);
                } else {
                    warnings.push(format!("bindings.{id}: no usable keys; using the defaults"));
                }
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
        let mut seen: Vec<(Context, KeyBinding)> = Vec::new();
        bindings.0.retain(|(k, c)| {
            let fresh = !seen.iter().any(|(sc, sk)| *sc == c.context() && sk == k);
            seen.push((c.context(), k.clone()));
            fresh
        });
        for &command in REBINDABLE {
            if bindings.keys_of(command).is_empty() && !emptied.contains(&command) {
                bindings.reset_command(command);
            }
        }
        bindings.ensure_vital();
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
        for &c in REBINDABLE {
            let id = c.id().expect("rebindable commands have ids");
            assert_eq!(KeyCommand::from_id(id), Some(c));
        }
        assert_eq!(KeyCommand::Cancel.id(), None);
        let defaults = Bindings::default();
        for &c in REBINDABLE {
            let unbound = UNBOUND.contains(&c);
            assert_eq!(
                defaults.keys_of(c).is_empty(),
                unbound,
                "{c:?}: a default key"
            );
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
    fn binding_a_used_key_moves_it_and_can_leave_the_old_command_keyless() {
        let mut b = Bindings::default();
        let kick = b.keys_of(KeyCommand::Kick)[0].clone();
        assert_eq!(b.keys_of(KeyCommand::Kick).len(), 1);
        let got = b.bind_primary(KeyCommand::Wait, kick.clone()).unwrap();
        assert_eq!(
            got,
            Bound::Moved {
                key: kick.clone(),
                from: KeyCommand::Kick,
                keyless: true
            }
        );
        assert!(b.keys_of(KeyCommand::Kick).is_empty());
        assert_eq!(b.keys_of(KeyCommand::Wait)[0], &kick);
        assert!(b.conflicts().is_empty());
    }

    #[test]
    fn moving_a_key_keeps_the_old_commands_other_keys() {
        let mut b = Bindings::default();
        let got = b.bind_primary(KeyCommand::Wait, ch("k")).unwrap();
        assert_eq!(
            got,
            Bound::Moved {
                key: ch("k"),
                from: KeyCommand::Move(Dir::N),
                keyless: false
            }
        );
        assert_eq!(b.keys_of(KeyCommand::Move(Dir::N)).len(), 2);
        assert_eq!(b.owner(&ch("k")), Some(KeyCommand::Wait));
        assert!(b.conflicts().is_empty());
    }

    #[test]
    fn moving_a_key_leaves_other_contexts_alone() {
        let mut b = Bindings::default();
        // `1` is Page 1 on the info screens; in play it is free.
        assert_eq!(
            b.owner_in(Context::Pages, &ch("1")),
            Some(KeyCommand::Page(1))
        );
        assert_eq!(
            b.bind_primary(KeyCommand::Wait, ch("1")).unwrap(),
            Bound::Free
        );
        assert_eq!(
            b.owner_in(Context::Pages, &ch("1")),
            Some(KeyCommand::Page(1))
        );
        assert_eq!(b.owner(&ch("1")), Some(KeyCommand::Wait));
    }

    #[test]
    fn a_fixed_command_never_loses_its_key() {
        let mut b = Bindings::default();
        let before = b.0.clone();
        let esc = KeyBinding::Code(KeyCode::Escape);
        let err = b.bind_primary(KeyCommand::Wait, esc.clone()).unwrap_err();
        assert_eq!(
            err,
            Conflict {
                key: esc,
                owner: KeyCommand::Cancel
            }
        );
        assert_eq!(b.0, before);
    }

    #[test]
    fn a_keyless_command_survives_a_save_and_load() {
        let mut b = Bindings::default();
        b.bind_primary(KeyCommand::Wait, ch("K")).unwrap();
        assert!(b.keys_of(KeyCommand::Kick).is_empty());
        let text = toml::to_string(&b.to_map()).unwrap();
        let table: toml::Table = toml::from_str(&text).unwrap();
        let mut warnings = Vec::new();
        let back = Bindings::from_table(&table, &mut warnings);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!(back.keys_of(KeyCommand::Kick).is_empty());
        assert_eq!(back.to_map(), b.to_map());
        // Commands unbound by default stay out of the file.
        assert!(!b.to_map().contains_key("quit"));
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
        b.bind_primary(KeyCommand::Descend, ch("D")).unwrap();
        let text = toml::to_string(&b.to_map()).unwrap();
        let table: toml::Table = toml::from_str(&text).unwrap();
        let mut warnings = Vec::new();
        let back = Bindings::from_table(&table, &mut warnings);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(back.to_map(), b.to_map());
    }

    /// Every key some command has by default, and a few more.
    fn some_keys() -> Vec<KeyBinding> {
        let mut keys: Vec<KeyBinding> = Vec::new();
        let defaults = Bindings::default().0.into_iter().map(|(k, _)| k);
        for key in defaults.chain(["w", "s", "x", "Q"].map(ch)) {
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        keys
    }

    #[test]
    fn the_vital_commands_never_lose_their_last_key() {
        // Binding every key to every command, one after another, never leaves a vital command
        // without a key, and Esc stays Cancel's.
        let keys = some_keys();
        let mut b = Bindings::default();
        for &command in REBINDABLE {
            for key in &keys {
                let _ = b.bind_primary(command, key.clone());
                for &vital in VITAL {
                    assert!(!b.keys_of(vital).is_empty(), "{vital:?} after {command:?}");
                }
            }
        }
        let esc = KeyBinding::Code(KeyCode::Escape);
        assert_eq!(b.keys_of(KeyCommand::Cancel), vec![&esc]);
        assert!(b.conflicts().is_empty());
        // Reset all keys gets everything back.
        b.reset_all();
        assert_eq!(b.to_map(), Bindings::default().to_map());
    }

    #[test]
    fn a_vital_commands_last_key_is_refused_with_its_owner() {
        let mut b = Bindings::default();
        let enter = KeyBinding::Code(KeyCode::Enter);
        // Choose has Enter and NumEnter: Enter may move to Erase, then NumEnter may not.
        assert!(matches!(
            b.bind_primary(KeyCommand::Erase, enter.clone()),
            Ok(Bound::Moved { keyless: false, .. })
        ));
        let numpad = KeyBinding::Code(KeyCode::NumpadEnter);
        assert_eq!(
            b.bind_primary(KeyCommand::Mark, numpad.clone()),
            Err(Conflict {
                key: numpad.clone(),
                owner: KeyCommand::Confirm
            })
        );
        // Resetting a vital command whose default keys are all taken keeps the keys it has.
        b.bind_primary(KeyCommand::Confirm, ch("x")).unwrap();
        assert_eq!(b.keys_of(KeyCommand::Confirm), vec![&ch("x")]);
        b.bind_primary(KeyCommand::Mark, numpad).unwrap();
        b.reset_command(KeyCommand::Confirm);
        assert_eq!(b.keys_of(KeyCommand::Confirm), vec![&ch("x")]);
    }

    #[test]
    fn a_settings_file_cannot_lock_the_player_out() {
        let table: toml::Table = toml::from_str(
            r#"
            menu_up = []
            confirm = ["Escape"]
            mark = ["ArrowDown", "j"]
            menu_down = ["ArrowDown"]
            "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let b = Bindings::from_table(&table, &mut warnings);
        for &vital in VITAL {
            assert!(!b.keys_of(vital).is_empty(), "{vital:?}");
        }
        assert_eq!(b.keys_of(KeyCommand::MenuUp).len(), 2, "its defaults");
        assert_eq!(
            b.keys_of(KeyCommand::Confirm)[0],
            &KeyBinding::Code(KeyCode::Enter)
        );
        assert!(b.conflicts().is_empty());
        for needle in ["bindings.menu_up", "bindings.confirm"] {
            assert!(warnings.iter().any(|w| w.contains(needle)), "{needle}");
        }
    }

    #[test]
    fn ensure_vital_takes_defaults_back_from_other_commands() {
        let mut b = Bindings::default();
        // Menu down without a key, one of its keys held by Mark (as a hand-edited file might
        // leave it).
        b.0.retain(|(_, c)| *c != KeyCommand::MenuDown);
        b.0.push((KeyBinding::Code(KeyCode::ArrowDown), KeyCommand::Mark));
        b.ensure_vital();
        assert_eq!(b.keys_of(KeyCommand::MenuDown).len(), 2);
        assert_eq!(
            b.keys_of(KeyCommand::Mark),
            vec![&KeyBinding::Code(KeyCode::Space)]
        );
        assert!(b.conflicts().is_empty());
    }
}
