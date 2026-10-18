//! Noise and hearing (spec §5 "Hearing").
//!
//! **Noise**: actions make noise. A step makes the walker's `footsteps` (creature data) scaled by
//! the `step_noise` percent of the terrain stepped onto (water splashes, grass is soft); opening or
//! closing a door makes the door's `toggle_noise` (terrain data). [`noise`] is the one function
//! that decides how loud an action is, so skills such as moving silently can change it there.
//! Loudness is an integer, at most [`MAX_LOUDNESS`]; a creature or door without the field is
//! silent.
//!
//! **Propagation** ([`propagate`]): a noise spreads from its cell to the 8 neighbors through every
//! cell that passes sound, losing [`STEP_LOSS`] per orthogonal step and [`DIAGONAL_LOSS`] per
//! diagonal one, plus the `muffle` of each cell it enters (a closed door takes 30). Solid terrain
//! (neither walkable nor transparent: walls, rock, trees) blocks sound unless it has a `muffle`,
//! and a diagonal step needs one of the two cells beside it to let sound through unmuffled, so
//! sound goes around corners but never through walls or past a closed door's corner. The
//! loudness at a cell is that of its loudest path (integer Dijkstra, so the result does not
//! depend on search order). The search stops below the quietest threshold of the listeners in
//! reach and never leaves the square `loudness / STEP_LOSS` cells around the noise, so its cost
//! follows the loudness, not the level size. The overworld is silent: its steps are journeys.
//!
//! **Listeners**: every actor whose creature type has `hearing` (the faintest loudness it hears;
//! the player too) hears a noise made by another actor on its level when the loudness at its cell
//! is at least that. It keeps what it heard ([`Heard`]: kind, where, how loud) in
//! [`crate::world::Actor::heard`] until it next acts: a creature as its turn starts (the input
//! the creature AI reads, #77), the player when the next turn starts passing.
//!
//! **The player** perceives heard sounds whose source they do not see through
//! [`crate::perception::PerceptionView::sounds`]: the kind, how loud ([`Volume`]), and roughly
//! where ([`rough_place`]); never who made them. [`Recent`] sounds decide which are news worth a
//! message (and so stop a rest).

use crate::content::Content;
use crate::geom::{Dir, Pos};
use crate::world::{Actor, ActorId, LevelState, WorldState, fnv1a};
use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;

/// Loudness a sound loses per orthogonal step.
pub const STEP_LOSS: u16 = 10;
/// Loudness a sound loses per diagonal step (√2 · [`STEP_LOSS`], rounded).
pub const DIAGONAL_LOSS: u16 = 14;
/// The loudest noise anything makes; content is validated against it and terrain modifiers are
/// capped at it.
pub const MAX_LOUDNESS: u16 = 300;
/// The farthest a noise carries, in cells (Chebyshev): a noise loses at least [`STEP_LOSS`] a
/// cell.
pub const MAX_REACH: i32 = (MAX_LOUDNESS / STEP_LOSS) as i32;

/// Turns a perceived sound stays in the player's mind ([`Recent`]).
pub const RECALL_TURNS: u64 = 10;
/// How far apart (Chebyshev) two rough places of the same sound can be, on the same turn, and
/// still be the same sound: two rough places up to 2 cells off each.
const SAME_PLACE: i64 = 4;
/// How far a sound's source may have moved per turn and still be the same sound: a quick
/// creature takes up to two steps a turn (speed 2000).
const MOVED_PER_TURN: i64 = 2;
/// Sounds of one kind made this close together (Chebyshev) during one turn are one sound: a
/// quick walker's steps.
const ONE_SOURCE: i32 = 2;

/// What a noise sounds like.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundKind {
    Footsteps,
    DoorOpened,
    DoorClosed,
}

impl SoundKind {
    /// The kind's id in dumps.
    pub const fn name(self) -> &'static str {
        match self {
            SoundKind::Footsteps => "footsteps",
            SoundKind::DoorOpened => "door_opened",
            SoundKind::DoorClosed => "door_closed",
        }
    }

    /// What the player hears, as in "You hear …": `footsteps`, `a door open`.
    pub const fn heard_as(self) -> &'static str {
        match self {
            SoundKind::Footsteps => "footsteps",
            SoundKind::DoorOpened => "a door open",
            SoundKind::DoorClosed => "a door close",
        }
    }
}

/// Something an actor does that makes noise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    /// Steps onto this cell.
    Step(Pos),
    /// Opens the closed door on this cell (before it opens).
    OpenDoor(Pos),
    /// Closes the open door on this cell (before it closes).
    CloseDoor(Pos),
}

/// A noise made by an actor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Noise {
    pub kind: SoundKind,
    /// Where it is made.
    pub at: Pos,
    /// How loud it is there.
    pub loudness: u16,
    /// Who made it (it does not hear itself).
    pub maker: ActorId,
}

/// A noise an actor heard (see [`crate::world::Actor::heard`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heard {
    pub kind: SoundKind,
    /// Where the noise was made (exact: creatures may go there to investigate; the player
    /// perceives only a rough place).
    pub at: Pos,
    /// How loud it was at the listener: at least its hearing threshold.
    pub loudness: u16,
}

/// How loud a sound is to the player, by how far it is above their hearing threshold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Volume {
    /// Less than a cell's worth of loudness above the threshold: placed up to 2 cells off.
    Faint,
    /// Placed up to a cell off.
    Clear,
    /// Three cells' worth above the threshold or more: placed where it was made.
    Loud,
}

impl Volume {
    fn of(loudness: u16, threshold: u16) -> Self {
        match loudness.saturating_sub(threshold) {
            m if m >= 3 * STEP_LOSS => Volume::Loud,
            m if m >= STEP_LOSS => Volume::Clear,
            _ => Volume::Faint,
        }
    }

    /// How many cells off its rough place may be.
    fn blur(self) -> i32 {
        match self {
            Volume::Faint => 2,
            Volume::Clear => 1,
            Volume::Loud => 0,
        }
    }
}

/// A sound the player heard during the last turn from a source they do not see.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PerceivedSound {
    pub kind: SoundKind,
    /// Roughly where it came from ([`rough_place`]): a cell the player does not see.
    pub pos: Pos,
    pub volume: Volume,
}

/// A sound the player perceived lately (kind and rough place), kept [`RECALL_TURNS`] turns: a
/// sound of the same kind near it is no news (within 4 cells, plus 2 for every turn since: a
/// creature takes up to two steps a turn), so a patrol heard turn after turn gives one message,
/// not one a turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recent {
    pub kind: SoundKind,
    pub pos: Pos,
    pub turn: u64,
}

/// How loud `act` by `actor` is on `level`: the one place an action's noise is decided. `None`
/// if it is silent.
pub fn noise(content: &Content, level: &LevelState, actor: &Actor, act: Act) -> Option<Noise> {
    let terrain = |p: Pos| level.terrain.get(p).map(|t| content.terrain(*t));
    let (kind, at, loudness) = match act {
        Act::Step(to) => {
            let footsteps = u32::from(content.creature(actor.creature).footsteps);
            let percent = terrain(to).map_or(100, |t| u32::from(t.step_noise));
            (SoundKind::Footsteps, to, footsteps * percent / 100)
        }
        Act::OpenDoor(p) => (
            SoundKind::DoorOpened,
            p,
            terrain(p).map_or(0, |t| u32::from(t.toggle_noise)),
        ),
        Act::CloseDoor(p) => (
            SoundKind::DoorClosed,
            p,
            terrain(p).map_or(0, |t| u32::from(t.toggle_noise)),
        ),
    };
    let loudness = loudness.min(u32::from(MAX_LOUDNESS)) as u16;
    (loudness > 0).then_some(Noise {
        kind,
        at,
        loudness,
        maker: actor.id,
    })
}

/// Loudness of one noise around where it was made (see [`propagate`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoundField {
    /// Top-left cell of the square searched.
    min: Pos,
    side: i32,
    loudness: Vec<u16>,
}

impl SoundField {
    /// Loudness at `p`; 0 where the sound does not reach (or is below the search's floor).
    pub fn at(&self, p: Pos) -> u16 {
        self.index(p).map_or(0, |i| self.loudness[i])
    }

    fn index(&self, p: Pos) -> Option<usize> {
        let (x, y) = (p.x - self.min.x, p.y - self.min.y);
        ((0..self.side).contains(&x) && (0..self.side).contains(&y))
            .then(|| (y * self.side + x) as usize)
    }
}

/// How much loudness a sound loses entering `p`: `None` if it cannot (solid terrain, outside the
/// level).
fn muffle(level: &LevelState, content: &Content, p: Pos) -> Option<u16> {
    level
        .terrain
        .get(p)
        .and_then(|t| content.terrain(*t).muffle)
}

/// Whether sound may step diagonally from `p` by `(dx, dy)`: one of the two cells beside the step
/// lets sound through unmuffled.
fn corner_open(level: &LevelState, content: &Content, p: Pos, dx: i32, dy: i32) -> bool {
    let free = |q: Pos| muffle(level, content, q) == Some(0);
    free(Pos::new(p.x + dx, p.y)) || free(Pos::new(p.x, p.y + dy))
}

/// Spreads a noise of `loudness` made at `at` over `level` (see the module docs). Cells it reaches
/// with less than `floor` (at least 1) are not searched.
pub fn propagate(
    level: &LevelState,
    content: &Content,
    at: Pos,
    loudness: u16,
    floor: u16,
) -> SoundField {
    let reach = i32::from(loudness / STEP_LOSS);
    let side = 2 * reach + 1;
    let mut field = SoundField {
        min: Pos::new(at.x - reach, at.y - reach),
        side,
        loudness: vec![0; (side * side) as usize],
    };
    let floor = floor.max(1);
    let Some(start) = field.index(at) else {
        return field;
    };
    if loudness < floor {
        return field;
    }
    field.loudness[start] = loudness;
    // Loudest first: Dijkstra over the loudness lost.
    let mut queue = BinaryHeap::from([(loudness, at)]);
    while let Some((l, p)) = queue.pop() {
        if l < field.at(p) {
            continue;
        }
        for dir in Dir::ALL {
            let q = p.step(dir);
            let (Some(i), Some(m)) = (field.index(q), muffle(level, content, q)) else {
                continue;
            };
            let (dx, dy) = dir.delta();
            let step = if dx != 0 && dy != 0 {
                if !corner_open(level, content, p, dx, dy) {
                    continue;
                }
                DIAGONAL_LOSS
            } else {
                STEP_LOSS
            };
            let left = l.saturating_sub(step.saturating_add(m));
            if left >= floor && left > field.loudness[i] {
                field.loudness[i] = left;
                queue.push((left, q));
            }
        }
    }
    field
}

/// Makes `noise` on the current level: every other actor there with `hearing` that it reaches
/// loudly enough keeps it in its [`Actor::heard`] list. The overworld is silent.
pub(crate) fn make(state: &mut WorldState, content: &Content, noise: Noise) {
    let level = state.current();
    if level.kind == crate::content::LevelKind::Overworld {
        return;
    }
    let reach = i32::from(noise.loudness / STEP_LOSS);
    let listeners: Vec<(ActorId, Pos, u16)> = state
        .actors
        .values()
        .filter(|a| a.id != noise.maker && a.level == level.id)
        .filter(|a| a.pos.chebyshev(noise.at) <= reach)
        .filter_map(|a| {
            let threshold = content.creature(a.creature).hearing?;
            (threshold <= noise.loudness).then_some((a.id, a.pos, threshold))
        })
        .collect();
    let Some(floor) = listeners.iter().map(|&(_, _, t)| t).min() else {
        return;
    };
    let field = propagate(level, content, noise.at, noise.loudness, floor);
    for (id, pos, threshold) in listeners {
        let loudness = field.at(pos);
        if loudness >= threshold
            && let Some(actor) = state.actors.get_mut(&id)
        {
            actor.heard.push(Heard {
                kind: noise.kind,
                at: noise.at,
                loudness,
            });
        }
    }
}

/// The sounds in `heard` (the player's during one turn, with hearing `threshold`) the player
/// perceives: those whose source cell is not visible, each placed at its [`rough_place`]. A
/// sound followed by one of the same kind within [`ONE_SOURCE`] cells is the same walker's
/// earlier step and is left out.
pub(crate) fn perceive(
    level: &LevelState,
    content: &Content,
    heard: &[Heard],
    threshold: u16,
    turn: u64,
    hidden: impl Fn(Pos) -> bool,
) -> Vec<PerceivedSound> {
    let followed = |i: usize| {
        let h = &heard[i];
        heard[i + 1..]
            .iter()
            .any(|l| l.kind == h.kind && l.at.chebyshev(h.at) <= ONE_SOURCE)
    };
    heard
        .iter()
        .enumerate()
        .filter(|&(i, h)| hidden(h.at) && !followed(i))
        .map(|(_, h)| {
            let volume = Volume::of(h.loudness, threshold);
            PerceivedSound {
                kind: h.kind,
                pos: rough_place(level, content, h.at, volume, turn, &hidden),
                volume,
            }
        })
        .collect()
}

/// Where the player places a sound made at `at`: a cell up to [`Volume`]'s blur steps from it (0
/// for loud sounds) through cells that pass sound unmuffled, which the player does not see (they
/// would see that nothing is there). Which one is picked by a hash of the source and the turn:
/// deterministic, the same for the same turn, and not the source more often than any other.
pub fn rough_place(
    level: &LevelState,
    content: &Content,
    at: Pos,
    volume: Volume,
    turn: u64,
    hidden: impl Fn(Pos) -> bool,
) -> Pos {
    // Breadth-first, `blur` steps out; the cells in the order found.
    let mut found = vec![at];
    let mut frontier = vec![at];
    for _ in 0..volume.blur() {
        let mut next = Vec::new();
        for &p in &frontier {
            for dir in Dir::ALL {
                let q = p.step(dir);
                let (dx, dy) = dir.delta();
                let diagonal_ok = dx == 0 || dy == 0 || corner_open(level, content, p, dx, dy);
                if diagonal_ok && muffle(level, content, q) == Some(0) && !found.contains(&q) {
                    found.push(q);
                    next.push(q);
                }
            }
        }
        frontier = next;
    }
    let candidates: Vec<Pos> = found.into_iter().filter(|&p| hidden(p)).collect();
    if candidates.is_empty() {
        return at;
    }
    let mut key = [0u8; 16];
    key[..4].copy_from_slice(&at.x.to_le_bytes());
    key[4..8].copy_from_slice(&at.y.to_le_bytes());
    key[8..].copy_from_slice(&turn.to_le_bytes());
    candidates[(fnv1a(&key) % candidates.len() as u64) as usize]
}

/// The sounds of `sounds` (perceived on `turn`) that are news: no sound of the same kind was
/// perceived near its place lately ([`Recent`]). Updates `recent`: forgets sounds older than
/// [`RECALL_TURNS`] and remembers these.
pub(crate) fn news(
    recent: &mut Vec<Recent>,
    sounds: &[PerceivedSound],
    turn: u64,
) -> Vec<PerceivedSound> {
    recent.retain(|r| turn.saturating_sub(r.turn) <= RECALL_TURNS);
    let known = |s: &PerceivedSound| {
        recent.iter().any(|r| {
            let since = turn.saturating_sub(r.turn) as i64;
            let reach = SAME_PLACE + MOVED_PER_TURN * since;
            r.kind == s.kind && i64::from(r.pos.chebyshev(s.pos)) <= reach
        })
    };
    let news: Vec<PerceivedSound> = sounds.iter().filter(|s| !known(s)).copied().collect();
    for s in sounds {
        let entry = Recent {
            kind: s.kind,
            pos: s.pos,
            turn,
        };
        if !recent.contains(&entry) {
            recent.push(entry);
        }
    }
    news
}

impl PerceivedSound {
    /// "You hear footsteps", or for a faint sound "You faintly hear footsteps".
    pub fn you_hear(&self) -> String {
        let hear = match self.volume {
            Volume::Faint => "You faintly hear",
            Volume::Clear | Volume::Loud => "You hear",
        };
        format!("{hear} {}", self.kind.heard_as())
    }

    /// The message for this sound heard from `viewer`: "You hear footsteps to the north-east."
    pub fn message(&self, viewer: Pos) -> String {
        match Dir::toward(viewer, self.pos) {
            Some(dir) => format!("{} to the {}.", self.you_hear(), dir.compass_name()),
            None => format!("{}.", self.you_hear()),
        }
    }
}

#[cfg(test)]
mod tests;
