//! Hunger in the turn pipeline (spec §8 "Hunger", [`crate::hunger`]).
//!
//! Game time burns the player's nutrition as the scheduler moves the clock ([`World::digest`],
//! so long actions, rests, and overworld journeys burn it too), at their race's hunger; fighting
//! burns a little more ([`World::exert`]). The state it puts them in takes effect:
//! - **Weak** (and worse) saps Strength and Dexterity where combat and burden use them
//!   ([`super::Actor::effective_stats`]);
//! - **Fainting** (and Starving): at the start of each turn the player may faint (one draw of the
//!   gameplay stream), losing turns while the others act ([`World::faint`]);
//! - **Starving**: no hit points come back (`fight.rs`), and some are lost by the clock, until the
//!   player starves to death;
//! - **Bloated**: every price the player pays is raised, as a burden raises it
//!   ([`World::bloated`], applied in `schedule.rs`).
//!
//! The player is told when their state changes; getting hungrier (into Hungry or worse) is a
//! message that stops a rest, the rest are only logged.

use super::{Cause, Death, World};
use crate::content::ActionKind;
use crate::hunger::Hunger;

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

    /// `seconds` of game time pass for the player (the scheduler moves the clock): their nutrition
    /// burns, and starving costs them hit points, maybe their life. Says how their state changed.
    pub(super) fn digest(&mut self, seconds: u64) {
        let content = self.content.clone();
        let (Some(def), Some(before)) = (content.hunger(), self.player_hunger()) else {
            return;
        };
        let percent = self.hunger_percent();
        let Some(fed) = self.player_mut().satiation.as_mut() else {
            return;
        };
        fed.burn(seconds, percent, def.burn_seconds);
        let lost = if def.state(fed.nutrition) == Hunger::Starving {
            fed.starve(seconds, def.starve.seconds, def.starve.damage)
        } else {
            fed.starving = 0;
            0
        };
        self.hunger_news(before);
        if lost > 0 {
            self.suffer(lost, Cause::Starvation);
        }
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

    /// The player loses `damage` hit points to `cause`, hunger or what they ate: at 0 they die,
    /// and the run ends there (no creature killed them, so no [`crate::Event::Died`]).
    pub(super) fn suffer(&mut self, damage: i32, cause: Cause) {
        if self.state.death.is_some() {
            return;
        }
        let player = self.player_mut();
        player.hp = player.hp.saturating_sub(damage);
        if player.hp > 0 {
            return;
        }
        self.state.death = Some(Death {
            turn: self.state.turn,
            time: self.state.time,
            place: self.state.current().name.clone(),
            by: None,
            cause: Some(cause),
        });
        let text = cause.message(&self.content);
        self.say(text);
    }
}
