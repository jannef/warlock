//! The birth sign's gifts that are more than one read at a hook (spec §8 "Birth sign"; the
//! talents of the `stars` tree): the two actives, Horn's grace and the Transfixing stare, used
//! from the abilities menu's Birth sign group ([`crate::ability::Ability::Sign`]) as
//! [`Action::Gift`]; Relentless's streak of blows at one creature ([`Streak`], on
//! [`Actor::streak`](super::Actor)); Barbed hide's barbs; and Slip away's cheaper step out of
//! reach. Each refuses or does nothing for an actor without its talent. Nothing here draws at
//! random.
//!
//! [`Action::Gift`]: crate::action::Action::Gift

use super::fight::Harm;
use super::talk::capitalized;
use super::{ActorId, Spent, World};
use crate::ability::{Ability, Unavailable};
use crate::action::{AttackKind, Event};
use crate::combat::Profile;
use crate::content::{ActionKind, SignActive, Stance};
use crate::geom::Pos;
use crate::status::Effect;
use serde::{Deserialize, Serialize};

/// How far the Transfixing stare reaches, in cells (a disc, as ranges are).
pub const STARE_RANGE: i32 = 6;

/// A streak of melee blows at one creature (Relentless): who they were at, how many in a row,
/// and the player's turn of the last ([`super::WorldState::turn`]). A blow continues it while
/// it is at the same creature in the same turn or the next; anything else starts it over.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Streak {
    pub target: ActorId,
    pub count: u8,
    pub turn: u64,
}

impl World {
    /// `use:ID[:X,Y]` ([`Action::Gift`](crate::action::Action::Gift)): the player uses the
    /// birth sign's active `active`, at cell `at` if it asks for one. Refused, taking no time,
    /// without it, while its cooldown lasts, and without a cell it asks for.
    pub(super) fn use_gift(
        &mut self,
        active: SignActive,
        at: Option<Pos>,
        events: &mut Vec<Event>,
    ) -> Spent {
        let content = self.content.clone();
        let ability = Ability::Sign(active);
        let me = self.state.player();
        if !ability.had_by(me, &content) {
            self.say("You have no such gift.");
            return Spent::Nothing;
        }
        if let Some(why) = ability.unavailable(me, &content) {
            self.say(match why {
                Unavailable::While(status) => format!(
                    "You are still {}.",
                    content.status(status).name.to_lowercase()
                ),
                other => format!("You cannot: {}.", other.reason(&content)),
            });
            return Spent::Nothing;
        }
        match (active, at) {
            (SignActive::HornsGrace, _) => self.horns_grace(events),
            (SignActive::TransfixingStare, Some(at)) => self.transfixing_stare(at, events),
            (SignActive::TransfixingStare, None) => {
                self.say("At whom?");
                Spent::Nothing
            }
            (SignActive::Blaze, _) => self.blaze(events),
        }
    }

    /// Horn's grace: the statuses it cures end (Bleeding, Poisoned) and the first diseases that
    /// show are shaken off, then its cooldown, for the time of a wait. Refused, taking no time,
    /// with nothing it would mend.
    fn horns_grace(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let me = self.state.player();
        let Some(grace) = me.talent_mods(&content).horns_grace else {
            return Spent::Nothing;
        };
        let ended: Vec<Effect> = (me.effects.iter())
            .filter(|e| grace.cures.contains(&e.status))
            .copied()
            .collect();
        let diseases: Vec<_> = (me.diseases.iter())
            .filter(|i| i.shown)
            .map(|i| i.disease)
            .take(usize::from(grace.diseases))
            .collect();
        if ended.is_empty() && diseases.is_empty() {
            self.say("There is nothing for your horn's grace to mend.");
            return Spent::Nothing;
        }
        self.say("A clean warmth runs through your blood.");
        for e in &ended {
            self.player_mut().effects.remove(e.status);
        }
        self.ended(player, &ended, events);
        for disease in diseases {
            self.end_disease(player, disease, events);
        }
        self.afflict(player, grace.cooldown, 1, None, events);
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }

    /// The Transfixing stare at `at`: a hostile there, within [`STARE_RANGE`], that the player
    /// sees and that sees them now suffers the stare's status, and the player its cooldown; one
    /// that does not see them is not caught, and the cooldown waits. Silent; the time of a
    /// wait. Refused, taking no time, at the player's own cell, out of reach, where they see no
    /// one, and at anyone not hostile.
    fn transfixing_stare(&mut self, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(stare) = self.state.player().talent_mods(&content).stare else {
            return Spent::Nothing;
        };
        if !self.within_reach(at, STARE_RANGE, "stare") {
            return Spent::Nothing;
        }
        let level = self.state.current().id.clone();
        let target = (self.state.actor_at(&level, at)).map(|a| a.id);
        let Some(target) = target.filter(|_| self.perception.sees_actors_at(at)) else {
            self.say("You see no one there to stare at.");
            return Spent::Nothing;
        };
        let who = capitalized(&self.called(&self.state.actors[&target]));
        if self.stance(player, target) != Stance::Hostile {
            self.say(format!("{who} means you no harm."));
            return Spent::Nothing;
        }
        let price = self.price_of(player, ActionKind::Wait);
        if !self.sees_now(target, player) {
            self.say(format!("{who} does not meet your gaze."));
            return Spent::Turn(price);
        }
        self.afflict(target, stare.status, stare.severity, Some(player), events);
        self.say(format!("{who} freezes under your gaze."));
        self.afflict(player, stare.cooldown, 1, None, events);
        Spent::Turn(price)
    }

    /// `profile`, a blow of `kind` by `attacker` at `target`, with Relentless's to-hit: +1 for
    /// each blow before it in the streak at `target`, to the talent's most. The blow continues
    /// the streak (or starts one), hit or miss. Melee only; an actor without the talent keeps
    /// no streak.
    pub(super) fn relentless(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        mut profile: Profile,
    ) -> Profile {
        let turn = self.state.turn;
        let Some(me) = self.state.actors.get_mut(&attacker) else {
            return profile;
        };
        if kind != AttackKind::Melee || me.talents.is_empty() {
            return profile;
        }
        let max = me.talent_mods(&self.content).streak;
        if max == 0 {
            return profile;
        }
        let before = (me.streak)
            .filter(|s| s.target == target && turn <= s.turn + 1)
            .map_or(0, |s| s.count);
        profile.to_hit += i32::from(before.min(max));
        me.streak = Some(Streak {
            target,
            count: before.saturating_add(1),
            turn,
        });
        profile
    }

    /// Barbed hide: `target`, hit in melee by `attacker` and alive, cuts it for its talent's
    /// damage. Not a blow: no roll, no noise, and if it kills, no one earns anything.
    pub(super) fn barbed(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        events: &mut Vec<Event>,
    ) {
        if kind != AttackKind::Melee || attacker == target {
            return;
        }
        let barbs = (self.state.actors.get(&target))
            .filter(|a| !a.talents.is_empty())
            .map_or(0, |a| a.talent_mods(&self.content).barbs);
        let Some(at) = self.state.actors.get(&attacker).map(|a| a.pos) else {
            return;
        };
        if barbs <= 0 {
            return;
        }
        if target == self.state.player && self.perception.sees_actors_at(at) {
            let who = capitalized(&self.called(&self.state.actors[&attacker]));
            self.say(format!("{who} is cut by your barbs."));
        }
        self.hurt(attacker, barbs, Harm::Barbs, events);
    }

    /// `price`, actor `id`'s step to `to`, as Slip away makes it: its percent off when some
    /// hostile stands beside it and the step takes it out of reach of every one.
    pub(super) fn slipped(&self, id: ActorId, to: Pos, price: i64) -> i64 {
        let Some(me) = self.state.actors.get(&id) else {
            return price;
        };
        let percent = if me.talents.is_empty() {
            0
        } else {
            me.talent_mods(&self.content).slip_percent
        };
        if percent == 0 {
            return price;
        }
        let beside: Vec<Pos> = (self.state.actors.values())
            .filter(|o| o.id != id && o.level == me.level && o.pos.chebyshev(me.pos) == 1)
            .filter(|o| self.stance(id, o.id) == Stance::Hostile)
            .map(|o| o.pos)
            .collect();
        if beside.is_empty() || beside.iter().any(|p| p.chebyshev(to) <= 1) {
            return price;
        }
        crate::content::percent_of(price, u32::try_from((100 + percent).max(1)).unwrap_or(1))
    }
}
