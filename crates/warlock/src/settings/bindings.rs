//! The rebindable key commands and their default table (spec §6): what a key does
//! ([`KeyCommand`]), where it is read ([`Context`]), a key it is bound to ([`KeyBinding`]), and the
//! binding table ([`Bindings`]). `input::route_keys` reads presses through it; the settings file
//! (`keys`, `keymap`) and Controls write it.

use crate::settings::mods::{self, Mods, Platform};
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use warlock_core::Dir;

/// What a key does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCommand {
    Move(Dir),
    /// Run in a direction: step after step until something worth stopping for (#1325,
    /// `Action::Run`). Alt with a movement key by default.
    Run(Dir),
    /// Wait a turn, which is searching the cells around the player for secret doors and hidden
    /// traps (spec §8 "Searching"; #1326: one action, `Action::Wait`).
    Wait,
    /// Search turn after turn, up to `data/search.toml`'s `turns`, stopping when something is
    /// found or something interrupts, like a rest.
    SearchLong,
    Open,
    Close,
    /// Kick in a direction: a door (to break it in), or a creature.
    Kick,
    Descend,
    Ascend,
    /// Pick up what lies here: a single stack at once, a choice of several on a screen.
    PickUp,
    /// Choose what to drop from the inventory.
    Drop,
    /// Show what the player carries and weighs (no time passes).
    Inventory,
    /// Show what the player wears and wields, slot by slot, to change it.
    Equipment,
    /// Show the player's name, stats, and weight (no time passes).
    Character,
    /// Show the player's trained skills and their values (no time passes).
    Skills,
    /// Show the player's weapon skills: ranks and marks (no time passes).
    WeaponSkills,
    /// Show the player's quests, active and done (no time passes).
    Quests,
    /// Show the player's talents, and every other with what it needs (no time passes). `G`, a
    /// key ADOM leaves free (ADOM has no talents command).
    Talents,
    /// Show the messages so far, the newest at the bottom (no time passes).
    Messages,
    /// Ask how long to rest (an hour, until dawn, until dusk).
    Rest,
    /// At `--more--`: show the next of this turn's messages.
    MoreNext,
    /// At `--more--`: skip the rest of this turn's messages (Esc always does it; no default key
    /// of its own).
    MoreSkip,
    /// Choose something to consume (eat), from the floor or the pack.
    Eat,
    /// Choose a scroll or grimoire to read: a spell is aimed next (a cursor on the map), the
    /// scroll of identify asks what to read it on, a grimoire asks whether to learn its spell.
    Read,
    /// Move a cursor over the map to read what is there (no time passes).
    Look,
    /// Shoot or throw what is in the missile weapon slot (a cursor on the map): a launcher
    /// shoots its ammunition, anything else is thrown.
    Missile,
    /// Choose something to throw from the pack, aim it, then throw.
    Throw,
    /// Open the abilities menu: racial, class, skill, weapon skill, and item abilities, to
    /// choose one to use (spec §6 "Abilities", #156).
    Abilities,
    /// Use the ability used last again (else open the abilities menu).
    RepeatAbility,
    /// Bash with the shield held, in a direction (asked for in the message line). An ability
    /// (`warlock_core::ability`): unbound by default, as the abilities menu lists it (#156).
    Bash,
    /// Throw your voice to a place you see (aimed with a cursor on the map). An ability:
    /// unbound by default.
    ThrowVoice,
    /// Disarm a trap the player knows of, in a direction (asked for in the message line; spec §8
    /// "Disarming traps"). An ability: unbound by default, as the abilities menu lists it.
    Disarm,
    /// Pick the lock of a door, in a direction (asked for in the message line; spec §8
    /// "Doors"). An ability (`warlock_core::ability`, the Open lock skill's use): unbound by
    /// default, as the abilities menu lists it (#156).
    PickLock,
    /// Steal from someone in a direction (asked for in the message line; spec §8 "Stealing").
    /// An ability: unbound by default, as the abilities menu lists it.
    Steal,
    /// While aiming: the cursor to the creature in view before (no default key: Shift with
    /// the fixed Tab does it, `settings::fixed`).
    PrevTarget,
    /// Take a more aggressive combat stance (Coward, Defensive, Neutral, Aggressive; no time
    /// passes).
    MoreAggressive,
    /// Take a more defensive combat stance (the other way).
    MoreDefensive,
    /// Use the tool the player has ready (ADOM's `U`): toward a direction, dig into a wall with
    /// a pick-axe; a light (a lantern) at once, from unlit to lit to hooded and back. An item's
    /// use, not an ability (abilities get a menu of their own).
    UseTool,
    /// Chat with someone next to you: the one in view if there is only one, else in a
    /// direction.
    Chat,
    /// Pay for what the player took in the shop they stand in (spec §7 "Shops"; no time passes).
    Pay,
    /// On a menu or screen: back out, as Esc always does (no default key of its own). The
    /// other menu keys are fixed (`settings::fixed`).
    Back,
    /// In a text field: back out, as Esc always does (no default key of its own). The other
    /// text field keys are fixed.
    TextBack,
    /// Dev: switch to the next vision profile.
    CycleVision,
    /// Dev: move the clock an hour ahead at once.
    SkipHour,
    /// Dev: set the next weather (clear, overcast, rain, heavy rain) for three hours.
    NextWeather,
    /// Dev: identify everything the player carries (magic items' affixes; the scroll of identify
    /// does it in play).
    IdentifyAll,
    /// Dev: show ground truth where the player does not see.
    RevealMap,
    /// Dev: show perceived light values on visible cells.
    LightValues,
    Help,
    Cancel,
    Quit,
    Snapshot,
    LoadSnapshot,
    /// On an info screen: the previous page (no default key: Shift with the fixed Tab does
    /// it; the digits and Tab are fixed).
    PrevPage,
}

/// Where a command is read. Keys must be unique within a context, not across contexts, so a key
/// can mean one thing in play and another on a screen (Tab: next target while aiming, next page
/// on an info screen; `y` and `n`: moving in play, yes and no in the dialog). A mode or screen
/// with a context of its own reads it first: the rest prompt and aiming before play's keys, an
/// info screen's page keys and the dialog's answers before the menu keys. A text field reads
/// only its own: every other key types what it types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context {
    /// In play: the map, the direction prompts, and look mode.
    Play,
    /// While aiming a shot or a throw (`ui::target`).
    Aim,
    /// At `--more--`, when this turn's messages need more than one look at the message line.
    More,
    /// On every menu and screen: moving the selection, choosing, backing out.
    Menu,
    /// In a text field (the character's name, `ui/name_screen.rs`): accepting, erasing, a random
    /// value, backing out. Keys bound to nothing here type.
    Text,
    /// On an info screen ([`crate::ui`]'s numbered pages).
    Pages,
    /// The developer keys (debug builds, or `--dev`), read in play after play's own.
    Dev,
}

impl Context {
    /// Every context, in the order the controls screen and the keymap file list them.
    pub const ALL: &[Context] = &[
        Context::Play,
        Context::Aim,
        Context::More,
        Context::Menu,
        Context::Text,
        Context::Pages,
        Context::Dev,
    ];

    /// The heading of its group on the controls screen.
    pub fn title(self) -> &'static str {
        match self {
            Context::Play => "In play",
            Context::Aim => "Aiming",
            Context::More => "More messages (--more--)",
            Context::Menu => "Menus and screens",
            Context::Text => "Text fields",
            Context::Pages => "Info pages",
            Context::Dev => "Developer",
        }
    }

    /// Its table in the keymap file (`settings::keymap`).
    pub fn id(self) -> &'static str {
        match self {
            Context::Play => "play",
            Context::Aim => "aim",
            Context::More => "more",
            Context::Menu => "menu",
            Context::Text => "text",
            Context::Pages => "pages",
            Context::Dev => "dev",
        }
    }
}

impl KeyCommand {
    pub fn context(self) -> Context {
        use KeyCommand::*;
        match self {
            PrevPage => Context::Pages,
            PrevTarget => Context::Aim,
            MoreNext | MoreSkip => Context::More,
            Back => Context::Menu,
            TextBack => Context::Text,
            CycleVision | SkipHour | NextWeather | IdentifyAll | RevealMap | LightValues
            | Snapshot | LoadSnapshot => Context::Dev,
            _ => Context::Play,
        }
    }
}

impl KeyCommand {
    /// What the command does; movement directions share one label (see `KeyCommand::name`).
    pub fn label(self) -> &'static str {
        match self {
            KeyCommand::Move(_) => "Move",
            KeyCommand::Run(_) => "Run",
            KeyCommand::Wait => "Wait/search",
            KeyCommand::SearchLong => "Search for a while",
            KeyCommand::Open => "Open a door",
            KeyCommand::Close => "Close a door",
            KeyCommand::Kick => "Kick",
            KeyCommand::Descend => "Go down stairs",
            KeyCommand::Ascend => "Go up stairs",
            KeyCommand::PickUp => "Pick up",
            KeyCommand::Drop => "Drop",
            KeyCommand::Inventory => "Inventory",
            KeyCommand::Equipment => "Equipment",
            KeyCommand::Character => "Character",
            KeyCommand::Skills => "Skills",
            KeyCommand::WeaponSkills => "Weapon skills",
            KeyCommand::Quests => "Quest log",
            KeyCommand::Talents => "Talents",
            KeyCommand::Messages => "Message history",
            KeyCommand::Rest => "Rest",
            KeyCommand::MoreNext => "Show the next messages",
            KeyCommand::MoreSkip => "Skip the rest (Esc always works)",
            KeyCommand::Eat => "Consume",
            KeyCommand::Read => "Read a scroll",
            KeyCommand::Look => "Look around",
            KeyCommand::Missile => "Throw or shoot the missile weapon",
            KeyCommand::Throw => "Throw from the pack",
            KeyCommand::MoreAggressive => "More aggressive stance",
            KeyCommand::MoreDefensive => "More defensive stance",
            KeyCommand::Abilities => "Abilities",
            KeyCommand::RepeatAbility => "Use the last ability again",
            KeyCommand::Bash => "Bash with your shield",
            KeyCommand::ThrowVoice => "Throw your voice",
            KeyCommand::Disarm => "Disarm a trap",
            KeyCommand::PickLock => "Pick a lock",
            KeyCommand::Steal => "Steal",
            KeyCommand::PrevTarget => "Previous target",
            KeyCommand::UseTool => "Use a tool (dig, light)",
            KeyCommand::Chat => "Chat",
            KeyCommand::Pay => "Pay the shopkeeper",
            KeyCommand::Back => "Back (Esc always works)",
            KeyCommand::TextBack => "Back (Esc always works)",
            KeyCommand::CycleVision => "Next vision profile (dev)",
            KeyCommand::SkipHour => "Skip an hour (dev)",
            KeyCommand::NextWeather => "Next weather (dev)",
            KeyCommand::IdentifyAll => "Identify what you carry (dev)",
            KeyCommand::RevealMap => "Reveal map (dev)",
            KeyCommand::LightValues => "Light values (dev)",
            KeyCommand::Help => "Show / hide controls",
            KeyCommand::Cancel => "Cancel",
            KeyCommand::Quit => "Save and quit",
            KeyCommand::Snapshot => "Save snapshot (dev)",
            KeyCommand::LoadSnapshot => "Load snapshot (dev)",
            KeyCommand::PrevPage => "Previous info page",
        }
    }
}

/// A key a command can be bound to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyBinding {
    /// The character produced by the active layout (case-sensitive).
    Char(String),
    /// A physical key (arrows, numpad, function keys).
    Code(KeyCode),
    /// A key held with modifiers, `Ctrl+r` (play commands only; `settings::mods`). Made by
    /// [`KeyBinding::chord`]: the modifiers are never none, the key never a chord, and Shift
    /// only with a physical key.
    Chord(Mods, Box<KeyBinding>),
}

impl KeyBinding {
    /// `key` held with `mods`: the plain key without modifiers.
    pub fn chord(mods: Mods, key: KeyBinding) -> Self {
        let mods = mods.union(key.mods());
        let base = key.base().clone();
        let mods = match base {
            KeyBinding::Code(_) => mods,
            _ => mods.without_shift(),
        };
        if mods.is_empty() {
            base
        } else {
            KeyBinding::Chord(mods, Box::new(base))
        }
    }

    /// The modifiers held with the key (none for a plain key).
    pub fn mods(&self) -> Mods {
        match self {
            KeyBinding::Chord(mods, _) => *mods,
            _ => Mods::NONE,
        }
    }

    /// The key without its modifiers.
    pub fn base(&self) -> &KeyBinding {
        match self {
            KeyBinding::Chord(_, key) => key,
            plain => plain,
        }
    }

    /// Human-readable key name for the controls screen and help, the modifiers named as this
    /// platform names them (`Cmd+s` on macOS).
    pub fn display(&self) -> String {
        self.display_on(Platform::current())
    }

    /// The key's name as `platform` names its modifiers.
    pub fn display_on(&self, platform: Platform) -> String {
        match self {
            KeyBinding::Char(s) => s.clone(),
            KeyBinding::Chord(mods, key) => {
                format!("{}{}", mods.display_on(platform), key.display())
            }
            KeyBinding::Code(code) => match code {
                KeyCode::ArrowUp => "↑".into(),
                KeyCode::ArrowDown => "↓".into(),
                KeyCode::ArrowLeft => "←".into(),
                KeyCode::ArrowRight => "→".into(),
                KeyCode::Escape => "Esc".into(),
                KeyCode::Numpad1 => "Num1".into(),
                KeyCode::Numpad2 => "Num2".into(),
                KeyCode::Numpad3 => "Num3".into(),
                KeyCode::Numpad4 => "Num4".into(),
                KeyCode::Numpad5 => "Num5".into(),
                KeyCode::Numpad6 => "Num6".into(),
                KeyCode::Numpad7 => "Num7".into(),
                KeyCode::Numpad8 => "Num8".into(),
                KeyCode::Numpad9 => "Num9".into(),
                KeyCode::NumpadEnter => "NumEnter".into(),
                KeyCode::Delete => "Del".into(),
                other => format!("{other:?}"),
            },
        }
    }
}

/// The binding table. The controls screen and `?` help are generated from it, so they are always
/// accurate.
#[derive(Resource, Debug, Clone)]
pub struct Bindings(pub Vec<(KeyBinding, KeyCommand)>);

impl Default for Bindings {
    fn default() -> Self {
        use KeyBinding::{Char, Code};
        use KeyCommand::*;
        let ch = |s: &str| Char(s.to_string());
        // ADOM's default keys where ADOM has the command (#350; ADOM's manual, "Miscellaneous
        // commands"), else a key ADOM does not use. Movement is ADOM's: the numpad and the arrows
        // (its letters are other commands: `k` kick, `l` look, `u` use a tool).
        let mut table = Self(vec![
            (Code(KeyCode::Numpad8), Move(Dir::N)),
            (Code(KeyCode::Numpad9), Move(Dir::NE)),
            (Code(KeyCode::Numpad6), Move(Dir::E)),
            (Code(KeyCode::Numpad3), Move(Dir::SE)),
            (Code(KeyCode::Numpad2), Move(Dir::S)),
            (Code(KeyCode::Numpad1), Move(Dir::SW)),
            (Code(KeyCode::Numpad4), Move(Dir::W)),
            (Code(KeyCode::Numpad7), Move(Dir::NW)),
            (Code(KeyCode::ArrowUp), Move(Dir::N)),
            (Code(KeyCode::ArrowRight), Move(Dir::E)),
            (Code(KeyCode::ArrowDown), Move(Dir::S)),
            (Code(KeyCode::ArrowLeft), Move(Dir::W)),
        ]);
        // Running: Alt with each movement key (#1325; Alt, not Ctrl, since #1385: macOS keeps
        // Ctrl+arrows for Spaces). Every default modifier is Alt, on every platform.
        let runs: Vec<_> = (table.0.iter())
            .filter_map(|(key, command)| match command {
                Move(dir) => Some((KeyBinding::chord(Mods::ALT, key.clone()), Run(*dir))),
                _ => None,
            })
            .collect();
        table.0.extend(runs);
        table.0.extend([
            (Code(KeyCode::Numpad5), Wait),
            (ch("s"), Wait),
            (
                KeyBinding::chord(Mods::ALT, Code(KeyCode::Numpad5)),
                SearchLong,
            ),
            (KeyBinding::chord(Mods::ALT, ch("s")), SearchLong),
            (ch("o"), Open),
            (ch("c"), Close),
            (ch("k"), Kick),
            (ch(">"), Descend),
            (ch("<"), Ascend),
            (ch(","), PickUp),
            (ch(";"), PickUp),
            (ch("d"), Drop),
            (ch("I"), Inventory),
            (ch("i"), Equipment),
            (ch("@"), Character),
            (ch("A"), Skills),
            (ch("W"), WeaponSkills),
            (ch("q"), Quests),
            (ch("G"), Talents),
            (ch("v"), Messages),
            (ch("b"), Rest),
            (ch("e"), Eat),
            (ch("r"), Read),
            (ch("l"), Look),
            (ch("t"), Missile),
            (ch("T"), Throw),
            (ch("+"), MoreAggressive),
            (ch("-"), MoreDefensive),
            (ch("a"), Abilities),
            (ch("X"), RepeatAbility),
            (ch("C"), Chat),
            (Code(KeyCode::Space), MoreNext),
            (Code(KeyCode::Enter), MoreNext),
            (Code(KeyCode::NumpadEnter), MoreNext),
            (ch("u"), UseTool),
            (ch("p"), Pay),
            (ch("?"), Help),
            (Code(KeyCode::Escape), Cancel),
        ]);
        // The developer keys: in every build's table, read only where they are on (`dev_keys`).
        table.0.extend([
            (Code(KeyCode::F2), RevealMap),
            (Code(KeyCode::F3), CycleVision),
            (Code(KeyCode::F4), LightValues),
            (Code(KeyCode::F5), Snapshot),
            (Code(KeyCode::F6), SkipHour),
            (Code(KeyCode::F7), NextWeather),
            (Code(KeyCode::F8), IdentifyAll),
            (Code(KeyCode::F9), LoadSnapshot),
        ]);
        table
    }
}

impl Bindings {
    /// The command a key press stands for in play, `mods` the modifiers held.
    pub fn lookup(&self, input: &KeyboardInput, mods: Mods) -> Option<KeyCommand> {
        self.lookup_in(Context::Play, input, mods)
    }

    /// The command a key press stands for in `context`, `mods` the modifiers held: the first
    /// of the bindings the press may stand for that a command of `context` has
    /// ([`mods::press_candidates`]; matching is exact, so `Ctrl+r` never fires `r`).
    pub fn lookup_in(
        &self,
        context: Context,
        input: &KeyboardInput,
        mods: Mods,
    ) -> Option<KeyCommand> {
        (mods::press_candidates(input, mods, crate::settings::keys::CODES).iter())
            .find_map(|key| self.owner_in(context, key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(b: &Bindings, command: KeyCommand) -> Vec<String> {
        b.keys_of(command).iter().map(|k| k.display()).collect()
    }

    #[test]
    fn default_keys_are_adoms_numpad_and_arrows() {
        let b = Bindings::default();
        assert_eq!(keys(&b, KeyCommand::Move(Dir::N)), ["Num8", "↑"]);
        assert_eq!(keys(&b, KeyCommand::Move(Dir::NE)), ["Num9"]);
        assert_eq!(keys(&b, KeyCommand::Run(Dir::N)), ["Alt+Num8", "Alt+↑"]);
        assert_eq!(keys(&b, KeyCommand::Run(Dir::NE)), ["Alt+Num9"]);
        assert_eq!(keys(&b, KeyCommand::Descend), [">"]);
        assert!(keys(&b, KeyCommand::Quit).is_empty(), "the menu quits");
        assert_eq!(keys(&b, KeyCommand::Kick), ["k"]);
        assert_eq!(keys(&b, KeyCommand::Help), ["?"]);
        assert_eq!(keys(&b, KeyCommand::Cancel), ["Esc"]);
        assert_eq!(keys(&b, KeyCommand::Rest), ["b"]);
        assert_eq!(keys(&b, KeyCommand::Wait), ["Num5", "s"]);
        assert_eq!(keys(&b, KeyCommand::SearchLong), ["Alt+Num5", "Alt+s"]);
        assert_eq!(keys(&b, KeyCommand::PickUp), [",", ";"]);
        assert_eq!(keys(&b, KeyCommand::Drop), ["d"]);
        assert_eq!(keys(&b, KeyCommand::Inventory), ["I"]);
        assert_eq!(keys(&b, KeyCommand::Equipment), ["i"]);
        assert_eq!(keys(&b, KeyCommand::Skills), ["A"]);
        assert_eq!(keys(&b, KeyCommand::Look), ["l"]);
        assert_eq!(keys(&b, KeyCommand::Messages), ["v"]);
        assert_eq!(keys(&b, KeyCommand::Character), ["@"]);
        assert_eq!(keys(&b, KeyCommand::Missile), ["t"]);
        assert_eq!(keys(&b, KeyCommand::Throw), ["T"]);
        assert_eq!(keys(&b, KeyCommand::WeaponSkills), ["W"]);
        assert_eq!(keys(&b, KeyCommand::Talents), ["G"]);
        assert!(keys(&b, KeyCommand::Bash).is_empty(), "an ability: unbound");
        assert!(
            keys(&b, KeyCommand::Disarm).is_empty(),
            "an ability: unbound"
        );
        assert!(
            keys(&b, KeyCommand::PickLock).is_empty(),
            "an ability: unbound"
        );
        assert!(
            keys(&b, KeyCommand::Steal).is_empty(),
            "an ability: unbound"
        );
        assert_eq!(keys(&b, KeyCommand::Abilities), ["a"]);
        assert_eq!(keys(&b, KeyCommand::RepeatAbility), ["X"]);
        assert_eq!(keys(&b, KeyCommand::MoreAggressive), ["+"]);
        assert_eq!(keys(&b, KeyCommand::MoreDefensive), ["-"]);
        assert_eq!(keys(&b, KeyCommand::Eat), ["e"]);
        assert_eq!(keys(&b, KeyCommand::UseTool), ["u"]);
        assert_eq!(keys(&b, KeyCommand::Chat), ["C"]);
        assert_eq!(keys(&b, KeyCommand::Pay), ["p"]);
    }

    #[test]
    fn every_context_has_its_own_keys() {
        let b = Bindings::default();
        // Aiming's and the screens' back-style commands are rebindable, unbound by default
        // (their other keys are fixed, `settings::fixed`); `--more--` has keys of its own.
        assert!(
            keys(&b, KeyCommand::PrevTarget).is_empty(),
            "Shift+Tab does it"
        );
        assert!(
            keys(&b, KeyCommand::Back).is_empty(),
            "Esc always backs out"
        );
        assert_eq!(
            keys(&b, KeyCommand::MoreNext),
            ["Space", "Enter", "NumEnter"]
        );
        // Every command but Cancel belongs to one context, listed in `Context::ALL`.
        for (_, command) in &b.0 {
            assert!(Context::ALL.contains(&command.context()), "{command:?}");
        }
    }

    #[test]
    fn dev_keys_are_in_every_table() {
        let b = Bindings::default();
        assert_eq!(keys(&b, KeyCommand::Snapshot), ["F5"]);
        assert_eq!(keys(&b, KeyCommand::LoadSnapshot), ["F9"]);
        assert_eq!(keys(&b, KeyCommand::CycleVision), ["F3"]);
        assert_eq!(keys(&b, KeyCommand::RevealMap), ["F2"]);
        assert_eq!(keys(&b, KeyCommand::LightValues), ["F4"]);
        assert_eq!(keys(&b, KeyCommand::SkipHour), ["F6"]);
        assert_eq!(keys(&b, KeyCommand::NextWeather), ["F7"]);
        assert_eq!(keys(&b, KeyCommand::IdentifyAll), ["F8"]);
    }
}
