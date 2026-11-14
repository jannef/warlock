//! The prompts' key presses (`Gameplay::press`): `--more--`, aiming, look mode, and the
//! direction prompts (a command's, an ability's, Give's, and Trapwright's way). The rest prompt
//! and aiming's fixed keys are read before these (`Gameplay::fixed_key`).

use super::{Bindings, Gameplay, InputMode, KeyCommand, Pressed};
use crate::game::PlayerAction;
use crate::ui::{Aim, more};
use warlock_core::ability::Ability;
use warlock_core::content::RigId;
use warlock_core::{Action, Pos};

impl Gameplay<'_> {
    /// `--more--`: the next page of this turn's messages, then play once the last shows;
    /// skipping goes to play at once (the last page shows). Nothing else acts meanwhile.
    pub(super) fn more(
        &self,
        page: usize,
        command: KeyCommand,
        bindings: &Bindings,
    ) -> Option<InputMode> {
        let game = self.game.as_ref()?;
        Some(match command {
            KeyCommand::MoreNext => {
                let cols = self.console.as_ref().map_or(0, |c| c.cols());
                more::advance(page, more::stops(game, cols, bindings))
            }
            KeyCommand::MoreSkip | KeyCommand::Cancel => InputMode::Normal,
            _ => InputMode::More(page),
        })
    }

    /// Aiming `aim`: the cursor's moves and the previous target. The key that started the aim
    /// confirms it, as Enter does (the owner, #1533).
    pub(super) fn aiming(&mut self, aim: Aim, command: KeyCommand) -> Option<Pressed> {
        let game = self.game.as_ref()?;
        let next = match command {
            KeyCommand::Move(dir) => InputMode::Aim(aim.step(dir, &game.world.perception())),
            KeyCommand::PrevTarget => {
                InputMode::Aim(aim.cycle(&game.world.perception(), self.shift()))
            }
            _ if aim.confirmed_by(command) => {
                let open = self.release(aim);
                return Some((*self.mode, open));
            }
            KeyCommand::Cancel | KeyCommand::Look => InputMode::Normal,
            _ => InputMode::Aim(aim),
        };
        Some((next, None))
    }

    /// Look mode, the cursor at `at`: moving it, within the level.
    pub(super) fn looking(&self, at: Pos, command: KeyCommand) -> Option<InputMode> {
        let game = self.game.as_ref()?;
        Some(match command {
            KeyCommand::Look | KeyCommand::Cancel => InputMode::Normal,
            KeyCommand::Move(dir) => {
                let view = game.world.perception();
                let to = at.step(dir);
                InputMode::Look(Pos::new(
                    to.x.clamp(0, view.width() - 1),
                    to.y.clamp(0, view.height() - 1),
                ))
            }
            _ => InputMode::Look(at),
        })
    }

    /// The way to use `ability` in. Heal with two kept beasts beside the player asked which way
    /// first (`tend_way`): the one there, or, at the wait key, the player themselves.
    pub(super) fn ability_way(&mut self, ability: Ability, command: KeyCommand) -> Option<Pressed> {
        let game = self.game.as_ref()?;
        let mut open = None;
        let next = match (ability, command) {
            (Ability::Heal, KeyCommand::Move(dir)) => {
                open = self.tend(Some(dir));
                InputMode::Normal
            }
            (Ability::Heal, KeyCommand::Wait) => {
                open = self.tend(None);
                InputMode::Normal
            }
            (ability, KeyCommand::Move(dir)) => {
                let content = game.world.content().clone();
                if let Some(action) = ability.action(&content, Some(dir)) {
                    self.actions.write(PlayerAction(action));
                }
                InputMode::Normal
            }
            _ => InputMode::Normal,
        };
        Some((next, open))
    }

    /// The way to the kept beast inventory stack `item` goes on (Give).
    pub(super) fn give_way(&mut self, item: usize, command: KeyCommand) -> InputMode {
        if let KeyCommand::Move(dir) = command {
            self.actions.write(PlayerAction(Action::Give {
                item,
                ally: Some(dir),
            }));
        }
        InputMode::Normal
    }

    /// The way to rig trap `rig` on (Trapwright).
    pub(super) fn rig_way(&mut self, rig: RigId, command: KeyCommand) -> Option<InputMode> {
        let game = self.game.as_ref()?;
        if let KeyCommand::Move(dir) = command {
            let content = game.world.content();
            if let Some(rules) = content.rigs() {
                self.actions.write(PlayerAction(Action::Rig {
                    rig: rules.rig(rig).id.clone(),
                    dir: Some(dir),
                }));
            }
        }
        Some(InputMode::Normal)
    }

    /// The way for `cmd`: open, close, kick, dig (use a tool), or chat.
    pub(super) fn direction(&mut self, cmd: KeyCommand, command: KeyCommand) -> InputMode {
        if let KeyCommand::Move(dir) = command {
            let action = match cmd {
                KeyCommand::Open => Action::Open(dir),
                KeyCommand::Kick => Action::Kick(dir),
                KeyCommand::UseTool => Action::Dig(dir),
                KeyCommand::Chat => Action::Chat(dir),
                _ => Action::Close(dir),
            };
            self.actions.write(PlayerAction(action));
        }
        InputMode::Normal
    }
}
