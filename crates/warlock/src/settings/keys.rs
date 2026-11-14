//! Rebindable commands and their text form in the settings file (spec §6).
//!
//! A key is written as the character it produces (`"k"`, `">"`) or as a physical key name
//! (`"ArrowUp"`, `"Numpad8"`, `"Enter"`, `"F5"`), a play command's perhaps after modifiers
//! (`"Ctrl+r"`, `"Alt+ArrowUp"`, `"Super+s"`; `settings::mods`). Commands have stable ids (`"move_n"`,
//! `"wait"`, `"back"`), unique across contexts.
//!
//! No lockout (#159): Esc cannot be bound (it always backs out, cancels, and opens the pause
//! menu), and the keys that reach the controls screen and its Reset all keys (the menu keys,
//! Enter among them) are fixed (`settings::fixed`, #1533).

use crate::input::{Bindings, Context, KeyBinding, KeyCommand};
use crate::settings::fixed;
use crate::settings::mods::{self, Mods, Platform};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use std::collections::BTreeMap;
use warlock_core::Dir;

/// The commands the player may rebind, in the order the controls screen lists them within each
/// context (it groups them by context). `Cancel` (Esc) is fixed so a rebinding can always be
/// cancelled and every screen backed out of; so are the keys of menus, text fields, the dialog,
/// the info pages, the rest prompt, and aiming, but for their back-style commands
/// (`settings::fixed`).
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
    KeyCommand::Run(Dir::N),
    KeyCommand::Run(Dir::NE),
    KeyCommand::Run(Dir::E),
    KeyCommand::Run(Dir::SE),
    KeyCommand::Run(Dir::S),
    KeyCommand::Run(Dir::SW),
    KeyCommand::Run(Dir::W),
    KeyCommand::Run(Dir::NW),
    KeyCommand::Wait,
    KeyCommand::SearchLong,
    KeyCommand::Open,
    KeyCommand::Close,
    KeyCommand::Kick,
    KeyCommand::Descend,
    KeyCommand::Ascend,
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
    KeyCommand::MoreNext,
    KeyCommand::MoreSkip,
    KeyCommand::Eat,
    KeyCommand::Read,
    KeyCommand::Look,
    KeyCommand::Missile,
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
    KeyCommand::Pay,
    KeyCommand::PrevTarget,
    KeyCommand::UseTool,
    KeyCommand::Chat,
    KeyCommand::Help,
    KeyCommand::Quit,
    KeyCommand::Back,
    KeyCommand::TextBack,
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

/// The abilities' own keys: play commands that each use one ability, as the abilities menu (`a`)
/// does, for a player who wants a key for one. Niche and unbound by default ([`UNBOUND`]), so
/// Controls and the `?` help list them last, under a heading of their own below every other
/// group (the owner, #644).
pub const ABILITY_KEYS: &[KeyCommand] = &[
    KeyCommand::Bash,
    KeyCommand::ThrowVoice,
    KeyCommand::Disarm,
    KeyCommand::PickLock,
    KeyCommand::Steal,
];

/// Ids of commands that were removed: an old settings or keymap file may still list them, and
/// they are dropped without a warning. Add an id here when a command is deleted. The menu, text
/// field, dialog, info page, rest prompt, and aiming keys became fixed in #1533.
pub const RETIRED: &[&str] = &[
    "confirm",
    "erase",
    "fire",
    "hood_lantern",
    "lantern",
    "loose",
    "mark",
    "mark_all",
    "menu_down",
    "menu_left",
    "menu_right",
    "menu_up",
    "next_list_page",
    "next_page",
    "next_target",
    "no",
    "page_1",
    "page_2",
    "page_3",
    "page_4",
    "page_5",
    "page_6",
    "page_7",
    "page_8",
    "page_9",
    "prev_list_page",
    "rest_dawn",
    "rest_dusk",
    "rest_hour",
    "text_confirm",
    "text_erase",
    "text_random",
    "yes",
];

/// The keymap file's context tables that were removed with their commands (#1533): an old file
/// may still have them, and they are skipped without a warning.
pub const RETIRED_CONTEXTS: &[&str] = &["dialog", "prompt"];

/// Ids of commands merged into another: an old settings or keymap file's keys for one go to the
/// command that took it over, after that command's own (Search into Wait/search, #1326).
const MERGED: &[(&str, KeyCommand)] = &[("search", KeyCommand::Wait)];

/// `table` (a `[bindings]` table, or a keymap with its context tables) with the keys of every
/// merged id ([`MERGED`]) moved to the command that took it over, or `table` itself when it lists
/// none. A command the file does not list starts from its defaults.
pub fn fold_merged(table: &toml::Table) -> std::borrow::Cow<'_, toml::Table> {
    let has_merged = |t: &toml::Table| MERGED.iter().any(|(old, _)| t.contains_key(*old));
    let nested = table.values().any(|v| v.as_table().is_some_and(has_merged));
    if !has_merged(table) && !nested {
        return std::borrow::Cow::Borrowed(table);
    }
    let mut folded = table.clone();
    fold_into(&mut folded);
    for (_, value) in folded.iter_mut() {
        if let toml::Value::Table(inner) = value {
            fold_into(inner);
        }
    }
    std::borrow::Cow::Owned(folded)
}

fn fold_into(table: &mut toml::Table) {
    for (old, command) in MERGED {
        let Some(toml::Value::Array(keys)) = table.remove(*old) else {
            continue;
        };
        let Some(id) = command.id() else { continue };
        let defaults = || {
            let defaults = Bindings::default();
            (defaults.keys_of(*command).iter())
                .map(|k| toml::Value::String(k.to_text()))
                .collect()
        };
        let entry = (table.entry(id.to_string())).or_insert_with(|| toml::Value::Array(defaults()));
        if let toml::Value::Array(own) = entry {
            own.extend(keys);
        }
    }
}

/// Rebindable commands with no key by default: abilities (`warlock_core::ability`), which the
/// abilities menu (#156) lists instead of a key each; save and quit, which the pause menu does;
/// the previous target and the previous info page, which Shift with the next one's key picks;
/// and Back (in menus and in text fields) and skipping the rest at `--more--`, which Esc always
/// does. The player may bind one in Controls.
pub const UNBOUND: &[KeyCommand] = &[
    KeyCommand::Bash,
    KeyCommand::ThrowVoice,
    KeyCommand::Disarm,
    KeyCommand::PickLock,
    KeyCommand::Steal,
    KeyCommand::Quit,
    KeyCommand::PrevTarget,
    KeyCommand::MoreSkip,
    KeyCommand::PrevPage,
    KeyCommand::Back,
    KeyCommand::TextBack,
];

/// Physical keys a command may be bound to.
pub(crate) const CODES: &[KeyCode] = &[
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
            Run(Dir::N) => "run_n",
            Run(Dir::NE) => "run_ne",
            Run(Dir::E) => "run_e",
            Run(Dir::SE) => "run_se",
            Run(Dir::S) => "run_s",
            Run(Dir::SW) => "run_sw",
            Run(Dir::W) => "run_w",
            Run(Dir::NW) => "run_nw",
            Wait => "wait",
            SearchLong => "search_long",
            Open => "open",
            Close => "close",
            Kick => "kick",
            Descend => "descend",
            Ascend => "ascend",
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
            MoreNext => "more_next",
            MoreSkip => "more_skip",
            Eat => "eat",
            Read => "read",
            Look => "look",
            Missile => "missile",
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
            PrevTarget => "prev_target",
            UseTool => "use_tool",
            Chat => "chat",
            Pay => "pay",
            Back => "back",
            TextBack => "text_back",
            Help => "help",
            Quit => "quit",
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
            KeyCommand::Run(dir) => format!("Run {}", direction_name(dir)),
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
    /// The settings-file spelling: `"k"`, `"ArrowUp"`, `"Ctrl+r"` (`settings::mods`).
    pub fn to_text(&self) -> String {
        match self {
            KeyBinding::Char(s) => s.clone(),
            KeyBinding::Code(code) => format!("{code:?}"),
            KeyBinding::Chord(mods, key) => format!("{}{}", mods.to_text(), key.to_text()),
        }
    }

    /// Parses the settings-file spelling. Esc is reserved, so it cannot be bound, alone or with
    /// modifiers.
    pub fn from_text(text: &str) -> Option<Self> {
        let (mods, key) = Mods::split(text);
        let mut chars = key.chars();
        let key = match (chars.next(), chars.next()) {
            (Some(c), None) => {
                (!c.is_control() && !c.is_whitespace()).then(|| KeyBinding::Char(c.to_string()))
            }
            (Some(_), Some(_)) => CODES
                .iter()
                .find(|code| format!("{code:?}") == key)
                .map(|code| KeyBinding::Code(*code)),
            (None, _) => None,
        }?;
        Some(KeyBinding::chord(mods, key))
    }

    /// The binding a key press stands for, if it can be bound, `mods` the modifiers held.
    /// Printable keys bind by the character they produce; numpad keys (which also produce
    /// digits), arrows, and function keys bind by physical key; Ctrl, Alt, and Super make a
    /// chord (`settings::mods::press_binding`). Shift is left out: see [`Press::shift`].
    ///
    /// [`Press::shift`]: crate::ui::Press
    pub fn from_input(input: &KeyboardInput, mods: Mods) -> Option<Self> {
        mods::press_binding(input, mods, CODES)
    }

    /// Why this key may not be a key of `command` here, if it may not
    /// (`settings::mods::refusal`), or it is fixed where `command` is read
    /// (`settings::fixed::refusal`).
    pub fn refused_for(&self, command: KeyCommand) -> Option<String> {
        mods::refusal(self, command, Platform::current())
            .or_else(|| fixed::refusal(self, command.context()))
    }
}

/// Why a key could not be bound: it belongs to a fixed command (one the player cannot rebind,
/// such as Cancel, which opens the pause menu and backs out of screens).
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

    /// Whether `owner` may not give up `key`: a fixed command's (Esc).
    pub(super) fn keeps(&self, owner: KeyCommand) -> bool {
        !REBINDABLE.contains(&owner)
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

    /// Makes `key` the primary (first) key of `command`: [`Bindings::bind_at`] its first key. The
    /// tests' shorthand (Controls binds by place).
    #[cfg(test)]
    pub fn bind_primary(
        &mut self,
        command: KeyCommand,
        key: KeyBinding,
    ) -> Result<Bound, Conflict> {
        self.bind_at(command, key, Some(0))
    }

    /// Binds `key` to `command`: in place of its key number `at` (from 0, the primary), or
    /// after its last key for `None` (or an `at` past them). A key the command already has
    /// trades places with the one replaced (an added one stays where it is). A key another
    /// command of the same context uses is taken from it ([`Bound::Moved`]), unless that command
    /// is fixed (not in [`REBINDABLE`]): that is never touched, so the player cannot lock
    /// themselves out.
    pub fn bind_at(
        &mut self,
        command: KeyCommand,
        key: KeyBinding,
        at: Option<usize>,
    ) -> Result<Bound, Conflict> {
        let mut bound = Bound::Free;
        if let Some(owner) = self.owner_in(command.context(), &key)
            && owner != command
        {
            if self.keeps(owner) {
                return Err(Conflict { key, owner });
            }
            self.0.retain(|(k, c)| !(*c == owner && *k == key));
            bound = Bound::Moved {
                key: key.clone(),
                from: owner,
                keyless: self.keys_of(owner).is_empty(),
            };
        }
        let own: Vec<usize> = (self.0.iter().enumerate())
            .filter(|(_, (_, c))| *c == command)
            .map(|(i, _)| i)
            .collect();
        let same = self.0.iter().position(|(k, c)| *c == command && *k == key);
        match (at.and_then(|n| own.get(n).copied()), same) {
            (Some(target), Some(same)) => {
                let old = std::mem::replace(&mut self.0[target].0, key);
                self.0[same].0 = old;
            }
            (Some(target), None) => self.0[target].0 = key,
            (None, Some(_)) => {}
            (None, None) => {
                let after = own.last().map_or(self.0.len(), |last| last + 1);
                self.0.insert(after, (key, command));
            }
        }
        Ok(bound)
    }

    /// Takes key number `at` (from 0, the primary) from `command`, and returns it; `None` when it
    /// has no such key. Refused with [`Conflict`] for a fixed command's key.
    pub fn remove_key(
        &mut self,
        command: KeyCommand,
        at: usize,
    ) -> Option<Result<KeyBinding, Conflict>> {
        let key = self.keys_of(command).get(at).map(|k| (*k).clone())?;
        if self.keeps(command) {
            return Some(Err(Conflict {
                key,
                owner: command,
            }));
        }
        self.0.retain(|(k, c)| !(*c == command && *k == key));
        Some(Ok(key))
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

    /// Restores the default keys of `command`, except keys another command now uses.
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
        let table = &*fold_merged(table);
        let mut bindings = Bindings::default();
        for id in table.keys() {
            if KeyCommand::from_id(id).is_none() && !RETIRED.contains(&id.as_str()) {
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
                    Some(key) => match key.refused_for(command) {
                        Some(why) => warnings.push(format!("bindings.{id}: {why}")),
                        None => keys.push(key),
                    },
                    None => warnings.push(format!("bindings.{id}: invalid key {item}")),
                }
            }
            if keys.is_empty() {
                if items.is_empty() {
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
        // The file's keys win over the defaults of the commands it does not list (a command new
        // since it was written, or one given a default key since, as Search for a while got `S`): such a
        // command loses a default key a listed command of its context has. The file's own keys
        // are never rewritten.
        let listed = |c: &KeyCommand| c.id().is_some_and(|id| table.contains_key(id));
        let claimed: Vec<(Context, KeyBinding)> = (bindings.0.iter())
            .filter(|(_, c)| listed(c))
            .map(|(k, c)| (c.context(), k.clone()))
            .collect();
        bindings.0.retain(|(k, c)| {
            listed(c)
                || !REBINDABLE.contains(c)
                || !claimed.iter().any(|(cc, ck)| *cc == c.context() && ck == k)
        });
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
        bindings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_retired_command_is_dropped_without_a_warning() {
        let table: toml::Table = toml::from_str("fire = [\"f\"]\nghost = [\"g\"]").unwrap();
        let mut warnings = Vec::new();
        Bindings::from_table(&table, &mut warnings);
        assert_eq!(warnings, ["bindings: unknown command `ghost` ignored"]);
        let mut b = Bindings::default();
        let report = crate::settings::keymap::import(&mut b, "fire = [\"f\"]").unwrap();
        assert!(report.warnings.is_empty() && report.changed.is_empty());
    }

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
    fn a_key_is_added_replaced_at_its_place_and_removed() {
        let mut b = Bindings::default();
        let north = KeyCommand::Move(Dir::N);
        let shown = |b: &Bindings| -> Vec<String> {
            b.keys_of(north).iter().map(|k| k.to_text()).collect()
        };
        assert_eq!(shown(&b), ["Numpad8", "ArrowUp"]);
        // Added after the last key; a key it has already stays where it is.
        assert_eq!(b.bind_at(north, ch("w"), None), Ok(Bound::Free));
        assert_eq!(b.bind_at(north, ch("w"), None), Ok(Bound::Free));
        assert_eq!(shown(&b), ["Numpad8", "ArrowUp", "w"]);
        // The second key replaced; one of its own trades places.
        b.bind_at(north, ch("W"), Some(1)).unwrap();
        assert_eq!(shown(&b), ["Numpad8", "W", "w"]);
        b.bind_at(north, ch("w"), Some(0)).unwrap();
        assert_eq!(shown(&b), ["w", "W", "Numpad8"]);
        // Another command's key moves, as for the primary.
        let kick = b.keys_of(KeyCommand::Kick)[0].clone();
        let got = b.bind_at(north, kick.clone(), None).unwrap();
        assert!(matches!(got, Bound::Moved { keyless: true, .. }));
        assert_eq!(b.keys_of(north).last(), Some(&&kick));
        // Removed by place; none past the last.
        assert_eq!(b.remove_key(north, 1), Some(Ok(ch("W"))));
        assert_eq!(shown(&b).len(), 3);
        assert_eq!(b.remove_key(north, 3), None);
        assert!(b.conflicts().is_empty());
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
        let up = KeyBinding::Code(KeyCode::ArrowUp);
        let got = b.bind_primary(KeyCommand::Wait, up.clone()).unwrap();
        assert_eq!(
            got,
            Bound::Moved {
                key: up.clone(),
                from: KeyCommand::Move(Dir::N),
                keyless: false
            }
        );
        assert_eq!(b.keys_of(KeyCommand::Move(Dir::N)).len(), 1);
        assert_eq!(b.owner(&up), Some(KeyCommand::Wait));
        assert!(b.conflicts().is_empty());
    }

    #[test]
    fn moving_a_key_leaves_other_contexts_alone() {
        let mut b = Bindings::default();
        // `q` goes to Back on the screens; in play it is Quests' still.
        b.bind_primary(KeyCommand::Back, ch("q")).unwrap();
        assert_eq!(
            b.bind_primary(KeyCommand::Wait, ch("q")).unwrap(),
            Bound::Moved {
                key: ch("q"),
                from: KeyCommand::Quests,
                keyless: true
            }
        );
        assert_eq!(b.owner_in(Context::Menu, &ch("q")), Some(KeyCommand::Back));
        assert_eq!(b.owner(&ch("q")), Some(KeyCommand::Wait));
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
        b.bind_primary(KeyCommand::Wait, ch("k")).unwrap();
        assert!(b.keys_of(KeyCommand::Kick).is_empty());
        let text = toml::to_string(&b.to_map()).unwrap();
        let table: toml::Table = toml::from_str(&text).unwrap();
        let mut warnings = Vec::new();
        let back = Bindings::from_table(&table, &mut warnings);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!(back.keys_of(KeyCommand::Kick).is_empty());
        assert_eq!(back.to_map(), b.to_map());
        // Commands unbound by default stay out of the file.
        assert!(!b.to_map().contains_key("steal"));
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
        assert_eq!(keys, ["w", "ArrowUp"]);
        assert!(b.conflicts().is_empty());
        // Choosing an alternate promotes it to primary.
        b.bind_primary(KeyCommand::Move(Dir::N), KeyBinding::Code(KeyCode::ArrowUp))
            .unwrap();
        assert_eq!(
            b.keys_of(KeyCommand::Move(Dir::N))[0],
            &KeyBinding::Code(KeyCode::ArrowUp)
        );
        assert_eq!(b.keys_of(KeyCommand::Move(Dir::N)).len(), 2);
    }

    #[test]
    fn reset_restores_defaults_in_place() {
        let mut b = Bindings::default();
        let before = b.to_map();
        b.bind_primary(KeyCommand::Wait, ch("w")).unwrap();
        b.bind_primary(KeyCommand::Open, ch("O")).unwrap();
        b.reset_command(KeyCommand::Wait);
        assert_eq!(b.keys_of(KeyCommand::Wait)[1], &ch("s"));
        b.reset_all();
        assert_eq!(b.to_map(), before);
    }

    #[test]
    fn table_applies_overrides_and_survives_garbage() {
        let table: toml::Table = toml::from_str(
            r#"
            wait = ["w", "Numpad5"]
            look = ["w"]
            close = ["Banana", 7]
            ghost = ["x"]
            help = "?"
            "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let b = Bindings::from_table(&table, &mut warnings);
        assert_eq!(b.keys_of(KeyCommand::Wait)[0], &ch("w"));
        // `w` stays with Wait; Look falls back to its default.
        assert_eq!(b.owner(&ch("w")), Some(KeyCommand::Wait));
        assert_eq!(b.keys_of(KeyCommand::Look), vec![&ch("l")]);
        assert_eq!(b.keys_of(KeyCommand::Close), vec![&ch("c")]);
        assert_eq!(b.keys_of(KeyCommand::Help), vec![&ch("?")]);
        assert!(b.conflicts().is_empty());
        for needle in ["ghost", "used by both", "bindings.close", "bindings.help"] {
            assert!(warnings.iter().any(|w| w.contains(needle)), "{needle}");
        }
    }

    #[test]
    fn an_old_files_keys_win_over_new_defaults() {
        // A file written before #350: the old keys, and no `quit` (unbound then). Its keys stay
        // the player's own; Search for a while, unlisted, gives up its new default `Alt+s` to
        // Skills.
        let table: toml::Table = toml::from_str(
            r#"
            move_n = ["k", "Numpad8", "ArrowUp"]
            kick = ["K"]
            look = ["x"]
            skills = ["S", "Alt+s"]
            "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let b = Bindings::from_table(&table, &mut warnings);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(b.owner(&ch("k")), Some(KeyCommand::Move(Dir::N)));
        assert_eq!(b.keys_of(KeyCommand::Kick), vec![&ch("K")]);
        assert_eq!(b.keys_of(KeyCommand::Look), vec![&ch("x")]);
        let ctrl = |text| KeyBinding::from_text(text).unwrap();
        assert_eq!(
            b.keys_of(KeyCommand::Skills),
            vec![&ch("S"), &ctrl("Alt+s")]
        );
        assert_eq!(
            b.keys_of(KeyCommand::SearchLong),
            vec![&ctrl("Alt+Numpad5")]
        );
        // Commands it does not list and nobody took a key from have the new defaults.
        assert_eq!(b.keys_of(KeyCommand::Equipment), vec![&ch("i")]);
        assert!(b.conflicts().is_empty());
    }

    #[test]
    fn an_old_files_search_keys_go_to_wait_search() {
        // A file from before #1326, when Search was a command of its own: its keys join
        // Wait/search after Wait's, without a warning.
        let table: toml::Table = toml::from_str(
            r#"
            wait = [".", "Numpad5"]
            search = ["s", "f"]
            "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let b = Bindings::from_table(&table, &mut warnings);
        assert!(warnings.is_empty(), "{warnings:?}");
        let keys: Vec<String> = (b.keys_of(KeyCommand::Wait).iter())
            .map(|k| k.to_text())
            .collect();
        assert_eq!(keys, [".", "Numpad5", "s", "f"]);
        // Without `wait`, Wait/search keeps its defaults and takes the old keys too.
        let table: toml::Table = toml::from_str("search = [\"f\"]").unwrap();
        let b = Bindings::from_table(&table, &mut warnings);
        assert!(warnings.is_empty(), "{warnings:?}");
        let keys: Vec<String> = (b.keys_of(KeyCommand::Wait).iter())
            .map(|k| k.to_text())
            .collect();
        assert_eq!(keys, ["Numpad5", "s", "f"]);
        // A keymap's context table too.
        let mut b = Bindings::default();
        let report = crate::settings::keymap::import(&mut b, "[play]\nsearch = [\"f\"]\n").unwrap();
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        assert_eq!(b.owner(&ch("f")), Some(KeyCommand::Wait));
    }

    #[test]
    fn map_round_trips_through_the_table() {
        let mut b = Bindings::default();
        b.bind_primary(KeyCommand::Descend, ch("D")).unwrap();
        let chord = KeyBinding::from_text("Ctrl+Alt+ArrowUp").unwrap();
        b.bind_primary(KeyCommand::Look, chord).unwrap();
        let text = toml::to_string(&b.to_map()).unwrap();
        assert!(text.contains(r#"look = ["Ctrl+Alt+ArrowUp"]"#), "{text}");
        assert!(
            text.contains(r#"search_long = ["Alt+Numpad5", "Alt+s"]"#),
            "{text}"
        );
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
    fn no_binding_takes_esc_and_reset_all_keys_undoes_everything() {
        // Binding every key to every command, one after another, never takes Esc from Cancel.
        let keys = some_keys();
        let mut b = Bindings::default();
        for &command in REBINDABLE {
            for key in &keys {
                let _ = b.bind_primary(command, key.clone());
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
    fn a_back_style_command_never_takes_a_fixed_key() {
        let mut b = Bindings::default();
        let enter = KeyBinding::Code(KeyCode::Enter);
        let why = enter.refused_for(KeyCommand::Back);
        assert_eq!(why.as_deref(), Some("Enter is fixed in menus and screens"));
        let tab = KeyBinding::Code(KeyCode::Tab);
        assert!(tab.refused_for(KeyCommand::PrevTarget).is_some());
        assert!(tab.refused_for(KeyCommand::PrevPage).is_some());
        assert!(tab.refused_for(KeyCommand::TextBack).is_some());
        // A play key may be anything free in play.
        assert!(enter.refused_for(KeyCommand::Wait).is_none());
        b.bind_primary(KeyCommand::Back, ch("q")).unwrap();
        assert_eq!(b.keys_of(KeyCommand::Back), vec![&ch("q")]);
    }

    #[test]
    fn an_old_settings_file_with_the_fixed_commands_loads_quietly() {
        // Written before #1533 made these keys fixed: dropped without a word, and nothing else
        // changes.
        let table: toml::Table = toml::from_str(
            r#"
            menu_up = []
            confirm = ["Escape"]
            mark = ["ArrowDown", "j"]
            yes = ["j"]
            page_1 = ["w"]
            loose = ["f"]
            rest_hour = ["F1"]
            text_confirm = []
            "#,
        )
        .unwrap();
        let mut warnings = Vec::new();
        let b = Bindings::from_table(&table, &mut warnings);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(b.to_map(), Bindings::default().to_map());
    }
}
