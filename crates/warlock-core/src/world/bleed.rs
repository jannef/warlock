//! Bleeding (spec §8 "Blood"): big hits and kills leave blood where they land.
//!
//! [`World::bleed`] turns a blow about to be dealt into stains on the victim's level
//! ([`crate::blood::Wound::spatter`], [`super::LevelState::stain`]): how much by the blow and the
//! victim's size, where by the blow's direction and a hash of the blow (never the gameplay random
//! stream), so blood never changes what happens next. No rule reads it.

use super::{ActorId, World};
use crate::action::AttackKind;
use crate::blood::{self, Wound};
use crate::combat::{Blow, Strike};
use crate::content::LevelKind;
use crate::geom::Pos;

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
        let Some(victim) = self.state.actors.get(&target) else {
            return;
        };
        let Some(color) = self.content.creature(victim.creature).blood else {
            return;
        };
        let size = self.state.size_of(&self.content, victim);
        let wound = Wound {
            from,
            at: victim.pos,
            missile: kind != AttackKind::Melee,
            damage: blow.damage,
            hp: victim.hp,
            max_hp: victim.max_hp(),
            critical: blow.strike == Strike::Hit { critical: true },
            size: self.content.size(size).blood,
            seed: blood::seed(self.state.turn, self.state.time, attacker.0, target.0),
        };
        let now = self.state.time;
        let content = &self.content;
        let Some(level) = self.state.levels.get_mut(&victim.level) else {
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
