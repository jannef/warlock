//! What every spell of a damage type does beyond its damage (the spell types design
//! `docs/design/spell-types-and-new-spells.md` §3, §4): the rules `[types]` in `data/spells.toml`
//! gives each type, read at one hook each on the cast path, whether the spell comes from mana, a
//! scroll, or a wand. Magic's alone: no blow reads them. They spare no one.
//!
//! - **On those it harmed** ([`World::spell_type_lands`], from `spell_lands`): the rider, by its
//!   chance, then what it ends. "Harmed" is a blow that took hit points: one that resisted it
//!   all is no one's to burn.
//! - **On the cells it reached** ([`World::spell_type_burst`] from `cast_burst`,
//!   [`World::spell_type_cell`] from `cast_single` and the line's cast, at radius 0 on the cell
//!   struck): the ice it melts and the water it freezes, the fires it puts out, and a burst's
//!   fire left on its centre.
//!
//! Determinism: one `below(100)` per living creature harmed, only for a `chance` below 100, in
//! the order `spell_lands` walks them. Saves: a status is data, a fire is `LevelState::fires`.

use super::level::Fire;
use super::{ActorId, World};
use crate::action::Event;
use crate::content::{DamageTypeId, LightId};
use crate::geom::Pos;
use crate::time::Duration;

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

    /// A burst of a spell of `kind` went off around `center`, its ice seen to: the fires it puts
    /// out on its cells, then the one it leaves on its centre.
    pub(super) fn spell_type_burst(
        &mut self,
        kind: Option<DamageTypeId>,
        center: Pos,
        radius: i32,
    ) {
        let content = self.content.clone();
        let Some(rule) = content.spell_type(kind) else {
            return;
        };
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

    /// The fires within `radius` of `center`, in its line of sight, go out.
    fn douse(&mut self, center: Pos, radius: i32) {
        for p in self.burst(center, radius) {
            if self.state.current().fires.contains_key(&p) {
                self.put_out(p);
            }
        }
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
