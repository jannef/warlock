//! The birth sign's gifts that are more than one read at a hook (spec §8 "Birth sign"; the
//! talents of the `stars` tree): the two actives, Horn's grace and the Transfixing stare, used
//! from the abilities menu's Birth sign group ([`crate::ability::Ability::Sign`]) as
//! [`Action::Gift`]; Relentless's streak of blows at one creature ([`Streak`], on
//! [`Actor::streak`](crate::world::Actor)); Barbed hide's barbs; and Slip away's cheaper step out of
//! reach. Each refuses or does nothing for an actor without its talent. Nothing here draws at
//! random.
//!
//! [`Action::Gift`]: crate::action::Action::Gift

use crate::ability::{Ability, Unavailable};
use crate::action::{AttackKind, Event};
use crate::combat::{Profile, Strike};
use crate::content::{ActionKind, SignActive, Stance};
use crate::damage::{Damage, Rider};
use crate::geom::Pos;
use crate::status::Effect;
use crate::text::capitalized;
use crate::world::combat::fight::Harm;
use crate::world::{ActorId, Spent, World};
use serde::{Deserialize, Serialize};

/// A streak of melee blows at one creature (Relentless): who they were at, how many in a row,
/// and the player's turn of the last ([`crate::world::WorldState::turn`]). A blow continues it while
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
    pub(in crate::world) fn use_gift(
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
            (SignActive::FireBreath, Some(at)) => self.fire_breath(at, events),
            (SignActive::FireBreath, None) => {
                self.say("Where to?");
                Spent::Nothing
            }
            (SignActive::HoundsBay, _) => self.hounds_bay(events),
        }
    }

    /// Horn's grace: the statuses it cures end (Bleeding, Poisoned) and the first diseases that
    /// show are shaken off, then its cooldown, for the time of a wait; with Horn of plenty
    /// (`sign_picks.rs`) hit points come back too, and the cooldown is shorter. Refused, taking
    /// no time, with nothing it would mend.
    fn horns_grace(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let me = self.state.player();
        let mods = me.talent_mods(&content).into_owned();
        let Some(grace) = mods.sign.horns_grace else {
            return Spent::Nothing;
        };
        let (heal, degrees) = self.horn_of_plenty(mods.sign.horn_of_plenty, grace.degrees);
        let me = self.state.player();
        let ended: Vec<Effect> = (me.effects.iter())
            .filter(|e| grace.cures.contains(&e.status))
            .copied()
            .collect();
        let diseases: Vec<_> = (me.diseases.iter())
            .filter(|i| i.shown)
            .map(|i| i.disease)
            .take(usize::from(grace.diseases))
            .collect();
        if ended.is_empty() && diseases.is_empty() && heal == 0 {
            self.say("There is nothing for your horn's grace to mend.");
            return Spent::Nothing;
        }
        self.say("A clean warmth runs through your blood.");
        self.player_mut().hp += heal;
        for e in &ended {
            self.player_mut().effects.remove(e.status);
        }
        self.ended(player, &ended, events);
        for disease in diseases {
            self.end_disease(player, disease, events);
        }
        self.gift_cooldown(grace.cooldown, degrees, events);
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }

    /// The Transfixing stare at `at`: a hostile there, within its talent's `range` (more under a
    /// full moon), that the player sees and that sees them now suffers the stare's status, and the
    /// player its cooldown; one that does not see them is not caught, and the cooldown waits.
    /// Silent; the time of a wait. Refused, taking no time, at the player's own cell, out of reach, where they see no
    /// one, and at anyone not hostile.
    fn transfixing_stare(&mut self, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let ability = Ability::Sign(SignActive::TransfixingStare);
        let player = self.state.player;
        let mods = self.state.player().talent_mods(&content);
        let Some(stare) = mods.sign.stare else {
            return Spent::Nothing;
        };
        // Petrifying gaze (`sign_picks.rs`): held longer, rested sooner.
        let (severity, degrees) = mods
            .sign
            .petrifying_gaze
            .unwrap_or((stare.severity, stare.degrees));
        // A full moon lengthens it (spec §8 "Birth sign", the flavour).
        let Some(range) = ability.reach_at(&content, self.state.time) else {
            return Spent::Nothing;
        };
        if !self.within_reach(at, range, "stare") {
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
        self.afflict(target, stare.status, severity, Some(player), events);
        self.say(format!("{who} freezes under your gaze."));
        self.gift_cooldown(stare.cooldown, degrees, events);
        Spent::Turn(price)
    }

    /// `profile`, a blow of `kind` by `attacker` at `target`, with Relentless's to-hit: +1 for
    /// each blow before it in the streak at `target`, to the talent's most. The blow continues
    /// the streak (or starts one), hit or miss. Melee only; an actor without the talent keeps
    /// no streak.
    pub(in crate::world) fn relentless(
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
        let mods = me.talent_mods(&self.content);
        let max = mods.offence.streak;
        if max == 0 {
            return profile;
        }
        let before = (me.streak)
            .filter(|s| s.target == target && turn <= s.turn + 1)
            .map_or(0, |s| s.count);
        profile.to_hit += i32::from(before.min(max));
        // Relentless fury: every so many blows before it bite +1 harder, as far as the streak
        // counts.
        if let Some(more) = before
            .min(max)
            .checked_div(mods.offence.streak_damage_every)
        {
            profile.bonus += i32::from(more);
        }
        me.streak = Some(Streak {
            target,
            count: before.saturating_add(1),
            turn,
        });
        profile
    }

    /// Barbed hide: `target`, hit in melee by `attacker` and alive, cuts it for its talent's
    /// damage. Not a blow: no roll, no noise, and if it kills, no one earns anything. A status
    /// that strikes back ([`World::retaliate`]) answers first.
    pub(in crate::world) fn barbed(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        events: &mut Vec<Event>,
    ) {
        if kind != AttackKind::Melee || attacker == target {
            return;
        }
        self.retaliate(attacker, target, events);
        let barbs = (self.state.actors.get(&target))
            .filter(|a| !a.talents.is_empty())
            .map_or(0, |a| a.talent_mods(&self.content).defence.barbs);
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

    /// A melee blow from `attacker` landed on `target`: each of its statuses that `retaliates`
    /// (Fire cloak) strikes back, a spell of the target's in all but the casting: said by the
    /// damage type's words ("The fire burns the kobold."), resisted, crediting the target with a
    /// kill, and the type's rule on the striker if it was harmed. A draw for the dice, and the
    /// rule's own; none for one without such a status.
    fn retaliate(&mut self, attacker: ActorId, target: ActorId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get(&target) else {
            return;
        };
        let from = me.pos;
        let riders: Vec<(Rider, &str)> = (me.effects.iter())
            .filter_map(|e| {
                let def = content.status(e.status);
                def.retaliates.map(|r| (r, def.name.as_str()))
            })
            .collect();
        for (rider, status) in riders {
            if !self.state.actors.contains_key(&attacker) || self.state.death.is_some() {
                return;
            }
            let damage = Damage::new(rider.dice.roll(&mut self.state.rng), Some(rider.kind));
            let hit = Strike::Hit { critical: false };
            let mut made = Vec::new();
            self.spell_strikes(target, attacker, (from, false), (damage, hit), &mut made);
            let harmed = (made.iter()).any(|e| {
                matches!(
                    e,
                    Event::Attacked {
                        strike: Some(Strike::Hit { .. }),
                        ..
                    }
                )
            });
            // Fire burns ("The fire burns you."); a physical kind strikes by the status's name
            // (an innate Barbed hide: "The barbed hide stabs you.").
            let def = content.damage_type(rider.kind);
            let noun = match def.physical {
                true => status.to_lowercase(),
                false => def.name.clone(),
            };
            self.say_spell((&noun, &def.verb), made, events);
            if harmed {
                self.spell_type_lands(target, Some(rider.kind), &[attacker], events);
            }
        }
    }

    /// `price`, actor `id`'s step to `to`, as Slip away makes it: its percent off when some
    /// hostile stands beside it and the step takes it out of reach of every one.
    pub(in crate::world) fn slipped(&self, id: ActorId, to: Pos, price: i64) -> i64 {
        let Some(me) = self.state.actors.get(&id) else {
            return price;
        };
        let percent = if me.talents.is_empty() {
            0
        } else {
            me.talent_mods(&self.content).defence.slip_percent
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
