//! Where an encounter's fight opens (spec §7 "Overworld encounters"; `docs/dev/encounters.md`):
//! the cell's local area, as `>` would open it, the player and the party placed by outcome.
//!
//! The **way in** is the straight line from the arrival cell **A** (the edge cell
//! `mapgen::local::arrival` gives for the side the player came from) to the area's middle. The
//! player stands `[positions] player.<outcome>` percent of the way along it (the owner: the more
//! the encounter favours them, the nearer the edge; ambushed, in the middle). The party, from
//! the player: a fight `fight` cells further in, spread one cell apart across the way; lying in
//! wait `ambush` cells further in, walking back along the way past the player's cover; ambushed
//! `ambushed` cells around them, the lead on the edge's side; a failed evade `close` cells off,
//! between them and the edge. Each lands on the free cell nearest its mark, among the cells
//! reachable on foot from A (walkable, not water), so nobody stands across a river from the
//! fight. No stream is drawn here but the members' gear (`gear/<actor id>`).

use super::encounter::{How, Opening, Party};
use crate::action::Event;
use crate::content::{BASE_TURN, Content, CreatureSpawn, local_area_id, price};
use crate::geom::{Dir, Pos};
use crate::world::{ActorId, World, levels::travel::make_creature};
use std::collections::BTreeSet;

/// The cells reachable on foot from `from` on `level` (8 neighbours; walkable and not water).
fn reachable(level: &crate::world::LevelState, content: &Content, from: Pos) -> BTreeSet<Pos> {
    let mut seen = BTreeSet::from([from]);
    let mut queue = std::collections::VecDeque::from([from]);
    while let Some(p) = queue.pop_front() {
        for d in Dir::ALL {
            let q = p.step(d);
            if !seen.contains(&q)
                && level.is_walkable(content, q)
                && level.water_at(content, q).is_none()
            {
                seen.insert(q);
                queue.push_back(q);
            }
        }
    }
    seen
}

/// `a` moved `percent` of the way to `b`, rounded to the nearest cell.
fn lerp(a: Pos, b: Pos, percent: i32) -> Pos {
    let part =
        |from: i32, to: i32| from + ((to - from) * percent * 2 + 100 * (to - from).signum()) / 200;
    Pos::new(part(a.x, b.x), part(a.y, b.y))
}

/// `p` moved `cells` along the way's direction `(dx, dy)` (Chebyshev length `len`); negative
/// `cells` go back toward the edge.
fn along(p: Pos, (dx, dy): (i32, i32), cells: i32) -> Pos {
    let len = dx.abs().max(dy.abs()).max(1);
    let round = |v: i32| (v * 2 + len * v.signum()) / (2 * len);
    Pos::new(p.x + round(dx * cells), p.y + round(dy * cells))
}

/// The nearest cell to `target` for which `ok` holds, in ring order (as `free_cell_near`).
fn nearest(target: Pos, ok: impl Fn(Pos) -> bool, reach: i32) -> Option<Pos> {
    if ok(target) {
        return Some(target);
    }
    for r in 1..=reach {
        for dy in -r..=r {
            for dx in -r..=r {
                let q = Pos::new(target.x + dx, target.y + dy);
                if q.chebyshev(target) == r && ok(q) {
                    return Some(q);
                }
            }
        }
    }
    None
}

/// The offsets across the way, one cell apart from the middle out: 0, 1, -1, 2, -2, ...
fn spread(i: usize) -> i32 {
    let k = i32::try_from(i.div_ceil(2)).unwrap_or(0);
    if i % 2 == 1 { k } else { -k }
}

impl World {
    /// Opens the area `opening` names and places everyone (module docs), then perceives; an
    /// ambushed player is startled by the lead and owes one action, which the party takes.
    pub(super) fn open_encounter(&mut self, opening: Opening, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let Some(rules) = content.meetings() else {
            return;
        };
        let Opening { pending, how } = opening;
        let id = local_area_id(pending.cell);
        let from = self.state.current_level.clone();
        let Ok(level) = self.visit(&id, Some(&from)) else {
            return;
        };
        let (w, h) = (level.width(), level.height());
        let middle = Pos::new(w / 2, h / 2);
        let a = crate::mapgen::local::arrival(&level.terrain, &content, pending.side)
            .or(level.start)
            .unwrap_or(middle);
        let reach = reachable(level, &content, a);
        let occupied: BTreeSet<Pos> = (self.state.actors.values())
            .filter(|x| x.level == id)
            .map(|x| x.pos)
            .collect();
        let pos = rules.positions;
        let percent = match how {
            How::Ambush => pos.player.ambush,
            How::Fight | How::Hail | How::Hunt => pos.player.fight,
            How::Close => pos.player.close,
            How::Ambushed => pos.player.ambushed,
        };
        let mark = lerp(a, middle, i32::try_from(percent).unwrap_or(100));
        let far = w.max(h);
        let free = |q: Pos| reach.contains(&q) && !occupied.contains(&q);
        let level = &self.state.levels[&id];
        // Lying in wait, the player takes cover beside something that hides them, if any is near.
        let cover = |q: Pos| {
            Dir::ALL.into_iter().any(|d| {
                (level.terrain.get(q.step(d))).is_some_and(|&t| !content.terrain(t).transparent)
            })
        };
        let me = match how {
            How::Ambush => nearest(mark, |q| free(q) && cover(q), 2),
            _ => None,
        };
        let Some(me) = me.or_else(|| nearest(mark, free, far)) else {
            return;
        };
        if self.travel(&id, Some(me)).is_err() {
            return;
        }
        events.push(Event::LevelChanged {
            from,
            to: id.clone(),
        });
        // The way's direction; ambushed in the middle, still from the edge in.
        let way = match (middle.x - a.x, middle.y - a.y) {
            (0, 0) => pending.side.opposite().delta(),
            d => d,
        };
        let across = (-way.1.signum(), way.0.signum());
        let (cells, back) = match how {
            How::Fight | How::Hail | How::Hunt => (pos.fight, false),
            How::Ambush => (pos.ambush, false),
            How::Close => (pos.close, true),
            How::Ambushed => (pos.ambushed, true),
        };
        let center = along(me, way, if back { -cells } else { cells });
        // Ambushed: the lead toward the edge, the rest around the player.
        let ring: Vec<(i32, i32)> = vec![
            (-way.0, -way.1),
            across,
            (-across.0, -across.1),
            way,
            (-way.0 + across.0, -way.1 + across.1),
            (-way.0 - across.0, -way.1 - across.1),
            (way.0 + across.0, way.1 + across.1),
            (way.0 - across.0, way.1 - across.1),
        ];
        let entry = (content.encounter_entry(&self.state.campaign, &pending.entry)).map(|(_, e)| e);
        let known = self.known_party(&pending.entry, pending.cell);
        let count = if known.is_empty() {
            pending.members.len()
        } else {
            known.len()
        };
        // Whoever stands here now: the player's retinue stepped out beside them (`ally.rs`).
        let mut taken: BTreeSet<Pos> = (self.state.actors.values())
            .filter(|x| x.level == id && !x.in_retinue())
            .map(|x| x.pos)
            .collect();
        taken.insert(me);
        let mut spots = Vec::new();
        for i in 0..count {
            let target = match how {
                How::Ambushed => {
                    let d = ring[i % ring.len()];
                    along(me, d, cells)
                }
                _ => {
                    let s = spread(i);
                    Pos::new(center.x + across.0 * s, center.y + across.1 * s)
                }
            };
            let ok = |q: Pos| reach.contains(&q) && !taken.contains(&q);
            let Some(spot) = nearest(target, ok, far) else {
                break;
            };
            taken.insert(spot);
            spots.push(spot);
        }
        let turn = self.state.turn;
        let route = |spot: Pos| -> Vec<Dir> {
            // Lying in wait: they walk back along the way, past the player, and on.
            let toward = Dir::from_delta((me.x - spot.x).signum(), (me.y - spot.y).signum());
            let n = usize::try_from(spot.chebyshev(me) + 3).unwrap_or(3);
            toward.map_or_else(Vec::new, |d| {
                std::iter::repeat_n(d, n)
                    .chain(std::iter::repeat_n(d.opposite(), n))
                    .collect()
            })
        };
        let mut members: Vec<ActorId> = Vec::new();
        for (i, &spot) in spots.iter().enumerate() {
            let route = if how == How::Ambush {
                route(spot)
            } else {
                Vec::new()
            };
            let actor = match known.get(i) {
                Some(&actor) => {
                    if let Some(x) = self.state.actors.get_mut(&actor) {
                        x.pos = spot;
                        x.energy = 0;
                        x.mind = Default::default();
                        x.patrol =
                            (!route.is_empty()).then_some(crate::world::Patrol { route, step: 0 });
                    }
                    actor
                }
                None => {
                    let creature = pending.members[i];
                    // A peaceful party's dialogue: the lead's type speaks it (villagers all).
                    let npc = (entry.and_then(|e| e.npc.clone()))
                        .filter(|_| pending.members.first() == Some(&creature));
                    let spawn = CreatureSpawn {
                        creature,
                        faction: None,
                        alignment: None,
                        npc,
                        pos: spot,
                        route,
                        carry: Vec::new(),
                        equipment: Default::default(),
                        rolls_gear: true,
                        asleep: false,
                        status: Vec::new(),
                    };
                    make_creature(&mut self.state, &content, &id, &spawn)
                }
            };
            members.push(actor);
        }
        if known.is_empty() && !members.is_empty() {
            self.state.parties.push(Party {
                entry: pending.entry.clone(),
                level: id.clone(),
                members: members.clone(),
            });
        }
        let debt = price(1000, BASE_TURN);
        for &m in &members {
            let Some(x) = self.state.actors.get_mut(&m) else {
                continue;
            };
            match how {
                // Slipped past already: unaware of the player until hurt or the slip ends, and
                // one action behind (the player acts first).
                How::Ambush => {
                    x.mind.missed = Some(turn);
                    x.energy = -debt;
                }
                How::Fight | How::Close | How::Ambushed => {
                    x.mind.last_seen = Some(me);
                }
                How::Hail | How::Hunt => {}
            }
            // The player knows they are there, seen or not.
            x.mind.noticed = Some(turn);
        }
        self.perceive(events);
        if how == How::Ambushed {
            if let Some(&lead) = members.first() {
                self.startle(self.state.player, Some(lead), events);
            }
            // The player owes one action: the party takes it.
            if self.state.death.is_none() {
                self.run_turn(debt, events);
                self.perceive(events);
            }
        }
    }
}
