//! Key presses in plain play (`Gameplay::press` with no prompt open): every play command, and
//! the developer keys where they are on.

use super::{Gameplay, InputMode, KeyCommand, Pressed};
use crate::game::PlayerAction;
use crate::save::SaveCommand;
use crate::ui::step_dialogs::{step_dialog, step_screen};
use crate::ui::{Aim, AimKind, ScreenAction, ScreenName, quit_dialog};
use warlock_core::ability::Ability;
use warlock_core::beside::Beside;
use warlock_core::content::VisionProfile;
use warlock_core::time::Duration;
use warlock_core::weather::Weather;
use warlock_core::{Action, Dir, Positions};

impl Gameplay<'_> {
    /// Applies `command` in plain play, `repeat` whether its key is held, `last` the ability
    /// used last (for the quick repeat): the mode next, and what it asks of the screens.
    pub(super) fn play(
        &mut self,
        command: KeyCommand,
        repeat: bool,
        last: Option<Ability>,
    ) -> Option<Pressed> {
        let game = self.game.as_ref()?;
        let mut open: Option<ScreenAction> = None;
        let next = match command {
            KeyCommand::Missile => {
                let view = game.world.perception();
                if let Some(item) = view.ready_wand().or(view.ready_rod()) {
                    InputMode::Aim(Aim::start(AimKind::Zap(item), &view, &self.last))
                } else if view.ready_to_fire().is_some() {
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
            KeyCommand::Throw => {
                open = Some(ScreenAction::Open(ScreenName::Throw));
                InputMode::Normal
            }
            KeyCommand::Look => InputMode::Look(game.world.player_pos()),
            KeyCommand::Rest => InputMode::Rest,
            KeyCommand::Move(dir) => {
                // Diving into deep water (spec §8 "Skills": Swim), leaving a shop owing anything
                // (spec §7 "Shops"), stepping onto a known trap (spec §8 "Traps"), setting out on
                // the overworld Fainting or Starving (spec §8 "Hunger"), or walking into something
                // that offers an action (spec §7 "Special rooms") asks first, in the dialog: one
                // dialog a step, the first that applies. The dive comes first because it always
                // asks (the owner's rule), and any other dialog's "go on" would step in unasked;
                // its own "Dive in" is a plain move, which the rules still judge (the keeper turns
                // back one who owes). A held key stops there (repeats never open the dialog).
                // Walking into an anvil opens Metalcraft's screen for the trained (#1387).
                let view = game.world.perception();
                let screen = step_dialog(&view, dir)
                    .map(ScreenAction::from)
                    .or_else(|| step_screen(&view, dir).map(ScreenAction::Open));
                match screen {
                    Some(screen) if !repeat => open = Some(screen),
                    Some(_) => {}
                    None => {
                        self.actions.write(PlayerAction(Action::Move(dir)));
                    }
                }
                InputMode::Normal
            }
            // A run's first step asks what a step would (#1325): the dialog's answer is a plain
            // step; with nothing to ask, the run.
            KeyCommand::Run(dir) => {
                let view = game.world.perception();
                let screen = step_dialog(&view, dir)
                    .map(ScreenAction::from)
                    .or_else(|| step_screen(&view, dir).map(ScreenAction::Open));
                match screen {
                    Some(screen) => open = Some(screen),
                    None => {
                        self.actions.write(PlayerAction(Action::Run(dir)));
                    }
                }
                InputMode::Normal
            }
            KeyCommand::Pay => {
                self.actions.write(PlayerAction(Action::Pay(None)));
                InputMode::Normal
            }
            KeyCommand::Wait => {
                self.actions.write(PlayerAction(Action::Wait));
                InputMode::Normal
            }
            KeyCommand::SearchLong => {
                let turns = game.world.content().search().turns;
                self.actions.write(PlayerAction(Action::Search(turns)));
                InputMode::Normal
            }
            KeyCommand::Descend => {
                self.actions.write(PlayerAction(Action::Descend));
                InputMode::Normal
            }
            KeyCommand::Ascend => {
                self.actions.write(PlayerAction(Action::Ascend));
                InputMode::Normal
            }
            KeyCommand::PickUp => {
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
            KeyCommand::Drop => {
                if game.world.perception().inventory().is_empty() {
                    self.actions
                        .write(PlayerAction(Action::Drop(Positions::default())));
                } else {
                    open = Some(ScreenAction::Open(ScreenName::Drop));
                }
                InputMode::Normal
            }
            KeyCommand::Inventory => {
                open = Some(ScreenAction::Open(ScreenName::Inventory));
                InputMode::Normal
            }
            KeyCommand::Equipment => {
                open = Some(ScreenAction::Open(ScreenName::Equipment));
                InputMode::Normal
            }
            KeyCommand::Character => {
                open = Some(ScreenAction::Open(ScreenName::Character));
                InputMode::Normal
            }
            KeyCommand::Skills => {
                open = Some(ScreenAction::Open(ScreenName::Skills));
                InputMode::Normal
            }
            KeyCommand::MoreAggressive | KeyCommand::MoreDefensive => {
                let now = game.world.perception().own_stance();
                let next = if command == KeyCommand::MoreAggressive {
                    now.more_aggressive()
                } else {
                    now.more_defensive()
                };
                self.actions.write(PlayerAction(Action::SetStance(next)));
                InputMode::Normal
            }
            KeyCommand::WeaponSkills => {
                open = Some(ScreenAction::Open(ScreenName::WeaponSkills));
                InputMode::Normal
            }
            KeyCommand::Talents => {
                open = Some(ScreenAction::Open(ScreenName::Talents));
                InputMode::Normal
            }
            KeyCommand::Eat => {
                open = Some(ScreenAction::Open(ScreenName::Eat));
                InputMode::Normal
            }
            KeyCommand::Read => {
                open = Some(ScreenAction::Open(ScreenName::Read));
                InputMode::Normal
            }
            KeyCommand::Quests => {
                open = Some(ScreenAction::Open(ScreenName::Quests));
                InputMode::Normal
            }
            KeyCommand::Messages => {
                open = Some(ScreenAction::Open(ScreenName::Messages));
                InputMode::Normal
            }
            KeyCommand::CycleVision => {
                let view = game.world.perception();
                let next = next_vision(view.content().vision_profiles(), &view.vision().id);
                self.actions.write(PlayerAction(Action::SetVision(next)));
                InputMode::Normal
            }
            KeyCommand::SkipHour => {
                self.actions
                    .write(PlayerAction(Action::SkipTime(Duration::hours(1))));
                InputMode::Normal
            }
            KeyCommand::NextWeather => {
                let now = game.world.perception().weather();
                self.actions
                    .write(PlayerAction(Action::SetWeather(next_weather(now))));
                InputMode::Normal
            }
            KeyCommand::IdentifyAll => {
                self.actions
                    .write(PlayerAction(Action::Identify(Positions::default())));
                InputMode::Normal
            }
            KeyCommand::RevealMap => {
                self.overlays.reveal = !self.overlays.reveal;
                InputMode::Normal
            }
            KeyCommand::LightValues => {
                self.overlays.light_values = !self.overlays.light_values;
                InputMode::Normal
            }
            // Kick asks always: any way may be kicked.
            KeyCommand::Kick => InputMode::Direction(command),
            KeyCommand::Open => self.beside(Beside::Open, InputMode::Direction(command)),
            KeyCommand::Close => self.beside(Beside::Close, InputMode::Direction(command)),
            KeyCommand::Abilities => {
                open = Some(ScreenAction::Open(ScreenName::Abilities));
                InputMode::Normal
            }
            // The ability used last, again; with none yet, the menu to choose one.
            KeyCommand::RepeatAbility => {
                open = Some(last.map_or(
                    ScreenAction::Open(ScreenName::Abilities),
                    ScreenAction::UseAbility,
                ));
                InputMode::Normal
            }
            // An ability's own key (unbound by default) uses it as the menu does.
            KeyCommand::Bash => {
                open = Some(ScreenAction::UseAbility(Ability::ShieldBash));
                InputMode::Normal
            }
            KeyCommand::ThrowVoice => {
                open = Some(ScreenAction::UseAbility(Ability::ThrowVoice));
                InputMode::Normal
            }
            KeyCommand::Disarm => {
                open = Some(ScreenAction::UseAbility(Ability::Disarm));
                InputMode::Normal
            }
            KeyCommand::PickLock => {
                open = Some(ScreenAction::UseAbility(Ability::PickLock));
                InputMode::Normal
            }
            KeyCommand::Steal => {
                open = Some(ScreenAction::UseAbility(Ability::Steal));
                InputMode::Normal
            }
            // The one creature in view next to the player, at once; among several, ask which way.
            KeyCommand::Chat => self.beside(Beside::Chat, InputMode::Direction(command)),
            KeyCommand::UseTool => {
                let view = game.world.perception();
                if view.ready_light().is_some() {
                    // A light ready (a lantern): its next mode, at once.
                    self.actions.write(PlayerAction(Action::UseTool));
                    InputMode::Normal
                } else if view.dig_refusal().is_none() {
                    self.beside(Beside::Dig, InputMode::Direction(command))
                } else {
                    // Nothing to dig with, or untrained in Mining: the rules say so ("You have no
                    // digging tool ready.").
                    self.actions.write(PlayerAction(Action::Dig(Dir::N)));
                    InputMode::Normal
                }
            }
            KeyCommand::Help => {
                open = Some(ScreenAction::Open(ScreenName::Help));
                InputMode::Normal
            }
            KeyCommand::Cancel => {
                open = Some(ScreenAction::Open(ScreenName::Pause));
                InputMode::Normal
            }
            KeyCommand::Quit => {
                open = Some(quit_dialog(true).into());
                InputMode::Normal
            }
            KeyCommand::Snapshot => {
                self.saves.write(SaveCommand::Snapshot);
                InputMode::Normal
            }
            KeyCommand::LoadSnapshot => {
                self.saves.write(SaveCommand::LoadSnapshot);
                InputMode::Normal
            }
            // Other contexts' commands (aiming's, `--more--`'s, the screens'): `command_in`
            // never finds them here.
            KeyCommand::PrevTarget
            | KeyCommand::MoreNext
            | KeyCommand::MoreSkip
            | KeyCommand::Back
            | KeyCommand::TextBack
            | KeyCommand::PrevPage => InputMode::Normal,
        };
        Some((next, open))
    }
}

/// The weather after `now` (clear when it is not known, underground), wrapping around.
fn next_weather(now: Option<Weather>) -> Weather {
    let i = now.map_or(0, |w| (w.index() + 1) % Weather::ALL.len());
    Weather::ALL[i]
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
    fn weather_cycles_from_clear_to_heavy_rain() {
        assert_eq!(next_weather(None), Weather::Clear);
        assert_eq!(next_weather(Some(Weather::Clear)), Weather::Overcast);
        assert_eq!(next_weather(Some(Weather::Rain)), Weather::HeavyRain);
        assert_eq!(next_weather(Some(Weather::HeavyRain)), Weather::Clear);
    }

    #[test]
    fn vision_cycles_through_profiles() {
        let content = warlock_assets::content().unwrap();
        let profiles = content.vision_profiles();
        assert_eq!(next_vision(profiles, "normal"), "low_light");
        let last = &profiles[profiles.len() - 1].id;
        assert_eq!(next_vision(profiles, last), "normal");
    }
}
