//! The overworld encounter's question (spec §7 "Overworld encounters", #805;
//! `docs/dev/encounters.md`): while the world holds one pending
//! ([`warlock_core::perception::PerceptionView::encounter`]), the one dialog asks it, a row per
//! choice that sends `Action::Encounter(choice)`. It must be answered (the owner, #796 Q9): Esc
//! does nothing and no info screen opens over it ([`Dialog::required`]). It opens after any
//! action that leaves one pending and on load, as the level-up screen does; an answer the world
//! refused would leave it pending, and the dialog opens again.

use super::dialog_screen::Dialog;
use super::screens::Screens;
use crate::game::Game;
use bevy::prelude::*;
use warlock_core::action::Action;
use warlock_core::world::EncounterView;

/// The dialog asking `view`: its title, its text (with the line for a party met again, as the
/// view gives it), and a plain row per choice.
pub(super) fn dialog(view: EncounterView) -> Dialog {
    let ask = Dialog::new(view.title, view.text);
    (view.choices.into_iter())
        .fold(ask, |ask, (label, choice)| {
            ask.plain(label, Action::Encounter(choice))
        })
        .required()
}

/// Opens the dialog when an encounter is pending and it is not open yet; the death screen wins.
/// It runs before the level-up screen opens, so a level's picks (which the world asks first)
/// show over it.
pub(super) fn on_encounter(game: Res<Game>, mut screens: Screens) {
    if !game.is_changed() || game.world.death().is_some() || screens.stack.shows_death() {
        return;
    }
    if screens.stack.asks_required() {
        return;
    }
    if let Some(view) = game.world.perception().encounter() {
        screens.ask(dialog(view));
    }
}
