//! Animated objects as they are made (spec §8 "Animated objects"; docs/dev/animated.md): what a
//! creature of an animated type is made of, rolled when it spawns.
//!
//! - A household type's creature is its body: one stack of its body's object (a clatter of
//!   cutlery: 3 to 6), of a material drawn by the materials' `gear` weights at the type's level
//!   (a silver candlestick is rare), carried first and in no slot ([`compose`]).
//! - A family's creature (a blade) is its slot lines' pieces ([`compose_slots`]), each line in
//!   order: one draw for its `chance`; one for its base item among the line's classes of level
//!   at most the type's ([`loot::base_in`]); one for its grade (rare by the tier's `rare`, else
//!   magic by its `magic`, else plain); then the piece made by [`loot::make_as`] at item level
//!   L with a creature's materials (`Source::Gear(L)`), put in the pack and in its slot. No
//!   burden check: it carries itself. Its rank in the wielded weapon's skill is set as it is
//!   placed ([`weapon_skills`]; no draws).
//!
//! Who rolls: a generated level composes its spawns from `animated/<level id>`, in spawn order,
//! after their gear ([`arm`]), so the rest of the level is the same whatever animated things it
//! holds; every other spawn (a level's legend, a runtime spawn, the dev `spawn`) from
//! `animated/<actor id>` as it is placed ([`kit`]). Neither touches the gameplay stream. A
//! creature placed with its body already (a legend's `animated = { material }`), or wielding or
//! wearing something already, is not rolled.

use crate::content::{Content, CreatureId, LevelTemplate};
use crate::equipment::{self, Equipment};
use crate::kit::Pack;
use crate::magic::{LootGrade, loot};
use crate::object::Stack;
use crate::rng::Rng;
use crate::variant::{self, Source};
use crate::weapon_skills::WeaponSkills;

/// The stream an animated creature is composed from: a generated level's id, or an actor's.
pub fn stream(run_seed: u64, key: &str) -> Rng {
    Rng::stream(run_seed, &format!("animated/{key}"))
}

/// What a creature of animated type `creature` is made of, drawn from `rng`: its household body
/// (one draw for a body of several, then its material's); `None` for a type without a body.
pub fn compose(content: &Content, rng: &mut Rng, creature: CreatureId) -> Option<Stack> {
    let body = content.animated_body(creature)?;
    let (least, most) = body.count;
    let count = if most > least {
        least + rng.below(most - least + 1)
    } else {
        least
    };
    let level = content.creature(creature).level;
    let def = content.object(body.object);
    let stack = Stack::new(body.object, count);
    Some(
        match variant::draw(content, rng, def, Source::Gear(level), level) {
            Some(part) => stack.with_parts(vec![part]),
            None => stack,
        },
    )
}

/// What a creature of family type `creature` (a blade) is made of, drawn from `rng` (the module
/// docs give the draws): its pieces, each in its slot. Empty for a type of no family.
pub fn compose_slots(content: &Content, rng: &mut Rng, creature: CreatureId) -> Pack {
    let def = content.creature(creature);
    let animated = content.animated();
    let mut pack = Pack::default();
    let Some(tier) = animated.family_tier(def.animated.as_ref()) else {
        return pack;
    };
    let level = def.level;
    let lines = def
        .animated
        .map_or(&[][..], |a| &animated.lines[a.tier][..]);
    let chance = |ramp: Option<crate::content::Ramp>| {
        u32::try_from(ramp.map_or(0, |r| r.at(level)).clamp(0, 100)).unwrap_or(0)
    };
    let (magic, rare) = (chance(tier.magic), chance(tier.rare));
    for line in lines {
        if !rng.chance(u32::from(line.chance)) {
            continue;
        }
        let Some(object) = loot::base_in(content, rng, &line.classes, level) else {
            continue;
        };
        let roll = rng.below(100);
        let grade = if roll < rare {
            Some(LootGrade::Rare)
        } else if roll < rare.max(magic) {
            Some(LootGrade::Magic)
        } else {
            None
        };
        let piece = match grade {
            Some(grade) => {
                loot::make_as(content, rng, object, 1, (level, grade), Source::Gear(level))
            }
            None => {
                let stack = Stack::new(object, 1);
                let def = content.object(object);
                match variant::draw(content, rng, def, Source::Gear(level), level) {
                    Some(part) => stack.with_parts(vec![part]),
                    None => stack,
                }
            }
        };
        let at = pack.put(piece, content);
        if pack.equipment.get(line.slot).is_none() && content.fits(object, line.slot) {
            let slots = equipment::slots_taken(content, object, line.slot);
            pack.equipment.put(&slots, at);
        }
    }
    pack
}

/// `carry` and `equipment` of a creature of type `creature` with what an animated type is made
/// of, composed from `rng`: a household type's body (first, unless something is worn or
/// wielded, whose places in the pack must not move: then last) when it carries none yet; a
/// family's pieces when it carries nothing yet. Any other type's as they are.
pub fn kit(
    content: &Content,
    rng: &mut Rng,
    creature: CreatureId,
    (mut carry, equipment): (Vec<Stack>, Equipment),
) -> (Vec<Stack>, Equipment) {
    let def = content.creature(creature);
    if content
        .animated()
        .family_tier(def.animated.as_ref())
        .is_some()
    {
        if !carry.is_empty() || !equipment.is_empty() {
            return (carry, equipment);
        }
        let pack = compose_slots(content, rng, creature);
        return (pack.items, pack.equipment);
    }
    let Some(body) = content.animated_body(creature) else {
        return (carry, equipment);
    };
    if carry.iter().any(|s| s.object == body.object) {
        return (carry, equipment);
    }
    if let Some(stack) = compose(content, rng, creature) {
        if equipment.items().is_empty() {
            carry.insert(0, stack);
        } else {
            carry.push(stack);
        }
    }
    (carry, equipment)
}

/// The weapon skills of a creature of family type `creature` that wields what `equipment`
/// holds of `carry`: its tier's `weapon_rank` at its level in the skill of each weapon it
/// wields (with the ranks that skill's defaults give). `None` for any other type, or one that
/// wields no weapon of a skill.
pub fn weapon_skills(
    content: &Content,
    creature: CreatureId,
    carry: &[Stack],
    equipment: &Equipment,
) -> Option<WeaponSkills> {
    let def = content.creature(creature);
    let tier = content.animated().family_tier(def.animated.as_ref())?;
    let rank = u8::try_from(tier.weapon_rank?.at(def.level).clamp(0, 255)).unwrap_or(0);
    let ranks: Vec<_> = (equipment.iter())
        .filter_map(|(_, item)| carry.get(item))
        .filter_map(|stack| content.object(stack.object).skill)
        .map(|skill| (skill, rank))
        .collect();
    (rank > 0 && !ranks.is_empty())
        .then(|| WeaponSkills::from_ranks(content.weapon_skills(), ranks))
}

/// Composes every animated spawn of generated level `template` from the level's own stream
/// `animated/<level id>`, in spawn order (after [`crate::gear::arm`]).
pub fn arm(template: &mut LevelTemplate, content: &Content, seed: u64) {
    let mut rng = stream(seed, &template.id);
    for spawn in &mut template.creatures {
        if content.creature(spawn.creature).animated.is_none() {
            continue;
        }
        let made = (std::mem::take(&mut spawn.carry), spawn.equipment.clone());
        (spawn.carry, spawn.equipment) = kit(content, &mut rng, spawn.creature, made);
    }
}

// Tests: `world/animated_tests.rs`.
