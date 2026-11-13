//! Escorts a quest places (#1141; `docs/dev/quests.md` "Escort quests"): an escort quest's
//! `waits` ([`Waits`]) puts its escort, a creature of the type it names speaking the escort's
//! npc dialogue, on the named place `at` of level `level`, for a level that cannot name a unique
//! character itself (a generated floor: a vault holds no npc).
//!
//! [`place_waiting`] runs as a level is made (`travel.rs::instantiate_level`) and as the quest
//! is given (`quest.rs::give`, for a level made before): for each active escort quest with a
//! `waits` on that level whose escort is nowhere in the run yet, the escort is made there, on
//! the place's cell or the nearest free one. Once made it is an actor like any other, so it is
//! never made twice: its death fails the quest, and the quest done, it is no longer active.
//!
//! Nothing here draws from any stream.

use super::WorldState;
use super::travel::make_creature;
use crate::content::{Content, CreatureSpawn, Goal, Speaker, Waits};
use crate::equipment::Equipment;
use crate::geom::Pos;

/// Makes the escorts waiting on level `level` (it must exist) that are not in the run yet (the
/// module docs).
pub(super) fn place_waiting(state: &mut WorldState, content: &Content, level: &str) {
    for def in content.quests() {
        let Goal::Escort {
            escort,
            waits: Some(waits),
            ..
        } = &def.goal
        else {
            continue;
        };
        if waits.level != level || !state.quests.is_active(def) {
            continue;
        }
        let Speaker::Npc(npc) = &content.dialogue(*escort).speaker else {
            continue;
        };
        if (state.actors.values()).any(|a| a.npc.as_deref() == Some(npc.as_str())) {
            continue;
        }
        let Some(pos) = spot(state, content, waits) else {
            continue;
        };
        let spawn = CreatureSpawn {
            creature: waits.creature,
            faction: None,
            alignment: None,
            npc: Some(npc.clone()),
            pos,
            route: Vec::new(),
            carry: Vec::new(),
            equipment: Equipment::default(),
            rolls_gear: false,
            asleep: false,
            status: Vec::new(),
        };
        make_creature(state, content, level, &spawn);
    }
}

/// The cell `waits` puts its escort on: its place's, or the nearest walkable one no one stands
/// on (ring search, in row order); `None` if there is none.
pub(super) fn spot(state: &WorldState, content: &Content, waits: &Waits) -> Option<Pos> {
    let here = &state.levels[&waits.level];
    let named = |places: &std::collections::BTreeMap<Pos, crate::content::Place>| {
        (places.iter())
            .find(|(_, p)| p.name == waits.at)
            .map(|(&at, _)| at)
    };
    let at = named(&here.places)
        .or_else(|| content.level(&waits.level).and_then(|l| named(&l.places)))?;
    let free = |q: Pos| here.is_walkable(content, q) && state.actor_at(&waits.level, q).is_none();
    let reach = here.width().max(here.height());
    (0..reach)
        .flat_map(|r| {
            (-r..=r).flat_map(move |dy| {
                (-r..=r)
                    .map(move |dx| Pos::new(at.x + dx, at.y + dy))
                    .filter(move |q| q.chebyshev(at) == r)
            })
        })
        .find(|&q| free(q))
}
