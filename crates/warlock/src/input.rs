//! Keyboard input: bindings, input modes, and the one router of key presses (spec §6).
//!
//! [`route_keys`] reads every key press exactly once. While a screen is open (the screen stack is
//! not empty) the press goes to the top screen, as a [`Press`] whose menu meaning comes from the
//! [`Context::Menu`] bindings (Esc always backs out); otherwise it is a gameplay command, read
//! through the bindings of the current [`InputMode`]'s contexts. Prompts in the message line (the
//! direction for open/close/kick, for digging, for a chat, or for an ability, how long to rest,
//! look mode, aiming a shot or a throw, `--more--` when a turn says more than the message line
//! holds) are input modes, not screens.
//!
//! **Every key the game reads is a [`KeyCommand`] in its [`Context`]** (#159), rebindable in
//! Controls and listed by the `?` help; nothing matches a key code or a character itself, but for
//! Esc (fixed: it always backs out and cancels, so no binding can lock the player out) and the
//! letters and digits that name the rows of a list.
//!
//! Letter and symbol commands bind by *logical* key (the character the keyboard layout produces),
//! so `<`, `>`, and `?` work on any layout. Arrows and the numpad bind by physical key code.

use crate::cli::Args;
use crate::game::{Game, PlayerAction, SimSet};
use crate::render::console::Console;
use crate::save::SaveCommand;
use crate::settings::Settings;
use crate::ui::{
    Aim, AimKind, DevOverlays, Dialog, LastTarget, MenuChoice, Press, ScreenAction, ScreenName,
    Screens, more, quit_dialog,
};
use bevy::ecs::system::SystemParam;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use warlock_core::ability::{self, Ability};
use warlock_core::content::VisionProfile;
use warlock_core::perception::{CellView, PerceptionView};
use warlock_core::time::Duration;
use warlock_core::weather::Weather;
use warlock_core::world::ActorKind;
use warlock_core::{Action, Dir, Pos, Positions, Rest};

/// What a key does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCommand {
    Move(Dir),
    Wait,
    /// Search the cells around the player for secret doors and hidden traps: a turn (spec §8
    /// "Searching").
    Search,
    /// Search turn after turn, up to `data/search.toml`'s `turns`, stopping when something is
    /// found or something interrupts, like a rest.
    SearchLong,
    Open,
    Close,
    /// Kick in a direction: a door (to break it in), or a creature.
    Kick,
    Descend,
    Ascend,
    Lantern,
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
    /// Show the messages so far, the newest at the bottom (no time passes).
    Messages,
    /// Ask how long to rest (an hour, until dawn, until dusk).
    Rest,
    /// In the rest prompt: rest an hour.
    RestHour,
    /// In the rest prompt: rest until dawn.
    RestDawn,
    /// In the rest prompt: rest until nightfall.
    RestDusk,
    /// At `--more--`: show the next of this turn's messages.
    MoreNext,
    /// At `--more--`: skip the rest of this turn's messages (Esc always does it; no default key
    /// of its own).
    MoreSkip,
    /// Choose something to eat, from the floor or the pack.
    Eat,
    /// Choose a scroll to read: a spell is aimed next (a cursor on the map), the scroll of
    /// identify asks what to read it on.
    Read,
    /// Move a cursor over the map to read what is there (no time passes).
    Look,
    /// Shoot or throw what is in the missile weapon slot (a cursor on the map): a launcher
    /// shoots its ammunition, anything else is thrown.
    Missile,
    /// Aim the readied missile weapon, then shoot (no default key; `Missile` does this too).
    Fire,
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
    /// Lower or raise the lantern's hood. An ability: unbound by default.
    HoodLantern,
    /// While aiming: the cursor to the next creature in view (with Shift, the one before).
    NextTarget,
    /// While aiming: the cursor to the creature in view before (no default key: Shift with
    /// `NextTarget` does it).
    PrevTarget,
    /// While aiming: shoot or throw at the cursor (Enter).
    Loose,
    /// Take a more aggressive combat stance (Coward, Defensive, Neutral, Aggressive; no time
    /// passes).
    MoreAggressive,
    /// Take a more defensive combat stance (the other way).
    MoreDefensive,
    /// Use the tool the player has ready toward a direction (ADOM's `U`): dig into a wall with a
    /// pick-axe. An item's use, not an ability (abilities get a menu of their own).
    UseTool,
    /// Chat with someone next to you: the one in view if there is only one, else in a
    /// direction.
    Chat,
    /// Pay for what the player took in the shop they stand in (spec §7 "Shops"; no time passes).
    Pay,
    /// On a menu or screen: move the selection up.
    MenuUp,
    /// On a menu or screen: move the selection down.
    MenuDown,
    /// On a menu or screen: change a value (a setting) one way.
    MenuLeft,
    /// On a menu or screen: change a value the other way.
    MenuRight,
    /// On a menu or screen: choose the selected row (Enter).
    Confirm,
    /// On a menu or screen: back out, as Esc always does (no default key of its own).
    Back,
    /// On a menu or screen: erase the last character typed (the seed), reset the selected
    /// command's keys (Controls), or delete the selected saved character (Characters).
    Erase,
    /// On the pick-up and drop lists: mark or unmark the selected stack.
    Mark,
    /// On the pick-up and drop lists: mark everything, or nothing if everything is marked.
    MarkAll,
    /// On an item list (the inventory, pick up, drop, ...): the next page of the list.
    NextListPage,
    /// On an item list: the previous page of the list.
    PrevListPage,
    /// In a text field (the character's name): accept what is typed (Enter).
    TextConfirm,
    /// In a text field: back out, as Esc always does (no default key of its own).
    TextBack,
    /// In a text field: erase the last character typed (Backspace, Delete).
    TextErase,
    /// In a text field: put a random value in (the name: one of the race's names; Tab).
    TextRandom,
    /// In the confirmation dialog (quitting; paying before leaving a shop, selling): yes.
    Yes,
    /// In the confirmation dialog: no.
    No,
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
    /// On an info screen: switch to page `n` (1 to 9) of the info screens.
    Page(u8),
    /// On an info screen: the next page (with Shift, the one before).
    NextPage,
    /// On an info screen: the previous page.
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
    /// In the rest prompt.
    Prompt,
    /// At `--more--`, when this turn's messages need more than one look at the message line.
    More,
    /// On every menu and screen: moving the selection, choosing, backing out.
    Menu,
    /// In a text field (the character's name, `ui/name_screen.rs`): accepting, erasing, a random
    /// value, backing out. Keys bound to nothing here type.
    Text,
    /// On an info screen ([`crate::ui`]'s numbered pages).
    Pages,
    /// The dialog's shortcuts (`ui/dialog_screen.rs`).
    Dialog,
    /// The developer keys (debug builds, or `--dev`), read in play after play's own.
    Dev,
}

impl Context {
    /// Every context, in the order the controls screen and the keymap file list them.
    pub const ALL: &[Context] = &[
        Context::Play,
        Context::Aim,
        Context::Prompt,
        Context::More,
        Context::Menu,
        Context::Text,
        Context::Pages,
        Context::Dialog,
        Context::Dev,
    ];

    /// The heading of its group on the controls screen.
    pub fn title(self) -> &'static str {
        match self {
            Context::Play => "In play",
            Context::Aim => "Aiming",
            Context::Prompt => "Rest prompt",
            Context::More => "More messages (--more--)",
            Context::Menu => "Menus and screens",
            Context::Text => "Text fields",
            Context::Pages => "Info pages",
            Context::Dialog => "Dialogs",
            Context::Dev => "Developer",
        }
    }

    /// Its table in the keymap file (`settings::keymap`).
    pub fn id(self) -> &'static str {
        match self {
            Context::Play => "play",
            Context::Aim => "aim",
            Context::Prompt => "prompt",
            Context::More => "more",
            Context::Menu => "menu",
            Context::Text => "text",
            Context::Pages => "pages",
            Context::Dialog => "dialog",
            Context::Dev => "dev",
        }
    }
}

impl KeyCommand {
    pub fn context(self) -> Context {
        use KeyCommand::*;
        match self {
            Page(_) | NextPage | PrevPage => Context::Pages,
            Yes | No => Context::Dialog,
            NextTarget | PrevTarget | Loose => Context::Aim,
            RestHour | RestDawn | RestDusk => Context::Prompt,
            MoreNext | MoreSkip => Context::More,
            MenuUp | MenuDown | MenuLeft | MenuRight | Confirm | Back | Erase | Mark | MarkAll
            | NextListPage | PrevListPage => Context::Menu,
            TextConfirm | TextBack | TextErase | TextRandom => Context::Text,
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
            KeyCommand::Wait => "Wait a turn",
            KeyCommand::Search => "Search",
            KeyCommand::SearchLong => "Search for a while",
            KeyCommand::Open => "Open a door",
            KeyCommand::Close => "Close a door",
            KeyCommand::Kick => "Kick",
            KeyCommand::Descend => "Go down stairs",
            KeyCommand::Ascend => "Go up stairs",
            KeyCommand::Lantern => "Light / douse lantern",
            KeyCommand::PickUp => "Pick up",
            KeyCommand::Drop => "Drop",
            KeyCommand::Inventory => "Inventory",
            KeyCommand::Equipment => "Equipment",
            KeyCommand::Character => "Character",
            KeyCommand::Skills => "Skills",
            KeyCommand::WeaponSkills => "Weapon skills",
            KeyCommand::Quests => "Quest log",
            KeyCommand::Messages => "Message history",
            KeyCommand::Rest => "Rest",
            KeyCommand::RestHour => "Rest an hour",
            KeyCommand::RestDawn => "Rest until dawn",
            KeyCommand::RestDusk => "Rest until nightfall",
            KeyCommand::MoreNext => "Show the next messages",
            KeyCommand::MoreSkip => "Skip the rest (Esc always works)",
            KeyCommand::Eat => "Eat",
            KeyCommand::Read => "Read a scroll",
            KeyCommand::Look => "Look around",
            KeyCommand::Missile => "Throw or shoot the missile weapon",
            KeyCommand::Fire => "Fire missile weapon",
            KeyCommand::Throw => "Throw from the pack",
            KeyCommand::MoreAggressive => "More aggressive stance",
            KeyCommand::MoreDefensive => "More defensive stance",
            KeyCommand::Abilities => "Abilities",
            KeyCommand::RepeatAbility => "Use the last ability again",
            KeyCommand::Bash => "Bash with your shield",
            KeyCommand::ThrowVoice => "Throw your voice",
            KeyCommand::Disarm => "Disarm a trap",
            KeyCommand::PickLock => "Pick a lock",
            KeyCommand::HoodLantern => "Hood your lantern",
            KeyCommand::NextTarget => "Next target (Shift: previous)",
            KeyCommand::PrevTarget => "Previous target",
            KeyCommand::Loose => "Shoot or throw",
            KeyCommand::UseTool => "Use a tool (dig)",
            KeyCommand::Chat => "Chat",
            KeyCommand::Pay => "Pay the shopkeeper",
            KeyCommand::MenuUp => "Up",
            KeyCommand::MenuDown => "Down",
            KeyCommand::MenuLeft => "Left (change a setting)",
            KeyCommand::MenuRight => "Right (change a setting)",
            KeyCommand::Confirm => "Choose",
            KeyCommand::Back => "Back (Esc always works)",
            KeyCommand::Erase => "Erase, reset keys, delete a save",
            KeyCommand::Mark => "Mark a stack (pick up, drop)",
            KeyCommand::MarkAll => "Mark everything (pick up, drop)",
            KeyCommand::NextListPage => "Next page of a list",
            KeyCommand::PrevListPage => "Previous page of a list",
            KeyCommand::TextConfirm => "Accept the text",
            KeyCommand::TextBack => "Back (Esc always works)",
            KeyCommand::TextErase => "Erase the last character",
            KeyCommand::TextRandom => "A random name",
            KeyCommand::Yes => "Yes (in a dialog)",
            KeyCommand::No => "No (in a dialog)",
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
            KeyCommand::Page(_) => "Info page",
            KeyCommand::NextPage => "Next info page (Shift: previous)",
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
}

impl KeyBinding {
    /// Human-readable key name for the controls screen and help.
    pub fn display(&self) -> String {
        match self {
            KeyBinding::Char(s) => s.clone(),
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

    fn matches(&self, input: &KeyboardInput) -> bool {
        match self {
            KeyBinding::Code(code) => input.key_code == *code,
            KeyBinding::Char(s) => {
                matches!(&input.logical_key, Key::Character(c) if c.as_str() == s)
            }
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
        let mut table = Self(vec![
            (ch("k"), Move(Dir::N)),
            (ch("u"), Move(Dir::NE)),
            (ch("l"), Move(Dir::E)),
            (ch("n"), Move(Dir::SE)),
            (ch("j"), Move(Dir::S)),
            (ch("b"), Move(Dir::SW)),
            (ch("h"), Move(Dir::W)),
            (ch("y"), Move(Dir::NW)),
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
            (ch("."), Wait),
            (Code(KeyCode::Numpad5), Wait),
            (ch("s"), Search),
            (ch("z"), SearchLong),
            (ch("o"), Open),
            (ch("c"), Close),
            (ch("K"), Kick),
            (ch(">"), Descend),
            (ch("<"), Ascend),
            (ch("L"), Lantern),
            (ch(","), PickUp),
            (ch("g"), PickUp),
            (ch("d"), Drop),
            (ch("i"), Inventory),
            (ch("e"), Equipment),
            (ch("@"), Character),
            (ch("S"), Skills),
            (ch("W"), WeaponSkills),
            (ch("q"), Quests),
            (ch("M"), Messages),
            (ch("R"), Rest),
            (ch("E"), Eat),
            (ch("r"), Read),
            (ch("x"), Look),
            (ch("t"), Missile),
            (ch("T"), Throw),
            (ch("+"), MoreAggressive),
            (ch("-"), MoreDefensive),
            (ch("a"), Abilities),
            (ch("A"), RepeatAbility),
            (ch("C"), Chat),
            (Code(KeyCode::Tab), NextTarget),
            (Code(KeyCode::Enter), Loose),
            (Code(KeyCode::NumpadEnter), Loose),
            (ch("h"), RestHour),
            (ch("d"), RestDawn),
            (ch("n"), RestDusk),
            (Code(KeyCode::Space), MoreNext),
            (Code(KeyCode::Enter), MoreNext),
            (Code(KeyCode::NumpadEnter), MoreNext),
            (ch("U"), UseTool),
            (ch("p"), Pay),
            (Code(KeyCode::ArrowUp), MenuUp),
            (ch("k"), MenuUp),
            (Code(KeyCode::ArrowDown), MenuDown),
            (ch("j"), MenuDown),
            (Code(KeyCode::ArrowLeft), MenuLeft),
            (ch("h"), MenuLeft),
            (Code(KeyCode::ArrowRight), MenuRight),
            (ch("l"), MenuRight),
            (Code(KeyCode::Enter), Confirm),
            (Code(KeyCode::NumpadEnter), Confirm),
            (Code(KeyCode::Delete), Erase),
            (Code(KeyCode::Backspace), Erase),
            (Code(KeyCode::Space), Mark),
            (ch(","), MarkAll),
            (ch(">"), NextListPage),
            (Code(KeyCode::PageDown), NextListPage),
            (ch("<"), PrevListPage),
            (Code(KeyCode::PageUp), PrevListPage),
            (Code(KeyCode::Enter), TextConfirm),
            (Code(KeyCode::NumpadEnter), TextConfirm),
            (Code(KeyCode::Backspace), TextErase),
            (Code(KeyCode::Delete), TextErase),
            (Code(KeyCode::Tab), TextRandom),
            (ch("y"), Yes),
            (ch("n"), No),
            (ch("?"), Help),
            (Code(KeyCode::Escape), Cancel),
        ]);
        for n in 1..=9u8 {
            table.0.push((ch(&n.to_string()), Page(n)));
        }
        table.0.push((Code(KeyCode::Tab), NextPage));
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

/// Whether the developer keys work in this launch: always in debug builds, with `--dev` in
/// release builds (spec §9). Their bindings are in every table; only where this is false are
/// they never read, nor listed by the controls screen and the `?` help. It also gives New run
/// its seed field (`NewRunScreen`): elsewhere a new run's seed is no one's choice (#256).
pub fn dev_keys(args: &Args) -> bool {
    cfg!(debug_assertions) || args.dev
}

impl Bindings {
    /// The command a key press stands for in play.
    pub fn lookup(&self, input: &KeyboardInput) -> Option<KeyCommand> {
        self.lookup_in(Context::Play, input)
    }

    /// The command a key press stands for in `context`.
    pub fn lookup_in(&self, context: Context, input: &KeyboardInput) -> Option<KeyCommand> {
        // Physical bindings first: numpad keys also produce digit characters.
        self.0
            .iter()
            .filter(|(_, c)| c.context() == context)
            .filter(|(k, _)| matches!(k, KeyBinding::Code(_)))
            .chain(
                self.0
                    .iter()
                    .filter(|(_, c)| c.context() == context)
                    .filter(|(k, _)| matches!(k, KeyBinding::Char(_))),
            )
            .find(|(k, _)| k.matches(input))
            .map(|(_, c)| *c)
    }
}

/// What keys currently mean.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    /// Waiting for a direction for `Open`, `Close`, `Kick`, `UseTool`, or `Chat`.
    Direction(KeyCommand),
    /// Waiting for a direction to use an ability in (one whose [`ability::Aim`] is a direction:
    /// a shield bash).
    Ability(Ability),
    /// Asking how long to rest ([`REST_CHOICES`]).
    Rest,
    /// Look mode: a cursor at this level position.
    Look(Pos),
    /// Aiming a shot or a throw (`ui::target`).
    Aim(Aim),
    /// `--more--`: the message line shows page `n` of this turn's messages, and waits for
    /// [`KeyCommand::MoreNext`] (or [`KeyCommand::MoreSkip`], or Esc) before anything else
    /// (`ui::more`).
    More(usize),
}

/// The dialog a step in `dir` opens first, if it leaves the shop the player stands in owing
/// anything (spec §7 "Shops"). Out of its door: pay and leave in one action (offered only with
/// the gold; else the dialog says what they owe and have), or not, and the keeper turns them
/// back one step (declining and Esc send the step, which the rules turn back). Any other way
/// out (a wall dug open): pay and leave, leave without paying (theft), or stay.
fn leaving_dialog(view: &PerceptionView<'_>, dir: Dir) -> Option<Dialog> {
    let to = view.viewer().step(dir);
    let (shop, owed) = view.owed_on_leaving(to)?;
    let (keeper, gold) = (shop.keeper(), view.own_gold());
    let pay = format!("Pay {owed} gold and leave");
    let dialog = if shop.is_exit(to) {
        if gold >= owed {
            Dialog::new(shop.sign(), format!("You owe {keeper} {owed} gold."))
                .accept(pay, Action::Pay(Some(dir)))
                .decline("Don't pay", Action::Move(dir))
        } else {
            let have = match gold {
                0 => "no gold".to_string(),
                n => format!("only {n}"),
            };
            let text = format!(
                "You owe {keeper} {owed} gold and have {have}. Put something back, or bring \
                 more gold."
            );
            Dialog::new(shop.sign(), text).decline("Go back", Action::Move(dir))
        }
    } else {
        let text = format!(
            "You owe {keeper} {owed} gold. Leave this way without paying, and {keeper} will \
             call it theft."
        );
        let dialog = Dialog::new(shop.sign(), text);
        let dialog = match gold >= owed {
            true => dialog.accept(pay, Action::Pay(Some(dir))),
            false => dialog,
        };
        dialog
            .plain("Leave without paying", Action::Move(dir))
            .cancel("Stay", ScreenAction::Close)
    };
    Some(dialog)
}

/// The dialog dropping the inventory stacks `letters` opens first, if the shop the player stands
/// in would buy some of them (spec §7 "Shops"): sell them, or keep them (nothing is dropped).
fn sale_dialog(view: &PerceptionView<'_>, letters: &Positions) -> Option<Dialog> {
    let sale = view.sale(letters)?;
    let shop = view.shop_here()?;
    let what = warlock_core::object::phrase(&sale.stacks, view.content());
    let text = format!("{} offers {} gold for {what}.", shop.keeper(), sale.offer);
    let sell = format!("Sell for {} gold", sale.offer);
    Some(
        Dialog::new(shop.sign(), text)
            .accept(sell, Action::Sell(letters.clone()))
            .cancel("Keep it", ScreenAction::Close),
    )
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // The settings file's bindings, if `main` loaded them.
        let bindings = app
            .world()
            .get_resource::<Bindings>()
            .cloned()
            .unwrap_or_default();
        app.insert_resource(bindings)
            .init_resource::<InputMode>()
            .init_resource::<LastTarget>()
            .add_systems(Startup, (aim_at_startup, press_keys_at_startup))
            .add_systems(Update, route_keys.before(SimSet));
    }
}

/// The one reader of key presses, so each press has exactly one consumer. While a screen is
/// open, every press goes to the top screen and gameplay sees none; otherwise presses are
/// gameplay commands. Screens open and close at once, so the next press in the same frame
/// already goes to the new top. A menu choice (starting or leaving a run) is applied after this
/// system, so the rest of that frame's presses are dropped.
pub fn route_keys(
    mut keys: MessageReader<KeyboardInput>,
    mut screens: Screens,
    mut settings: ResMut<Settings>,
    mut bindings: ResMut<Bindings>,
    mut play: Gameplay,
    mut choices: MessageWriter<MenuChoice>,
    console: Option<Res<Console>>,
) {
    // Item lists page by the console's size: the screens learn it before any press.
    let size = console.map(|c| (c.cols(), c.rows()));
    if let Some(size) = size
        && !keys.is_empty()
        && screens.stack.console() != Some(size)
    {
        screens.stack.set_console(size);
    }
    let mut chosen = false;
    for key in keys.read() {
        if chosen || key.state != ButtonState::Pressed {
            continue;
        }
        let action = if screens.stack.is_empty() {
            let last = screens.stack.last_ability();
            play.press(key, &bindings, last)
                .unwrap_or(ScreenAction::Stay)
        } else {
            // Screens edit copies, so the resources only change (and are saved) when a press
            // changed them.
            let (mut new_settings, mut new_bindings) = (settings.clone(), bindings.clone());
            let action = screens.stack.handle(
                &Press {
                    shift: play.shift(),
                    ..Press::from_input(key, &bindings)
                },
                &mut new_settings,
                &mut new_bindings,
            );
            if new_settings != *settings {
                *settings = new_settings;
            }
            if new_bindings.0 != bindings.0 {
                *bindings = new_bindings;
            }
            action
        };
        match action {
            ScreenAction::Stay => {}
            ScreenAction::Close => screens.stack.close(),
            ScreenAction::Act(action) => {
                screens.stack.close();
                if let Some(dialog) = play.act(action) {
                    screens.ask(dialog);
                }
            }
            ScreenAction::Aim(kind) => {
                screens.stack.close();
                play.aim(kind);
            }
            ScreenAction::UseAbility(ability) => {
                // From the abilities menu, or from play (its key, the quick repeat).
                screens.stack.close();
                screens.stack.remember_ability(ability);
                play.use_ability(ability);
            }
            ScreenAction::Open(name) => screens.open(name),
            ScreenAction::Replace(name) => screens.replace(name),
            ScreenAction::Return(name) => screens.stack.return_to(name),
            ScreenAction::Dialog(dialog) => screens.ask(*dialog),
            ScreenAction::Choose(choice) => {
                choices.write(choice);
                chosen = true;
            }
        }
    }
}

/// What gameplay input reads and writes.
#[derive(SystemParam)]
pub struct Gameplay<'w> {
    args: Res<'w, Args>,
    game: Option<Res<'w, Game>>,
    mode: ResMut<'w, InputMode>,
    last: ResMut<'w, LastTarget>,
    held: Res<'w, ButtonInput<KeyCode>>,
    overlays: ResMut<'w, DevOverlays>,
    actions: MessageWriter<'w, PlayerAction>,
    saves: MessageWriter<'w, SaveCommand>,
    /// The UI console, whose width lays out the message line's pages (`--more--`).
    console: Option<Res<'w, Console>>,
}

impl Gameplay<'_> {
    /// Whether Shift is held.
    fn shift(&self) -> bool {
        self.held.pressed(KeyCode::ShiftLeft) || self.held.pressed(KeyCode::ShiftRight)
    }

    /// Applies one key press in play, `last` the ability used last (for the quick repeat).
    /// Returns what it asks of the screens: one to open (`?` help, the pause menu on Esc when no
    /// prompt is open), a dialog, or an ability to use.
    fn press(
        &mut self,
        key: &KeyboardInput,
        bindings: &Bindings,
        last: Option<Ability>,
    ) -> Option<ScreenAction> {
        let game = self.game.as_ref()?;
        let command = command_in(*self.mode, key, bindings, dev_keys(&self.args))?;
        // Held keys repeat only for movement, waiting, and searching, and the aiming cursor's
        // moves.
        let moves = matches!(
            command,
            KeyCommand::Move(_) | KeyCommand::Wait | KeyCommand::Search
        );
        if key.repeat && !moves {
            return None;
        }
        let mut open: Option<ScreenAction> = None;
        let next = match (*self.mode, command) {
            // `--more--`: the next page of this turn's messages, then play once the last shows;
            // skipping goes to play at once (the last page shows). Nothing else acts meanwhile.
            (InputMode::More(page), KeyCommand::MoreNext) => {
                let cols = self.console.as_ref().map_or(0, |c| c.cols());
                more::advance(page, more::page_count(game, cols, bindings))
            }
            (InputMode::More(_), KeyCommand::MoreSkip | KeyCommand::Cancel) => InputMode::Normal,
            (InputMode::More(page), _) => InputMode::More(page),
            (
                InputMode::Rest,
                KeyCommand::RestHour | KeyCommand::RestDawn | KeyCommand::RestDusk,
            ) => {
                if let Some((_, rest, _)) = REST_CHOICES.iter().find(|(c, ..)| *c == command) {
                    self.actions.write(PlayerAction(Action::Rest(*rest)));
                }
                InputMode::Normal
            }
            (InputMode::Aim(aim), KeyCommand::Loose) => return self.release(aim),
            (InputMode::Aim(aim), KeyCommand::Move(dir)) => {
                InputMode::Aim(aim.step(dir, &game.world.perception()))
            }
            (InputMode::Aim(aim), KeyCommand::NextTarget | KeyCommand::PrevTarget) => {
                let forward = (command == KeyCommand::NextTarget) != self.shift();
                InputMode::Aim(aim.cycle(&game.world.perception(), forward))
            }
            (
                InputMode::Aim(aim),
                KeyCommand::Missile | KeyCommand::Fire | KeyCommand::Throw | KeyCommand::Read,
            ) => {
                // The key that started the aim confirms it, and so does `t`, which starts a shot
                // and a throw.
                let (confirm, missile) = match aim.kind {
                    AimKind::Fire => (KeyCommand::Fire, true),
                    AimKind::Throw(_) => (KeyCommand::Throw, true),
                    AimKind::Read(_) => (KeyCommand::Read, false),
                    // None of these uses an ability: its own key does (below), and Enter.
                    AimKind::Ability(_) => (KeyCommand::Loose, false),
                };
                if command == confirm || (missile && command == KeyCommand::Missile) {
                    return self.release(aim);
                }
                InputMode::Aim(aim)
            }
            (InputMode::Aim(aim), KeyCommand::ThrowVoice)
                if aim.kind == AimKind::Ability(Ability::ThrowVoice) =>
            {
                return self.release(aim);
            }
            (InputMode::Aim(_), KeyCommand::Cancel | KeyCommand::Look) => InputMode::Normal,
            (InputMode::Aim(aim), _) => InputMode::Aim(aim),
            (InputMode::Normal, KeyCommand::Fire) => {
                let view = game.world.perception();
                if view.ready_to_fire().is_some() {
                    InputMode::Aim(Aim::start(AimKind::Fire, &view, &self.last))
                } else {
                    // Nothing to shoot: the rules say why.
                    let here = view.viewer();
                    self.actions.write(PlayerAction(Action::Fire(here)));
                    InputMode::Normal
                }
            }
            (InputMode::Normal, KeyCommand::Missile) => {
                let view = game.world.perception();
                if view.ready_to_fire().is_some() {
                    InputMode::Aim(Aim::start(AimKind::Fire, &view, &self.last))
                } else if let Some(item) = view.ready_to_throw() {
                    InputMode::Aim(Aim::start(AimKind::Throw(item), &view, &self.last))
                } else {
                    // Nothing ready (or no ammunition): the rules say so, and take no time.
                    let here = view.viewer();
                    self.actions.write(PlayerAction(Action::Fire(here)));
                    InputMode::Normal
                }
            }
            (InputMode::Normal, KeyCommand::Throw) => {
                open = Some(ScreenAction::Open(ScreenName::Throw));
                InputMode::Normal
            }
            (InputMode::Look(_), KeyCommand::Look | KeyCommand::Cancel) => InputMode::Normal,
            (InputMode::Look(at), KeyCommand::Move(dir)) => {
                let view = game.world.perception();
                let to = at.step(dir);
                InputMode::Look(Pos::new(
                    to.x.clamp(0, view.width() - 1),
                    to.y.clamp(0, view.height() - 1),
                ))
            }
            (InputMode::Look(at), _) => InputMode::Look(at),
            (InputMode::Normal, KeyCommand::Look) => InputMode::Look(game.world.player_pos()),
            (InputMode::Ability(ability), KeyCommand::Move(dir)) => {
                if let Some(action) = ability.action(Some(dir)) {
                    self.actions.write(PlayerAction(action));
                }
                InputMode::Normal
            }
            (InputMode::Ability(_), _) => InputMode::Normal,
            (InputMode::Direction(cmd), KeyCommand::Move(dir)) => {
                let action = match cmd {
                    KeyCommand::Open => Action::Open(dir),
                    KeyCommand::Kick => Action::Kick(dir),
                    KeyCommand::UseTool => Action::Dig(dir),
                    KeyCommand::Chat => Action::Chat(dir),
                    _ => Action::Close(dir),
                };
                self.actions.write(PlayerAction(action));
                InputMode::Normal
            }
            (InputMode::Direction(_), _) => InputMode::Normal,
            // Any other command (Esc, or `R` again) cancels the rest prompt.
            (InputMode::Rest, _) => InputMode::Normal,
            (InputMode::Normal, KeyCommand::Rest) => InputMode::Rest,
            (InputMode::Normal, KeyCommand::Move(dir)) => {
                // Leaving a shop owing anything (spec §7 "Shops") or stepping onto a known trap
                // (spec §8 "Traps") asks first, in the dialog. A held key stops there (repeats
                // never open the dialog).
                let view = game.world.perception();
                match leaving_dialog(&view, dir).or_else(|| trap_dialog(&view, dir)) {
                    Some(dialog) if !key.repeat => open = Some(dialog.into()),
                    Some(_) => {}
                    None => {
                        self.actions.write(PlayerAction(Action::Move(dir)));
                    }
                }
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Pay) => {
                self.actions.write(PlayerAction(Action::Pay(None)));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Wait) => {
                self.actions.write(PlayerAction(Action::Wait));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Search) => {
                self.actions.write(PlayerAction(Action::Search(1)));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::SearchLong) => {
                let turns = game.world.content().search().turns;
                self.actions.write(PlayerAction(Action::Search(turns)));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Descend) => {
                self.actions.write(PlayerAction(Action::Descend));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Ascend) => {
                self.actions.write(PlayerAction(Action::Ascend));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Lantern) => {
                self.actions.write(PlayerAction(Action::ToggleLantern));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::PickUp) => {
                // One stack is picked up at once; several are chosen from on a screen. With
                // nothing here, the core says so.
                let view = game.world.perception();
                let letters = match view.objects(view.viewer()).len() {
                    0 => Some(Positions::default()),
                    1 => Some(Positions::of([0])),
                    _ => None,
                };
                match letters {
                    Some(letters) => {
                        self.actions.write(PlayerAction(Action::PickUp(letters)));
                    }
                    None => open = Some(ScreenAction::Open(ScreenName::PickUp)),
                }
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Drop) => {
                if game.world.perception().inventory().is_empty() {
                    self.actions
                        .write(PlayerAction(Action::Drop(Positions::default())));
                } else {
                    open = Some(ScreenAction::Open(ScreenName::Drop));
                }
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Inventory) => {
                open = Some(ScreenAction::Open(ScreenName::Inventory));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Equipment) => {
                open = Some(ScreenAction::Open(ScreenName::Equipment));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Character) => {
                open = Some(ScreenAction::Open(ScreenName::Character));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Skills) => {
                open = Some(ScreenAction::Open(ScreenName::Skills));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::MoreAggressive | KeyCommand::MoreDefensive) => {
                let now = game.world.perception().own_stance();
                let next = if command == KeyCommand::MoreAggressive {
                    now.more_aggressive()
                } else {
                    now.more_defensive()
                };
                self.actions.write(PlayerAction(Action::SetStance(next)));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::WeaponSkills) => {
                open = Some(ScreenAction::Open(ScreenName::WeaponSkills));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Eat) => {
                open = Some(ScreenAction::Open(ScreenName::Eat));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Read) => {
                open = Some(ScreenAction::Open(ScreenName::Read));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Quests) => {
                open = Some(ScreenAction::Open(ScreenName::Quests));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Messages) => {
                open = Some(ScreenAction::Open(ScreenName::Messages));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::CycleVision) => {
                let view = game.world.perception();
                let next = next_vision(view.content().vision_profiles(), &view.vision().id);
                self.actions.write(PlayerAction(Action::SetVision(next)));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::SkipHour) => {
                self.actions
                    .write(PlayerAction(Action::SkipTime(Duration::hours(1))));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::NextWeather) => {
                let now = game.world.perception().weather();
                self.actions
                    .write(PlayerAction(Action::SetWeather(next_weather(now))));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::IdentifyAll) => {
                self.actions
                    .write(PlayerAction(Action::Identify(Positions::default())));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::RevealMap) => {
                self.overlays.reveal = !self.overlays.reveal;
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::LightValues) => {
                self.overlays.light_values = !self.overlays.light_values;
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Open | KeyCommand::Close | KeyCommand::Kick) => {
                InputMode::Direction(command)
            }
            (InputMode::Normal, KeyCommand::Abilities) => {
                open = Some(ScreenAction::Open(ScreenName::Abilities));
                InputMode::Normal
            }
            // The ability used last, again; with none yet, the menu to choose one.
            (InputMode::Normal, KeyCommand::RepeatAbility) => {
                open = Some(last.map_or(
                    ScreenAction::Open(ScreenName::Abilities),
                    ScreenAction::UseAbility,
                ));
                InputMode::Normal
            }
            // An ability's own key (unbound by default) uses it as the menu does.
            (InputMode::Normal, KeyCommand::Bash) => {
                open = Some(ScreenAction::UseAbility(Ability::ShieldBash));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::ThrowVoice) => {
                open = Some(ScreenAction::UseAbility(Ability::ThrowVoice));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Disarm) => {
                open = Some(ScreenAction::UseAbility(Ability::Disarm));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::PickLock) => {
                open = Some(ScreenAction::UseAbility(Ability::PickLock));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::HoodLantern) => {
                open = Some(ScreenAction::UseAbility(Ability::HoodLantern));
                InputMode::Normal
            }
            // The one creature in view next to the player, at once; else ask which way.
            (InputMode::Normal, KeyCommand::Chat) => match chat_partner(&game.world.perception()) {
                Some(dir) => {
                    self.actions.write(PlayerAction(Action::Chat(dir)));
                    InputMode::Normal
                }
                None => InputMode::Direction(command),
            },
            (InputMode::Normal, KeyCommand::UseTool) => {
                if game.world.perception().ready_to_dig().is_some() {
                    InputMode::Direction(command)
                } else {
                    // Nothing to dig with: the rules say so ("You have no digging tool ready.").
                    self.actions.write(PlayerAction(Action::Dig(Dir::N)));
                    InputMode::Normal
                }
            }
            (InputMode::Normal, KeyCommand::Help) => {
                open = Some(ScreenAction::Open(ScreenName::Help));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Cancel) => {
                open = Some(ScreenAction::Open(ScreenName::Pause));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Quit) => {
                open = Some(quit_dialog(true).into());
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Snapshot) => {
                self.saves.write(SaveCommand::Snapshot);
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::LoadSnapshot) => {
                self.saves.write(SaveCommand::LoadSnapshot);
                InputMode::Normal
            }
            // Other contexts' commands (aiming's, the rest prompt's, the screens'): `command_in`
            // never finds them here.
            (
                InputMode::Normal,
                KeyCommand::NextTarget
                | KeyCommand::PrevTarget
                | KeyCommand::Loose
                | KeyCommand::RestHour
                | KeyCommand::RestDawn
                | KeyCommand::RestDusk
                | KeyCommand::MoreNext
                | KeyCommand::MoreSkip
                | KeyCommand::MenuUp
                | KeyCommand::MenuDown
                | KeyCommand::MenuLeft
                | KeyCommand::MenuRight
                | KeyCommand::Confirm
                | KeyCommand::Back
                | KeyCommand::Erase
                | KeyCommand::Mark
                | KeyCommand::MarkAll
                | KeyCommand::NextListPage
                | KeyCommand::PrevListPage
                | KeyCommand::TextConfirm
                | KeyCommand::TextBack
                | KeyCommand::TextErase
                | KeyCommand::TextRandom
                | KeyCommand::Page(_)
                | KeyCommand::NextPage
                | KeyCommand::PrevPage
                | KeyCommand::Yes
                | KeyCommand::No,
            ) => InputMode::Normal,
        };
        if *self.mode != next {
            *self.mode = next;
        }
        open
    }

    /// Asks the simulation to apply `action`, chosen on a screen. Dropping in a shop what it
    /// would buy asks first whether to sell (spec §7 "Shops"): the dialog is returned instead.
    fn act(&mut self, action: Action) -> Option<Dialog> {
        if let (Action::Drop(letters), Some(game)) = (&action, &self.game)
            && let Some(dialog) = sale_dialog(&game.world.perception(), letters)
        {
            return Some(dialog);
        }
        self.actions.write(PlayerAction(action));
        None
    }

    /// Uses `ability`: asks for what it needs (a direction in the message line, a cell in the
    /// aiming mode), then applies its action. One the player cannot use now is applied at once
    /// all the same, so the rules say why, taking no time (a shield bash without a shield).
    fn use_ability(&mut self, ability: Ability) {
        let Some(game) = &self.game else { return };
        let usable = (game.world.perception().abilities().iter())
            .any(|own| own.ability == ability && own.unavailable.is_none());
        match (usable, ability.aim()) {
            (true, ability::Aim::Direction) => *self.mode = InputMode::Ability(ability),
            (true, ability::Aim::Cell) => self.aim(AimKind::Ability(ability)),
            (true, ability::Aim::Nothing) => {
                if let Some(action) = ability.action(None) {
                    self.actions.write(PlayerAction(action));
                }
            }
            (false, _) => {
                let here = game.world.player_pos();
                let action = (ability.action(Some(Dir::N))).or_else(|| ability.action_at(here));
                if let Some(action) = action {
                    self.actions.write(PlayerAction(action));
                }
            }
        }
    }

    /// Starts aiming `kind` (a throw or a scroll chosen on a screen).
    fn aim(&mut self, kind: AimKind) {
        if let Some(game) = &self.game {
            let aim = Aim::start(kind, &game.world.perception(), &self.last);
            *self.mode = InputMode::Aim(aim);
        }
    }

    /// Looses what `aim` aims at its cursor; a burst that may catch the player (a scroll of
    /// fireball read too close) asks first, in the dialog, which is returned to open (spec §6
    /// "Targeting").
    fn release(&mut self, aim: Aim) -> Option<ScreenAction> {
        let dialog = (self.game.as_ref()).and_then(|g| burst_dialog(&g.world.perception(), aim));
        let Some(dialog) = dialog else {
            self.loose(aim);
            return None;
        };
        if let Some(game) = &self.game {
            self.last.remember(&game.world.perception(), aim);
        }
        *self.mode = InputMode::Normal;
        Some(dialog.into())
    }

    /// Looses the missile `aim` aims, remembering its target for the next shot.
    fn loose(&mut self, aim: Aim) {
        if let Some(game) = &self.game {
            self.last.remember(&game.world.perception(), aim);
        }
        if let Some(action) = aim.action() {
            self.actions.write(PlayerAction(action));
        }
        *self.mode = InputMode::Normal;
    }
}

/// The way to the one creature the player sees next to them, if there is exactly one (whom `C`
/// talks to without asking).
fn chat_partner(view: &PerceptionView<'_>) -> Option<Dir> {
    let me = view.viewer();
    let mut near = (view.actors().iter())
        .filter(|a| a.kind == ActorKind::Creature && a.pos.chebyshev(me) == 1)
        .map(|a| Dir::from_delta(a.pos.x - me.x, a.pos.y - me.y));
    match (near.next(), near.next()) {
        (Some(dir), None) => dir,
        _ => None,
    }
}

/// Dev (`--fire`): starts the run aiming the readied missile weapon, as `f` would, so captures
/// show the targeting line; (`--look X,Y`) in look mode with the cursor there.
fn aim_at_startup(args: Res<Args>, game: Option<Res<Game>>, mut mode: ResMut<InputMode>) {
    if let (true, Some(game)) = (args.fire, &game) {
        let view = game.world.perception();
        *mode = InputMode::Aim(Aim::start(AimKind::Fire, &view, &LastTarget::default()));
    }
    if let (Some(at), Some(_)) = (args.look, &game) {
        *mode = InputMode::Look(at);
    }
}

/// Dev (`--keys`): presses the keys as if typed, once the run (and the screens) have started, so
/// captures show what they open.
fn press_keys_at_startup(args: Res<Args>, mut presses: MessageWriter<KeyboardInput>) {
    let Some(keys) = &args.keys else {
        return;
    };
    for key in &keys.0 {
        let (key_code, logical_key) = match key.as_str() {
            "Enter" => (KeyCode::Enter, Key::Enter),
            "Esc" => (KeyCode::Escape, Key::Escape),
            "Tab" => (KeyCode::Tab, Key::Tab),
            "Up" => (KeyCode::ArrowUp, Key::ArrowUp),
            "Down" => (KeyCode::ArrowDown, Key::ArrowDown),
            "Left" => (KeyCode::ArrowLeft, Key::ArrowLeft),
            "Right" => (KeyCode::ArrowRight, Key::ArrowRight),
            ch => (
                KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified),
                Key::Character(ch.into()),
            ),
        };
        // A character key types itself, as a keyboard's would (text fields read the text).
        let text = match &logical_key {
            Key::Character(ch) => Some(ch.clone()),
            _ => None,
        };
        presses.write(KeyboardInput {
            key_code,
            logical_key,
            state: ButtonState::Pressed,
            text,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
    }
}

/// The rest prompt's answers: the command (`Context::Prompt`), what it rests for, and how the
/// prompt words it.
pub const REST_CHOICES: [(KeyCommand, Rest, &str); 3] = [
    (KeyCommand::RestHour, Rest::For(Duration::hours(1)), "hour"),
    (KeyCommand::RestDawn, Rest::Dawn, "until dawn"),
    (KeyCommand::RestDusk, Rest::Dusk, "until nightfall"),
];

/// The command a key press stands for in input mode `mode`: the rest prompt's answers,
/// `--more--`'s keys, and aiming's keys before play's, which the other modes read alone; the
/// developer keys last, in plain play, where they are on (`dev`).
fn command_in(
    mode: InputMode,
    key: &KeyboardInput,
    bindings: &Bindings,
    dev: bool,
) -> Option<KeyCommand> {
    let own = match mode {
        InputMode::Rest => Some(Context::Prompt),
        InputMode::Aim(_) => Some(Context::Aim),
        InputMode::More(_) => Some(Context::More),
        InputMode::Normal
        | InputMode::Direction(_)
        | InputMode::Ability(_)
        | InputMode::Look(_) => None,
    };
    (own.and_then(|context| bindings.lookup_in(context, key)))
        .or_else(|| bindings.lookup(key))
        .or_else(|| {
            let dev = dev && mode == InputMode::Normal;
            dev.then(|| bindings.lookup_in(Context::Dev, key)).flatten()
        })
}

/// The question before reading a scroll whose burst may catch the player too, aimed by `aim`
/// (as far as they know, [`Aim::catches_reader`]): read it anyway, aim again (back at the same
/// cursor, the target remembered), or not read it.
fn burst_dialog(view: &PerceptionView<'_>, aim: Aim) -> Option<Dialog> {
    let AimKind::Read(item) = aim.kind else {
        return None;
    };
    let noun = aim.catches_reader(view)?;
    let read = aim.action()?;
    let stack = view.inventory().get(item)?;
    let name = &view.content().object(stack.object).name;
    Some(
        Dialog::new(
            "Too close",
            format!("The {noun} may catch you too. Read the {name} anyway?"),
        )
        .accept("Read it", read)
        .decline("Aim again", ScreenAction::Aim(aim.kind))
        .cancel("Don't read it", ScreenAction::Close),
    )
}

/// The dialog a step in `dir` opens first, if it goes onto a trap the player knows, where no one
/// they see stands (moving there would be an attack, or stop; spec §8 "Traps"): "Really step
/// onto the fireball trap?", to step onto it or stay back (the safe answer, selected first). A
/// door's trap is not stepped on (spec §8 "Trapped doors"), so a door never asks.
fn trap_dialog(view: &PerceptionView<'_>, dir: Dir) -> Option<Dialog> {
    let to = view.viewer().step(dir);
    let door = match view.cell(to) {
        CellView::Visible(t) | CellView::Remembered(t) => view.content().terrain(t).is_door(),
        CellView::Unknown => false,
    };
    let trap = view
        .trap(to)
        .filter(|_| !door && view.actor_at(to).is_none())?;
    let name = &view.content().trap(trap).name;
    Some(
        Dialog::new("Trap", format!("Really step onto the {name}?"))
            .accept("Step onto it", Action::Move(dir))
            .cancel("Stay back", ScreenAction::Close),
    )
}

/// The id of the profile after `current` in `profiles`, wrapping around.
/// The weather after `now` (clear when it is not known, underground), wrapping around.
fn next_weather(now: Option<Weather>) -> Weather {
    let i = now.map_or(0, |w| (w.index() + 1) % Weather::ALL.len());
    Weather::ALL[i]
}

fn next_vision(profiles: &[VisionProfile], current: &str) -> String {
    let i = profiles.iter().position(|p| p.id == current).unwrap_or(0);
    profiles
        .get((i + 1) % profiles.len().max(1))
        .map_or_else(|| current.to_string(), |p| p.id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(b: &Bindings, command: KeyCommand) -> Vec<String> {
        b.keys_of(command).iter().map(|k| k.display()).collect()
    }

    #[test]
    fn default_keys_cover_vi_numpad_and_arrows() {
        let b = Bindings::default();
        assert_eq!(keys(&b, KeyCommand::Move(Dir::N)), ["k", "Num8", "↑"]);
        assert_eq!(keys(&b, KeyCommand::Move(Dir::NE)), ["u", "Num9"]);
        assert_eq!(keys(&b, KeyCommand::Descend), [">"]);
        assert!(keys(&b, KeyCommand::Quit).is_empty(), "unbound by default");
        assert_eq!(keys(&b, KeyCommand::Lantern), ["L"]);
        assert_eq!(keys(&b, KeyCommand::Kick), ["K"]);
        assert_eq!(keys(&b, KeyCommand::Help), ["?"]);
        assert_eq!(keys(&b, KeyCommand::Cancel), ["Esc"]);
        assert_eq!(keys(&b, KeyCommand::Rest), ["R"]);
        assert_eq!(keys(&b, KeyCommand::Search), ["s"]);
        assert_eq!(keys(&b, KeyCommand::SearchLong), ["z"]);
        assert_eq!(keys(&b, KeyCommand::PickUp), [",", "g"]);
        assert_eq!(keys(&b, KeyCommand::Drop), ["d"]);
        assert_eq!(keys(&b, KeyCommand::Inventory), ["i"]);
        assert_eq!(keys(&b, KeyCommand::Equipment), ["e"]);
        assert_eq!(keys(&b, KeyCommand::Character), ["@"]);
        assert_eq!(keys(&b, KeyCommand::Missile), ["t"]);
        assert!(
            keys(&b, KeyCommand::Fire).is_empty(),
            "f has no default key"
        );
        assert_eq!(keys(&b, KeyCommand::Throw), ["T"]);
        assert_eq!(keys(&b, KeyCommand::WeaponSkills), ["W"]);
        assert!(keys(&b, KeyCommand::Bash).is_empty(), "an ability: unbound");
        assert!(
            keys(&b, KeyCommand::Disarm).is_empty(),
            "an ability: unbound"
        );
        assert!(
            keys(&b, KeyCommand::PickLock).is_empty(),
            "an ability: unbound"
        );
        assert_eq!(keys(&b, KeyCommand::Abilities), ["a"]);
        assert_eq!(keys(&b, KeyCommand::RepeatAbility), ["A"]);
        assert_eq!(keys(&b, KeyCommand::MoreAggressive), ["+"]);
        assert_eq!(keys(&b, KeyCommand::MoreDefensive), ["-"]);
        assert_eq!(keys(&b, KeyCommand::Eat), ["E"]);
        assert_eq!(keys(&b, KeyCommand::UseTool), ["U"]);
        assert_eq!(keys(&b, KeyCommand::Chat), ["C"]);
        assert_eq!(keys(&b, KeyCommand::Pay), ["p"]);
        // The dialog's answers are their own context: `y` and `n` move the player in play.
        assert_eq!(keys(&b, KeyCommand::Yes), ["y"]);
        assert_eq!(keys(&b, KeyCommand::No), ["n"]);
    }

    #[test]
    fn every_context_has_its_own_keys() {
        let b = Bindings::default();
        // Aiming, the rest prompt, and the screens read keys of their own.
        assert_eq!(keys(&b, KeyCommand::NextTarget), ["Tab"]);
        assert!(
            keys(&b, KeyCommand::PrevTarget).is_empty(),
            "Shift+Tab does it"
        );
        assert_eq!(keys(&b, KeyCommand::Loose), ["Enter", "NumEnter"]);
        assert_eq!(keys(&b, KeyCommand::RestHour), ["h"]);
        assert_eq!(keys(&b, KeyCommand::RestDawn), ["d"]);
        assert_eq!(keys(&b, KeyCommand::RestDusk), ["n"]);
        assert_eq!(keys(&b, KeyCommand::MenuUp), ["↑", "k"]);
        assert_eq!(keys(&b, KeyCommand::MenuDown), ["↓", "j"]);
        assert_eq!(keys(&b, KeyCommand::MenuLeft), ["←", "h"]);
        assert_eq!(keys(&b, KeyCommand::MenuRight), ["→", "l"]);
        assert_eq!(keys(&b, KeyCommand::Confirm), ["Enter", "NumEnter"]);
        assert!(
            keys(&b, KeyCommand::Back).is_empty(),
            "Esc always backs out"
        );
        assert_eq!(keys(&b, KeyCommand::Erase), ["Del", "Backspace"]);
        assert_eq!(keys(&b, KeyCommand::Mark), ["Space"]);
        assert_eq!(keys(&b, KeyCommand::MarkAll), [","]);
        // Every command but Cancel belongs to one context, listed in `Context::ALL`.
        for (_, command) in &b.0 {
            assert!(Context::ALL.contains(&command.context()), "{command:?}");
        }
    }

    fn press(code: KeyCode, text: &str) -> KeyboardInput {
        KeyboardInput {
            key_code: code,
            logical_key: if text.is_empty() {
                Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified)
            } else {
                Key::Character(text.into())
            },
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        }
    }

    #[test]
    fn modes_read_their_own_context_first() {
        let b = Bindings::default();
        let (h, n, tab) = (
            press(KeyCode::KeyH, "h"),
            press(KeyCode::KeyN, "n"),
            press(KeyCode::Tab, ""),
        );
        let enter = KeyboardInput {
            logical_key: Key::Enter,
            ..press(KeyCode::Enter, "")
        };
        let aim = InputMode::Aim(Aim::start(
            AimKind::Fire,
            &warlock_core::World::new(
                std::sync::Arc::new(warlock_core::content::Content::builtin().unwrap()),
                "showcase",
                None,
            )
            .unwrap()
            .perception(),
            &LastTarget::default(),
        ));
        let read = |mode, key: &KeyboardInput, dev| command_in(mode, key, &b, dev);
        assert_eq!(
            read(InputMode::Normal, &h, false),
            Some(KeyCommand::Move(Dir::W))
        );
        assert_eq!(read(InputMode::Rest, &h, false), Some(KeyCommand::RestHour));
        assert_eq!(read(InputMode::Rest, &n, false), Some(KeyCommand::RestDusk));
        assert_eq!(read(aim, &tab, false), Some(KeyCommand::NextTarget));
        assert_eq!(read(aim, &enter, false), Some(KeyCommand::Loose));
        assert_eq!(read(aim, &h, false), Some(KeyCommand::Move(Dir::W)));
        assert_eq!(read(InputMode::Normal, &tab, false), None);
        assert_eq!(read(InputMode::Normal, &enter, false), None);
        // The developer keys only where they are on, and only in plain play.
        let f2 = press(KeyCode::F2, "");
        assert_eq!(read(InputMode::Normal, &f2, false), None);
        assert_eq!(
            read(InputMode::Normal, &f2, true),
            Some(KeyCommand::RevealMap)
        );
        assert_eq!(read(InputMode::Rest, &f2, true), None);
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

    #[test]
    fn weather_cycles_from_clear_to_heavy_rain() {
        assert_eq!(next_weather(None), Weather::Clear);
        assert_eq!(next_weather(Some(Weather::Clear)), Weather::Overcast);
        assert_eq!(next_weather(Some(Weather::Rain)), Weather::HeavyRain);
        assert_eq!(next_weather(Some(Weather::HeavyRain)), Weather::Clear);
    }

    #[test]
    fn leaving_a_shop_owing_and_selling_open_dialogs() {
        let content = std::sync::Arc::new(warlock_core::content::Content::builtin().unwrap());
        let mut world = warlock_core::World::new(content, "courtyard", None).expect("run");
        let walk = |world: &mut warlock_core::World, actions: &str| {
            for action in warlock_core::action::parse_actions(actions).expect("actions") {
                world.apply(&action);
            }
        };
        // Into Grimbold's store with the purse (a) and the scroll of light (b), on the cell
        // inside the door (to the west).
        walk(
            &mut world,
            "move:E*36 move:S pickup move:NE move:NE pickup:b move:SE move:E*5 move:S*5 move:E*5",
        );
        assert!(
            leaving_dialog(&world.perception(), Dir::W).is_none(),
            "owing nothing"
        );
        // With an unpaid dagger (c): pay and leave in one action, or not (the step, which the
        // rules turn back).
        walk(&mut world, "move:E move:NE pickup move:SW move:W");
        let view = world.perception();
        let owed = view.shop_here().map(|s| s.owed()).expect("owing");
        assert!(owed <= view.own_gold());
        assert_eq!(
            leaving_dialog(&view, Dir::W),
            Some(
                Dialog::new(
                    "Grimbold's General Store",
                    format!("You owe Grimbold {owed} gold.")
                )
                .accept(
                    format!("Pay {owed} gold and leave"),
                    Action::Pay(Some(Dir::W))
                )
                .decline("Don't pay", Action::Move(Dir::W))
            )
        );
        assert!(leaving_dialog(&view, Dir::E).is_none(), "a step inside");
        // Selling the scroll asks; putting back the dagger does not.
        let offer = view.sale(&Positions::of([1])).expect("an offer").offer;
        assert_eq!(
            sale_dialog(&view, &Positions::of([1])),
            Some(
                Dialog::new(
                    "Grimbold's General Store",
                    format!("Grimbold offers {offer} gold for a scroll of light.")
                )
                .accept(
                    format!("Sell for {offer} gold"),
                    Action::Sell(Positions::of([1]))
                )
                .cancel("Keep it", ScreenAction::Close)
            )
        );
        assert!(sale_dialog(&view, &Positions::of([2])).is_none());
        // With the short sword too, more than the purse holds: no way to pay, only back.
        walk(
            &mut world,
            "move:E move:NE move:E*2 pickup move:W*3 move:SW",
        );
        let view = world.perception();
        let owed = view.shop_here().map(|s| s.owed()).expect("owing");
        assert!(owed > view.own_gold());
        assert_eq!(
            leaving_dialog(&view, Dir::W),
            Some(
                Dialog::new(
                    "Grimbold's General Store",
                    format!(
                        "You owe Grimbold {owed} gold and have only 60. Put something back, or \
                         bring more gold."
                    )
                )
                .decline("Go back", Action::Move(Dir::W))
            )
        );
    }

    #[test]
    fn stepping_onto_a_known_trap_asks_first() {
        use std::sync::Arc;
        use warlock_core::World;
        use warlock_core::action::parse_actions;
        let content = Arc::new(warlock_core::content::Content::builtin().unwrap());
        let mut world = World::new(content, "traps", None).expect("the trap hall");
        // Below the arrow trap at the west end of the proving row.
        for action in parse_actions("move:NW*14 move:W*7 move:N*2").unwrap() {
            world.apply(&action);
        }
        assert_eq!(world.player_pos(), Pos::new(16, 3));
        let view = world.perception();
        assert!(trap_dialog(&view, Dir::E).is_none());
        assert_eq!(
            trap_dialog(&view, Dir::N),
            Some(
                Dialog::new("Trap", "Really step onto the arrow trap?")
                    .accept("Step onto it", Action::Move(Dir::N))
                    .cancel("Stay back", ScreenAction::Close)
            )
        );
    }

    #[test]
    fn a_trapped_door_never_asks() {
        use std::sync::Arc;
        use warlock_core::World;
        use warlock_core::action::parse_actions;
        let content = Arc::new(warlock_core::content::Content::builtin().unwrap());
        let mut world = World::new(content, "traps", None).expect("the trap hall");
        // Below the west closet's door, which shows its arrow trap: walking into the door opens
        // it (and sets the trap off) without asking, as `o` does.
        for action in parse_actions("move:NW*14 move:W*21").unwrap() {
            world.apply(&action);
        }
        assert_eq!(world.player_pos(), Pos::new(2, 5));
        let view = world.perception();
        assert!(view.trap(Pos::new(2, 4)).is_some());
        assert!(trap_dialog(&view, Dir::N).is_none());
    }

    #[test]
    fn vision_cycles_through_profiles() {
        let content = warlock_core::content::Content::builtin().unwrap();
        let profiles = content.vision_profiles();
        assert_eq!(next_vision(profiles, "normal"), "low_light");
        let last = &profiles[profiles.len() - 1].id;
        assert_eq!(next_vision(profiles, last), "normal");
    }
}
