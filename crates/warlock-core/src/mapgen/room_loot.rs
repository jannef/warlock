//! A room's own loot (spec §7 "Special rooms"; `docs/dev/special-rooms.md`): a generated
//! floor's special room whose effect has a `loot` (a scriptorium's scrolls) gets between its
//! fewest and its most items of that class on its plain floor (no stairs, light, or trap), each
//! a base item of the class that is loot at the floor's item level, by frequency, made as loot
//! is (unidentified), on a cell drawn anew.
//!
//! The items come from the level's swarm stream [`Rng::swarm`] (`swarm/<level id>`), as a den's
//! and a hoard's do, so the rest of the level is the same with or without such a room; each
//! starts the stream afresh. Draws, per room: the count, then per item the base, its making, and
//! its cell.

use super::Palette;
use crate::content::{Content, LevelTemplate};
use crate::geom::Pos;
use crate::magic::{LootGrade, loot};
use crate::object::add_to_pile;
use crate::rng::Rng;
use crate::variant::Source;

/// Lays the loot of every special room of generated level `template` whose effect has one (see
/// the module docs).
pub(super) fn fill(template: &mut LevelTemplate, content: &Content, seed: u64, pal: Palette) {
    let rooms: Vec<_> = (template.rooms.iter().enumerate())
        .filter_map(|(i, r)| Some((i, content.room_effect(r.effect).loot?)))
        .collect();
    if rooms.is_empty() {
        return;
    }
    let mut rng = Rng::swarm(seed, &template.id);
    let level = template.item_level;
    for (i, room_loot) in rooms {
        let floor: Vec<Pos> = (template.rooms[i].cells().into_iter())
            .filter(|&p| template.terrain.get(p) == Some(&pal.floor))
            .filter(|&p| !template.stairs.iter().any(|s| s.pos == p))
            .filter(|&p| !template.lights.contains_key(&p))
            .filter(|p| !template.traps.contains_key(p))
            .collect();
        if floor.is_empty() {
            continue;
        }
        let [least, most] = room_loot.count;
        let count = rng.range(i32::from(least), i32::from(most));
        for _ in 0..count {
            let Some(base) = loot::base_in(content, &mut rng, &[room_loot.class], level) else {
                break;
            };
            let stack = loot::make_as(
                content,
                &mut rng,
                base,
                1,
                (level, LootGrade::Any),
                Source::Loot,
            );
            let p = floor[rng.index(floor.len())];
            add_to_pile(template.objects.entry(p).or_default(), &[stack], content);
        }
    }
}
