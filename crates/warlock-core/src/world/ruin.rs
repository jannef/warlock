//! Ruin (spec §8 "Spells": "Ruin"; #1100): what an area spell does to the items lying on the
//! floor it reaches. A Fireball burns the scrolls on it, a blast of frost shatters the potions.
//!
//! - **Who**: an actor's area cast only ([`Caster::Actor`]): a spell's burst, cone, or line,
//!   from mana, a scroll, or a wand, and a dragon's breath. The one hook is in the shared area
//!   code of `spell.rs` ([`World::cast_burst_at`], [`World::cast_bolt`], [`World::cast_cone`]),
//!   after the damage; a trap's or a place's burst ruins nothing, and neither does a spell with
//!   no damage type.
//! - **What**: every stack lying on a cell it reaches, in position order, top of each pile
//!   first: each item of it is lost at the chance of the table's row for the spell's damage type
//!   and the stack (`data/ruin.toml`, [`crate::content::RuinRules::row`]); no row, untouched.
//!   Carried, worn, and chest-held things are not on the floor and never touched.
//! - **Spared**: what is protected from all item damage (`item_damage.rs`,
//!   [`World::item_protected`]; #1170): a unique item, an item of Proof, and every object a
//!   quest asks to be brought or delivered, so no stray Fireball can break a quest. A quest's
//!   reward is not spared unless another quest asks for it.
//! - **Draws**: the ruin stream ([`crate::rng::Rng::ruin`]), kept in the state from the first
//!   draw on, so a cast over no pile the table touches leaves every other stream and the hash
//!   as they were.
//! - **Told**: for the cells the player sees, one line per kind of item lost, after the cast:
//!   "The scroll labelled ZUNBAR burns up.", "3 potions of healing freeze and shatter." Unseen,
//!   nothing is said, and the pile simply shows less when next seen.

use super::World;
use super::item_damage::protected;
use super::spell::Caster;
use super::talk::capitalized;
use crate::content::DamageTypeId;
use crate::geom::Pos;
use crate::object::Stack;
use crate::rng::Rng;
use std::collections::BTreeSet;

impl World {
    /// `caster`'s area spell of damage type `kind` reached `cells` of the current level: the
    /// items lying there that its type ruins are lost by the table's chances (module docs).
    pub(super) fn ruin_items(&mut self, caster: Caster, kind: Option<DamageTypeId>, cells: &[Pos]) {
        let (Caster::Actor(_), Some(kind)) = (caster, kind) else {
            return;
        };
        let content = self.content.clone();
        let rules = content.ruin();
        if rules.rows.iter().all(|r| r.kind != kind) {
            return;
        }
        let quest = self.quest_objects();
        let cells: BTreeSet<Pos> = cells.iter().copied().collect();
        let seed = self.state.seed;
        // What the player saw go, by what it is and what is said of it, in the order first lost.
        let mut told: Vec<(Stack, (&str, &str))> = Vec::new();
        for p in cells {
            let Some(pile) = self.state.current().objects.get(&p) else {
                continue;
            };
            let mut pile = pile.clone();
            let seen = self.perception.sees_actors_at(p);
            let mut changed = false;
            for stack in &mut pile {
                if protected(&content, &quest, stack) {
                    continue;
                }
                let Some(row) = rules.row(&content, kind, stack) else {
                    continue;
                };
                if row.percent == 0 {
                    continue;
                }
                let rng = (self.state.ruin_rng).get_or_insert_with(|| Rng::ruin(seed));
                let lost = (0..stack.count).filter(|_| rng.chance(row.percent)).count();
                let lost = u32::try_from(lost).unwrap_or(u32::MAX);
                if lost == 0 {
                    continue;
                }
                changed = true;
                if seen {
                    let mut gone = stack.clone();
                    gone.count = lost;
                    let said = (row.one.as_str(), row.many.as_str());
                    match (told.iter_mut()).find(|(s, w)| s.same_goods(&gone) && *w == said) {
                        Some((s, _)) => s.count += lost,
                        None => told.push((gone, said)),
                    }
                }
                stack.count -= lost;
            }
            if changed {
                pile.retain(|s| s.count > 0);
                self.state.current_mut().set_pile(p, pile);
            }
        }
        for (stack, (one, many)) in told {
            let line = if stack.count == 1 {
                format!("The {} {one}.", self.lore().title(&stack))
            } else {
                format!("{} {many}.", capitalized(&self.lore().describe(&stack)))
            };
            self.say(line);
        }
    }
}
