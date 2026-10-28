//! Treasure rooms in generated levels (spec §7 "Special rooms"; docs/design/chests-and-treasure.md
//! §6; `docs/dev/special-rooms.md`): a special room whose effect has `secret_doors = "all"` hides
//! every door it has, and one whose effect has `treasure` gets its chests, heaps of gold, and
//! gems.
//!
//! - On a rooms level the room is a room of the generator, eligible only with a door, no stairs,
//!   nothing lying in it (a key), and no locked door ([`fits_room`]). On a cave level it is a box
//!   stamped into the rock ([`stamp`]): its floor, a ring of rock (walls once the walls are
//!   built), and one closed door in the ring facing the cave; eligible only where such a box
//!   fits ([`spots`]). Both come from the level's map stream, as every special room's draws.
//! - A sealed room (`sealed = "dig"`, a greater treasure room) is a box on either generator
//!   ([`stamp`]; on a rooms level in the rock the corridors left, so no corridor leads to it):
//!   no door at all, and one cell of its ring beside a walkable cell reached from the up
//!   staircase is its `breach`, the one wall guaranteed to open it. Eligible only where such a
//!   box fits ([`fits_box`]); `connect::validate` lets its cells go unreached for that breach.
//! - [`seal`], once the level is valid: every door of such a room (a doorway or an open door made
//!   a closed one first) becomes a secret door ([`secret_look`]) of the branch's secret-door
//!   difficulty, not counted against the branch's `secret_doors`. If the level is no longer valid
//!   (the stairs or a key reached only through the room), the doors are put back and the room is
//!   plain. No draw.
//! - [`fill`], after the floor's chests: from the level's treasure stream [`Rng::treasure`]
//!   (`treasure/<level id>`), so the rest of the level, its other chests too, is the same with or
//!   without a treasure room. Per room in order: the `chests` dice, then per chest its cell
//!   (never where something lies) and [`super::chest::lay`]'s draws (a chest of the floor's item
//!   level: lock, trap, contents, the mimic draw); the `gold.heaps` dice, then per heap its cell and its `gold.dice`; the `gems`
//!   dice, then per gem its cell and [`crate::gem::roll`]'s draws.

use super::{Canvas, Palette, connect};
use crate::chest::ChestState;
use crate::content::{
    Content, DungeonDef, GOLD, LevelTemplate, RoomEffectDef, RoomEffectId, Secret, SpecialRoom,
    secret_look,
};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::item_parts::ItemPart;
use crate::magic::loot::Luck;
use crate::object::{Stack, add_to_pile};
use crate::rng::Rng;

/// The interior (width, height) of a stamped box of at least `min_cells` floor cells (at least
/// nine): one wider than square.
fn box_size(min_cells: u16) -> (i32, i32) {
    let n = i32::from(min_cells.max(9));
    let mut w = 1;
    while w * w < n {
        w += 1;
    }
    let w = w + 1;
    (w, (n + w - 1) / w)
}

/// Whether a room of a rooms level, its box `min` to `max` (walls included) with `doors`, may
/// hold an effect with secret doors: it has a door, no staircase, nothing lying in it (the key
/// of the level's locked rooms), no locked door, no creature starting or patrolling in it, and
/// it is on no way the level must lead ([`aside`]).
pub(super) fn fits_room(canvas: &Canvas, (min, max): (Pos, Pos), doors: &[Pos]) -> bool {
    let inside = |p: Pos| (min.x..=max.x).contains(&p.x) && (min.y..=max.y).contains(&p.y);
    !doors.is_empty()
        && !doors.iter().any(|d| canvas.locks.contains_key(d))
        && ![canvas.up, canvas.down].into_iter().flatten().any(inside)
        && !canvas.objects.keys().any(|p| inside(*p))
        && !canvas.creatures.iter().any(|c| {
            let mut p = c.pos;
            inside(p)
                || c.route.iter().any(|d| {
                    p = p.step(*d);
                    inside(p)
                })
        })
        && aside(canvas, doors)
}

/// Whether the down staircase and every key lying outside a vault are still reached from the up
/// staircase without `doors`, a locked door, or a secret one: a room whose doors are hidden is
/// never on the way anywhere the level must lead (`connect::validate`).
fn aside(canvas: &Canvas, doors: &[Pos]) -> bool {
    let Some(up) = canvas.up else {
        return false;
    };
    let content = canvas.content;
    let open = |p: Pos| {
        canvas.terrain.in_bounds(p)
            && !doors.contains(&p)
            && !canvas.locks.contains_key(&p)
            && !canvas.secrets.contains_key(&p)
            && (canvas.is_walkable(p) || content.terrain(canvas.get(p)).is_closed_door())
    };
    let reached = connect::flood(canvas.width(), canvas.height(), up, open);
    let keys = (canvas.objects.iter())
        .filter(|(p, pile)| {
            !canvas.is_stamped(**p) && pile.iter().any(|s| content.object(s.object).key.is_some())
        })
        .map(|(p, _)| *p);
    canvas.down.into_iter().chain(keys).all(|p| reached[p])
}

/// The top-left corners of the interior of every box of `size` that fits in the level's rock: the
/// box with its ring all rock (not stamped, in no special room, off the level's edge), and at
/// least one [`door_spots`] (beside a cell `reached` holds, when given: a sealed box's breach).
/// In row order.
pub(super) fn spots(canvas: &Canvas, size: (i32, i32), reached: Option<&Grid<bool>>) -> Vec<Pos> {
    let (w, h) = size;
    (2..canvas.height() - h - 1)
        .flat_map(|y| (2..canvas.width() - w - 1).map(move |x| Pos::new(x, y)))
        .filter(|&at| {
            ring(at, size).all(|p| {
                canvas.is_inner(p)
                    && canvas.get(p) == canvas.pal.void
                    && !canvas.is_stamped(p)
                    && !canvas.rooms.iter().any(|r| r.contains(p))
            })
        })
        .filter(|&at| !door_spots(canvas, at, size, reached).is_empty())
        .collect()
}

/// The cells reached from the level's up staircase (4-connected, through walkable cells and
/// closed doors, as `connect::validate` reaches them); `None` before the stairs are placed.
fn reached(canvas: &Canvas) -> Option<Grid<bool>> {
    let content = canvas.content;
    let passable = |p: Pos| {
        canvas.terrain.in_bounds(p)
            && (canvas.is_walkable(p) || content.terrain(canvas.get(p)).is_closed_door())
    };
    let up = canvas.up?;
    Some(connect::flood(
        canvas.width(),
        canvas.height(),
        up,
        passable,
    ))
}

/// The cells of the box of `size` with its interior's top-left corner at `at`, its ring
/// included, in row order.
fn ring(at: Pos, (w, h): (i32, i32)) -> impl Iterator<Item = Pos> {
    (at.y - 1..=at.y + h).flat_map(move |y| (at.x - 1..=at.x + w).map(move |x| Pos::new(x, y)))
}

/// The cells of a box's ring (corners left out) next to walkable floor outside it (not stamped,
/// in no special room, and reached from the up staircase when `reached` is given), where its
/// door, or a sealed box's breach, may go. In row order.
fn door_spots(
    canvas: &Canvas,
    at: Pos,
    (w, h): (i32, i32),
    reached: Option<&Grid<bool>>,
) -> Vec<Pos> {
    let open = |p: Pos| {
        canvas.terrain.in_bounds(p)
            && canvas.is_walkable(p)
            && !canvas.is_stamped(p)
            && !canvas.rooms.iter().any(|r| r.contains(p))
            && reached.is_none_or(|r| r[p])
    };
    let mut found = Vec::new();
    for x in at.x..at.x + w {
        found.push((Pos::new(x, at.y - 1), Dir::N));
        found.push((Pos::new(x, at.y + h), Dir::S));
    }
    for y in at.y..at.y + h {
        found.push((Pos::new(at.x - 1, y), Dir::W));
        found.push((Pos::new(at.x + w, y), Dir::E));
    }
    let mut found: Vec<Pos> = (found.into_iter())
        .filter(|&(p, out)| open(p.step(out)))
        .map(|(p, _)| p)
        .collect();
    found.sort();
    found
}

/// Whether the level has room for a box for `effect` (see [`spots`]): a cave's treasure room,
/// or a sealed room (a greater treasure room) on either generator, whose breach must lie beside
/// a cell reached from the up staircase.
pub(super) fn fits_box(canvas: &Canvas, effect: &RoomEffectDef) -> bool {
    let reached = if effect.sealed.is_some() {
        match reached(canvas) {
            Some(reached) => Some(reached),
            None => return false,
        }
    } else {
        None
    };
    !spots(canvas, box_size(effect.min_cells), reached.as_ref()).is_empty()
}

/// Stamps the box of a special room of `effect` into the level's rock: one draw for its spot
/// among [`spots`], one for its way in among [`door_spots`]; its floor, its ring rock (walls once
/// the walls are built). A box with secret doors (a cave's treasure room) gets its door there,
/// closed (hidden by [`seal`]); a sealed one (a greater treasure room, `sealed = "dig"`) no door
/// at all, the cell its `breach`, beside a cell reached from the up staircase. Returns whether
/// it did.
pub(super) fn stamp(canvas: &mut Canvas, rng: &mut Rng, effect: RoomEffectId) -> bool {
    let def = canvas.content.room_effect(effect);
    let size = box_size(def.min_cells);
    let reached = if def.sealed.is_some() {
        match reached(canvas) {
            Some(reached) => Some(reached),
            None => return false,
        }
    } else {
        None
    };
    let spots = spots(canvas, size, reached.as_ref());
    let Some(&at) = rng.pick(&spots) else {
        return false;
    };
    let doors = door_spots(canvas, at, size, reached.as_ref());
    let Some(&door) = rng.pick(&doors) else {
        return false;
    };
    let (w, h) = size;
    let (min, max) = (Pos::new(at.x - 1, at.y - 1), Pos::new(at.x + w, at.y + h));
    let mut room = SpecialRoom::rect(effect, min, max);
    if reached.is_some() {
        room.breach = Some(door);
    }
    if !super::special::make(canvas, room) {
        return false;
    }
    for y in at.y..at.y + h {
        for x in at.x..at.x + w {
            canvas.set(Pos::new(x, y), canvas.pal.floor);
        }
    }
    if reached.is_none() {
        canvas.set(door, canvas.pal.door_closed);
    }
    true
}

/// Hides every door of the level's rooms whose effect has `secret_doors` (see the module docs),
/// on a level that is valid; a room whose sealing would leave it invalid stays plain.
pub(super) fn seal(template: &mut LevelTemplate, content: &Content, pal: Palette) {
    let difficulty = content.search().secret_door;
    let mut i = 0;
    while i < template.rooms.len() {
        let room = &template.rooms[i];
        if content.room_effect(room.effect).secret_doors.is_none() {
            i += 1;
            continue;
        }
        let (min, max) = (room.min, room.max);
        let rim = |p: Pos| p.x == min.x || p.x == max.x || p.y == min.y || p.y == max.y;
        let doors: Vec<Pos> = (room.cells().into_iter())
            .filter(|&p| rim(p))
            .filter(|&p| {
                let def = content.terrain(template.terrain[p]);
                def.walkable || def.is_closed_door()
            })
            .collect();
        let before: Vec<_> = doors.iter().map(|&p| (p, template.terrain[p])).collect();
        for &p in &doors {
            template.terrain.set(p, pal.door_closed);
        }
        let mut hidden = Vec::new();
        for &p in &doors {
            if let Some(look) = secret_look(&template.terrain, content, p) {
                hidden.push((p, look));
            }
        }
        for &(p, look) in &hidden {
            template.terrain.set(p, look);
            let secret = Secret {
                door: pal.door_closed,
                difficulty,
            };
            template.secrets.insert(p, secret);
        }
        if hidden.len() == doors.len() && connect::validate(template, content).is_ok() {
            i += 1;
            continue;
        }
        for (p, t) in before {
            template.terrain.set(p, t);
            template.secrets.remove(&p);
        }
        template.rooms.remove(i);
    }
}

/// Lays what every treasure room of generated level `template` of branch `def` holds (see the
/// module docs), the finder's `luck` reaching its chests.
pub(super) fn fill(
    template: &mut LevelTemplate,
    content: &Content,
    (seed, luck): (u64, Luck),
    (pal, def): (Palette, &DungeonDef),
) {
    let rooms: Vec<(usize, crate::content::Treasury)> = (template.rooms.iter().enumerate())
        .filter_map(|(i, r)| Some((i, content.room_effect(r.effect).treasure?)))
        .collect();
    if rooms.is_empty() {
        return;
    }
    let mut rng = Rng::treasure(seed, &template.id);
    let traps: Vec<_> = (def.traps.iter().copied())
        .filter(|&t| content.trap(t).effect.fits_fixture())
        .collect();
    let shut = content.chest_terrain(ChestState::Shut);
    let currency = (content.shops().map(|s| s.currency)).or_else(|| content.object_id(GOLD));
    let level = template.item_level;
    for (i, treasure) in rooms {
        let room = template.rooms[i].clone();
        let mut free = floor(template, &room, pal);
        for _ in 0..treasure.chests.roll(&mut rng).max(0) {
            let Some(shut) = shut else { break };
            let walkable: Vec<Pos> = (room.cells().into_iter())
                .filter(|&p| content.terrain(template.terrain[p]).walkable)
                .collect();
            // A chest never goes where something already lies (the level's loot): its terrain
            // is not walkable, so the pile would lie on a wall.
            let fits: Vec<Pos> = (free.iter().copied())
                .filter(|p| !template.objects.contains_key(p))
                .filter(|&p| !by_door(template, content, &room, p))
                .filter(|&p| super::special::joined(&walkable, p))
                .collect();
            let Some(&at) = rng.pick(&fits) else { break };
            free.retain(|&p| p != at);
            super::chest::lay(template, content, &mut rng, (at, shut), (&traps, luck));
        }
        for _ in 0..treasure.gold.heaps.roll(&mut rng).max(0) {
            let Some(&at) = rng.pick(&free) else { break };
            let coins = treasure.gold.dice.roll(&mut rng).max(0) as u32 * treasure.gold.each;
            if let (true, Some(currency)) = (coins > 0, currency) {
                let pile = template.objects.entry(at).or_default();
                add_to_pile(pile, &[Stack::new(currency, coins)], content);
            }
        }
        for _ in 0..treasure.gems.roll(&mut rng).max(0) {
            let Some(&at) = rng.pick(&free) else { break };
            let (Some(gems), Some((gem, cutting))) = (
                content.gems(),
                crate::gem::roll(content, &mut rng, level, false),
            ) else {
                continue;
            };
            let parts = vec![ItemPart::gem(content, None, gem, cutting)];
            let gem = Stack::one(gems.object).with_parts(parts);
            add_to_pile(template.objects.entry(at).or_default(), &[gem], content);
        }
    }
}

/// The plain floor cells of `room` where treasure may lie: no stairs, light, trap, lock, or
/// chest. In row order.
fn floor(template: &LevelTemplate, room: &SpecialRoom, pal: Palette) -> Vec<Pos> {
    (room.cells().into_iter())
        .filter(|&p| template.terrain.get(p) == Some(&pal.floor))
        .filter(|&p| !template.stairs.iter().any(|s| s.pos == p))
        .filter(|p| !template.lights.contains_key(p))
        .filter(|p| !template.traps.contains_key(p))
        .filter(|p| !template.locks.contains_key(p))
        .filter(|p| !template.chests.contains_key(p))
        .collect()
}

/// Whether `p` touches (8-way) a door, secret door, or breach of `room`'s rim, where a chest would
/// stand in the way in.
fn by_door(template: &LevelTemplate, content: &Content, room: &SpecialRoom, p: Pos) -> bool {
    Dir::ALL.iter().map(|d| p.step(*d)).any(|q| {
        room.contains(q)
            && (template.secrets.contains_key(&q)
                || room.breach == Some(q)
                || template
                    .terrain
                    .get(q)
                    .is_some_and(|t| content.terrain(*t).is_door()))
    })
}
