//! Keyboard input: bindings, input modes, and translating keys into commands (spec §6).
//!
//! Letter and symbol commands bind by *logical* key (the character the keyboard layout produces),
//! so `<`, `>`, and `?` work on any layout. Arrows and the numpad bind by physical key code.

use crate::cli::Args;
use crate::game::{Game, PlayerAction, SimSet};
use crate::save::SaveCommand;
use crate::ui::{DevOverlays, Menu, OpenPause};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use warlock_core::content::VisionProfile;
use warlock_core::{Action, Dir, Pos};

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
    /// Move a cursor over the map to read what is there (no time passes).
    Look,
    /// Dev: switch to the next vision profile.
    CycleVision,
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
    /// Label for the controls overlay; movement directions share one entry.
    pub fn label(self) -> &'static str {
        match self {
            KeyCommand::Move(_) => "Move",
            KeyCommand::Wait => "Wait a turn",
            KeyCommand::Open => "Open a door",
            KeyCommand::Close => "Close a door",
            KeyCommand::Descend => "Go down stairs",
            KeyCommand::Ascend => "Go up stairs",
            KeyCommand::Lantern => "Light / douse lantern",
            KeyCommand::Look => "Look around",
            KeyCommand::CycleVision => "Next vision profile (dev)",
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
    /// Human-readable key name for the controls overlay.
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

/// The binding table. The controls overlay is generated from it, so it is always accurate.
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
    /// Adds the developer keys — reveal map, vision profile switching, light values, snapshots
    /// (always present in debug builds, `--dev` otherwise).
    fn add_dev_keys(&mut self) {
        let dev = [
            (KeyBinding::Code(KeyCode::F2), KeyCommand::RevealMap),
            (KeyBinding::Code(KeyCode::F3), KeyCommand::CycleVision),
            (KeyBinding::Code(KeyCode::F4), KeyCommand::LightValues),
            (KeyBinding::Code(KeyCode::F5), KeyCommand::Snapshot),
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

    /// Keys bound to commands with the same overlay label, in table order.
    pub fn keys_for(&self, label: &str) -> Vec<String> {
        self.0
            .iter()
            .filter(|(_, c)| c.label() == label)
            .map(|(k, _)| k.display())
            .collect()
    }

    /// Overlay labels in first-appearance order.
    pub fn labels(&self) -> Vec<&'static str> {
        let mut labels: Vec<&'static str> = Vec::new();
        for (_, c) in &self.0 {
            if !labels.contains(&c.label()) {
                labels.push(c.label());
            }
        }
        labels
    }
}

/// What keys currently mean.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    /// Waiting for a direction for `Open` or `Close`.
    Direction(KeyCommand),
    /// The controls overlay is open.
    Help,
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
            .add_systems(
                Update,
                read_keys.before(SimSet).run_if(resource_exists::<Game>),
            );
    }
}

fn read_keys(
    mut keys: MessageReader<KeyboardInput>,
    bindings: Res<Bindings>,
    game: Res<Game>,
    mut mode: ResMut<InputMode>,
    mut overlays: ResMut<DevOverlays>,
    mut actions: MessageWriter<PlayerAction>,
    mut saves: MessageWriter<SaveCommand>,
    mut pause: MessageWriter<OpenPause>,
    menu: Option<Res<Menu>>,
) {
    // The pause menu owns the keyboard (and eats these presses), so time cannot advance.
    if menu.is_some() {
        keys.clear();
        return;
    }
    for key in keys.read() {
        if key.state != ButtonState::Pressed {
            continue;
        }
        let Some(command) = bindings.lookup(key) else {
            continue;
        };
        // Held keys repeat only for movement and waiting.
        if key.repeat && !matches!(command, KeyCommand::Move(_) | KeyCommand::Wait) {
            continue;
        }
        let next = match (*mode, command) {
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
            (InputMode::Help, KeyCommand::Help | KeyCommand::Cancel) => InputMode::Normal,
            (InputMode::Help, _) => InputMode::Help,
            (InputMode::Direction(cmd), KeyCommand::Move(dir)) => {
                let action = if cmd == KeyCommand::Open {
                    Action::Open(dir)
                } else {
                    Action::Close(dir)
                };
                actions.write(PlayerAction(action));
                InputMode::Normal
            }
            (InputMode::Direction(_), _) => InputMode::Normal,
            (InputMode::Normal, KeyCommand::Move(dir)) => {
                actions.write(PlayerAction(Action::Move(dir)));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Wait) => {
                actions.write(PlayerAction(Action::Wait));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Descend) => {
                actions.write(PlayerAction(Action::Descend));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Ascend) => {
                actions.write(PlayerAction(Action::Ascend));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Lantern) => {
                actions.write(PlayerAction(Action::ToggleLantern));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::CycleVision) => {
                let view = game.world.perception();
                let next = next_vision(view.content().vision_profiles(), &view.vision().id);
                actions.write(PlayerAction(Action::SetVision(next)));
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::RevealMap) => {
                overlays.reveal = !overlays.reveal;
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::LightValues) => {
                overlays.light_values = !overlays.light_values;
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Open | KeyCommand::Close) => {
                InputMode::Direction(command)
            }
            (InputMode::Normal, KeyCommand::Help) => InputMode::Help,
            (InputMode::Normal, KeyCommand::Cancel) => {
                pause.write(OpenPause);
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Quit) => {
                saves.write(SaveCommand::SaveAndQuit);
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::Snapshot) => {
                saves.write(SaveCommand::Snapshot);
                InputMode::Normal
            }
            (InputMode::Normal, KeyCommand::LoadSnapshot) => {
                saves.write(SaveCommand::LoadSnapshot);
                InputMode::Normal
            }
        };
        if *mode != next {
            *mode = next;
        }
    }
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

    #[test]
    fn overlay_lists_every_command_once_with_all_keys() {
        let b = Bindings::default();
        let labels = b.labels();
        assert_eq!(labels.iter().filter(|l| **l == "Move").count(), 1);
        let move_keys = b.keys_for("Move");
        assert!(move_keys.contains(&"h".to_string()));
        assert!(move_keys.contains(&"Num8".to_string()));
        assert!(move_keys.contains(&"↑".to_string()));
        assert_eq!(b.keys_for("Go down stairs"), vec![">".to_string()]);
        assert_eq!(b.keys_for("Save and quit"), vec!["Q".to_string()]);
        assert_eq!(b.keys_for("Light / douse lantern"), vec!["L".to_string()]);
    }

    #[test]
    fn dev_keys_are_added_once() {
        let mut b = Bindings::default();
        b.add_dev_keys();
        b.add_dev_keys();
        assert_eq!(b.keys_for("Save snapshot (dev)"), vec!["F5".to_string()]);
        assert_eq!(b.keys_for("Load snapshot (dev)"), vec!["F9".to_string()]);
        assert_eq!(
            b.keys_for("Next vision profile (dev)"),
            vec!["F3".to_string()]
        );
        assert_eq!(b.keys_for("Reveal map (dev)"), vec!["F2".to_string()]);
        assert_eq!(b.keys_for("Light values (dev)"), vec!["F4".to_string()]);
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
