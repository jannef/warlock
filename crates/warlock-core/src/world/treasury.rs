//! The hint on arrival (spec §7 "Special rooms"; docs/design/chests-and-treasure.md §6.5): on
//! every arrival on a level (and at the start of a run), for each effect with an `arrive` line
//! that has a room on the level the player has not entered (nor stands in), the line, once per
//! effect however many such rooms there are. Nothing is stored: `SpecialRoom::entered` is the
//! record, so the line comes again on every return until the room is entered. With Treasure
//! sense (`TalentMods::treasure_bearing`) the line ends with the way to the room's middle from
//! where the player arrived, in eight winds (the nearest such room of the effect). No draw.

use super::WorldState;
use crate::content::{Content, RoomEffectId};
use crate::geom::{Dir, Pos};

/// The lines said as the player arrives on the current level (see the module docs).
pub(super) fn arrive_lines(state: &WorldState, content: &Content) -> Vec<String> {
    let player = state.player();
    let here = player.pos;
    let bearing = !player.talents.is_empty() && player.talent_mods(content).treasure_bearing;
    let mut effects: Vec<(RoomEffectId, Pos)> = Vec::new();
    for room in &state.current().rooms {
        if room.entered || room.contains(here) {
            continue;
        }
        if content.room_effect(room.effect).arrive.is_none() {
            continue;
        }
        let middle = Pos::new((room.min.x + room.max.x) / 2, (room.min.y + room.max.y) / 2);
        match effects.iter_mut().find(|(e, _)| *e == room.effect) {
            Some((_, nearest)) if middle.dist2(here) < nearest.dist2(here) => *nearest = middle,
            Some(_) => {}
            None => effects.push((room.effect, middle)),
        }
    }
    effects.sort_by_key(|(e, _)| *e);
    (effects.into_iter())
        .filter_map(|(effect, middle)| {
            let line = content.room_effect(effect).arrive.clone()?;
            let way = Dir::toward(here, middle).filter(|_| bearing);
            Some(match way {
                Some(dir) => match line.strip_suffix('.') {
                    Some(stem) => format!("{stem}, to the {}.", dir.compass_name()),
                    None => format!("{line} To the {}.", dir.compass_name()),
                },
                None => line,
            })
        })
        .collect()
}
