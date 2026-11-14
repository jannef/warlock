//! The Pits' records and the `portal` miscast (spec §7 "The Pits", #1297; the miscasts design
//! §7.2–§7.3). A Pit is an instance of the portal branch (Hellegeat), its floors
//! `hellegeat@S:N` made from the run seed and the id alone (`mapgen`, `endless.rs`), so all a run
//! keeps of one is its record here and the mouth on the level it opened on.
//!
//! - **The record** ([`WorldState::pits`], saved; left out while empty): `S`, the Pit's integer,
//!   drawn from the gameplay stream at the miscast (`rng.below(u32::MAX) + 1`, drawn again while
//!   it is taken), and [`Pit`]: `from`, the level the mouth is on, and `sealed` (its warden is
//!   dead, below).
//! - **The portal** ([`World::open_pit`]): on the caster's level, in play, the first plain free
//!   cell around the caster ([`World::pit_mouth`]: walkable, no one on it, no stairs, door,
//!   chest, statue, grave, light, trap, hidden location, place, or flow; out to the `[pit]`
//!   `mouth_reach` of `data/rules/tuning.toml`) becomes stairs down to `hellegeat@S:1` named "the
//!   Pit", as [`World::open_lair`] opens a lair's; the entry's gate comes to the first free cell
//!   around the caster but the mouth. **Refused**, the gate alone made and said so
//!   ([`World::pit_refused`]), on the overworld, on a Pit's own floor, and where no cell is plain.
//!   A creature's miscast on a level not in play does nothing.
//! - **The way out**: a Pit's first floor's `<` leads to its record's `from` whenever the floor
//!   is made ([`WorldState::pit_way_out`], read by `World::visit`), so a climb from the second
//!   floor makes the first with its `<` to the mouth, not back down.
//! - **The orphan rule** ([`World::drop_orphan_pits`], after an action that changed level, once
//!   the floors left are forgotten): on an arrival outside every Pit, a record whose `from` level
//!   is gone or holds no `>` into its Pit is dropped (a Pit opened on an endless floor, left and
//!   forgotten with its mouth).
//! - **The warden** (#1298): the portal branch's boss row places each Pit's warden in its hall on
//!   the last floor (`mapgen::generate_for`). Its death ([`World::seal_pit`], one line in
//!   `fight.rs::creature_dies`) seals its own Pit and no other: `sealed` is set, and the slain
//!   list (`Quests::slain`, by type) is never written. A sealed Pit's floor made again reads the
//!   warden as slain ([`WorldState::pit_slain`], `Visit::slain`): the hall without it.
//! - **The mouth closes** ([`World::close_mouth`], in [`World::drop_orphan_pits`]): on an arrival
//!   outside every Pit, a sealed Pit's `>` on its `from` level becomes rubble and its link goes,
//!   so the orphan rule drops the record. Closing on arrival, not at the kill, never seals the
//!   player in.
//! - **The dev scenario** `hellegeat` (the showcase's) starts on `hellegeat@1:1`: a run that
//!   starts on a Pit's floor records it with `from` the level its campaign's own start scenario
//!   begins on ([`start_from`]), the hub.
//!
//! Determinism: one draw (more on a collision) from the gameplay stream after the entry's
//! message; the cells are found in a fixed order.

use super::{World, WorldState};
use crate::action::Event;
use crate::content::{
    Content, CreatureId, LevelKind, StairsDir, StairsLink, parse_branch_level, parse_instance,
};
use crate::geom::Pos;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// What a run keeps of one Pit (module docs).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pit {
    /// The level its mouth is on: where its first floor's `<` leads.
    pub(crate) from: String,
    /// Its warden is dead (#1298).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub(crate) sealed: bool,
}

impl Pit {
    /// An open Pit whose mouth is on level `from`.
    pub(super) fn new(from: String) -> Self {
        Pit {
            from,
            sealed: false,
        }
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn sealed(&self) -> bool {
        self.sealed
    }
}

/// Why a portal opens no Pit (module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Refused {
    /// The overworld, or no plain cell near: the ground will not open.
    Ground,
    /// A Pit's own floor.
    InPit,
}

/// The Pit a level id is a floor of, and its depth: `hellegeat@7:3` → `(7, 3)`.
pub fn pit_of(content: &Content, id: &str) -> Option<(u32, u32)> {
    if !content.is_portal_floor(id) {
        return None;
    }
    let (branch, depth) = parse_branch_level(id)?;
    parse_instance(branch).map(|(_, s)| (s, depth))
}

/// The level a run of campaign `campaign` starting on `start` records as its Pit's `from`, when
/// `start` is a Pit's floor: where the campaign's own start scenario begins (module docs).
pub(super) fn start_from(content: &Content, campaign: &str, start: &str) -> Option<String> {
    pit_of(content, start)?;
    let scenario = content.scenario(&content.campaign(campaign)?.start)?;
    (!content.is_portal_floor(&scenario.start_level)).then(|| scenario.start_level.clone())
}

impl WorldState {
    /// Where the `<` of level `id` leads when it is made, if it is a Pit's first floor with a
    /// record: the record's `from`.
    pub(super) fn pit_way_out(&self, content: &Content, id: &str) -> Option<String> {
        match pit_of(content, id)? {
            (s, 1) => self.pits.get(&s).map(|p| p.from.clone()),
            _ => None,
        }
    }

    /// What a floor of level `id` made now counts as slain, if it is a Pit's floor: its warden
    /// when the Pit is sealed, else no one (module docs; `mapgen::Visit::slain`).
    pub(super) fn pit_slain(&self, content: &Content, id: &str) -> Option<BTreeSet<String>> {
        let (s, _) = pit_of(content, id)?;
        let sealed = self.pits.get(&s).is_some_and(|p| p.sealed);
        let (def, _) = content.branch_level(id)?;
        Some(
            (def.bosses.iter())
                .filter(|_| sealed)
                .map(|b| content.creature(b.creature).id.clone())
                .collect(),
        )
    }

    /// Dev: the Pits' records, by their integers.
    pub fn pits(&self) -> &std::collections::BTreeMap<u32, Pit> {
        &self.pits
    }
}

impl World {
    /// Why a portal at `here` on the current level opens no Pit, if it does not.
    pub(super) fn pit_refused(&self, here: Pos) -> Option<Refused> {
        let level = self.state.current();
        if self.content.is_portal_floor(&level.id) {
            return Some(Refused::InPit);
        }
        if level.kind == LevelKind::Overworld || self.pit_mouth(here).is_none() {
            return Some(Refused::Ground);
        }
        None
    }

    /// The cell a Pit's mouth opens on around `here` on the current level (module docs).
    fn pit_mouth(&self, here: Pos) -> Option<Pos> {
        let level = self.state.current();
        let content = &self.content;
        let plain = |p: Pos| {
            let Some(&t) = level.terrain.get(p) else {
                return false;
            };
            let def = content.terrain(t);
            def.stairs.is_none()
                && def.chest.is_none()
                && def.statue.is_none()
                && !def.grave
                && def.light.is_none()
                && !def.is_door()
                && level.stairs_at(p).is_none()
                && !level.traps.contains_key(&p)
                && !level.hidden.contains_key(&p)
                && !level.places.contains_key(&p)
                && !level.flow.contains_key(&p)
                && !level.lights.contains_key(&p)
        };
        let reach = self.content.tuning().pit.mouth_reach;
        self.free_cell_within(&level.id, here, reach, plain)
    }

    /// A `portal` miscast by a caster at `here` (module docs): a Pit opened with `gate` at its
    /// mouth, or the gate alone where it is refused.
    pub(super) fn open_pit(&mut self, here: Pos, gate: CreatureId) {
        let mouth = match self.pit_refused(here) {
            Some(_) => None,
            None => self.pit_mouth(here),
        };
        if let Some(at) = mouth {
            let mut s = self.state.rng.below(u32::MAX) + 1;
            while self.state.pits.contains_key(&s) {
                s = self.state.rng.below(u32::MAX) + 1;
            }
            let Some(def) = self.content.dungeons().find(|d| d.portal.is_some()) else {
                return;
            };
            let to = def.pit_level_id(s, 1);
            let from = self.state.current_level.clone();
            self.state.pits.insert(s, Pit::new(from));
            if let Some(stairs) = self.content.ids().stairs_down {
                self.set_terrain(at, stairs);
            }
            self.state.current_mut().stairs.push(StairsLink {
                pos: at,
                dir: StairsDir::Down,
                to: Some(to),
                name: Some("the Pit".into()),
            });
        }
        self.miscast_summon(here, gate, mouth);
    }

    /// Creature type `creature` died on level `level`: if it is the warden of the Pit that level
    /// is a floor of, that Pit is sealed, and the player is told if they are there (module docs).
    pub(super) fn seal_pit(&mut self, level: &str, creature: CreatureId) {
        let Some((s, depth)) = pit_of(&self.content, level) else {
            return;
        };
        let Some((def, _)) = self.content.branch_level(level) else {
            return;
        };
        if def.boss_at(depth).is_none_or(|b| b.creature != creature) {
            return;
        }
        let Some(pit) = self.state.pits.get_mut(&s) else {
            return;
        };
        pit.sealed = true;
        if self.state.current_level == level {
            self.say("The Pit shudders. Far above, stone grinds on stone.");
        }
    }

    /// Sealed Pit `s`'s mouth closes: its `>` on the level it opened on becomes rubble and leads
    /// nowhere, and the player is told if they stand on that level (module docs).
    fn close_mouth(&mut self, s: u32) {
        let Some(from) = self.state.pits.get(&s).map(|p| p.from.clone()) else {
            return;
        };
        let content = self.content.clone();
        let Some(level) = self.state.levels.get_mut(&from) else {
            return;
        };
        let Some(i) = (level.stairs.iter()).position(|link| {
            link.dir == StairsDir::Down
                && (link.to.as_deref()).and_then(|to| pit_of(&content, to)) == Some((s, 1))
        }) else {
            return;
        };
        let at = level.stairs.remove(i).pos;
        let Some(rubble) = content.ids().rubble else {
            return;
        };
        if self.state.current_level != from {
            if let Some(level) = self.state.levels.get_mut(&from) {
                level.terrain.set(at, rubble);
                // The cache keeps every level visited (#1518).
                self.lights.terrain_changed(level, &content, at);
            }
            return;
        }
        self.set_terrain(at, rubble);
        self.say("The stair crumbles to rubble behind you.");
    }

    /// After an action that changed level, on an arrival outside every Pit: a sealed Pit's mouth
    /// closes, and the orphan rule (module docs).
    pub(super) fn drop_orphan_pits(&mut self, events: &[Event]) {
        if self.state.pits.is_empty()
            || !(events.iter()).any(|e| matches!(e, Event::LevelChanged { .. }))
            || self.content.is_portal_floor(&self.state.current_level)
        {
            return;
        }
        let sealed: Vec<u32> = (self.state.pits.iter())
            .filter_map(|(&s, pit)| pit.sealed.then_some(s))
            .collect();
        for s in sealed {
            self.close_mouth(s);
        }
        let content = self.content.clone();
        let levels = &self.state.levels;
        self.state.pits.retain(|&s, pit| {
            levels.get(&pit.from).is_some_and(|level| {
                (level.stairs.iter()).any(|link| {
                    link.dir == StairsDir::Down
                        && (link.to.as_deref()).and_then(|to| pit_of(&content, to)) == Some((s, 1))
                })
            })
        });
    }
}
