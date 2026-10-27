//! Using a rod (spec §8 "Magic items": "Rods", the magic design §9): a rod in the missile weapon
//! slot is used as a missile weapon is (`t`, `fire:X,Y`), as a wand is, and does the effect it
//! holds (`data/rod_effects/`, [`crate::content::RodEffect`]):
//!
//! - **Mana**: each use costs the user the rod's `mana`, cut by Channeler
//!   (`TalentMods::rod_cost`, [`Actor::rod_base`]) and then spent through
//!   [`World::spend_mana`] (Frugal caster's cut, a Willpower training mark), so the rod never
//!   runs out but its user does. Refused, taking no time, without the mana or while a status
//!   blocks casting (Raging).
//! - **Caster level**: the user's ([`World::caster_level`]): the effect reaches a cell further
//!   per its `per_levels` levels, up to its `max_range`, and a status effect that says
//!   `up_to_caster_level` takes only creatures whose level is at most it.
//! - **Effects**, each on existing code: `dig` breaks the walls on its line
//!   ([`World::break_wall`], as dug through) until one too hard for its `power`, a door, or the
//!   level's edge stops it; `teleport` moves the first creature on its bolt to a free cell of the
//!   level ([`World::teleport`], the teleport trap's), an attack on it as a blow is (a shopkeeper
//!   turns on the user); `status` gives everyone in its burst but the user its status
//!   ([`World::afflict`], as an area hex does).
//! - **Identification**: a rod type is unknown ("a bone rod", [`crate::scroll_label`]) until one
//!   is used or identified; there is nothing more to know of one rod than its type.
//! - **Time**: the `cast` action's.
//!
//! No skill applies (the owner: there is no Use magic device skill): a rod costs its own mana and
//! reaches by its user's caster level, whoever holds it.
//!
//! Determinism: nothing here draws; a teleport's landing cell is the teleport trap's hash.

use super::spell::burst_cells;
use super::talk::capitalized;
use super::{Actor, ActorId, Spent, World};
use crate::action::{Breach, Event};
use crate::content::{ActionKind, Content, Rod, RodDoes, RodEffect, RodShape};
use crate::geom::Pos;

impl Actor {
    /// Where in its inventory the rod it has ready is: the first stack in a `ready` slot (the
    /// missile weapon slot) that is a rod.
    pub fn rod_item(&self, content: &Content) -> Option<usize> {
        content.slot_ids().find_map(|(slot, def)| {
            if def.action != ActionKind::Ready {
                return None;
            }
            let item = self.equipment.get(slot)?;
            let stack = self.inventory.get(item)?;
            content.object(stack.object).rod.map(|_| item)
        })
    }

    /// What a use of `rod` costs it before [`World::spend_mana`]'s cut: the rod's `mana` with
    /// Channeler's (`TalentMods::rod_cost`).
    pub fn rod_base(&self, content: &Content, rod: &Rod) -> i32 {
        self.talent_mods(content).rod_cost(i32::from(rod.mana))
    }

    /// What a use of `rod` costs it in all: [`Actor::rod_base`], then Frugal caster's cut
    /// ([`Actor::mana_cost`]).
    pub fn rod_cost(&self, content: &Content, rod: &Rod) -> i32 {
        self.mana_cost(content, self.rod_base(content, rod))
    }
}

impl World {
    /// The player uses the rod in inventory stack `item` at cell `at` (module docs).
    pub(super) fn use_rod(&mut self, item: usize, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        let Some(rod) = content.object(stack.object).rod else {
            return Spent::Nothing;
        };
        if !self.can_loose_here() {
            return Spent::Nothing;
        }
        let here = self.player_pos();
        if at == here {
            self.say("You cannot zap at yourself.");
            return Spent::Nothing;
        }
        let effect = content.rod_effect(rod.effect);
        let level = self.caster_level(player);
        let reach = effect.reach(level);
        if effect.shape == RodShape::Burst && !self.within_reach(at, reach, "zap") {
            return Spent::Nothing;
        }
        let me = self.state.player();
        if let Some(status) = me.effects.blocking(&content, ActionKind::Cast) {
            let name = content.status(status).name.to_lowercase();
            self.say(format!("You cannot use a rod while {name}."));
            return Spent::Nothing;
        }
        let base = me.rod_base(&content, &rod);
        if !self.spend_mana(player, base) {
            self.say("You have not the mana to use it.");
            return Spent::Nothing;
        }
        self.say_rod(item);
        match effect.does {
            RodDoes::Dig { power } => self.rod_dig(effect, (here, at, reach), power, events),
            RodDoes::Teleport => self.rod_teleport(effect, (here, at, reach), events),
            RodDoes::Status {
                status,
                severity,
                up_to_caster_level,
            } => {
                let limit = up_to_caster_level.then_some(level);
                let center = self.burst_center(here, at, reach);
                self.rod_status(effect, center, (status, severity), limit, events);
            }
        }
        Spent::Turn(self.price_of(player, ActionKind::Cast))
    }

    /// Says the rod in inventory stack `item` was used: "You zap the bone rod." A rod type used
    /// for the first time is identified: "It is a rod of digging."
    fn say_rod(&mut self, item: usize) {
        let Some(rod) = self.state.player().inventory.get(item).cloned() else {
            return;
        };
        let title = self.lore().title(&rod);
        self.say(format!("You zap the {title}."));
        if self.learn_scroll(rod.object) {
            let what = self.content.object(rod.object).describe(1);
            self.say(format!("It is {what}."));
        }
    }

    /// A digging effect's line from `from` toward `at`, `reach` cells: every wall on it that
    /// breaks and is softer than `power` gives way; the first that does not, and anything else
    /// that blocks sight (a closed door), stops it. Open cells it passes through.
    fn rod_dig(
        &mut self,
        effect: &RodEffect,
        (from, at, reach): (Pos, Pos, i32),
        power: u16,
        events: &mut Vec<Event>,
    ) {
        self.sound_at(effect.noise, from);
        let content = self.content.clone();
        let mut broken = 0;
        for p in from.ray(at).take(reach.max(1) as usize) {
            let level = self.state.current();
            let hardness = (level.terrain.get(p))
                .and_then(|t| content.terrain(*t).breaks)
                .map(|b| b.hardness);
            match hardness {
                Some(h) if h < power && level.breaks_at(&content, p) => {
                    self.break_wall(p, Breach::Dug, events);
                    broken += 1;
                }
                _ if level.is_opaque(&content, p) || level.terrain.get(p).is_none() => break,
                _ => {}
            }
        }
        self.say(if broken > 0 {
            "The stone crumbles away before you."
        } else {
            "Nothing gives way."
        });
    }

    /// A teleport effect's bolt from `from` toward `at`, `reach` cells or to the first wall: the
    /// first creature on it is moved to a free cell of the level.
    fn rod_teleport(
        &mut self,
        effect: &RodEffect,
        (from, at, reach): (Pos, Pos, i32),
        events: &mut Vec<Event>,
    ) {
        self.sound_at(effect.noise, from);
        let path = self.bolt_line(from, at, reach);
        let level = self.state.current_level.clone();
        let hit = (path.iter()).find_map(|&p| self.state.actor_at(&level, p).map(|a| a.id));
        let Some(target) = hit else {
            self.say("The bolt strikes nothing.");
            return;
        };
        let pos = self.state.actors[&target].pos;
        let seen = self.perception.sees_actors_at(pos);
        let who = capitalized(&self.called(&self.state.actors[&target]));
        self.teleport(target, pos, events);
        // An attack, as a blow is: a shopkeeper or a court turns on the user.
        self.provoke(target);
        self.provoke_court(target);
        self.break_empathy(target);
        if seen {
            self.say(format!("{who} vanishes!"));
        }
    }

    /// A status effect's burst at `center`: everyone in it but the user suffers `status` at
    /// `severity`; with a `limit`, a creature of a higher level shrugs it off.
    fn rod_status(
        &mut self,
        effect: &RodEffect,
        center: Pos,
        (status, severity): (crate::content::StatusId, u16),
        limit: Option<i32>,
        events: &mut Vec<Event>,
    ) {
        self.sound_at(effect.noise, center);
        let player = self.state.player;
        let level = self.state.current();
        let opaque = |p: Pos| p != center && level.is_opaque(&self.content, p);
        let cells = burst_cells(center, i32::from(effect.radius), opaque);
        let level = self.state.current_level.clone();
        let caught: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.level == level && a.id != player && cells.contains(&a.pos))
            .map(|a| a.id)
            .collect();
        let content = self.content.clone();
        let word = (content.status(status).seen.clone()).unwrap_or_else(|| "affected".into());
        for id in caught {
            let actor = &self.state.actors[&id];
            let seen = self.perception.sees_actors_at(actor.pos);
            let who = capitalized(&self.called(actor));
            let creature_level = i32::from(content.creature(actor.creature).level);
            if limit.is_some_and(|l| creature_level > l) {
                if seen {
                    self.say(format!("{who} shrugs it off."));
                }
                continue;
            }
            self.afflict(id, status, severity, Some(player), events);
            if seen {
                self.say(format!("{who} is {word}."));
            }
        }
    }
}
