//! `cargo xtask gen-vast`: deterministic generator for `assets/levels/vast.toml` (spec §3 Level 2).
//!
//! Pipeline: cellular-automata caverns, keep the largest connected cave, carve a winding river of
//! running water (its flow follows the channel south; a few boulders break it), scatter pillars,
//! stamp walled rooms with doors (joined to the caves by a dug corridor), place the stairs (up to
//! the hub, beside them down into the generated `depths` branch and up out to the overworld at the
//! cavern mouth) and player start, drop everything unreachable, then place patrolling creatures
//! whose routes go out along walkable cells and return along the same path, and finally lights
//! (lamps and glowing fungus). Uses its own SplitMix64 so the output never depends on the game's
//! RNG or on platform float behavior.

use std::collections::{BTreeMap, VecDeque};
use std::fmt::Write as _;

pub const WIDTH: usize = 420;
pub const HEIGHT: usize = 300;
const SEED: u64 = 0x5741_524c_4f43_4b15;
/// Seed of the boulders' own stream, so placing them leaves the rest of the level as it was.
const BOULDER_SEED: u64 = 0x426f_756c_6465_7273;
/// Percent of the river's rows that get a boulder, where the river is wide enough.
const BOULDER_ROWS: usize = 8;
const ROOM_ATTEMPTS: usize = 60;
const ROOMS: usize = 8;
const CREATURES: usize = 16;
const ROOM_CREATURES: usize = 4;

const WALL: u8 = b'#';
const FLOOR: u8 = b'.';
const WATER: u8 = b'~';
const DOOR: u8 = b'+';
const ALTAR: u8 = b'_';
const STAIRS: u8 = b'<';
/// Creature types hostile to the player: placed in the `peaceful` faction on this level.
const HOSTILE_KINDS: [&str; 2] = ["rat", "kobold"];

/// Stairs down into the procedurally generated `depths` branch (`assets/data/dungeons.toml`).
const DEPTHS: u8 = b'>';
/// Stairs up and out to the overworld (the cavern mouth there leads back down).
const MOUTH: u8 = b'^';
const START: u8 = b'@';
/// A hanging lamp over floor (legend light).
const LAMP: u8 = b'*';
/// Glowing fungus (terrain that emits faint light).
const FUNGUS: u8 = b',';
/// Most `LAMP_BLOCK`×`LAMP_BLOCK` blocks of cave get a lamp.
const LAMP_BLOCK: usize = 14;
const FUNGUS_PATCHES: usize = 80;

struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform-ish in `0..n` (modulo bias is irrelevant here).
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    /// Inclusive range.
    fn range(&mut self, lo: usize, hi: usize) -> usize {
        lo + self.below(hi - lo + 1)
    }

    fn percent(&mut self, p: usize) -> bool {
        self.below(100) < p
    }
}

#[derive(Clone)]
struct Map {
    cells: Vec<u8>,
}

type P = (usize, usize);

const CARDINALS: [(i32, i32, char); 4] = [(0, -1, 'N'), (1, 0, 'E'), (0, 1, 'S'), (-1, 0, 'W')];

impl Map {
    fn get(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
            WALL
        } else {
            self.cells[y as usize * WIDTH + x as usize]
        }
    }

    /// `test(cell)` for in-bounds cells; false outside the map.
    fn get_inside(&self, x: i32, y: i32, test: impl Fn(u8) -> bool) -> bool {
        x >= 0 && y >= 0 && x < WIDTH as i32 && y < HEIGHT as i32 && test(self.get(x, y))
    }

    fn set(&mut self, x: i32, y: i32, c: u8) {
        if x >= 0 && y >= 0 && x < WIDTH as i32 && y < HEIGHT as i32 {
            self.cells[y as usize * WIDTH + x as usize] = c;
        }
    }

    fn passable(c: u8) -> bool {
        matches!(
            c,
            FLOOR | WATER | DOOR | ALTAR | STAIRS | DEPTHS | MOUTH | START
        )
    }

    /// Cells a patrolling creature may step on (closed doors are not walkable).
    fn walkable(&self, x: i32, y: i32) -> bool {
        matches!(self.get(x, y), FLOOR | WATER)
    }

    fn count_walls8(&self, x: i32, y: i32) -> usize {
        let mut n = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if (dx, dy) != (0, 0) && self.get(x + dx, y + dy) == WALL {
                    n += 1;
                }
            }
        }
        n
    }

    fn all_floor3(&self, x: i32, y: i32) -> bool {
        (-1..=1).all(|dy| (-1..=1).all(|dx| self.get(x + dx, y + dy) == FLOOR))
    }

    /// 4-connected reachability from `from` over passable cells.
    fn flood(&self, from: P) -> Vec<bool> {
        let mut seen = vec![false; WIDTH * HEIGHT];
        if !Self::passable(self.get(from.0 as i32, from.1 as i32)) {
            return seen;
        }
        let mut queue = VecDeque::from([from]);
        seen[from.1 * WIDTH + from.0] = true;
        while let Some((x, y)) = queue.pop_front() {
            for (dx, dy, _) in CARDINALS {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                if nx < 0 || ny < 0 || nx >= WIDTH as i32 || ny >= HEIGHT as i32 {
                    continue;
                }
                let i = ny as usize * WIDTH + nx as usize;
                if !seen[i] && Self::passable(self.cells[i]) {
                    seen[i] = true;
                    queue.push_back((nx as usize, ny as usize));
                }
            }
        }
        seen
    }
}

fn caverns(rng: &mut SplitMix64) -> Map {
    let mut map = Map {
        cells: vec![WALL; WIDTH * HEIGHT],
    };
    for y in 1..HEIGHT - 1 {
        for x in 1..WIDTH - 1 {
            if !rng.percent(46) {
                map.cells[y * WIDTH + x] = FLOOR;
            }
        }
    }
    for _ in 0..5 {
        let mut next = map.clone();
        for y in 0..HEIGHT as i32 {
            for x in 0..WIDTH as i32 {
                let n = map.count_walls8(x, y);
                let wall = n >= 5 || (map.get(x, y) == WALL && n >= 4);
                next.set(x, y, if wall { WALL } else { FLOOR });
            }
        }
        map = next;
    }
    // Keep only the largest cave.
    let mut best: Vec<bool> = Vec::new();
    let mut best_size = 0;
    let mut claimed = vec![false; WIDTH * HEIGHT];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if claimed[y * WIDTH + x] || map.cells[y * WIDTH + x] != FLOOR {
                continue;
            }
            let region = map.flood((x, y));
            let size = region.iter().filter(|r| **r).count();
            for (c, r) in claimed.iter_mut().zip(&region) {
                *c |= *r;
            }
            if size > best_size {
                best_size = size;
                best = region;
            }
        }
    }
    for (cell, keep) in map.cells.iter_mut().zip(&best) {
        if !*keep {
            *cell = WALL;
        }
    }
    map
}

/// A winding north-south river of running water with a floor bank carved where rock would hug it.
/// Returns the flow of each river cell (see [`flow_arrows`]).
fn river(map: &mut Map, rng: &mut SplitMix64) -> Vec<Option<char>> {
    let mut cx = (WIDTH * 11 / 20) as i32;
    let mut vx = 0i32;
    let mut half = 1i32;
    // Each row and the center of its channel.
    let mut rows = Vec::with_capacity(HEIGHT);
    for y in 1..HEIGHT as i32 - 1 {
        if rng.percent(18) {
            vx = (vx + rng.range(0, 2) as i32 - 1).clamp(-1, 1);
        }
        if rng.percent(10) {
            half = (half + rng.range(0, 2) as i32 - 1).clamp(1, 2);
        }
        cx = (cx + vx).clamp(70, WIDTH as i32 - 70);
        for x in cx - half..=cx + half {
            map.set(x, y, WATER);
        }
        // Keep the channel contiguous when it steps sideways.
        if vx != 0 {
            map.set(cx - vx, y, WATER);
        }
        for bank in [cx - half - 1, cx + half + 1] {
            if map.get(bank, y) == WALL && rng.percent(70) {
                map.set(bank, y, FLOOR);
            }
        }
        rows.push((y, cx));
    }
    let flow = flow_arrows(map, &rows);
    boulders(map);
    flow
}

/// Boulders in the river (rock): the water right above one runs into it, where it foams. Only
/// where the river is wide, on water with water on all four sides, so the river stays connected,
/// no corridor or room ever reaches one before reaching the water around it, and later passes
/// behave as without them.
fn boulders(map: &mut Map) {
    let mut rng = SplitMix64(BOULDER_SEED);
    for y in 4..HEIGHT as i32 - 4 {
        if !rng.percent(BOULDER_ROWS) {
            continue;
        }
        let open = |m: &Map, x: i32, y: i32| {
            let water = |dx: i32, dy: i32| m.get(x + dx, y + dy) == WATER;
            water(0, 0)
                && CARDINALS.iter().all(|(dx, dy, _)| water(*dx, *dy))
                && (water(-2, 0) || water(2, 0))
        };
        let spots: Vec<i32> = (0..WIDTH as i32).filter(|&x| open(map, x, y)).collect();
        if !spots.is_empty() {
            let x = spots[rng.below(spots.len())];
            map.set(x, y, WALL);
        }
    }
}

/// The river flows south: each cell of a row toward where the channel continues in the next row
/// (`↓`, or `↘` / `↙` where it steps sideways); a cell at the edge with no water there turns
/// toward water below it (where the channel narrows). The last row runs into the rock at the
/// bottom of the map. The arrows are running water in the default legend
/// (`assets/data/legend.toml`).
fn flow_arrows(map: &Map, rows: &[(i32, i32)]) -> Vec<Option<char>> {
    let mut flow = vec![None; WIDTH * HEIGHT];
    for (i, &(y, cx)) in rows.iter().enumerate() {
        let step = rows.get(i + 1).map_or(0, |&(_, next)| next - cx);
        for x in 0..WIDTH as i32 {
            if map.get(x, y) != WATER {
                continue;
            }
            let ahead = |dx: i32| map.get(x + dx, y + 1) == WATER;
            let dx = [step, 0, -1, 1]
                .into_iter()
                .find(|dx| ahead(*dx))
                .unwrap_or(step);
            flow[y as usize * WIDTH + x as usize] = Some(match dx {
                -1 => '↙',
                1 => '↘',
                _ => '↓',
            });
        }
    }
    flow
}

fn pillars(map: &mut Map, rng: &mut SplitMix64) {
    let snapshot = map.clone();
    for y in 2..HEIGHT as i32 - 2 {
        for x in 2..WIDTH as i32 - 2 {
            if snapshot.all_floor3(x, y) && rng.percent(4) {
                map.set(x, y, WALL);
            }
        }
    }
}

struct Room {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl Room {
    /// Interior cells (inside the walls).
    fn interior(&self) -> impl Iterator<Item = P> + '_ {
        (self.y + 1..self.y + self.h - 1).flat_map(move |y| {
            (self.x + 1..self.x + self.w - 1).map(move |x| (x as usize, y as usize))
        })
    }

    fn overlaps(&self, o: &Room, pad: i32) -> bool {
        self.x - pad < o.x + o.w
            && o.x - pad < self.x + self.w
            && self.y - pad < o.y + o.h
            && o.y - pad < self.y + self.h
    }
}

/// Stamps walled rooms with one door each, connected to existing open ground by a dug corridor.
fn rooms(map: &mut Map, rng: &mut SplitMix64) -> Vec<Room> {
    let mut placed: Vec<Room> = Vec::new();
    for _ in 0..ROOM_ATTEMPTS {
        if placed.len() >= ROOMS {
            break;
        }
        let (w, h) = (rng.range(9, 17) as i32, rng.range(7, 11) as i32);
        let x = rng.range(30, WIDTH - 30 - w as usize) as i32;
        let y = rng.range(12, HEIGHT - 12 - h as usize) as i32;
        let room = Room { x, y, w, h };
        let clear_of_water =
            (y - 2..y + h + 2).all(|yy| (x - 2..x + w + 2).all(|xx| map.get(xx, yy) != WATER));
        if !clear_of_water || placed.iter().any(|p| p.overlaps(&room, 4)) {
            continue;
        }
        // Try door sides in a random order; the corridor must reach open ground quickly without
        // clipping another room.
        let first = rng.below(4);
        let mut chosen = None;
        for k in 0..4 {
            let (dx, dy, _) = CARDINALS[(first + k) % 4];
            let span = if dx == 0 { w } else { h } as usize;
            let along = rng.range(2, span - 3) as i32;
            let door = match (dx, dy) {
                (0, -1) => (x + along, y),
                (0, 1) => (x + along, y + h - 1),
                (-1, 0) => (x, y + along),
                _ => (x + w - 1, y + along),
            };
            let mut path = Vec::new();
            let mut cur = door;
            let mut reached = false;
            for _ in 0..45 {
                cur = (cur.0 + dx, cur.1 + dy);
                let probe = Room {
                    x: cur.0,
                    y: cur.1,
                    w: 1,
                    h: 1,
                };
                if cur.0 < 2
                    || cur.1 < 2
                    || cur.0 >= WIDTH as i32 - 2
                    || cur.1 >= HEIGHT as i32 - 2
                    || placed.iter().any(|p| p.overlaps(&probe, 2))
                {
                    break;
                }
                if matches!(map.get(cur.0, cur.1), FLOOR | WATER) {
                    reached = true;
                    break;
                }
                path.push(cur);
            }
            if reached {
                chosen = Some((door, path));
                break;
            }
        }
        let Some((door, path)) = chosen else { continue };
        for yy in y..y + h {
            for xx in x..x + w {
                let edge = xx == x || yy == y || xx == x + w - 1 || yy == y + h - 1;
                map.set(xx, yy, if edge { WALL } else { FLOOR });
            }
        }
        for (cx, cy) in path {
            map.set(cx, cy, FLOOR);
        }
        map.set(door.0, door.1, DOOR);
        if placed.len().is_multiple_of(2) {
            map.set(x + w / 2, y + h / 2, ALTAR);
        }
        placed.push(room);
    }
    placed
}

/// Run-length encodes a cardinal step list: `E3 N2`.
fn route_string(steps: &[usize]) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < steps.len() {
        let mut j = i;
        while j < steps.len() && steps[j] == steps[i] {
            j += 1;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        let _ = write!(out, "{}{}", CARDINALS[steps[i]].2, j - i);
        i = j;
    }
    out
}

/// Random walk out from `start`, then back along the same cells (so it always loops).
fn patrol_route(map: &Map, rng: &mut SplitMix64, start: P) -> Option<Vec<usize>> {
    for _ in 0..30 {
        let len = rng.range(5, 14);
        let mut pos = (start.0 as i32, start.1 as i32);
        let mut steps: Vec<usize> = Vec::new();
        let mut dir = rng.below(4);
        for _ in 0..len {
            if rng.percent(25) {
                dir = rng.below(4);
            }
            let mut moved = false;
            for turn in 0..4 {
                let d = (dir + turn) % 4;
                let (dx, dy, _) = CARDINALS[d];
                if map.walkable(pos.0 + dx, pos.1 + dy) {
                    pos = (pos.0 + dx, pos.1 + dy);
                    steps.push(d);
                    dir = d;
                    moved = true;
                    break;
                }
            }
            if !moved {
                break;
            }
        }
        if steps.len() >= 4 {
            let back: Vec<usize> = steps.iter().rev().map(|d| (d + 2) % 4).collect();
            steps.extend(back);
            return Some(steps);
        }
    }
    None
}

/// Lights (spec §3: the vast level has many light sources): a lamp beside the stairs and in every
/// room, lamps scattered through most blocks of cave, and patches of glowing fungus. Only plain
/// reachable floor is used, and both kinds stay walkable, so patrol routes remain valid. Returns
/// the number of light-emitting cells.
fn lights(
    map: &mut Map,
    rng: &mut SplitMix64,
    reach: &[bool],
    rooms: &[&Room],
    stairs: P,
) -> usize {
    let free = |map: &Map, (x, y): P| map.get(x as i32, y as i32) == FLOOR && reach[y * WIDTH + x];
    let mut placed = 0;
    let mut put = |map: &mut Map, p: P, c: u8| {
        if free(map, p) {
            map.cells[p.1 * WIDTH + p.0] = c;
            placed += 1;
            true
        } else {
            false
        }
    };
    // The stairs sit in an open 3×3 patch of floor; light it.
    put(map, (stairs.0, stairs.1 + 1), LAMP);
    for room in rooms {
        put(
            map,
            (
                (room.x + room.w / 2) as usize,
                (room.y + room.h / 2) as usize,
            ),
            LAMP,
        );
    }
    for by in (0..HEIGHT).step_by(LAMP_BLOCK) {
        for bx in (0..WIDTH).step_by(LAMP_BLOCK) {
            if !rng.percent(75) {
                continue;
            }
            for _ in 0..8 {
                let x = (bx + rng.below(LAMP_BLOCK)).min(WIDTH - 1);
                let y = (by + rng.below(LAMP_BLOCK)).min(HEIGHT - 1);
                if put(map, (x, y), LAMP) {
                    break;
                }
            }
        }
    }
    for _ in 0..FUNGUS_PATCHES {
        let (x, y) = (rng.range(2, WIDTH - 3), rng.range(2, HEIGHT - 3));
        if !put(map, (x, y), FUNGUS) {
            continue;
        }
        for _ in 0..rng.below(3) {
            let (dx, dy) = (rng.below(3), rng.below(3));
            put(map, (x + dx - 1, y + dy - 1), FUNGUS);
        }
    }
    placed
}

pub fn generate() -> Result<String, String> {
    let mut rng = SplitMix64(SEED);
    let mut map = caverns(&mut rng);
    let flow = river(&mut map, &mut rng);
    pillars(&mut map, &mut rng);
    let rooms = rooms(&mut map, &mut rng);

    // Stairs and start: the open 3x3 patch of floor nearest the west-central part of the level (so the camera scrolls from the start).
    let target = (80i32, HEIGHT as i32 / 2);
    let stairs = (2..HEIGHT as i32 - 2)
        .flat_map(|y| (2..WIDTH as i32 - 2).map(move |x| (x, y)))
        .filter(|&(x, y)| map.all_floor3(x, y))
        .min_by_key(|&(x, y)| ((x - target.0).pow(2) + (y - target.1).pow(2), y, x))
        .ok_or("no open floor for the stairs")?;
    map.set(stairs.0, stairs.1, STAIRS);
    map.set(stairs.0 + 1, stairs.1, START);
    map.set(stairs.0 - 1, stairs.1, DEPTHS);
    map.set(stairs.0, stairs.1 - 1, MOUTH);

    // Drop everything the player cannot walk to.
    let reach = map.flood((stairs.0 as usize, stairs.1 as usize));
    for (cell, ok) in map.cells.iter_mut().zip(&reach) {
        if !*ok && Map::passable(*cell) {
            *cell = WALL;
        }
    }
    let open = reach.iter().filter(|r| **r).count();
    if open < 30_000 {
        return Err(format!("only {open} reachable cells; change the seed"));
    }
    let reachable_rooms: Vec<&Room> = rooms
        .iter()
        .filter(|r| r.interior().all(|(x, y)| reach[y * WIDTH + x]))
        .collect();
    if reachable_rooms.len() < 3 {
        return Err(format!("only {} reachable rooms", reachable_rooms.len()));
    }

    // Creatures: some inside rooms, the rest scattered through the caves away from the start.
    let mut taken: Vec<P> = Vec::new();
    let mut legend: BTreeMap<char, (&'static str, String)> = BTreeMap::new();
    let pool: Vec<char> = ('a'..='z').chain('A'..='Z').chain('0'..='9').collect();
    let kinds = ["rat", "bat", "kobold"];
    let mut attempts = 0;
    while taken.len() < CREATURES && attempts < 5000 && legend.len() < pool.len() {
        attempts += 1;
        let in_room = taken.len() < ROOM_CREATURES;
        let (x, y) = if in_room {
            let room = reachable_rooms[taken.len() % reachable_rooms.len()];
            (
                rng.range(room.x as usize + 1, (room.x + room.w) as usize - 2),
                rng.range(room.y as usize + 1, (room.y + room.h) as usize - 2),
            )
        } else {
            (rng.range(2, WIDTH - 3), rng.range(2, HEIGHT - 3))
        };
        let dist2 = (x as i32 - stairs.0).pow(2) + (y as i32 - stairs.1).pow(2);
        if map.get(x as i32, y as i32) != FLOOR || !reach[y * WIDTH + x] || dist2 < 15 * 15 {
            continue;
        }
        let Some(steps) = patrol_route(&map, &mut rng, (x, y)) else {
            continue;
        };
        let ch = pool[legend.len()];
        legend.insert(ch, (kinds[rng.below(kinds.len())], route_string(&steps)));
        map.cells[y * WIDTH + x] = ch as u8;
        taken.push((x, y));
    }
    if taken.len() < CREATURES {
        return Err(format!("placed only {} patrols", taken.len()));
    }
    let stairs_at = (stairs.0 as usize, stairs.1 as usize);
    let lit = lights(&mut map, &mut rng, &reach, &reachable_rooms, stairs_at);
    if lit < 200 {
        return Err(format!("placed only {lit} lights"));
    }

    let mut out = String::new();
    out.push_str(
        "# GENERATED by `cargo xtask gen-vast` (xtask/src/vast.rs). Do not edit by hand;\n\
         # `cargo xtask gen-vast --check` fails when this file is stale.\n\
         # Spec §3 Level 2: caverns, a winding river, pillars, walled rooms with doors, patrols,\n\
         # and many lights: hanging lamps (*) and glowing fungus (,); no ambient light. Its rats and\n\
         # kobolds are peaceful folk (the showcase is safe; the arena is where fighting happens).\n\
         # Beside the stairs up to the hub, stairs down (>) lead into the generated depths and\n\
         # stairs up (^) out to the overworld, where the cavern mouth leads back down.\n\
         # The river is running water drawn as arrows the way it flows (data/legend.toml).\n",
    );
    out.push_str("name = \"The Vast Caverns\"\nmap = '''\n");
    // Rock that touches open ground is a visible wall; deeper rock stays void. The final cell is
    // always a wall so the loader (which drops blank trailing rows) keeps the full height.
    for y in 0..HEIGHT as i32 {
        let mut row = String::with_capacity(WIDTH + 1);
        for x in 0..WIDTH as i32 {
            let c = map.get(x, y);
            let touches_open =
                (-1..=1).any(|dy| (-1..=1).any(|dx| map.get_inside(x + dx, y + dy, |n| n != WALL)));
            row.push(match c {
                WALL if touches_open || (y == HEIGHT as i32 - 1 && x == WIDTH as i32 - 1) => '#',
                WALL => ' ',
                WATER => flow[y as usize * WIDTH + x as usize].unwrap_or('~'),
                other => other as char,
            });
        }
        out.push_str(&row);
        out.push('\n');
    }
    out.push_str("'''\n\n[legend]\n");
    out.push_str("\"@\" = { terrain = \"floor\", spawn = \"player\" }\n");
    out.push_str("\"<\" = { terrain = \"stairs_up\", to = \"hub\" }\n");
    out.push_str("\">\" = { terrain = \"stairs_down\", to = \"depths:1\" }\n");
    out.push_str("\"^\" = { terrain = \"stairs_up\", to = \"overworld\" }\n");
    out.push_str("\"*\" = { terrain = \"floor\", light = \"lamp\" }\n");
    out.push_str("\",\" = \"fungus\"\n");
    for (ch, (kind, route)) in &legend {
        // The showcase is safe: creatures whose type is hostile to the player live here as
        // "peaceful folk" (the arena and the depths are where fighting happens).
        let faction = if HOSTILE_KINDS.contains(kind) {
            ", faction = \"peaceful\""
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "\"{ch}\" = {{ terrain = \"floor\", creature = \"{kind}\"{faction}, route = \"{route}\" }}"
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use warlock_core::content::{Content, ContentFiles};

    #[test]
    fn generation_is_deterministic_and_loads() {
        let a = generate().expect("generates");
        assert_eq!(a, generate().expect("generates"));
        let mut files = ContentFiles::builtin();
        files.insert("levels/vast.toml", &a);
        let content = Content::load(&files).expect("vast level loads");
        let level = content.level("vast").expect("vast level exists");
        assert_eq!(
            (level.terrain.width(), level.terrain.height()),
            (WIDTH as i32, HEIGHT as i32)
        );
        assert!(level.creatures.len() >= 8);
        assert!(level.player_start.is_some());
        let targets: Vec<_> = level.stairs.iter().map(|s| s.to.as_deref()).collect();
        assert_eq!(targets, [Some("overworld"), Some("depths:1"), Some("hub")]);
        assert!(level.lights.len() >= 150, "{} lamps", level.lights.len());
        // The river is running water flowing south, with boulders for it to foam against.
        use warlock_core::Dir;
        assert!(level.flow.len() > 1000, "{} river cells", level.flow.len());
        assert!(
            level
                .flow
                .values()
                .all(|f| matches!(f.dir, Dir::S | Dir::SE | Dir::SW) && f.speed == 100)
        );
        let blocked = level
            .flow
            .iter()
            .filter(|(p, f)| !level.flow.contains_key(&p.step(f.dir)))
            .count();
        assert!(blocked >= 5, "{blocked} cells run into rock");
    }
}
