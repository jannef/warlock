//! Material drops (spec §7 "Material drops"; the design: docs/design/recipes.md §9; #1341): the
//! crafting stock a creature's body leaves where it dies, rolled from its type's resolved rows
//! (`content/drops.rs`: its tags' and its own, its size applied) after its carried gear falls
//! and [`World::loot_of`] rolls. Each row is its own chance draw and dice, from the stream
//! `materials/<actor id>` (spec §8 "Determinism"): never the gameplay, loot (`drop/`), or gem
//! streams, so a kill moves no other roll. Pieces of one material from several rows join one
//! stack. Whoever made the kill: the body's, not a reward.

use super::{Actor, World};
use crate::content::{DropGoods, PER_MILLE};
use crate::object::Stack;
use crate::piece;
use crate::rng::Rng;

impl World {
    /// What `actor`, a creature that just died leaving its body (no summoned ally, nothing
    /// turned to stone), drops of crafting stock: each of its type's rows by its chance and
    /// dice, one stack per kind of goods.
    pub(super) fn drops_of(&self, actor: &Actor) -> Vec<Stack> {
        let rows = &self.content.creature(actor.creature).drops;
        if rows.is_empty() {
            return Vec::new();
        }
        let mut rng = Rng::stream(self.state.seed, &format!("materials/{}", actor.id.0));
        let mut out: Vec<Stack> = Vec::new();
        for row in rows {
            if rng.below(PER_MILLE) >= u32::from(row.chance) {
                continue;
            }
            let Ok(count) = u32::try_from(row.count.roll(&mut rng)) else {
                continue;
            };
            if count == 0 {
                continue;
            }
            let stack = match row.goods {
                DropGoods::Material(m) => piece::make(&self.content, m, count),
                DropGoods::Object(o) => Some(Stack::new(o, count)),
            };
            let Some(stack) = stack else {
                continue;
            };
            match out.iter_mut().find(|s| s.same_goods(&stack)) {
                Some(same) => same.count += stack.count,
                None => out.push(stack),
            }
        }
        out
    }
}
