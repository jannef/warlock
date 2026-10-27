//! Foraging in the wild (spec §8 "Foraging"; #547): [`Action::Forage`], the Forage ability,
//! Survival's use.
//!
//! - **Who**: an actor trained in a skill with the `forage` benefit (Survival,
//!   `data/skills.toml`), [`crate::skills::Skills::trains_effect`], while `data/forage.toml`
//!   exists. Anyone else is told "You do not know how to live off the land." and no time passes.
//! - **Where** ([`forage_ground`]): on the overworld, by the landscape of the terrain underfoot;
//!   in a local area (`local:x,y`), by the landscape of its overworld cell (its edge exit), so an
//!   area forages as its cell does. Only a landscape with a row in `data/forage.toml` (land: not
//!   a road or water), and never standing in water or on ice. Anywhere else is refused, taking
//!   no time ("There is nothing to forage here."), and the abilities menu greys it out ("not
//!   usable here", [`crate::ability::Unavailable::NotHere`]). Towns, dungeons, and hand-made
//!   outdoor levels are not the wild.
//! - **Time**: the file's `seconds`, in turns of the `forage` action ([`World::keep_at`], the
//!   rest's machinery; the last turn cut short to end on time): a creature coming into view, a
//!   message, or a blow stops it, and a forage cut short finds nothing (the time is spent). It
//!   makes no noise of its own, as a rest makes none.
//! - **The find**, once the time is up: the ground's `chance` plus what Survival gives
//!   ([`Effect::Forage`], through [`crate::world::Actor::skill_effect`]), at most 100, one draw
//!   of the gameplay stream; found, one of the ground's `finds` by weight (one draw) and its
//!   `count` (a draw per die). The food is made now (fresh, `rot::made_at`) and goes into the
//!   pack (packs have no limit but the burden). "You gather 2 handfuls of berries." / "You find
//!   nothing worth eating." Then Survival may earn a training mark ([`Use::Forage`], the
//!   training stream). Nothing is remembered of a forage: the same spot may be tried again at
//!   once (no anti-farming limit, the owner's rule: time is the price).
//!
//! [`Action::Forage`]: crate::action::Action::Forage

use super::rest::Ended;
use super::{LevelState, Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, Content, ForageGround, LevelKind, parse_local_area};
use crate::geom::Pos;
use crate::magic::loot::pick_weighted;
use crate::object::{self, Stack};
use crate::skills::{Effect, Use};
use crate::time::Duration;

/// The ground a forage at `at` on `level` would search (module docs): `None` where nothing can
/// be foraged, or without `data/forage.toml`.
pub(crate) fn forage_ground<'a>(
    content: &'a Content,
    level: &LevelState,
    at: Pos,
) -> Option<&'a ForageGround> {
    let rules = content.forage()?;
    if level.water_at(content, at).is_some() {
        return None;
    }
    let terrain = match level.kind {
        LevelKind::Overworld => level.terrain_at(at)?,
        _ => {
            parse_local_area(&level.id)?;
            let exit = level.exit.as_ref()?;
            *content.level(&exit.to)?.terrain.get(exit.pos)?
        }
    };
    rules.ground(content.terrain(terrain).landscape?)
}

impl World {
    /// The player begins to forage; refused, taking no time, untrained or out of the wild.
    pub(super) fn start_forage(&mut self) -> Spent {
        let content = self.content.clone();
        let me = self.state.player();
        if content.forage().is_none() || !crate::ability::Ability::Forage.had_by(me, &content) {
            self.say("You do not know how to live off the land.");
            return Spent::Nothing;
        }
        let Some(rules) = content.forage() else {
            return Spent::Nothing;
        };
        if forage_ground(&content, self.state.current(), self.player_pos()).is_none() {
            self.say("There is nothing to forage here.");
            return Spent::Nothing;
        }
        self.say("You search the land about you for something to eat.");
        Spent::Forage(rules.seconds)
    }

    /// Forages turn after turn until its time is up or something stops it (module docs): its
    /// last turn cut short to end exactly when its time is up, whatever the player's speed, as
    /// a rest's is.
    pub(super) fn forage(&mut self, seconds: u32, events: &mut Vec<Event>) {
        let until = self.state.time + Duration::seconds(u64::from(seconds));
        let turn = self.price_of(self.state.player, ActionKind::Forage);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => self.forage_done(),
            Ended::Came(id) => {
                let text = format!(
                    "{} comes into view; you stop foraging.",
                    self.creature_called(id)
                );
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You stop foraging."),
        }
    }

    /// A forage ran its time: the chance, and the find (module docs).
    fn forage_done(&mut self) {
        let content = self.content.clone();
        let Some(ground) = forage_ground(&content, self.state.current(), self.player_pos()) else {
            self.say("You find nothing worth eating.");
            return;
        };
        let bonus = self.state.player().skill_effect(&content, Effect::Forage);
        let chance = (i32::from(ground.chance) + bonus).clamp(0, 100);
        let found = self.state.rng.chance(chance.unsigned_abs());
        let find = found
            .then(|| pick_weighted(&mut self.state.rng, &ground.finds).copied())
            .flatten();
        match find {
            Some(object) => {
                let count = ground.count.roll(&mut self.state.rng).max(1);
                let mut stacks = [Stack::new(object, count.unsigned_abs())];
                crate::rot::made_at(&mut stacks, &content, self.state.time);
                let text = format!("You gather {}.", object::phrase(&stacks, self.lore()));
                self.say(text);
                let [stack] = stacks;
                object::add_to_inventory(self.state.inventory_mut(), stack, &content);
            }
            None => self.say("You find nothing worth eating."),
        }
        if self.trains(Use::Forage) {
            self.train(Use::Forage);
        }
    }
}
