//! Animated objects as they are made (spec §8 "Animated objects"; docs/dev/animated.md): what a
//! creature of an animated type is made of, rolled when it spawns. A household type's creature is
//! its body: one stack of its body's object (a clatter of cutlery: 3 to 6), of a material drawn by
//! the materials' `gear` weights at the type's level (a silver candlestick is rare), carried first
//! and in no slot.
//!
//! Who rolls: a generated level composes its spawns from `animated/<level id>`, in spawn order,
//! after their gear ([`arm`]), so the rest of the level is the same whatever animated things it
//! holds; every other spawn (a level's legend, a runtime spawn, the dev `spawn`) from
//! `animated/<actor id>` as it is placed ([`bodied`]). Neither touches the gameplay stream. A
//! creature placed with its body already (a legend's `animated = { material }`) is not rolled.

use crate::content::{Content, CreatureId, LevelTemplate};
use crate::equipment::Equipment;
use crate::object::Stack;
use crate::rng::Rng;
use crate::variant::{self, Source};

/// The stream an animated creature is composed from: a generated level's id, or an actor's.
pub fn stream(run_seed: u64, key: &str) -> Rng {
    Rng::stream(run_seed, &format!("animated/{key}"))
}

/// What a creature of animated type `creature` is made of, drawn from `rng`: its household body
/// (one draw for a body of several, then its material's); `None` for a type that is not animated.
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

/// `carry` with creature type `creature`'s body first, composed from `rng`, when it is an animated
/// type that carries no body yet; else `carry` as it is. The body goes first unless something is
/// worn or wielded (whose places in the pack must not move): then last.
pub fn with_body(
    content: &Content,
    rng: &mut Rng,
    creature: CreatureId,
    (mut carry, equipment): (Vec<Stack>, Equipment),
) -> (Vec<Stack>, Equipment) {
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

/// Composes every animated spawn of generated level `template` from the level's own stream
/// `animated/<level id>`, in spawn order (after [`crate::gear::arm`]).
pub fn arm(template: &mut LevelTemplate, content: &Content, seed: u64) {
    let mut rng = stream(seed, &template.id);
    for spawn in &mut template.creatures {
        if content.animated_body(spawn.creature).is_none() {
            continue;
        }
        let kit = (std::mem::take(&mut spawn.carry), spawn.equipment.clone());
        (spawn.carry, spawn.equipment) = with_body(content, &mut rng, spawn.creature, kit);
    }
}

// Tests: `world/animated_tests.rs`.
