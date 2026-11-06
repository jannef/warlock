//! Ways up from below (spec §7 "Ways up"; `DungeonDef::below`): a level that another branch's
//! deepest level leads down to gets a staircase up to it, besides its own. It stands in the
//! roomy spot nearest, by walking, to the entrance of a vault the level names (Fyrdelf 12: the
//! drake's hoard), ties in row order, so it draws nothing; on a level that names no vault, nearest
//! to its own way up. A level no branch leads down to is untouched.

use super::Canvas;
use super::connect::distances;
use crate::content::{StairsDir, StairsLink};
use crate::geom::Pos;
use crate::grid::Grid;

/// Places a staircase up to each of `ways` (level ids) on `canvas`, built and not yet walled,
/// and returns their links; `None` if a staircase finds no room (the attempt fails).
pub(super) fn place(canvas: &mut Canvas, ways: &[String]) -> Option<Vec<StairsLink>> {
    let mut links = Vec::new();
    for to in ways {
        let pos = nearest_roomy(canvas)?;
        canvas.set(pos, canvas.pal.stairs_up);
        links.push(StairsLink {
            pos,
            dir: StairsDir::Up,
            to: Some(to.clone()),
            name: None,
        });
    }
    Some(links)
}

/// The roomy spot (3×3 of free floor) nearest by walking to the named vaults' entrances (or the
/// level's up staircase), first in row order among the nearest.
fn nearest_roomy(canvas: &Canvas) -> Option<Pos> {
    let anchors: Vec<Pos> = if canvas.named_doors.is_empty() {
        canvas.up.into_iter().collect()
    } else {
        canvas.named_doors.clone()
    };
    let (w, h) = (canvas.width(), canvas.height());
    let mut best = Grid::new(w, h, u32::MAX);
    for at in anchors {
        let dist = distances(w, h, at, |p| canvas.is_passable(p));
        for p in canvas.cells() {
            best[p] = best[p].min(dist[p]);
        }
    }
    (canvas.cells())
        .filter(|p| best[*p] != u32::MAX && canvas.is_open_area(*p))
        .min_by_key(|p| best[*p])
}
