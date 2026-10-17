//! Keyboard input: bindings, input modes, and translating keys into commands (spec §6).
//!
//! Letter and symbol commands bind by *logical* key (the character the keyboard layout produces),
//! so `<`, `>`, and `?` work on any layout. Arrows and the numpad bind by physical key code.

use crate::game::{PlayerAction, SimSet};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use warlock_core::{Action, Dir};

/// What a key does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCommand {
    Move(Dir),
    Wait,
    Open,
    Close,
    Descend,
    Ascend,
    Help,
    Cancel,
    Quit,
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
            KeyCommand::Help => "Show / hide controls",
            KeyCommand::Cancel => "Cancel",
            KeyCommand::Quit => "Quit",
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
        Self(vec![
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
            (ch("?"), Help),
            (Code(KeyCode::Escape), Cancel),
            (ch("Q"), Quit),
        ])
    }
}

impl Bindings {
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
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Bindings>()
            .init_resource::<InputMode>()
            .add_systems(Update, read_keys.before(SimSet));
    }
}

fn read_keys(
    mut keys: MessageReader<KeyboardInput>,
    bindings: Res<Bindings>,
    mut mode: ResMut<InputMode>,
    mut actions: MessageWriter<PlayerAction>,
    mut exit: MessageWriter<AppExit>,
) {
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
            (InputMode::Normal, KeyCommand::Open | KeyCommand::Close) => {
                InputMode::Direction(command)
            }
            (InputMode::Normal, KeyCommand::Help) => InputMode::Help,
            (InputMode::Normal, KeyCommand::Cancel) => InputMode::Normal,
            (InputMode::Normal, KeyCommand::Quit) => {
                exit.write(AppExit::Success);
                InputMode::Normal
            }
        };
        if *mode != next {
            *mode = next;
        }
    }
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
    }
}
