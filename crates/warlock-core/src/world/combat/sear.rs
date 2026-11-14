//! Searing (spec §8 "Blood"): fire leaves char where it burns ([`crate::scorch`]).
//!
//! One hook a place fire burns: the shared area code of `spell.rs` (a burst over its cells,
//! half at its edge, [`World::sear_burst`]; a cone over its cells, [`World::sear_area`]), a fire
//! zone as it is laid (`zone.rs`), and a blow of a charring type on its target's cell
//! (`bleed.rs`, [`World::sear_blow`]). The type's `char` (`data/damage_types.toml`) says whether
//! it chars and in what colours. Where and how much comes from [`crate::scorch::seed`], never the
//! gameplay random stream, and no rule reads char. Not on the overworld, a map of the land.

use super::World;
use crate::content::{CharColors, DamageTypeId, LevelKind};
use crate::fov::radius2;
use crate::geom::Pos;
use crate::scorch::{self, AREA};

impl World {
    /// The char colours of damage type `kind`, if it chars.
    fn chars(&self, kind: Option<DamageTypeId>) -> Option<CharColors> {
        kind.and_then(|k| self.content.damage_type(k).char)
    }

    /// A burst of `kind` of `radius` around `center` reached `cells` of the current level: each
    /// takes [`AREA`] char, half beyond radius − 1, as its damage falls.
    pub(super) fn sear_burst(
        &mut self,
        kind: Option<DamageTypeId>,
        (center, radius): (Pos, i32),
        cells: &[Pos],
    ) {
        let inner = radius2(radius - 1);
        let share = |p: Pos| {
            if p.dist2(center) > inner {
                AREA / 2
            } else {
                AREA
            }
        };
        let shares: Vec<(Pos, u32)> = cells.iter().map(|&p| (p, share(p))).collect();
        self.sear(kind, center, &shares);
    }

    /// A fire of `kind` centred on `center` reached `cells` of the current level, each taking
    /// `share` char.
    pub(super) fn sear_area(
        &mut self,
        kind: Option<DamageTypeId>,
        center: Pos,
        (cells, share): (&[Pos], u32),
    ) {
        let shares: Vec<(Pos, u32)> = cells.iter().map(|&p| (p, share)).collect();
        self.sear(kind, center, &shares);
    }

    fn sear(&mut self, kind: Option<DamageTypeId>, center: Pos, shares: &[(Pos, u32)]) {
        let Some(colors) = self.chars(kind) else {
            return;
        };
        let seed = scorch::seed(self.state.turn, self.state.time, center);
        let now = self.state.time;
        let content = &self.content;
        let level = self.state.current_mut();
        if level.kind == LevelKind::Overworld {
            return;
        }
        for burn in scorch::area(shares, |p| level.is_opaque(content, p), seed) {
            level.char(content, burn, colors, now);
        }
    }

    /// A blow of `kind` struck whoever stands at `at` on level `level`, one that would spill
    /// `volume` blood ([`crate::blood::Wound::volume`]): if the kind chars, the cell does.
    pub(super) fn sear_blow(
        &mut self,
        kind: Option<DamageTypeId>,
        (level, at): (&str, Pos),
        volume: u32,
    ) {
        let Some(colors) = self.chars(kind) else {
            return;
        };
        let seed = scorch::seed(self.state.turn, self.state.time, at);
        let burn = scorch::blow(at, volume, seed);
        let now = self.state.time;
        let content = &self.content;
        let Some(level) = self.state.levels.get_mut(level) else {
            return;
        };
        if level.kind != LevelKind::Overworld {
            level.char(content, burn, colors, now);
        }
    }
}
