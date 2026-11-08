//! Creatures' spells (spec §8 "Creature behavior": "Casters", #1102): a `combat = "caster"`
//! creature casts the spells of its type's `spells` list (`content/creature_spells.rs`) through
//! the player's spell code: the same reach, roll, and degrees at its own stats
//! ([`Cast::by_creature`]), the same areas, blows, spell types, and riders (`spell.rs`,
//! `cast.rs::cast_single`, [`World::spell_lands`]). Enemy-only as a choice of its AI; an ally
//! caster casts the same way at its master's enemies.
//!
//! **Paid in mana**: each cast spends the spell's cost ([`super::Actor::spell_cost`], through
//! [`World::spend_mana`]); a creature's pool is its type's `mana` and its mental stats', and
//! comes back at the usual slow pace. Out of mana it fights as a melee creature (or shoots, with
//! a missile weapon and ammunition), so it never stands idle.
//!
//! **When** ([`World::creature_spell`], asked in `ai.rs::fight` after a summoning, a throw,
//! and a breath): the first spell of its list, in its order, that it may cast now (it has the
//! mana, its casting is not blocked (Silenced, a silent vault), it is not on the overworld) and
//! that goes from where it stands:
//! - a **buff** it does not have yet, on itself;
//! - a **heal** by touch on itself or a friend beside it below half its hit points (itself first);
//! - a **hex** at the hostile it fights, which does not have its status yet and whose type is
//!   not immune to it, in reach;
//! - a **damage** spell at that hostile, in reach.
//!
//! "In reach" is the spell's shape's: a ray or a bolt within its range with a clear line
//! ([`World::clear_shot`]); a touch beside it; a burst within its range with a clear line, the
//! caster outside its cells; a burst about the caster with the hostile in its cells; a cone or
//! a line whose cells hold the hostile, with a clear line to it. An area spell is never cast
//! where its cells hold a friend of the caster's (anyone not hostile to it). Deciding draws
//! nothing. Otherwise the AI closes to the farthest reach of the aimed spells it could cast at
//! the hostile ([`World::spell_reach_at`], at most its type's `range`), and with none, fights
//! as a melee creature.
//!
//! **What** ([`World::creature_casts`]): the mana is spent, the chanting heard (a spell's
//! noise), and the cast said; then the spell does what the player's would. An `attack` spell's
//! roll adds its level's quarter, as its natural attacks do ([`World::creature_aim`]). A cast
//! trains nothing. The action costs a `cast`.
//!
//! **Words**: the player who sees the caster is told what it casts ("The kobold shaman casts
//! spark at you!", "The lich casts haste."); its blows are said as the player's spells' are
//! ("The spark shocks you."), a hex on a creature the player sees as theirs ("The goblin is
//! weakened."), a heal as "The kobold looks better.". Unseen, the chanting is heard.

use super::cast::Cast;
use super::spell::{Caster, burst_cells, cone_cells};
use super::talk::capitalized;
use super::{ActorId, AttackSkills, World};
use crate::action::{AttackKind, Event, SpellShape};
use crate::content::{
    ActionKind, Bolt, Burst, LevelKind, Shape, SpellData, SpellId, SpellPurpose, Stance,
};
use crate::fov::radius2;
use crate::geom::Pos;
use crate::noise::Act;

impl World {
    /// What a creature adds to its `attack` spells' roll: its level's quarter, as its natural
    /// attacks' `to_hit` (#1097); 0 for the player.
    pub(super) fn creature_aim(&self, id: ActorId) -> i32 {
        if id == self.state.player {
            return 0;
        }
        (self.state.actors.get(&id)).map_or(0, |a| {
            i32::from(self.content.creature(a.creature).level) / 4
        })
    }

    /// Whether creature `id` may cast at all now: its casting is not blocked and it is not on
    /// the overworld.
    fn casts_now(&self, id: ActorId) -> bool {
        let me = &self.state.actors[&id];
        self.state.current().kind != LevelKind::Overworld
            && (me.effects.blocking(&self.content, ActionKind::Cast)).is_none()
    }

    /// Whether creature `id` has the mana for `def`.
    fn affords(&self, id: ActorId, def: &SpellData) -> bool {
        let me = &self.state.actors[&id];
        me.mana >= me.spell_cost(&self.content, def)
    }

    /// The spell creature `id` casts now, fighting `target` on `at`, and on whom (module docs).
    pub(super) fn creature_spell(
        &self,
        id: ActorId,
        target: ActorId,
        at: Pos,
    ) -> Option<(SpellId, ActorId)> {
        let me = &self.state.actors[&id];
        let spells = &self.content.creature(me.creature).spells;
        if spells.is_empty() || !self.casts_now(id) {
            return None;
        }
        spells.iter().find_map(|&spell| {
            let def = self.content.spell(spell);
            if !self.affords(id, def) {
                return None;
            }
            let on = match def.kind {
                SpellPurpose::Buff => {
                    let (status, _) = def.effect.status?;
                    (!me.effects.has(status)).then_some(id)
                }
                SpellPurpose::Heal => self.wounded_friend(id),
                SpellPurpose::Hex | SpellPurpose::Damage => (self.worth_aiming(def, target)
                    && self.reaches(id, def, (target, at)))
                .then_some(target),
                _ => None,
            }?;
            Some((spell, on))
        })
    }

    /// The farthest creature `id` would cast at `target` from: the longest reach of the aimed
    /// spells of its list it may cast at it now (module docs), at most its type's `range`;
    /// `None` without one.
    pub(super) fn spell_reach_at(&self, id: ActorId, target: ActorId) -> Option<i32> {
        let me = &self.state.actors[&id];
        let def = self.content.creature(me.creature);
        if def.spells.is_empty() || !self.casts_now(id) {
            return None;
        }
        (def.spells.iter())
            .map(|&s| self.content.spell(s))
            .filter(|s| matches!(s.kind, SpellPurpose::Hex | SpellPurpose::Damage))
            .filter(|s| self.affords(id, s) && self.worth_aiming(s, target))
            .map(|s| {
                let reach = me.spell_reach(&self.content, s);
                match s.shape {
                    Shape::Touch => 1,
                    Shape::Burst if s.about_caster() => reach.radius.unwrap_or(1),
                    _ => reach.range.unwrap_or(1),
                }
            })
            .max()
            .map(|reach| reach.clamp(1, def.range.max(1)))
    }

    /// Whether aiming `def` at `target` would do anything: a damage spell always; a hex only
    /// at one without its status whose type is not immune to it.
    fn worth_aiming(&self, def: &SpellData, target: ActorId) -> bool {
        if def.kind != SpellPurpose::Hex {
            return true;
        }
        let (Some((status, _)), Some(them)) = (def.effect.status, self.state.actors.get(&target))
        else {
            return false;
        };
        !them.effects.has(status)
            && !self
                .content
                .creature(them.creature)
                .immune
                .contains(&status)
    }

    /// Whom creature `id` would heal by touch: itself below half its hit points, else the
    /// first friend beside it (lowest id) below half theirs.
    fn wounded_friend(&self, id: ActorId) -> Option<ActorId> {
        let me = &self.state.actors[&id];
        let hurt = |a: &super::Actor| a.hp * 2 < a.max_hp(&self.content);
        if hurt(me) {
            return Some(id);
        }
        (self.state.actors.values())
            .filter(|a| a.level == me.level && a.id != id && a.pos.chebyshev(me.pos) <= 1)
            .filter(|a| self.stance(id, a.id) != Stance::Hostile && hurt(a))
            .map(|a| a.id)
            .next()
    }

    /// Whether `def`, cast by creature `id` from where it stands, reaches `target` on `at`
    /// (module docs).
    fn reaches(&self, id: ActorId, def: &SpellData, (target, at): (ActorId, Pos)) -> bool {
        let me = &self.state.actors[&id];
        let here = me.pos;
        let reach = me.spell_reach(&self.content, def);
        let range = reach.range.unwrap_or(1);
        let radius = reach.radius.unwrap_or(1);
        let level = self.state.current();
        let in_range = at.dist2(here) <= radius2(range);
        let clear = || self.clear_shot(here, at, id, target);
        match def.shape {
            Shape::Touch => here.chebyshev(at) <= 1,
            Shape::Ray | Shape::Bolt => in_range && clear(),
            Shape::Burst if def.about_caster() => {
                let cells = burst_cells(here, radius, |p| {
                    p != here && level.is_opaque(&self.content, p)
                });
                cells.contains(&at) && !self.friend_in(id, &cells)
            }
            Shape::Burst => {
                let cells =
                    burst_cells(at, radius, |p| p != at && level.is_opaque(&self.content, p));
                in_range && !cells.contains(&here) && clear() && !self.friend_in(id, &cells)
            }
            Shape::Cone => {
                let cells = cone_cells(here, at, range, |p| level.is_opaque(&self.content, p));
                cells.contains(&at) && clear() && !self.friend_in(id, &cells)
            }
            Shape::Line => {
                let cells = self.bolt_line(here, at, range);
                cells.contains(&at) && clear() && !self.friend_in(id, &cells)
            }
            Shape::Myself => false,
        }
    }

    /// Whether any of `cells` holds a friend of creature `id`'s: someone else not hostile to it.
    fn friend_in(&self, id: ActorId, cells: &[Pos]) -> bool {
        let level = &self.state.actors[&id].level;
        (self.state.actors.values()).any(|a| {
            &a.level == level
                && a.id != id
                && cells.contains(&a.pos)
                && self.stance(id, a.id) != Stance::Hostile
        })
    }

    /// Creature `id` casts `spell` on `on` (its AI saw to it that it can). Returns the price: a
    /// `cast`.
    pub(super) fn creature_casts(
        &mut self,
        id: ActorId,
        (spell, on): (SpellId, ActorId),
        events: &mut Vec<Event>,
    ) -> i64 {
        let content = self.content.clone();
        let price = self.price_of(id, ActionKind::Cast);
        let def = content.spell(spell);
        let (Some(me), Some(at)) = (
            self.state.actors.get(&id),
            self.state.actors.get(&on).map(|a| a.pos),
        ) else {
            return price;
        };
        let here = me.pos;
        let cast = Cast::by_creature(me, &content, (spell, def));
        if !self.spend_mana(id, me.spell_base(&content, def)) {
            return price;
        }
        self.make_noise(id, Act::Attack(AttackKind::Spell));
        // A spell at the player or cast near them stops a rest; one farther off is only told
        // (spec §8 "Rest"): what it says meanwhile is held, then logged. A hurt to the player
        // stops a rest all the same (`rest.rs`, `World::stirs`).
        let far = on != self.state.player && !self.near_player(id) && !self.near_player(on);
        let outer = far.then(|| self.held.replace(Vec::new()));
        self.say_cast(id, def, on);
        match def.kind {
            SpellPurpose::Buff => {
                if let (Some((status, _)), Some(degrees)) = (def.effect.status, cast.degrees) {
                    self.afflict(id, status, degrees, Some(id), events);
                }
            }
            SpellPurpose::Heal => self.creature_heals(&cast, on),
            SpellPurpose::Hex => self.creature_hexes(id, &cast, (here, at), events),
            SpellPurpose::Damage => self.creature_harms(id, &cast, (here, at), events),
            _ => {}
        }
        if let Some(outer) = outer {
            let lines = std::mem::replace(&mut self.held, outer).unwrap_or_default();
            match &mut self.held {
                Some(outer) => outer.extend(lines),
                None => lines.into_iter().for_each(|line| self.note(line)),
            }
        }
        price
    }

    /// Whether actor `id` stands within [`super::rest::CLOSE`] of the player.
    fn near_player(&self, id: ActorId) -> bool {
        let here = self.player_pos();
        (self.state.actors.get(&id)).is_some_and(|a| a.pos.chebyshev(here) <= super::rest::CLOSE)
    }

    /// What the player is told of creature `id` casting `def` on `on`, if they see it.
    fn say_cast(&mut self, id: ActorId, def: &SpellData, on: ActorId) {
        let me = &self.state.actors[&id];
        if !self.perception.sees_actors_at(me.pos) {
            return;
        }
        let who = capitalized(&self.called(me));
        let name = def.name.to_lowercase();
        let player = self.state.player;
        let them = (self.state.actors.get(&on))
            .filter(|a| on == player || self.perception.sees_actors_at(a.pos))
            .map(|a| self.called(a));
        let text = match (on, them) {
            _ if on == id => format!("{who} casts {name}."),
            _ if on == player => format!("{who} casts {name} at you!"),
            (_, Some(them)) if def.kind == SpellPurpose::Heal => {
                format!("{who} casts {name} on {them}.")
            }
            (_, Some(them)) => format!("{who} casts {name} at {them}."),
            (_, None) => format!("{who} casts {name}."),
        };
        self.say(text);
    }

    /// A heal by touch: hit points back to `on`, said as the player sees it.
    fn creature_heals(&mut self, cast: &Cast<'_>, on: ActorId) {
        let Some(heal) = cast.roll else {
            return;
        };
        let back = heal.roll(&mut self.state.rng, cast.reroll).max(0);
        if on == self.state.player {
            self.spell_heals(on, back);
            return;
        }
        let content = self.content.clone();
        let Some(actor) = self.state.actors.get_mut(&on) else {
            return;
        };
        let max = actor.max_hp(&content);
        actor.hp = (actor.hp + back).min(max).max(actor.hp);
        let actor = &self.state.actors[&on];
        if self.perception.sees_actors_at(actor.pos) {
            let who = capitalized(&self.called(actor));
            self.say(format!("{who} looks better."));
        }
    }

    /// A hex at the hostile on `at`: its status on it, or on everyone in a burst's cells but
    /// the caster; each the player sees said ("The goblin is weakened."), the player's own by
    /// the status's words.
    fn creature_hexes(
        &mut self,
        id: ActorId,
        cast: &Cast<'_>,
        (here, at): (Pos, Pos),
        events: &mut Vec<Event>,
    ) {
        let def = cast.def;
        let (Some((status, _)), Some(severity)) = (def.effect.status, cast.degrees) else {
            return;
        };
        let level = self.state.current_level.clone();
        let caught: Vec<ActorId> = if def.shape == Shape::Burst {
            let range = cast.reach.range.unwrap_or(1);
            let radius = cast.reach.radius.unwrap_or(1);
            let center = self.burst_center(here, at, range);
            self.sound_at(def.look.noise, center);
            let lvl = self.state.current();
            let cells = burst_cells(center, radius, |p| {
                p != center && lvl.is_opaque(&self.content, p)
            });
            if let Some(effect) = &def.look.effect {
                events.push(Event::Spell {
                    effect: effect.clone(),
                    origin: here,
                    shape: SpellShape::Burst { center, radius },
                    cells: cells.clone(),
                });
            }
            (self.state.actors.values())
                .filter(|a| a.level == level && a.id != id && cells.contains(&a.pos))
                .filter(|a| self.sees_flash(a, def.effect.only_sighted))
                .map(|a| a.id)
                .collect()
        } else {
            (self.state.actor_at(&level, at).map(|a| a.id))
                .into_iter()
                .collect()
        };
        let content = self.content.clone();
        let word = (content.status(status).seen.clone()).unwrap_or_else(|| "hexed".into());
        for target in caught {
            if !self.state.actors.contains_key(&target) || self.state.death.is_some() {
                continue;
            }
            self.afflict(target, status, severity, Some(id), events);
            let them = &self.state.actors[&target];
            if target != self.state.player && self.perception.sees_actors_at(them.pos) {
                let who = capitalized(&self.called(them));
                self.say(format!("{who} is {word}."));
            }
        }
    }

    /// A damage spell at `at`, by its shape, as the player's cast strikes (`cast.rs`): the
    /// shared bursts, lines, and cones, a ray's or a bolt's single blow (reflected by a shield
    /// that turns spells), then its type's spread and what lands.
    fn creature_harms(
        &mut self,
        id: ActorId,
        cast: &Cast<'_>,
        (here, at): (Pos, Pos),
        events: &mut Vec<Event>,
    ) {
        let def = cast.def;
        let range = cast.reach.range.unwrap_or(1);
        let (roll, reroll) = (cast.roll.unwrap_or_default(), cast.reroll);
        let effect = def.look.effect.clone().unwrap_or_default();
        let caster = Caster::Actor(id);
        let mut made = Vec::new();
        let mut turned = None;
        let rolled = match def.shape {
            Shape::Burst => {
                let burst = Burst {
                    spell: effect,
                    radius: cast.reach.radius.unwrap_or(1),
                    damage: roll,
                    reroll,
                    kind: def.damage_type,
                    blast: def.look.blast,
                    ice: def.look.ice,
                };
                let center = self.burst_center(here, at, range);
                self.sound_at(def.look.noise, center);
                let only = &def.effect.only_tags;
                self.cast_burst_at(caster, (here, center), (&burst, only), &mut made)
            }
            Shape::Line => {
                let bolt = Bolt {
                    spell: effect,
                    length: range,
                    damage: roll,
                    reroll,
                    kind: def.damage_type,
                };
                let path = self.bolt_line(here, at, range);
                self.sound_at(def.look.noise, here);
                let rolled = self.cast_bolt(caster, here, &path, &bolt, &mut made);
                self.spell_type_cell(def.damage_type, path.last().copied(), &mut made);
                rolled
            }
            Shape::Cone => {
                self.sound_at(def.look.noise, here);
                let cone = (&effect[..], roll, reroll, def.damage_type);
                self.cast_cone(caster, (here, at, range), cone, &mut made)
            }
            Shape::Ray | Shape::Bolt | Shape::Touch => {
                self.sound_at(def.look.noise, here);
                let shot = (&effect[..], roll, reroll);
                let by = (def, Some(AttackSkills::default()));
                let rolled;
                (turned, rolled) = self.cast_single(id, (here, at, range), shot, by, &mut made);
                rolled
            }
            Shape::Myself => 0,
        };
        self.spell_type_spreads(id, (def, cast.reach.jumps), rolled, &mut made);
        self.spell_lands(id, (def, &cast.riders), made, Some(cast), events);
        // Spell reflection: a shield that turns spells sends it back at the creature.
        if let Some(turned) = turned {
            let mut back = Vec::new();
            self.turn_back(&turned, &mut back);
            self.spell_lands(turned.reflector, (def, &cast.riders), back, None, events);
        }
    }
}
