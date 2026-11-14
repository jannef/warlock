//! What every spell of a damage type does beyond its damage (the spell types design
//! `docs/design/spell-types-and-new-spells.md` §3, §4): the rules `[types]` in `data/spells.toml`
//! gives each type, read at one hook each on the cast path, whether the spell comes from mana, a
//! scroll, or a wand. Magic's alone: no blow reads them. They spare no one.
//!
//! - **On those it harmed** ([`World::spell_type_lands`], from `spell_lands`): the rider, by its
//!   chance, then what it ends. "Harmed" is a blow that took hit points: one that resisted it
//!   all is no one's to burn.
//! - **In the blow** ([`World::spell_type_bite`] before resistance, [`World::spell_type_shove`]
//!   after the hurt, both from `spell_strikes`): more harm against the wounded, and a big hit's
//!   or a critical's knock a cell away.
//! - **On the cells it reached** ([`World::spell_type_burst`] from `cast_burst`,
//!   [`World::spell_type_cell`] from `cast_single` and the line's cast, at radius 0 on the cell
//!   struck): the ice it melts and the water it freezes, the fires it puts out, and a burst's
//!   fire left on its centre; a burst's blast shatters thin ice, and breaks the shut doors it
//!   reaches ([`World::blast_doors`]).
//! - **On who is immune**: `immune_tags` is no hook here. `Content::load` lays it on the `resist`
//!   of the creature types of those tags, which every reader then knows.
//!
//! Determinism: one `below(100)` per living creature harmed, only for a `chance` below 100, in
//! the order `spell_lands` walks them; the rest draw nothing. Saves: a status is data, a fire is
//! `LevelState::fires`, a broken door is terrain.

use super::level::Fire;
use super::{ActorId, World};
use crate::action::Event;
use crate::combat::{self, Blow, Strike};
use crate::content::{ActionKind, DamageTypeId, LightId, ZoneKind};
use crate::geom::{Dir, Pos};
use crate::ice::Shock;
use crate::sight::LineOfSight;
use crate::time::Duration;

/// The ice a spell's blast shatters, in µm per point of its `blast`: thinner than 10 cm at 1000,
/// what holds a person and a pack, so a blast that breaks brick drops whoever stands on it.
const SHATTERS_UM: u32 = 100;

impl World {
    /// `by`'s spell of `kind` harmed `harmed`: its type's rider on each who lives (one draw for
    /// a chance), then what the type ends.
    pub(super) fn spell_type_lands(
        &mut self,
        by: ActorId,
        kind: Option<DamageTypeId>,
        harmed: &[ActorId],
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(rule) = content.spell_type(kind) else {
            return;
        };
        for &id in harmed {
            if !self.state.actors.contains_key(&id) || self.state.death.is_some() {
                continue;
            }
            if let Some(rider) = rule.rider
                && (rider.chance >= 100 || self.state.rng.below(100) < u32::from(rider.chance))
            {
                self.afflict(id, rider.status, rider.severity, Some(by), events);
            }
            if rule.corrodes && id == self.state.player {
                self.spell_eats_gear();
            }
            let ended: Vec<_> = (rule.ends.iter())
                .filter_map(|&s| self.state.actors.get(&id)?.effects.get(s).copied())
                .collect();
            if let Some(actor) = self.state.actors.get_mut(&id) {
                for e in &ended {
                    actor.effects.remove(e.status);
                }
            }
            self.ended(id, &ended, events);
        }
    }

    /// What a spell of `kind` harming `target` for `amount`, before resistance, comes to: more
    /// against a target low on hit points by its type's `bite`.
    pub(super) fn spell_type_bite(
        &self,
        target: ActorId,
        amount: i32,
        kind: Option<DamageTypeId>,
    ) -> i32 {
        let bite = self.content.spell_type(kind).and_then(|rule| rule.bite);
        let (Some(bite), Some(actor)) = (bite, self.state.actors.get(&target)) else {
            return amount;
        };
        let (hp, most) = (i64::from(actor.hp), i64::from(actor.max_hp(&self.content)));
        if hp * 100 >= most * i64::from(bite.below_percent) {
            return amount;
        }
        i32::try_from(i64::from(amount) * i64::from(bite.percent) / 100).unwrap_or(i32::MAX)
    }

    /// `blow` of a spell landed on `target`, who lives: a type with `shove` knocks it a cell away
    /// on a big hit or a critical, from the `origin` of the cast (a burst's centre, a ray's
    /// caster), or from `caster` for the one at a burst's centre. The one who cast that burst on
    /// themselves has no "away" and stays; so does one a status holds. The player is told with
    /// the blow's own report (`say_spell`).
    pub(super) fn spell_type_shove(
        &mut self,
        target: ActorId,
        (origin, caster): (Pos, Pos),
        blow: &Blow,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let shoves = content.spell_type(blow.kind).is_some_and(|rule| rule.shove);
        let Some(actor) = self.state.actors.get(&target).filter(|_| shoves) else {
            return;
        };
        let critical = matches!(blow.strike, Strike::Hit { critical: true });
        let big = combat::big_hit(blow.damage, actor.max_hp(&content));
        if self.rooted(target) || !(big || critical) {
            return;
        }
        let at = actor.pos;
        if let Some(dir) = Dir::toward(origin, at).or_else(|| Dir::toward(caster, at)) {
            self.drive_back(target, dir, events);
        }
    }

    /// Whether a status or its kind (`rooted.rs`) holds actor `id` where it stands, so nothing
    /// drives it on.
    pub(super) fn rooted(&self, id: ActorId) -> bool {
        self.is_rooted(id)
            || (self.state.actors.get(&id)).is_some_and(|a| {
                (a.effects)
                    .blocking(&self.content, ActionKind::Step)
                    .is_some()
            })
    }

    /// The shut doors a blast of a spell of `kind` around `center` would break, if its type's
    /// `doors` says so: those in its line of sight with something to break to.
    pub(super) fn doors_blasted(
        &self,
        kind: Option<DamageTypeId>,
        center: Pos,
        radius: i32,
    ) -> Vec<Pos> {
        let content = &self.content;
        if !content.spell_type(kind).is_some_and(|rule| rule.doors) {
            return Vec::new();
        }
        let level = self.state.current();
        let burst = LineOfSight::compute(center, radius, |p| level.is_opaque(content, p));
        let mut doors = Vec::new();
        for y in center.y - radius..=center.y + radius {
            for x in center.x - radius..=center.x + radius {
                let p = Pos::new(x, y);
                let def = level.terrain.get(p).map(|t| content.terrain(*t));
                if burst.contains(p)
                    && def.is_some_and(|d| d.is_closed_door() && d.kicked_in.is_some())
                {
                    doors.push(p);
                }
            }
        }
        doors
    }

    /// The shut doors at `doors` break, with no sturdiness to it: the lock and any trap go with
    /// the door, sprung at no one, as a blown chest's do (spec §8 "Doors").
    pub(super) fn blast_doors(&mut self, doors: &[Pos], events: &mut Vec<Event>) {
        for &p in doors {
            let level = self.state.current_mut();
            level.trap_memory.remove(&p);
            let broken = (level.terrain.get(p))
                .and_then(|t| self.content.terrain(*t).kicked_in)
                .map(|(to, _)| to);
            if let Some(to) = broken {
                self.set_terrain(p, to);
                events.push(Event::DoorBroken { pos: p });
            }
        }
    }

    /// A burst of a spell of `kind` went off (`center` and radius), its ice seen to: the thin ice
    /// its `blast` shatters, the fires it puts out on its cells, then the one it leaves on its
    /// centre.
    pub(super) fn spell_type_burst(
        &mut self,
        kind: Option<DamageTypeId>,
        (center, radius): (Pos, i32),
        blast: u16,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(rule) = content.spell_type(kind) else {
            return;
        };
        if rule.shatter && blast > 0 {
            let up_to = u32::from(blast).saturating_mul(SHATTERS_UM);
            self.shock(center, radius, Shock::Blast(up_to), events);
        }
        if rule.douses {
            self.douse(center, radius);
        }
        if let Some((light, burns)) = rule.fire {
            self.lay_fire(center, light, burns);
        }
    }

    /// A ray, a bolt, a touch, or a line of a spell of `kind` ended on `cell`, if it went
    /// anywhere: what its type does to the ice, the water, and the fire there.
    pub(super) fn spell_type_cell(
        &mut self,
        kind: Option<DamageTypeId>,
        cell: Option<Pos>,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let (Some(rule), Some(cell)) = (content.spell_type(kind), cell) else {
            return;
        };
        self.burst_ice(cell, 0, rule.ice, events);
        if rule.douses {
            self.douse(cell, 0);
        }
    }

    /// The fires within `radius` of `center`, in its line of sight, go out, and a fire zone
    /// loses those cells (`zone.rs`).
    pub(super) fn douse(&mut self, center: Pos, radius: i32) {
        let cells: Vec<Pos> = self.burst(center, radius).into_iter().collect();
        for &p in &cells {
            if self.state.current().fires.contains_key(&p) {
                self.put_out(p);
            }
        }
        (self.state.current_mut()).clear_zone_cells(&cells, &[ZoneKind::Fire]);
    }

    /// A fire on `cell` of the current level, if it is walkable, burning `seconds` with light
    /// `light`, or the fire there burning on until then (as a burning missile's, `come_down`).
    /// Light only: no rule reads it, and it never spreads.
    pub(super) fn lay_fire(&mut self, cell: Pos, light: LightId, seconds: u32) {
        let until = self.state.time + Duration::seconds(u64::from(seconds));
        let content = self.content.clone();
        let level = self.state.current_mut();
        if !level.is_walkable(&content, cell) {
            return;
        }
        let fire = (level.fires.entry(cell)).or_insert(Fire { light, until });
        fire.light = light;
        fire.until = fire.until.max(until);
    }

    /// Actor `id` is in the water: every status of its that water ends (Burning) ends.
    pub(super) fn quench(&mut self, id: ActorId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        let ended: Vec<_> = (actor.effects.iter())
            .filter(|e| content.status(e.status).quenched)
            .copied()
            .collect();
        for e in &ended {
            actor.effects.remove(e.status);
        }
        self.ended(id, &ended, events);
    }
}
