//! Rooms and corridors (ADOM-like): walled rectangular rooms joined by corridors dug through rock,
//! with doors where corridors enter rooms.
//!
//! 1. Stamp the vaults the branch level names (each for certain, else the attempt fails), then
//!    maybe one other; a vault joins the level like a room whose only doors are its entrances.
//! 2. Scatter non-overlapping rooms (random attempts, at least two cells of rock apart).
//! 3. Join the rooms along a minimum spanning tree of their centers, so everything is connected,
//!    plus a few extra corridors for loops. A corridor is the cheapest path from a door spot on one
//!    room to a door spot on the other: bends cost extra and existing corridors are cheap to
//!    reuse. Its ends become closed doors, open doors, or plain doorways.
//! 4. The up staircase goes in one room, the down staircase in a room far from it.
//! 5. Wall torches (sconces) in room corners light most rooms, always the arrival room.
//! 6. A few creatures walk laps or back and forth in rooms.
//! 7. With the branch's key, up to two rooms with a single door get it locked, and the key
//!    lies in a room that is not locked.
//! 8. Now and then a room other than the arrival room is special (`special.rs`): the room with
//!    its walls and doors.
//! 8. Rarely, a room with a single door becomes a shop (`shop.rs`).

use super::connect::cheapest_path;
use super::{Canvas, Needs, shop, special, vault};
use crate::content::{ObjectId, SpecialRoom, StairsDir, Vault};
use crate::door::{Key, Lock};
use crate::geom::{Dir, Pos};
use crate::object::Stack;
use crate::rng::Rng;
use std::collections::BTreeMap;

/// Rock between rooms, so walls are never shared and corridors fit between them.
const GAP: i32 = 2;
/// The most rooms a level locks (with the branch's key).
const MAX_LOCKED: usize = 2;
/// Chance (percent) that a level gets a vault.
const VAULT_CHANCE: u32 = 50;
/// Chance (percent) that a room other than the arrival room has torches.
const LIT_CHANCE: u32 = 80;

#[derive(Clone, Debug)]
struct Room {
    /// Bounds including the walls.
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    /// Doors (or doorways) in the walls, each with the direction leading out.
    doors: Vec<(Pos, Dir)>,
    /// A stamped vault: only its entrances serve as doors, and nothing is placed inside.
    vault: bool,
}

impl Room {
    /// Interior corners, inclusive: `(x0, y0, x1, y1)`.
    fn interior(&self) -> (i32, i32, i32, i32) {
        (
            self.x + 1,
            self.y + 1,
            self.x + self.w - 2,
            self.y + self.h - 2,
        )
    }

    fn center(&self) -> Pos {
        Pos::new(self.x + self.w / 2, self.y + self.h / 2)
    }

    fn contains(&self, p: Pos) -> bool {
        p.x >= self.x && p.y >= self.y && p.x < self.x + self.w && p.y < self.y + self.h
    }

    /// Whether the rooms are closer than `GAP` cells of rock.
    fn crowds(&self, o: &Room) -> bool {
        self.x - GAP < o.x + o.w
            && o.x - GAP < self.x + self.w
            && self.y - GAP < o.y + o.h
            && o.y - GAP < self.y + self.h
    }

    /// Where a corridor may enter: existing doors, and wall cells that are neither at nor next to
    /// a corner nor next to another door. A vault offers only its entrances.
    fn door_spots(&self) -> Vec<(Pos, Dir)> {
        if self.vault {
            return self.doors.clone();
        }
        let (x1, y1) = (self.x + self.w - 1, self.y + self.h - 1);
        let mut spots = Vec::new();
        for x in self.x + 2..=x1 - 2 {
            spots.push((Pos::new(x, self.y), Dir::N));
            spots.push((Pos::new(x, y1), Dir::S));
        }
        for y in self.y + 2..=y1 - 2 {
            spots.push((Pos::new(self.x, y), Dir::W));
            spots.push((Pos::new(x1, y), Dir::E));
        }
        spots.retain(|(p, _)| {
            self.doors
                .iter()
                .all(|(d, _)| d == p || d.chebyshev(*p) > 1)
        });
        spots
    }

    /// The cell just inside each door.
    fn door_fronts(&self) -> impl Iterator<Item = Pos> + '_ {
        self.doors.iter().map(|(d, out)| d.step(out.opposite()))
    }
}

fn manhattan(a: Pos, b: Pos) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

pub(super) fn build(canvas: &mut Canvas, rng: &mut Rng, needs: Needs) -> Option<()> {
    let mut rooms = Vec::new();
    for v in vault::named(canvas, needs.vaults) {
        place_named(canvas, rng, &mut rooms, v)?;
    }
    if rng.chance(VAULT_CHANCE) {
        place_vault(canvas, rng, &mut rooms);
    }
    place_rooms(canvas, rng, &mut rooms);
    if rooms.iter().filter(|r| !r.vault).count() < 3 {
        return None;
    }
    for room in rooms.iter().filter(|r| !r.vault) {
        let (x0, y0, x1, y1) = room.interior();
        for y in room.y..room.y + room.h {
            for x in room.x..room.x + room.w {
                let inside = (x0..=x1).contains(&x) && (y0..=y1).contains(&y);
                let t = if inside {
                    canvas.pal.floor
                } else {
                    canvas.pal.wall
                };
                canvas.set(Pos::new(x, y), t);
            }
        }
    }
    for (a, b, required) in connections(&rooms, rng) {
        if !join(canvas, &mut rooms, a, b, rng) && required {
            return None;
        }
    }
    let up_room = place_stairs(canvas, rng, &rooms, needs)?;
    light_rooms(canvas, rng, &rooms, up_room);
    patrols(canvas, rng, &rooms, up_room, needs);
    if let Some(key) = needs.key {
        lock_rooms(canvas, rng, &rooms, up_room, key);
    }
    special_rooms(canvas, rng, &rooms, up_room);
    shops(canvas, rng, &rooms, up_room);
    Some(())
}

/// Locks the door of up to [`MAX_LOCKED`] rooms that have only one (not the arrival room, not a
/// room with stairs, not a vault) with the lock `key` opens, and leaves the key on a free spot
/// in a room that is not locked, so the player can always reach it (spec §8 "Doors"). A
/// doorway or an open door becomes a closed door to be locked. Whatever lies or walks in a
/// locked room stays there.
fn lock_rooms(canvas: &mut Canvas, rng: &mut Rng, rooms: &[Room], up_room: usize, key: ObjectId) {
    let lock = match canvas.content.object(key).key {
        Some(Key::Lock(ref id)) => Lock::new(Some(id)),
        _ => Lock::new(None),
    };
    let stairs = [canvas.up, canvas.down];
    let mut dead_ends: Vec<usize> = (0..rooms.len())
        .filter(|&i| {
            let room = &rooms[i];
            i != up_room
                && !room.vault
                && room.doors.len() == 1
                && !stairs.iter().flatten().any(|p| room.contains(*p))
        })
        .collect();
    rng.shuffle(&mut dead_ends);
    let count = rng.range(0, MAX_LOCKED as i32) as usize;
    let locked: Vec<usize> = dead_ends.into_iter().take(count).collect();
    let open: Vec<usize> = (0..rooms.len())
        .filter(|i| !rooms[*i].vault && !locked.contains(i))
        .collect();
    if locked.is_empty() {
        return;
    }
    let Some(at) = rng
        .pick(&open)
        .and_then(|&i| free_spot(canvas, rng, &rooms[i]))
    else {
        return;
    };
    canvas.objects.insert(at, vec![Stack::one(key)]);
    for i in locked {
        let (door, _) = rooms[i].doors[0];
        canvas.set(door, canvas.pal.door_closed);
        canvas.locks.insert(door, lock.clone());
    }
}

/// Rooms but the arrival room and vaults with a single door, not locked, may become a shop,
/// rarely and last of all (spec §7 "Shops"; `shop.rs`).
fn shops(canvas: &mut Canvas, rng: &mut Rng, rooms: &[Room], up_room: usize) {
    let candidates: Vec<shop::Candidate> = (rooms.iter().enumerate())
        .filter(|(i, r)| !r.vault && *i != up_room && r.doors.len() == 1)
        .filter(|(_, r)| !canvas.locks.contains_key(&r.doors[0].0))
        .map(|(_, r)| {
            let (x0, y0, x1, y1) = r.interior();
            let (door, out) = r.doors[0];
            shop::Candidate {
                min: Pos::new(x0, y0),
                max: Pos::new(x1, y1),
                door,
                out,
            }
        })
        .collect();
    shop::place(canvas, rng, &candidates);
}

/// Rooms but the arrival room and vaults may be special, each with the content's chance (spec §7
/// "Special rooms"): the room with its walls and doors, its effect's floor over its own.
fn special_rooms(canvas: &mut Canvas, rng: &mut Rng, rooms: &[Room], up_room: usize) {
    for (i, room) in rooms.iter().enumerate() {
        if room.vault || i == up_room {
            continue;
        }
        if !special::roll(canvas, rng) {
            continue;
        }
        let (min, max) = (
            Pos::new(room.x, room.y),
            Pos::new(room.x + room.w - 1, room.y + room.h - 1),
        );
        let floor = (min.x..=max.x)
            .flat_map(|x| (min.y..=max.y).map(move |y| Pos::new(x, y)))
            .filter(|p| canvas.is_walkable(*p))
            .count();
        let Some((effect, whole)) = special::pick(canvas, rng, floor) else {
            continue;
        };
        special::place(canvas, rng, SpecialRoom::rect(effect, min, max), whole);
    }
}

/// Stamps one vault at a random spot that leaves room for corridors to its entrances.
fn place_vault(canvas: &mut Canvas, rng: &mut Rng, rooms: &mut Vec<Room>) {
    for v in vault::shuffled(canvas, rng) {
        let (max_x, max_y) = (
            canvas.width() - 2 - v.width(),
            canvas.height() - 2 - v.height(),
        );
        if max_x < 2 || max_y < 2 {
            continue;
        }
        let at = Pos::new(rng.range(2, max_x), rng.range(2, max_y));
        if !vault::fits(canvas, v, at) {
            continue;
        }
        let doors = vault::stamp(canvas, v, at);
        rooms.push(Room {
            x: at.x,
            y: at.y,
            w: v.width(),
            h: v.height(),
            doors,
            vault: true,
        });
        return;
    }
}

/// Stamps `vault`, which the branch level names, on a random spot among all where it fits;
/// `None` if it fits nowhere.
fn place_named(canvas: &mut Canvas, rng: &mut Rng, rooms: &mut Vec<Room>, v: &Vault) -> Option<()> {
    let at = *rng.pick(&vault::spots(canvas, v))?;
    let doors = vault::stamp(canvas, v, at);
    rooms.push(Room {
        x: at.x,
        y: at.y,
        w: v.width(),
        h: v.height(),
        doors,
        vault: true,
    });
    Some(())
}

/// Scatters rooms by random attempts until the target count (about one per 200 cells).
fn place_rooms(canvas: &Canvas, rng: &mut Rng, rooms: &mut Vec<Room>) {
    let (w, h) = (canvas.width(), canvas.height());
    let max_iw = (w / 5).clamp(4, 16);
    let max_ih = (h / 3).clamp(3, 8);
    let target = ((w * h) / 200).clamp(4, 40) as usize + rooms.len();
    for _ in 0..target * 40 {
        if rooms.len() >= target {
            break;
        }
        let (rw, rh) = (rng.range(3, max_iw) + 2, rng.range(3, max_ih) + 2);
        if w - 1 - rw < 1 || h - 1 - rh < 1 {
            continue;
        }
        let room = Room {
            x: rng.range(1, w - 1 - rw),
            y: rng.range(1, h - 1 - rh),
            w: rw,
            h: rh,
            doors: Vec::new(),
            vault: false,
        };
        if rooms.iter().all(|r| !r.crowds(&room)) {
            rooms.push(room);
        }
    }
}

/// Pairs of rooms to join: a minimum spanning tree over room centers (Manhattan distance, ties by
/// index; required) followed by a few extra links to near rooms (optional, for loops).
fn connections(rooms: &[Room], rng: &mut Rng) -> Vec<(usize, usize, bool)> {
    let n = rooms.len();
    let dist = |i: usize, j: usize| manhattan(rooms[i].center(), rooms[j].center());
    let mut in_tree = vec![false; n];
    in_tree[0] = true;
    let mut links = Vec::new();
    for _ in 1..n {
        let mut best: Option<(i32, usize, usize)> = None;
        for i in (0..n).filter(|i| in_tree[*i]) {
            for j in (0..n).filter(|j| !in_tree[*j]) {
                if best.is_none_or(|(d, _, _)| dist(i, j) < d) {
                    best = Some((dist(i, j), i, j));
                }
            }
        }
        let Some((_, i, j)) = best else { break };
        in_tree[j] = true;
        links.push((i, j, true));
    }
    for _ in 0..n / 4 {
        let i = rng.index(n);
        let linked = |j: usize| {
            links
                .iter()
                .any(|&(a, b, _)| (a, b) == (i, j) || (a, b) == (j, i))
        };
        let nearest = (0..n)
            .filter(|&j| j != i && !linked(j))
            .min_by_key(|&j| (dist(i, j), j));
        if let Some(j) = nearest {
            links.push((i, j, false));
        }
    }
    links
}

/// Digs a corridor from room `a` to room `b` and puts doors at both ends. False if no corridor
/// fits.
fn join(canvas: &mut Canvas, rooms: &mut [Room], a: usize, b: usize, rng: &mut Rng) -> bool {
    let mut starts = Vec::new();
    let mut start_door: BTreeMap<Pos, (Pos, Dir)> = BTreeMap::new();
    for (door, out) in rooms[a].door_spots() {
        let existing = rooms[a].doors.iter().any(|(d, _)| *d == door);
        let cost = if existing { 0 } else { rng.range(1, 4) as u32 };
        let outside = door.step(out);
        starts.push((outside, cost));
        start_door.entry(outside).or_insert((door, out));
    }
    let mut goal_door: BTreeMap<Pos, (Pos, Dir)> = BTreeMap::new();
    for (door, out) in rooms[b].door_spots() {
        goal_door.entry(door.step(out)).or_insert((door, out));
    }
    let (floor, void) = (canvas.pal.floor, canvas.pal.void);
    let cost = |p: Pos| {
        if !canvas.is_inner(p) || canvas.is_stamped(p) || rooms.iter().any(|r| r.contains(p)) {
            None
        } else if canvas.get(p) == floor {
            Some(1)
        } else if canvas.get(p) == void {
            Some(3)
        } else {
            None
        }
    };
    let Some(path) = cheapest_path(
        canvas.width(),
        canvas.height(),
        &starts,
        cost,
        |p| goal_door.contains_key(&p),
        2,
    ) else {
        return false;
    };
    let ends = path
        .first()
        .and_then(|p| start_door.get(p))
        .zip(path.last().and_then(|p| goal_door.get(p)));
    let Some((&start, &goal)) = ends else {
        return false;
    };
    for p in &path {
        canvas.set(*p, floor);
    }
    for (room, (door, out)) in [(a, start), (b, goal)] {
        let room = &mut rooms[room];
        if room.doors.iter().any(|(d, _)| *d == door) {
            continue;
        }
        if !room.vault {
            let roll = rng.below(100);
            let terrain = match roll {
                0..60 => canvas.pal.door_closed,
                60..80 => canvas.pal.door_open,
                _ => canvas.pal.floor,
            };
            canvas.set(door, terrain);
        }
        room.doors.push((door, out));
    }
    true
}

/// A random free interior cell of `room` that is not just inside a door.
fn free_spot(canvas: &Canvas, rng: &mut Rng, room: &Room) -> Option<Pos> {
    let (x0, y0, x1, y1) = room.interior();
    (0..30)
        .map(|_| Pos::new(rng.range(x0, x1), rng.range(y0, y1)))
        .find(|p| canvas.is_free_floor(*p) && room.door_fronts().all(|f| f != *p))
}

/// Up stairs in a random room; down stairs (if needed) in a room among the farthest third.
/// Returns the index of the arrival room.
fn place_stairs(canvas: &mut Canvas, rng: &mut Rng, rooms: &[Room], needs: Needs) -> Option<usize> {
    let plain: Vec<usize> = (0..rooms.len()).filter(|i| !rooms[*i].vault).collect();
    let up_room = *rng.pick(&plain)?;
    let up = free_spot(canvas, rng, &rooms[up_room])?;
    canvas.place_stairs(up, StairsDir::Up);
    if !needs.down {
        return Some(up_room);
    }
    let from = rooms[up_room].center();
    let mut others: Vec<usize> = plain.into_iter().filter(|i| *i != up_room).collect();
    others.sort_by_key(|i| (-manhattan(from, rooms[*i].center()), *i));
    let far = &others[..others.len().div_ceil(3)];
    let down_room = *rng.pick(far)?;
    let down = free_spot(canvas, rng, &rooms[down_room])?;
    canvas.place_stairs(down, StairsDir::Down);
    Some(up_room)
}

/// Sconces in the interior corners of most rooms (one in small rooms, two in opposite corners of
/// large ones); a room that should be lit but has no free corner gets a lamp instead.
fn light_rooms(canvas: &mut Canvas, rng: &mut Rng, rooms: &[Room], up_room: usize) {
    for (i, room) in rooms.iter().enumerate() {
        if room.vault || (i != up_room && !rng.chance(LIT_CHANCE)) {
            continue;
        }
        let (x0, y0, x1, y1) = room.interior();
        let want = if (x1 - x0 + 1) * (y1 - y0 + 1) < 24 {
            1
        } else {
            2
        };
        // Clockwise; a second torch goes diagonally opposite the first, so the room is lit evenly.
        let corners = [
            Pos::new(x0, y0),
            Pos::new(x1, y0),
            Pos::new(x1, y1),
            Pos::new(x0, y1),
        ];
        let first = rng.index(4);
        let mut placed = 0;
        for c in [0, 2, 1, 3].map(|k| corners[(first + k) % 4]) {
            if placed < want && canvas.is_free_floor(c) && room.door_fronts().all(|f| f != c) {
                canvas.set(c, canvas.pal.sconce);
                placed += 1;
            }
        }
        if placed == 0
            && let Some(p) = free_spot(canvas, rng, room)
        {
            canvas.lights.insert(p, canvas.pal.lamp);
        }
    }
}

/// One to three creatures in rooms other than the arrival room: laps around a large room, or
/// back and forth across a small one.
fn patrols(canvas: &mut Canvas, rng: &mut Rng, rooms: &[Room], up_room: usize, needs: Needs) {
    if needs.creatures.is_empty() {
        return;
    }
    let mut candidates: Vec<usize> = (0..rooms.len())
        .filter(|i| *i != up_room && !rooms[*i].vault)
        .collect();
    rng.shuffle(&mut candidates);
    let count = rng.range(1, 3) as usize;
    for i in candidates.into_iter().take(count) {
        let Some(&creature) = rng.pick(needs.creatures) else {
            return;
        };
        let (x0, y0, x1, y1) = rooms[i].interior();
        let (iw, ih) = (x1 - x0 + 1, y1 - y0 + 1);
        let repeat = |dir: Dir, n: i32| std::iter::repeat_n(dir, n.max(0) as usize);
        let (start, route): (Pos, Vec<Dir>) = if iw >= 5 && ih >= 5 {
            let (a, b) = (iw - 3, ih - 3);
            let route = repeat(Dir::E, a)
                .chain(repeat(Dir::S, b))
                .chain(repeat(Dir::W, a))
                .chain(repeat(Dir::N, b))
                .collect();
            (Pos::new(x0 + 1, y0 + 1), route)
        } else {
            let k = iw - 1;
            let route = repeat(Dir::E, k).chain(repeat(Dir::W, k)).collect();
            (Pos::new(x0, (y0 + y1) / 2), route)
        };
        canvas.add_patrol(creature, start, route);
    }
}
