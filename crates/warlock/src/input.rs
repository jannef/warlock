//! Keyboard input: bindings, input modes, and the one router of key presses (spec §6).
//!
//! [`route_keys`] reads every key press exactly once. While a screen is open (the screen stack is
//! not empty) the press goes to the top screen; otherwise it is a gameplay command, read through
//! the bindings in the current [`InputMode`]. Prompts in the message line (the direction for
//! open/close, how long to rest, look mode) are input modes, not screens.
//!
//! Letter and symbol commands bind by *logical* key (the character the keyboard layout produces),
//! so `<`, `>`, and `?` work on any layout. Arrows and the numpad bind by physical key code.

use crate::cli::Args;
use crate::game::{Game, PlayerAction, SimSet};
use crate::save::SaveCommand;
use crate::settings::Settings;
use crate::ui::{DevOverlays, MenuChoice, Press, ScreenAction, ScreenName, Screens};
use bevy::ecs::system::SystemParam;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use warlock_core::content::VisionProfile;
use warlock_core::time::Duration;
use warlock_core::{Action, Dir, Letters, Pos, Rest};

/// What a key does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCommand {
    Move(Dir),
    Wait,
    Open,
    Close,
    Descend,
    Ascend,
    Lantern,
    /// Pick up what lies here: a single stack at once, a choice of several on a screen.
    PickUp,
    /// Choose what to drop from the inventory.
    Drop,
    /// Show what the player carries and weighs (no time passes).
    Inventory,
    /// Ask how long to rest (an hour, until dawn, until dusk).
    Rest,
    /// Move a cursor over the map to read what is there (no time passes).
    Look,
    /// Dev: switch to the next vision profile.
    CycleVision,
    /// Dev: move the clock an hour ahead at once.
    SkipHour,
    /// Dev: show ground truth where the player does not see.
    RevealMap,
    /// Dev: show perceived light values on visible cells.
    LightValues,
    Help,
    Cancel,
    Quit,
    Snapshot,
    LoadSnapshot,
}

impl KeyCommand {
    /// What the command does; movement directions share one label (see `KeyCommand::name`).
    pub fn label(self) -> &'static str {
        match self {
            KeyCommand::Move(_) => "Move",
            KeyCommand::Wait => "Wait a turn",
            KeyCommand::Open => "Open a door",
            KeyCommand::Close => "Close a door",
            KeyCommand::Descend => "Go down stairs",
            KeyCommand::Ascend => "Go up stairs",
            KeyCommand::Lantern => "Light / douse lantern",
            KeyCommand::PickUp => "Pick up",
            KeyCommand::Drop => "Drop",
            KeyCommand::Inventory => "Inventory",
            KeyCommand::Rest => "Rest",
            KeyCommand::Look => "Look around",
            KeyCommand::CycleVision => "Next vision profile (dev)",
            KeyCommand::SkipHour => "Skip an hour (dev)",
            KeyCommand::RevealMap => "Reveal map (dev)",
            KeyCommand::LightValues => "Light values (dev)",
            KeyCommand::Help => "Show / hide controls",
            KeyCommand::Cancel => "Cancel",
            KeyCommand::Quit => "Save and quit",
            KeyCommand::Snapshot => "Save snapshot (dev)",
            KeyCommand::LoadSnapshot => "Load snapshot (dev)",
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
            (ch("o"), Open),
            (ch("c"), Close),
            (ch(">"), Descend),
            (ch("<"), Ascend),
            (ch("L"), Lantern),
            (ch(","), PickUp),
            (ch("g"), PickUp),
            (ch("d"), Drop),
            (ch("i"), Inventory),
            (ch("R"), Rest),
            (ch("x"), Look),
            (ch("?"), Help),
            (Code(KeyCode::Escape), Cancel),
            (ch("Q"), Quit),
        ]);
        if cfg!(debug_assertions) {
            table.add_dev_keys();
        }
        table
    }
}

impl Bindings {
    /// Adds the developer keys — reveal map, vision profile switching, light values, snapshots,
    /// skipping an hour (always present in debug builds, `--dev` otherwise).
    pub(crate) fn add_dev_keys(&mut self) {
        let dev = [
            (KeyBinding::Code(KeyCode::F2), KeyCommand::RevealMap),
            (KeyBinding::Code(KeyCode::F3), KeyCommand::CycleVision),
            (KeyBinding::Code(KeyCode::F4), KeyCommand::LightValues),
            (KeyBinding::Code(KeyCode::F5), KeyCommand::Snapshot),
            (KeyBinding::Code(KeyCode::F6), KeyCommand::SkipHour),
            (KeyBinding::Code(KeyCode::F9), KeyCommand::LoadSnapshot),
        ];
        for entry in dev {
            // A key the player bound to something else stays theirs.
            if !self.0.iter().any(|(k, _)| *k == entry.0) {
                self.0.push(entry);
            }
        }
    }

    pub fn lookup(&self, input: &KeyboardInput) -> Option<KeyCommand> {
        // Physical bindings first: numpad keys also produce digit characters.
        self.0
            .iter()
            .filter(|(k, _)| matches!(k, KeyBinding::Code(_)))
            .chain(
                self.0
                    .iter()
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
    /// Waiting for a direction for `Open` or `Close`.
    Direction(KeyCommand),
    /// Asking how long to rest ([`REST_CHOICES`]).
    Rest,
    /// Look mode: a cursor at this level position.
    Look(Pos),
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // The settings file's bindings, if `main` loaded them.
        let mut bindings = app
            .world()
            .get_resource::<Bindings>()
            .cloned()
            .unwrap_or_default();
        if app.world().resource::<Args>().dev {
            bindings.add_dev_keys();
        }
        app.insert_resource(bindings)
            .init_resource::<InputMode>()
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
) {
    let mut chosen = false;
    for key in keys.read() {
        if chosen || key.state != ButtonState::Pressed {
            continue;
        }
        let action = if screens.stack.is_empty() {
            play.press(key, &bindings)
                .map_or(ScreenAction::Stay, ScreenAction::Open)
        } else {
            // Screens edit copies, so the resources only change (and are saved) when a press
            // changed them.
            let (mut new_settings, mut new_bindings) = (settings.clone(), bindings.clone());
            let action = screens.stack.handle(
                &Press::from_input(key),
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
                play.act(action);
            }
            ScreenAction::Open(name) => screens.open(name),
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
    game: Option<Res<'w, Game>>,
    mode: ResMut<'w, InputMode>,
    overlays: ResMut<'w, DevOverlays>,
    actions: MessageWriter<'w, PlayerAction>,
    saves: MessageWriter<'w, SaveCommand>,
}

impl Gameplay<'_> {
    /// Applies one key press in play. Returns the screen it opens: `?` help, or the pause menu
    /// on Esc when no prompt is open.
    fn press(&mut self, key: &KeyboardInput, bindings: &Bindings) -> Option<ScreenName> {
        let game = self.game.as_ref()?;
        // The rest prompt answers by letter, whatever those letters are bound to.
        if *self.mode == InputMode::Rest
            && let Some(rest) = rest_choice(key)
        {
            self.actions.write(PlayerAction(Action::Rest(rest)));
            *self.mode = InputMode::Normal;
            return None;
        }
        let command = bindings.lookup(key)?;
        // Held keys repeat only for movement and waiting.
        if key.repeat && !matches!(command, KeyCommand::Move(_) | KeyCommand::Wait) {
            return None;
        }
        let mut open = None;
        let next = match (*self.mode, command) {
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
            (InputMode::Direction(cmd), KeyCommand::Move(dir)) => {
                let action = if cmd == KeyCommand::Open {
                    Action::Open(dir)
                } else {
                    Action::Close(dir)
                };
                self.actions.write(PlayerAction(action));
                InputMode::Normal
            }
            (InputMode::Direction(_), _) => InputMode::Normal,
            // Any other command (Esc, or `R` again) cancels the rest prompt.
            (InputMode::Rest, _) => InputMode::Normal,
            (InputMode::Normal, KeyCommand::Rest) => InputMode::Rest,
            (InputMode::Normal, KeyCommand::Move(dir)) => {
                self.actions.write(PlayerAction(Action::Move(dir)));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Wait) => {
                self.actions.write(PlayerAction(Action::Wait));
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
                    0 => Some(Letters::default()),
                    1 => Some(Letters::of([0])),
                    _ => None,
                };
                match letters {
                    Some(letters) => {
                        self.actions.write(PlayerAction(Action::PickUp(letters)));
                    }
                    None => open = Some(ScreenName::PickUp),
                }
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Drop) => {
                if game.world.perception().inventory().is_empty() {
                    self.actions
                        .write(PlayerAction(Action::Drop(Letters::default())));
                } else {
                    open = Some(ScreenName::Drop);
                }
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Inventory) => {
                open = Some(ScreenName::Inventory);
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
            (InputMode::Normal, KeyCommand::RevealMap) => {
                self.overlays.reveal = !self.overlays.reveal;
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::LightValues) => {
                self.overlays.light_values = !self.overlays.light_values;
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Open | KeyCommand::Close) => {
                InputMode::Direction(command)
            }
            (InputMode::Normal, KeyCommand::Help) => {
                open = Some(ScreenName::Help);
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Cancel) => {
                open = Some(ScreenName::Pause);
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Quit) => {
                self.saves.write(SaveCommand::SaveAndQuit);
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
        };
        if *self.mode != next {
            *self.mode = next;
        }
        open
    }

    /// Asks the simulation to apply `action`, chosen on a screen.
    fn act(&mut self, action: Action) {
        self.actions.write(PlayerAction(action));
    }
}

/// The rest prompt's answers: the key (a character) and what it rests for.
pub const REST_CHOICES: [(&str, Rest); 3] = [
    ("h", Rest::For(Duration::hours(1))),
    ("d", Rest::Dawn),
    ("n", Rest::Dusk),
];

/// The rest a key press picks in the rest prompt, if any.
fn rest_choice(key: &KeyboardInput) -> Option<Rest> {
    let Key::Character(c) = &key.logical_key else {
        return None;
    };
    let c = c.as_str().to_lowercase();
    REST_CHOICES
        .iter()
        .find(|(k, _)| *k == c)
        .map(|(_, rest)| *rest)
}

/// The id of the profile after `current` in `profiles`, wrapping around.
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
        assert_eq!(keys(&b, KeyCommand::Quit), ["Q"]);
        assert_eq!(keys(&b, KeyCommand::Lantern), ["L"]);
        assert_eq!(keys(&b, KeyCommand::Help), ["?"]);
        assert_eq!(keys(&b, KeyCommand::Cancel), ["Esc"]);
        assert_eq!(keys(&b, KeyCommand::Rest), ["R"]);
        assert_eq!(keys(&b, KeyCommand::PickUp), [",", "g"]);
        assert_eq!(keys(&b, KeyCommand::Drop), ["d"]);
        assert_eq!(keys(&b, KeyCommand::Inventory), ["i"]);
    }

    #[test]
    fn dev_keys_are_added_once() {
        let mut b = Bindings::default();
        b.add_dev_keys();
        b.add_dev_keys();
        assert_eq!(keys(&b, KeyCommand::Snapshot), ["F5"]);
        assert_eq!(keys(&b, KeyCommand::LoadSnapshot), ["F9"]);
        assert_eq!(keys(&b, KeyCommand::CycleVision), ["F3"]);
        assert_eq!(keys(&b, KeyCommand::RevealMap), ["F2"]);
        assert_eq!(keys(&b, KeyCommand::LightValues), ["F4"]);
        assert_eq!(keys(&b, KeyCommand::SkipHour), ["F6"]);
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
