//! Shops in generated levels (spec §7 "Shops"): last of all, after special rooms, with the
//! content's `chance` a rooms level turns one room into a shop, at most one a level. Like special
//! rooms it draws from the level's map stream after everything else, so a level without a shop is
//! exactly the map it would be without them.
//!
//! The room is one with a single door (or doorway), not the arrival room, a vault, or a locked
//! room (the rooms generator offers those, [`Candidate`]), with no stairs, nothing lying, no
//! special room, and no patrol in it.
//! Its kind and keeper's name are drawn uniformly; the keeper stands beside the cell inside the
//! door, so the way in stays free; a lamp hangs in the middle of the room; and its floor
//! is laid with stock: between the kind's least and most stacks (as many as there are free
//! cells), one a cell, each drawn uniformly from what it stocks, a stackable one in a bundle.

use super::Canvas;
use crate::content::{ObjectId, ShopId, ShopPlan};
use crate::geom::{Dir, Pos};
use crate::object::Stack;
use crate::rng::Rng;

/// A room the rooms generator offers for a shop: its interior corners (inclusive) and its one
/// door with the way out of it.
pub(super) struct Candidate {
    pub min: Pos,
    pub max: Pos,
    pub door: Pos,
    pub out: Dir,
}

impl Candidate {
    fn contains(&self, p: Pos) -> bool {
        (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
    }

    /// Its interior, row by row.
    fn cells(&self) -> impl Iterator<Item = Pos> + use<> {
        let (min, max) = (self.min, self.max);
        (min.y..=max.y).flat_map(move |y| (min.x..=max.x).map(move |x| Pos::new(x, y)))
    }

    /// Whether `p` lies in the room with its walls.
    fn walled(&self, p: Pos) -> bool {
        (self.min.x - 1..=self.max.x + 1).contains(&p.x)
            && (self.min.y - 1..=self.max.y + 1).contains(&p.y)
    }
}

/// Maybe makes one of `candidates` a shop (see the module docs).
pub(super) fn place(canvas: &mut Canvas, rng: &mut Rng, candidates: &[Candidate]) {
    let Some(shops) = canvas.content.shops().filter(|s| s.generated()) else {
        return;
    };
    if !rng.chance(u32::from(shops.chance)) {
        return;
    }
    let fit: Vec<&Candidate> = candidates.iter().filter(|c| suits(canvas, c)).collect();
    let Some(&room) = rng.pick(&fit) else {
        return;
    };
    let kind = ShopId(u8::try_from(rng.index(shops.defs().len())).unwrap_or(0));
    let keeper = rng.pick(&shops.names).cloned().unwrap_or_default();
    let front = room.door.step(room.out.opposite());
    let beside = match room.out {
        Dir::N | Dir::S => [Dir::W, Dir::E],
        _ => [Dir::N, Dir::S],
    };
    let Some(post) = (beside.iter())
        .map(|d| front.step(*d))
        .find(|p| room.contains(*p) && canvas.is_free_floor(*p))
    else {
        return;
    };
    light(canvas, room, front, post);
    let content = canvas.content;
    let goods: Vec<ObjectId> = (0..content.object_defs().len())
        .map(|o| ObjectId(o as u16))
        .filter(|&o| shops.stocks(content, kind, o))
        .collect();
    let mut cells: Vec<Pos> = (room.cells())
        .filter(|&p| p != front && p != post && canvas.is_free_floor(p))
        .filter(|p| !canvas.objects.contains_key(p))
        .collect();
    rng.shuffle(&mut cells);
    let (least, most) = shops.get(kind).items;
    let count = rng.range(i32::from(least), i32::from(most)) as usize;
    for &p in cells.iter().take(count) {
        let Some(&object) = rng.pick(&goods) else {
            break;
        };
        let count = if content.object(object).stackable {
            let (lo, hi) = shops.bundle;
            rng.range(lo as i32, hi as i32) as u32
        } else {
            1
        };
        canvas.objects.insert(p, vec![Stack::new(object, count)]);
    }
    canvas.creatures.push(shops.keeper_spawn(post));
    canvas.shops.push(ShopPlan {
        kind,
        keeper,
        min: room.min,
        max: room.max,
        post,
    });
}

/// Whether `room` may be a shop: no stairs or vault in it, nothing lying in it yet (a key left
/// there), no special room over it, and no creature starting or patrolling in it.
fn suits(canvas: &Canvas, room: &Candidate) -> bool {
    let (up, down) = (canvas.pal.stairs_up, canvas.pal.stairs_down);
    let plain = room.cells().all(|p| {
        let t = canvas.get(p);
        t != up && t != down && !canvas.is_stamped(p) && !canvas.objects.contains_key(&p)
    });
    let special = (canvas.rooms.iter()).any(|r| r.cells().into_iter().any(|p| room.walled(p)));
    let patrolled = canvas.creatures.iter().any(|c| {
        let mut p = c.pos;
        room.walled(p)
            || c.route.iter().any(|d| {
                p = p.step(*d);
                room.walled(p)
            })
    });
    plain && !special && !patrolled
}

/// A shop is lit: a lamp hangs over the free cell nearest the middle of the room (the first in
/// row order among the nearest), never in the way in or on the keeper's post, so its goods and
/// its keeper are seen whatever the room's own torches light.
fn light(canvas: &mut Canvas, room: &Candidate, front: Pos, post: Pos) {
    let middle = Pos::new((room.min.x + room.max.x) / 2, (room.min.y + room.max.y) / 2);
    let lamp = (room.cells())
        .filter(|&p| p != front && p != post && canvas.is_free_floor(p))
        .min_by_key(|p| p.dist2(middle));
    if let Some(p) = lamp {
        canvas.lights.insert(p, canvas.pal.lamp);
    }
}
