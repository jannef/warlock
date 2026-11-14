//! Bleeding (spec §8 "Blood"): big hits and kills leave blood where they land.
//!
//! [`World::bleed`] turns a blow about to be dealt into stains on the victim's level
//! ([`crate::blood::Wound::spatter`], [`crate::world::LevelState::stain`]): how much by the blow and the
//! victim's size, where by the blow's direction and a hash of the blow (never the gameplay random
//! stream), so blood never changes what happens next. No rule reads it. A blow of a kind that
//! draws no blood (`blood = false` in `data/damage_types.toml`: fire, cold, electricity, poison,
//! necrotic) spills none; one that chars sears its target's cell instead (`sear.rs`).

use crate::action::AttackKind;
use crate::blood::{self, Wound};
use crate::combat::{Blow, Strike};
use crate::content::{CreatureId, LevelKind};
use crate::geom::Pos;
use crate::world::{ActorId, World};

impl World {
    /// Actor `target` is about to take `blow`, an attack of `kind` that `attacker` made from
    /// `from`: if it is a big hit or kills, the target's blood (if it bleeds) lands around it. Not
    /// on the overworld, a map of the land.
    pub(super) fn bleed(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        from: Pos,
        kind: AttackKind,
        blow: Blow,
    ) {
        self.spill(attacker.0, target, (from, kind != AttackKind::Melee), blow);
    }

    /// A corpse of `creature` at `at` bursts, the blast from `from` (`corpse_spell.rs`): its
    /// blood is spattered as a kill's, presentation data as all blood is.
    pub(in crate::world) fn spill_corpse(&mut self, creature: CreatureId, at: Pos, from: Pos) {
        let def = self.content.creature(creature);
        let Some(color) = def.blood else {
            return;
        };
        let wound = Wound {
            from,
            at,
            missile: false,
            damage: 0,
            hp: 0,
            max_hp: 1,
            critical: false,
            size: self.content.size(def.size).blood,
            seed: blood::seed(self.state.turn, self.state.time, 0, u32::from(creature.0)),
        };
        let now = self.state.time;
        let content = &self.content;
        let level = self.state.current_mut();
        if level.kind == LevelKind::Overworld {
            return;
        }
        for spatter in wound.spatter(|p| level.is_opaque(content, p)) {
            level.stain(content, spatter, color, now);
        }
    }

    /// [`World::bleed`] for any blow: `source` only seeds where the blood lands (the attacker's
    /// id; a trap's kind for a trap's blow, spec §8 "Traps"), and the blow came from `from`,
    /// along a missile's flight or not.
    pub(in crate::world) fn spill(
        &mut self,
        source: u32,
        target: ActorId,
        (from, missile): (Pos, bool),
        blow: Blow,
    ) {
        let Some(victim) = self.state.actors.get(&target) else {
            return;
        };
        let size = self.state.size_of(&self.content, victim);
        let wound = Wound {
            from,
            at: victim.pos,
            missile,
            damage: blow.damage,
            hp: victim.hp,
            max_hp: victim.max_hp(&self.content),
            critical: blow.strike == Strike::Hit { critical: true },
            size: self.content.size(size).blood,
            seed: blood::seed(self.state.turn, self.state.time, source, target.0),
        };
        let color = self.content.creature(victim.creature).blood;
        let level = victim.level.clone();
        // Fire sears where it strikes (`sear.rs`); a kind that draws no blood stops here (spec
        // §8 "Blood": a blow of two kinds draws it if the kind that struck does).
        self.sear_blow(blow.kind, (&level, wound.at), wound.volume());
        let draws = blow.kind.is_none_or(|k| self.content.damage_type(k).blood);
        let Some(color) = color.filter(|_| draws) else {
            return;
        };
        let now = self.state.time;
        let content = &self.content;
        let Some(level) = self.state.levels.get_mut(&level) else {
            return;
        };
        if level.kind == LevelKind::Overworld {
            return;
        }
        for spatter in wound.spatter(|p| level.is_opaque(content, p)) {
            level.stain(content, spatter, color, now);
        }
    }
}
