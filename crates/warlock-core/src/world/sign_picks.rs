//! The birth signs' picks that are more than one read at a hook (spec §8 "Birth sign"; the
//! sign-limited talents of the `stars` tree, offered on the level-up screen and the creation
//! Talents step only to those born under their sign): the two new actives, **Fire breath** (the
//! Wyvern's, `use:fire_breath:X,Y`) and **Hound's bay** (the Barghest's, `use:hounds_bay`), used
//! from the abilities menu's Birth sign group and dispatched with the other actives by
//! `World::use_gift` in `gifts.rs`; what a frightened creature does ([`World::frightened`],
//! read by the AI); and Horn of plenty's healing. The picks that change a gift's numbers are
//! read where the gift's rule lives: Relentless fury in `World::relentless`, Petrifying gaze in
//! the stare, Wildfire in `World::blaze`, Second rising in `World::rise_from_ashes`; the
//! passive picks are one read each at their hooks (Another head in `World::healing_rate`, Deep
//! roots in `eat.rs`, Undertow in `World::step_price`, Tentacles in `Actor::off_hand_blow`,
//! Tempered in `Actor::benefit`).
//!
//! **Fire breath** is a bolt cast with the scrolls' shared code ([`World::cast_bolt`]): from the
//! player along the line toward the cell aimed at (within [`BREATH_RANGE`]) and on, as far as
//! its length or the first opaque cell, one gameplay roll of its damage for everyone on it; its
//! noise is a spell's; then its cooldown. Refused, taking no time, without the pick, while the
//! cooldown lasts, at the player's own cell, out of range, and on the overworld.
//!
//! **Hound's bay** is a howl ([`Act::Howl`], [`crate::noise::HOWL_NOISE`]): every hostile that
//! hears it (the hearing rules of every noise, its threshold and the walls between) and whose
//! creature level is below the player's suffers its status (Frightened, one that `flees`),
//! caused by the player; then its cooldown. No draw: no check (the owner's "unfair"). Refused,
//! taking no time, on the overworld, which is silent.
//!
//! **Frightened**: a creature suffering a status that `flees` steps away from the one who gave
//! it, to the free cell beside it farthest from them (Chebyshev, then straight-line distance;
//! the first in direction order on a tie), while a step takes it farther; cornered, it does what
//! it would have done.

use super::ai::Plan;
use super::spell::Caster;
use super::{ActorId, Spent, World};
use crate::action::{AttackKind, Event};
use crate::content::{SignActive, Stance};
use crate::geom::{Dir, Pos};
use crate::noise::{self, Act};

/// How far the cell Fire breath is aimed at may be, in cells (a disc, as ranges are); the bolt
/// itself runs its own length along that line.
pub const BREATH_RANGE: i32 = 8;

impl World {
    /// Horn of plenty (`plenty`: hit points per level and the cooldown's degrees) on a Horn's
    /// grace whose cooldown is `degrees`: the hit points it gives back (never past the
    /// maximum) and the degrees of cooldown it leaves. Without it, none and `degrees`.
    pub(super) fn horn_of_plenty(&self, plenty: Option<(u8, u16)>, degrees: u16) -> (i32, u16) {
        let Some((per_level, fewer)) = plenty else {
            return (0, degrees);
        };
        let me = self.state.player();
        let missing = (me.max_hp(&self.content) - me.hp).max(0);
        let heal = i32::from(per_level) * i32::from(self.player_level());
        (heal.min(missing), fewer)
    }

    /// Fire breath at `at` (module docs), for the time of a `cast`.
    pub(super) fn fire_breath(&mut self, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(breath) = self
            .state
            .player()
            .talent_mods(&content)
            .into_owned()
            .fire_breath
        else {
            return Spent::Nothing;
        };
        if !self.can_loose_here() || !self.within_reach(at, BREATH_RANGE, "breathe fire") {
            return Spent::Nothing;
        }
        let price = self.price_of(player, SignActive::FireBreath.cost());
        let here = self.player_pos();
        let path = self.bolt_line(here, at, breath.bolt.length);
        self.say("Fire roars from your throat!");
        self.make_noise(player, Act::Attack(AttackKind::Spell));
        let mut made = Vec::new();
        self.cast_bolt(Caster::Actor(player), here, &path, &breath.bolt, &mut made);
        let noun = content
            .sign_active(SignActive::FireBreath)
            .map_or_else(|| "fire breath".to_string(), |def| def.name.to_lowercase());
        let verb = (breath.bolt.kind).map_or("burns", |k| content.damage_type(k).verb.as_str());
        self.say_spell((&noun, verb), made, events);
        if self.state.death.is_none() {
            self.gift_cooldown(breath.cooldown, breath.degrees, events);
        }
        Spent::Turn(price)
    }

    /// Hound's bay (module docs), for the time of a wait.
    pub(super) fn hounds_bay(&mut self, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(bay) = self.state.player().talent_mods(&content).hounds_bay else {
            return Spent::Nothing;
        };
        if !self.can_loose_here() {
            return Spent::Nothing;
        }
        let price = self.price_of(player, SignActive::HoundsBay.cost());
        self.say("You throw back your head and howl!");
        // Who hears it: the same noise the howl makes, before it is made.
        let me = self.state.player();
        let burden = self.state.load(&content, me).penalty();
        let howl = noise::noise(&content, self.state.current(), me, burden, Act::Howl);
        let heard = howl.map_or_else(Vec::new, |n| noise::hearers(&self.state, &content, n));
        self.make_noise(player, Act::Howl);
        let mine = self.player_level();
        let mut fled = Vec::new();
        for (id, _) in heard {
            let Some(them) = self.state.actors.get(&id) else {
                continue;
            };
            if self.stance(player, id) != Stance::Hostile
                || content.creature(them.creature).level >= mine
            {
                continue;
            }
            if self.perception.sees_actors_at(them.pos) {
                fled.push(super::talk::capitalized(&self.called(them)));
            }
            self.afflict(id, bay.status, bay.severity, Some(player), events);
        }
        for who in fled {
            self.say(format!("{who} cowers and flees from you."));
        }
        self.gift_cooldown(bay.cooldown, 1, events);
        Spent::Turn(price)
    }

    /// `plan`, what creature `id` was about to do, as fear changes it (module docs): a step
    /// away from the one who frightened it while one takes it farther; else `plan`.
    pub(super) fn frightened(&self, id: ActorId, plan: Plan) -> Plan {
        let content = &self.content;
        let Some(me) = self.state.actors.get(&id) else {
            return plan;
        };
        let Some(fear) = (me.effects.iter()).find(|e| content.status(e.status).flees) else {
            return plan;
        };
        let from = (fear.by.and_then(|by| self.state.actors.get(&by)))
            .filter(|them| them.level == me.level)
            .map(|them| them.pos);
        let (Some(from), Some(level)) = (from, self.state.levels.get(&me.level)) else {
            return plan;
        };
        let away = |p: Pos| (p.chebyshev(from), p.dist2(from));
        let shuns = self.shuns(id);
        let best = (Dir::ALL.iter().copied())
            .map(|dir| (dir, me.pos.step(dir)))
            .filter(|&(_, to)| level.is_walkable(content, to) && !shuns(to))
            .filter(|&(_, to)| self.state.actor_at(&me.level, to).is_none())
            .filter(|&(_, to)| away(to) > away(me.pos))
            .fold(None::<(Dir, Pos)>, |best, step| match best {
                Some(b) if away(b.1) >= away(step.1) => Some(b),
                _ => Some(step),
            });
        best.map_or(plan, |(dir, _)| Plan::Step(dir))
    }
}
