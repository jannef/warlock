//! Creature gear (spec §8 "Creature gear"): what a creature made from a type without a fixed kit
//! spawns carrying, wielding, and wearing, by its type's gear table
//! ([`crate::content::Gear`]). Combat reads it as it reads anyone's equipment, and it falls where
//! the creature dies with the rest of what it carried.
//!
//! [`roll`] goes through the lines in order. Per line: one draw for its `chance`; one to pick
//! among its alternatives when it has several; for a line with a `magic` chance, one draw for
//! it; for a spell scroll one draw for its grimoire ([`Content::grimoire_or`]); for a magic
//! piece the loot draws that make it ([`loot::make_as`]: at least magic, at
//! the floor's item level, with nobody's luck; finished after its condition, [`finish`]) with its material; for a plain one only its
//! material, by the materials' `gear` weights at the creature's level and its type's faction's
//! favoured materials ([`variant::draw`]; none
//! for a placement that names its material). A piece that would overload the creature (what it
//! carries over its carrying limit, by its stats and size) is left out; one with no free slot
//! stays carried.
//!
//! Who rolls: every creature spawn that is not given a fixed kit (`CreatureSpawn::rolls_gear`).
//! A generated level rolls its spawns' gear from `gear/<level id>`, in spawn order, after its
//! loot ([`arm`]), so the rest of the level is the same with or without gear; a hand-made
//! level's from `gear/<actor id>` when it is placed ([`crate::world`]'s `instantiate_level`).
//! Neither touches the gameplay stream.

use crate::burden;
use crate::content::{Content, CreatureId, Keep, LevelTemplate};
use crate::kit::Pack;
use crate::magic::{LootGrade, loot};
use crate::object::{self, Stack};
use crate::rng::Rng;
use crate::stats::Stat;
use crate::variant::{self, Source};

pub(crate) mod condition;
#[cfg(test)]
mod condition_tests;

/// The gear a creature of type `creature` spawns with on a floor of item level `item_level`,
/// drawn from `rng` (the module docs give the draws).
pub fn roll(content: &Content, rng: &mut Rng, creature: CreatureId, item_level: u8) -> Pack {
    let def = content.creature(creature);
    let limit = burden::limit(
        def.stats.score(Stat::Strength),
        def.stats.score(Stat::Toughness),
        content.size(def.size).carry,
    );
    let source = Source::Gear {
        level: def.level,
        faction: def.faction,
    };
    let mut pack = Pack::default();
    for line in &def.gear.lines {
        if !rng.chance(u32::from(line.chance)) {
            continue;
        }
        let picked = match line.one_of.len() {
            0 | 1 => 0,
            n => rng.index(n),
        };
        let (stacks, keep) = &line.one_of[picked];
        let magic = line.magic > 0 && rng.chance(u32::from(line.magic));
        for stack in stacks {
            let piece = make(content, rng, stack, magic, (item_level, source));
            let load = object::weight_of(&pack.items, content) + piece.weight(content);
            if load > limit {
                continue;
            }
            let at = pack.put(piece, content);
            if *keep != Keep::Carry {
                pack.equip(at, content);
            }
        }
    }
    pack
}

/// One piece of gear as `stack` places it: made magic at `item_level`, or plain with a material
/// drawn as `source` has it unless the placement names one; a wand's rung drawn last.
fn make(
    content: &Content,
    rng: &mut Rng,
    stack: &Stack,
    magic: bool,
    placed: (u8, Source),
) -> Stack {
    // A spell scroll may be its grimoire (spec §8 "Spells": "Learning"): one draw, a scroll's.
    let object = content.grimoire_or(rng, stack.object);
    let grimoire = (object != stack.object).then(|| Stack::new(object, stack.count));
    let stack = grimoire.as_ref().unwrap_or(stack);
    let mut made = make_piece(content, rng, stack, magic, placed);
    made.draw_wand_rung(content, rng);
    made
}

/// [`make`] but for a wand's rung.
fn make_piece(
    content: &Content,
    rng: &mut Rng,
    stack: &Stack,
    magic: bool,
    (item_level, source): (u8, Source),
) -> Stack {
    // A unique item (`unique:ID`) is made as it is.
    if stack.magic().is_some() {
        return stack.clone();
    }
    if magic {
        let grade = (item_level, LootGrade::Magic);
        return loot::make_as_unfinished(content, rng, stack.object, stack.count, grade, source);
    }
    if stack.own_parts().is_some() {
        return stack.clone();
    }
    let def = content.object(stack.object);
    match variant::draw(content, rng, def, source, item_level) {
        Some(part) => stack.clone().with_parts(vec![part]),
        None => stack.clone(),
    }
}

/// Finishes every piece of `pack`, gear [`roll`] made and its condition drew
/// ([`loot::finished`]: the trim to its capacity, the very last step), from the gear's own
/// stream `rng`, in pack order: no draw for a piece that fits.
pub fn finish(content: &Content, rng: &mut Rng, pack: &mut Pack) {
    for piece in &mut pack.items {
        let made = std::mem::replace(piece, Stack::new(piece.object, piece.count));
        *piece = loot::finished(content, rng, made);
    }
}

/// The gear of a creature of type `creature` on a level of item level `level`, with every last
/// step, in order: [`roll`] from `rng`, its condition from `condition`
/// (`gear/condition.rs`), then [`finish`] from `rng` again. Every creature's rolled gear
/// is made here, a generated level's ([`arm`]) and a placed creature's alike.
pub fn equip(
    content: &Content,
    rng: &mut Rng,
    condition: &mut Rng,
    creature: CreatureId,
    level: u8,
) -> Pack {
    let mut pack = roll(content, rng, creature, level);
    condition::apply(content, condition, (creature, level), &mut pack);
    finish(content, rng, &mut pack);
    pack
}

/// Rolls the gear of every spawn of generated level `template` that has no fixed kit, from the
/// level's own stream `gear/<level id>`, in spawn order, on top of anything it carries already.
pub fn arm(template: &mut LevelTemplate, content: &Content, seed: u64) {
    let mut rng = Rng::gear(seed, &template.id);
    let mut condition = condition::stream(seed, &template.id);
    let level = template.item_level;
    for spawn in &mut template.creatures {
        if !spawn.rolls_gear {
            continue;
        }
        let pack = equip(content, &mut rng, &mut condition, spawn.creature, level);
        // What it was given already (a monster pool's key, `mapgen/population.rs`) comes after
        // its gear, so the gear's equipment still points at it.
        let given = std::mem::replace(&mut spawn.carry, pack.items);
        spawn.carry.extend(given);
        spawn.equipment = pack.equipment;
        spawn.rolls_gear = false;
    }
}

// Tests: `world/items/gear_tests.rs`.
