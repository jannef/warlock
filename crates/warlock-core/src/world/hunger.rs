//! Hunger in the turn pipeline (spec §8 "Hunger", [`crate::hunger`]).
//!
//! Game time burns the player's nutrition as the scheduler moves the clock, so long actions,
//! rests, and overworld journeys burn it too, at their race's hunger: it is one of the things
//! that happen while the player lives through the time ([`World::pass_time`], `status.rs`), each
//! state change a moment of its own ([`World::until_hungrier`]), so it makes no difference how
//! the time is cut up. Fighting burns a little more ([`World::exert`]). The state it puts them
//! in takes effect:
//! - **Weak** (and worse) saps Strength and Dexterity where combat and burden use them
//!   ([`super::Actor::effective_stats`]);
//! - **Fainting** (and Starving): at the start of each turn the player may faint (one draw of the
//!   gameplay stream), losing turns while the others act ([`World::faint`]);
//! - **Starving**: no hit points come back ([`World::healing_rate`], the one pace), and some are
//!   lost by the clock ([`World::digest`]), until the player starves to death;
//! - **Bloated**: every price the player pays is raised, as a burden raises it
//!   ([`World::bloated`], applied in `schedule.rs`).
//!
//! The player is told when their state changes; getting hungrier (into Hungry or worse) is a
//! message that stops a rest, the rest are only logged.

use super::{ActorId, Cause, Death, World};
use crate::content::ActionKind;
use crate::hunger::Hunger;
use crate::time::GameTime;

impl World {
    /// How much the player needs to eat, in percent of a human: their race's (`hunger` in
    /// `data/races.toml`), 100 without a character.
    pub(super) fn hunger_percent(&self) -> u16 {
        (self.state.character.as_ref())
            .and_then(|c| c.spec.race(&self.content).ok())
            .map_or(100, |r| r.hunger)
    }

    /// How hungry the player is; `None` when nobody hungers.
    pub(super) fn player_hunger(&self) -> Option<Hunger> {
        self.state.player().hunger(&self.content)
    }

    /// How long from now, in seconds, until hunger next changes something for actor `id` (at
    /// least 1): its next state ([`Hunger`]) while it is not Starving, else the next hit point
    /// starving costs it. `None` for an actor that does not hunger.
    pub(super) fn until_hungrier(&self, id: ActorId) -> Option<u64> {
        let def = self.content.hunger()?;
        let fed = self.state.actors.get(&id)?.satiation?;
        let state = def.state(fed.nutrition);
        Some(
            match def.least(state) {
                Some(least) => {
                    let points = u64::try_from(i64::from(fed.nutrition) - i64::from(least) + 1);
                    let per_point = u64::from(def.burn_seconds.max(1)) * 100;
                    let need = points.unwrap_or(1).saturating_mul(per_point) - fed.banked;
                    need.div_ceil(u64::from(self.hunger_percent().max(1)))
                }
                None => {
                    u64::from(def.starve.seconds.max(1)).saturating_sub(u64::from(fed.starving))
                }
            }
            .max(1),
        )
    }

    /// `seconds` of game time pass for actor `id`'s hunger, within which its state stays as it is
    /// ([`World::until_hungrier`]): its nutrition burns, and while it is Starving the time costs
    /// hit points. Returns the hit points lost, and its state before.
    pub(super) fn digest(&mut self, id: ActorId, seconds: u64) -> (i32, Option<Hunger>) {
        let content = self.content.clone();
        let Some(def) = content.hunger() else {
            return (0, None);
        };
        let percent = self.hunger_percent();
        let Some(fed) = (self.state.actors.get_mut(&id)).and_then(|a| a.satiation.as_mut()) else {
            return (0, None);
        };
        let before = def.state(fed.nutrition);
        fed.burn(seconds, percent, def.burn_seconds);
        let lost = if before == Hunger::Starving {
            fed.starve(seconds, def.starve.seconds, def.starve.damage)
        } else {
            fed.starving = 0;
            0
        };
        (lost, Some(before))
    }

    /// The player exerts themselves doing `kind` (a blow, a shot, a throw): it burns the extra
    /// nutrition `data/hunger.toml` gives it, at their race's hunger.
    pub(super) fn exert(&mut self, kind: ActionKind) {
        let content = self.content.clone();
        let (Some(def), Some(before)) = (content.hunger(), self.player_hunger()) else {
            return;
        };
        let points = def.exertion(kind);
        if points == 0 {
            return;
        }
        let percent = self.hunger_percent();
        if let Some(fed) = self.player_mut().satiation.as_mut() {
            let seconds = u64::from(points) * u64::from(def.burn_seconds);
            fed.burn(seconds, percent, def.burn_seconds);
        }
        self.hunger_news(before);
    }

    /// As a turn begins, a player who is Fainting or Starving may faint (one draw of the gameplay
    /// stream, while the chance is not 0): they lose turns, a wait each, while the others act.
    /// Returns the price of the turns lost (0 if they did not faint).
    pub(super) fn faint(&mut self) -> i64 {
        let content = self.content.clone();
        let Some(def) = content.hunger() else {
            return 0;
        };
        if def.faint.chance == 0 || !self.player_hunger().is_some_and(Hunger::faints) {
            return 0;
        }
        if !self.state.rng.chance(u32::from(def.faint.chance)) {
            return 0;
        }
        self.say("You faint from hunger.");
        let wait = self.price_of(self.state.player, ActionKind::Wait);
        wait.saturating_mul(i64::from(def.faint.turns))
    }

    /// `price` as the player pays it while Bloated: raised by `[bloated] slow` percent (rounded
    /// up to a whole unit of energy), like a burden; as it is for anyone else.
    pub(super) fn bloated(&self, id: super::ActorId, price: i64) -> i64 {
        let Some(def) = self.content.hunger() else {
            return price;
        };
        let bloated = (self.state.actors.get(&id))
            .is_some_and(|a| a.hunger(&self.content) == Some(Hunger::Bloated));
        if !bloated {
            return price;
        }
        let slow = i64::from(def.bloated_slow);
        price.saturating_mul(slow).saturating_add(99) / 100
    }

    /// Says how the player's hunger changed since it was `before`: getting hungrier, into Hungry
    /// or worse, stops a rest; satiation wearing off and being fed are only logged.
    pub(super) fn hunger_news(&mut self, before: Hunger) {
        let Some(now) = self.player_hunger() else {
            return;
        };
        let Some(text) = Hunger::news(before, now) else {
            return;
        };
        if Hunger::stops_rest(before, now) {
            self.say(text);
        } else {
            self.note(text);
        }
    }

    /// The player dies of `cause` at moment `when`, hunger or overeating: the run ends there (no
    /// creature killed them, so no [`crate::Event::Died`]).
    pub(super) fn die_of(&mut self, cause: Cause, when: GameTime) {
        if self.state.death.is_some() {
            return;
        }
        let player = self.player_mut();
        player.hp = player.hp.min(0);
        self.state.death = Some(Death {
            turn: self.state.turn,
            time: when,
            place: self.state.current().name.clone(),
            by: None,
            of: None,
            cause: Some(cause),
        });
        let text = cause.message(&self.content);
        self.say(text);
    }
}
