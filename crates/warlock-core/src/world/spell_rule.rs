//! The spells that each need one small rule (spec §8 "Spells"; the spell types design
//! `docs/design/spell-types-and-new-spells.md` §7), each from its one hook on the cast path:
//!
//! - **Leech** ([`World::leech`], from `spell_attack`): a share of the harm done comes back.
//! - **Benison** ([`World::cast_heal_about`], from `cast_heal`): one roll, for the caster and
//!   each of their allies in the burst about them. Hostiles nothing.
//! - **Rebuke and Bramble**: `only_tags` ([`World::is_of_tags`]) and `only_terrain`
//!   ([`World::stands_on`]) filter whom a burst takes; everything else of it stands.
//! - **Flare**: `only_sighted` ([`World::sees_flash`]) passes the eyeless by: a flash is light.
//! - **Gale** ([`World::cast_gale`], from `cast_hex`): everyone on the line is driven on, nearest
//!   first, and its fires are out. No harm, no roll, no draw; a shut door stands.
//! - **Blink** ([`World::cast_blink`]): to a free cell in reach, and the statuses it ends end.
//!   Held, webbed, and pinned block a step, not a cast.
//! - **Cloudburst** ([`World::cast_weather`]): the weather, as the dev override sets it, for a
//!   spell's time; rain puts out what burns under the open sky as it begins.
//! - **Sense the quick** ([`World::heartbeats`], from the end of `run_turn`): a status's `senses`
//!   pushes a sound on the player's heard list, which perception places roughly and only where
//!   they do not see. No creature hears it, and it is no noise.
//!
//! Determinism: Blink's one `below(n)` over its cells in cell order, Benison's one roll; the rest
//! draw nothing. Saves: nothing new but [`crate::weather::Override`]'s start.

use super::cast::Cast;
use super::spell::burst_cells;
use super::talk::capitalized;
use super::{Actor, ActorId, World};
use crate::action::{Event, SpellShape};
use crate::content::{Content, OnlyTerrain, SpellData};
use crate::fov::radius2;
use crate::geom::{Dir, Pos};
use crate::noise::Heard;
use crate::weather::{Override, Weather};
use std::collections::BTreeSet;

/// The tags of what has no heartbeat, beside the animated things (Sense the quick).
const LIFELESS: [&str; 3] = ["undead", "construct", "elemental"];

impl World {
    /// `by`'s single-target spell `def` took `harm` hit points: its `leech` share, rounded down,
    /// comes back to `by`, never past their maximum.
    pub(super) fn leech(&mut self, by: ActorId, def: &SpellData, harm: i32) {
        let Some(percent) = def.effect.leech else {
            return;
        };
        let back = harm * i32::from(percent) / 100;
        if back <= 0 || self.state.death.is_some() {
            return;
        }
        let content = self.content.clone();
        let Some(actor) = self.state.actors.get_mut(&by) else {
            return;
        };
        let before = actor.hp;
        actor.hp = (actor.hp + back).min(actor.max_hp(&content)).max(actor.hp);
        if by == self.state.player && self.state.actors[&by].hp > before {
            self.say("What it takes mends you.");
        }
    }

    /// Whether actor `a` carries one of the tags `only` (none: anyone will do).
    pub(super) fn is_of_tags(&self, a: &Actor, only: &[String]) -> bool {
        only.is_empty() || (only.iter()).any(|t| self.content.has_tag(a.creature, t))
    }

    /// Whether `a` is reached by a hex that reaches `only_sighted` (Flare): any creature with
    /// eyes, never one tagged [`crate::content::EYELESS`].
    pub(super) fn sees_flash(&self, a: &Actor, only_sighted: bool) -> bool {
        !only_sighted || !self.content.has_tag(a.creature, crate::content::EYELESS)
    }

    /// Whether cell `p` of the current level is the ground `only` asks for (none: anywhere).
    pub(super) fn stands_on(&self, p: Pos, only: Option<OnlyTerrain>) -> bool {
        let Some(only) = only else {
            return true;
        };
        let terrain = self.state.current().terrain.get(p);
        terrain.is_some_and(|t| match only {
            OnlyTerrain::Plants => self.content.terrain(*t).plants,
        })
    }

    /// A heal over a burst about the caster (Benison): one roll, back to the caster and every
    /// ally of theirs in it, each said. Refused (said, `false`) when none of them is hurt.
    pub(super) fn cast_heal_about(&mut self, cast: &Cast<'_>) -> bool {
        let Some(heal) = cast.roll else {
            return false;
        };
        let content = self.content.clone();
        let player = self.state.player;
        let here = self.player_pos();
        let level = self.state.current();
        let opaque = |p: Pos| p != here && level.is_opaque(&content, p);
        let cells = burst_cells(here, cast.reach.radius.unwrap_or(1), opaque);
        let hurt: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.level == level.id && cells.contains(&a.pos) && !a.in_retinue())
            .filter(|a| a.id == player || a.ally.as_ref().is_some_and(|x| x.master == player))
            .filter(|a| a.hp < a.max_hp(&content))
            .map(|a| a.id)
            .collect();
        if hurt.is_empty() {
            self.say("No one here is hurt.");
            return false;
        }
        if !self.pay_cast(cast) {
            return true;
        }
        if let Some(text) = &cast.def.look.message {
            self.say(text.clone());
        }
        self.sound_at(cast.def.look.noise, here);
        let back = heal.roll(&mut self.state.rng, cast.reroll).max(0);
        for id in hurt {
            self.spell_heals(id, back);
            self.spell_landed(cast, id, back > 0);
        }
        true
    }

    /// A wind down a line toward `at` (Gale): everyone on it, nearest first and each once, is
    /// driven `push` cells straight away from the caster, as far as they will go (no one is
    /// pushed into another, and one a status holds stays), then its fires go out.
    pub(super) fn cast_gale(
        &mut self,
        cast: &Cast<'_>,
        at: Pos,
        push: u8,
        events: &mut Vec<Event>,
    ) -> bool {
        let range = cast.reach.range.unwrap_or(1);
        if !self.can_loose_here() || !self.within_reach(at, range, "cast") {
            return false;
        }
        let here = self.player_pos();
        if !self.pay_cast(cast) {
            return true;
        }
        if let Some(text) = &cast.def.look.message {
            self.say(text.clone());
        }
        self.sound_at(cast.def.look.noise, here);
        let path = self.bolt_line(here, at, range);
        if let Some(effect) = &cast.def.look.effect {
            events.push(Event::Spell {
                effect: effect.clone(),
                origin: here,
                shape: SpellShape::Bolt { path: path.clone() },
                cells: path.clone(),
            });
        }
        let level = self.state.current().id.clone();
        let caught: Vec<ActorId> = (path.iter())
            .filter_map(|p| self.state.actor_at(&level, *p).map(|a| a.id))
            .collect();
        for id in caught {
            let moved = self.driven(id, here, push, events);
            if self.state.actors.contains_key(&id) {
                // A keeper or a court's own turns on the caster, as one a blow lands on does.
                self.provoke(id);
                self.provoke_court(id);
            }
            self.spell_landed(cast, id, moved);
        }
        if cast.def.effect.douses {
            for &p in &path {
                if self.state.current().fires.contains_key(&p) {
                    self.put_out(p);
                }
            }
        }
        // Clouds are torn away: the zones it clears lose the cells of its line (`zone.rs`).
        let clears = &cast.def.effect.clears;
        if !clears.is_empty() {
            (self.state.current_mut()).clear_zone_cells(&path, clears);
        }
        true
    }

    /// `id` is driven up to `cells` cells straight away from `from`, unless a status holds it,
    /// said where the player sees it. Whether it moved.
    fn driven(&mut self, id: ActorId, from: Pos, cells: u8, events: &mut Vec<Event>) -> bool {
        let Some(at) = self.state.actors.get(&id).map(|a| a.pos) else {
            return false;
        };
        let Some(dir) = Dir::toward(from, at).filter(|_| !self.rooted(id)) else {
            return false;
        };
        let mut moved = false;
        for _ in 0..cells {
            moved |= self.drive_back(id, dir, events).is_some();
        }
        if moved && self.perception.sees_actors_at(at) {
            let who = capitalized(&self.called(&self.state.actors[&id]));
            self.say(format!("{who} is driven back."));
        }
        moved
    }

    /// Blink: the caster lands on a free cell within its `cells` (walkable, no trap, no ice, no
    /// one there, not their own; through a wall or not), one drawn evenly in cell order, and what
    /// it `ends` ends. Refused (said, `false`) with nowhere to go.
    pub(super) fn cast_blink(&mut self, cast: &Cast<'_>, events: &mut Vec<Event>) -> bool {
        let Some(blink) = &cast.def.effect.blink else {
            return false;
        };
        let player = self.state.player;
        let here = self.player_pos();
        let cells = self.blink_cells(here, i32::from(blink.cells));
        if cells.is_empty() {
            self.say("There is nowhere to go.");
            return false;
        }
        if !self.pay_cast(cast) {
            return true;
        }
        if let Some(text) = &cast.def.look.message {
            self.say(text.clone());
        }
        self.sound_at(cast.def.look.noise, here);
        let n = u32::try_from(cells.len()).unwrap_or(u32::MAX);
        let to = cells[self.state.rng.below(n) as usize];
        self.teleport_to(player, (here, to), events);
        let me = self.player_mut();
        let ended: Vec<_> = (blink.ends.iter())
            .filter_map(|&s| me.effects.get(s).copied())
            .collect();
        for e in &ended {
            me.effects.remove(e.status);
        }
        self.ended(player, &ended, events);
        self.spell_landed(cast, player, true);
        true
    }

    /// The free cells a blink from `here` may land on within `reach`, in cell order.
    fn blink_cells(&self, here: Pos, reach: i32) -> Vec<Pos> {
        let level = self.state.current();
        let taken: BTreeSet<Pos> = (self.state.actors.values())
            .filter(|a| a.level == level.id)
            .map(|a| a.pos)
            .collect();
        let mut cells = Vec::new();
        for y in here.y - reach..=here.y + reach {
            for x in here.x - reach..=here.x + reach {
                let p = Pos::new(x, y);
                if p.dist2(here) <= radius2(reach)
                    && level.is_walkable(&self.content, p)
                    && !level.traps.contains_key(&p)
                    && level.ice_at(p).is_none()
                    && !taken.contains(&p)
                {
                    cells.push(p);
                }
            }
        }
        cells
    }

    /// Cloudburst: the weather is the spell's for its time, as the dev override sets it, only
    /// under the open sky if the spell says so. Rain begun, every fire and every Burning creature
    /// under the open sky is put out.
    pub(super) fn cast_weather(&mut self, cast: &Cast<'_>, events: &mut Vec<Event>) -> bool {
        let Some(weather) = cast.def.effect.weather else {
            return false;
        };
        if cast.def.open_sky && !self.under_open_sky() {
            self.say("You need the open sky above you for that.");
            return false;
        }
        if !self.pay_cast(cast) {
            return true;
        }
        if let Some(text) = &cast.def.look.message {
            self.say(text.clone());
        }
        self.sound_at(cast.def.look.noise, self.player_pos());
        let over = Override::lasting(weather.kind, self.state.time, weather.lasts);
        self.state.weather_override = Some(over);
        if weather.kind >= Weather::Rain {
            self.rain_begins(events);
        }
        self.spell_landed(cast, self.state.player, true);
        true
    }

    /// Rain falls on the current level: what burns under the open sky goes out.
    fn rain_begins(&mut self, events: &mut Vec<Event>) {
        let (content, level) = (self.content.clone(), self.state.current());
        if !level.kind.is_sky_exposed() {
            return;
        }
        let open = |p: Pos| (level.terrain.get(p)).is_some_and(|t| !content.terrain(*t).roofed);
        let fires: Vec<Pos> = level.fires.keys().copied().filter(|&p| open(p)).collect();
        let wet: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.level == level.id && !a.in_retinue() && open(a.pos))
            .map(|a| a.id)
            .collect();
        for p in fires {
            self.put_out(p);
        }
        for id in wet {
            self.quench(id, events);
        }
    }

    /// The end of a turn, for a player whose status `senses`: every living creature within its
    /// reach is on their heard list, faint (perception places it roughly, and only where they do
    /// not see; the deaf hear nothing). A lookup and no more without such a status.
    pub(super) fn heartbeats(&mut self) {
        let content = self.content.clone();
        let me = self.state.player();
        let senses = (me.effects.iter())
            .filter_map(|e| content.status(e.status).senses)
            .max_by_key(|s| s.reach);
        let (Some(senses), Some(loudness)) = (senses, me.hearing(&content)) else {
            return;
        };
        let (here, player) = (me.pos, me.id);
        let level = self.state.current_level.clone();
        let beats: Vec<Heard> = (self.state.actors.values())
            .filter(|a| a.level == level && a.id != player && !a.in_retinue())
            .filter(|a| a.pos.dist2(here) <= radius2(senses.reach))
            .filter(|a| living(&content, a))
            .map(|a| Heard {
                kind: senses.kind,
                at: a.pos,
                loudness,
            })
            .collect();
        self.player_mut().heard.extend(beats);
    }
}

/// Whether `a` has a heartbeat to feel: no undead, animated thing, or construct.
fn living(content: &Content, a: &Actor) -> bool {
    content.creature(a.creature).animated.is_none()
        && !LIFELESS.iter().any(|t| content.has_tag(a.creature, t))
}
