//! Tempting hoards (spec §7 "Special rooms"; `docs/dev/special-rooms.md`): a generated floor's
//! special room whose effect has a `hoard` gets its items on the floor: `hoard.items` of them at
//! the floor's item level + `hoard.loot_level`, with the finder's luck, each on a cell of the
//! room's plain floor drawn anew (no stairs, light, or trap). Lifting them is what seals the room
//! (`world/hoard.rs`).
//!
//! The items come from the level's swarm stream [`Rng::swarm`] (`swarm/<level id>`), as a den's
//! hoard does, drawn after its population, so the rest of the level is the same with or without a
//! hoard. A level holds one special room unless its odds allow more, so a den and a hoard rarely
//! share the stream; when they do, each starts it afresh.

use super::{Palette, Visit};
use crate::content::{Content, LevelTemplate};
use crate::geom::Pos;
use crate::magic::{LootGrade, loot};
use crate::object::add_to_pile;
use crate::rng::Rng;
use crate::variant::Source;

/// Lays the items of every tempting hoard of generated level `template` (see the module docs):
/// `danger` its danger, `visit` whom it is made for (the finder's luck).
pub(super) fn fill(
    template: &mut LevelTemplate,
    content: &Content,
    seed: u64,
    (pal, danger): (Palette, u8),
    visit: &Visit,
) {
    let hoards: Vec<(usize, u8, u8)> = (template.rooms.iter().enumerate())
        .filter_map(|(i, r)| {
            let h = content.room_effect(r.effect).hoard?;
            Some((i, h.items, h.loot_level))
        })
        .collect();
    if danger == 0 || hoards.is_empty() {
        return;
    }
    let mut rng = Rng::swarm(seed, &template.id);
    for (i, items, loot_level) in hoards {
        let floor: Vec<Pos> = (template.rooms[i].cells().into_iter())
            .filter(|&p| template.terrain.get(p) == Some(&pal.floor))
            .filter(|&p| !template.stairs.iter().any(|s| s.pos == p))
            .filter(|&p| !template.lights.contains_key(&p))
            .filter(|p| !template.traps.contains_key(p))
            .collect();
        if floor.is_empty() {
            continue;
        }
        let level = template.item_level.saturating_add(loot_level);
        for _ in 0..items {
            let p = floor[rng.index(floor.len())];
            let item = loot::roll_with(
                content,
                &mut rng,
                level,
                LootGrade::Any,
                Source::Loot,
                visit.luck,
            );
            if let Some(stack) = item {
                add_to_pile(template.objects.entry(p).or_default(), &[stack], content);
            }
        }
    }
}
