//! The witch's familiar (spec §8 "Allies": "The familiar"; docs/design/witch-familiar.md, §16
//! the owner's round; `docs/dev/familiar.md`): a small beast of the old craft at her heel. It is
//! an ally of the core (`ally.rs`) with `source = Familiar(CompanionData)`, `until = None`: a
//! **kept** beast ([`super::AllySource::kept`]) as the ranger's companion is, so it is named,
//! grows with its master's level, always comes along, and leaves a corpse by the companion's
//! rules (`companion.rs`). What is its own:
//!
//! - **how it comes**: Familiar (the witch's level 1, `TalentMods::familiar`) brings one as the
//!   run begins ([`World::bring_familiar`]): a beast drawn from the character's race's list in
//!   `data/familiars.toml`, or at `rare_per_mille` from the rares, on the stream
//!   `familiar/<race>,<gender>,<class>` (two draws); its name on `companion/name/<actor id>`
//!   (one draw). A run restored makes none; with a familiar already, none;
//! - **the touch** ([`World::familiar_touch`], from `fight.rs::resolve`): a hit of its that
//!   leaves the target alive lays its master's hex (`[familiar] touch`, Evil eye's status) at the
//!   degrees she would cast it now, **with her as the giver**, and refreshes every other hex of
//!   hers on the target at its own degree (a hex that stacks by refreshing): her hexes stick, and
//!   everything that reads "a hex of hers" (Cackle, Malison) sees them;
//! - **the harrying** ([`World::quarry`], from `ally.rs::ally_plan`): a familiar picks the nearest
//!   creature under a hex of its master's before any other, under its order's reach.
//!
//! Nothing here draws from any stream but the start's. A run without a familiar calls none of it
//! to any effect.

use super::{Actor, ActorId, AllySource, CompanionData, World, WorldState};
use crate::action::{AttackKind, Event};
use crate::character::CharacterSpec;
use crate::content::{Content, CreatureId, LevelKind, Stacking};
use crate::geom::Pos;
use crate::rng::Rng;

impl Actor {
    /// Its familiar data, if it is someone's familiar.
    pub fn familiar(&self) -> Option<&CompanionData> {
        match &self.ally.as_ref()?.source {
            AllySource::Familiar(data) => Some(data),
            _ => None,
        }
    }
}

/// The familiar a character of `spec` draws in the run of `seed`: at `rare_per_mille` one of the
/// rares, else one of the race's list (`None` without the data or a list for the race). Two
/// draws on `familiar/<spec>`, and nothing else.
pub fn draw(content: &Content, seed: u64, spec: &CharacterSpec) -> Option<CreatureId> {
    let rules = content.familiars()?;
    let list = rules.list_of(&spec.race)?;
    let mut rng = Rng::stream(seed, &format!("familiar/{spec}"));
    let rare = rng.below(1000) < rules.rare_per_mille;
    let pool = if rare { rules.rare.as_slice() } else { list };
    let at = rng.below(u32::try_from(pool.len()).unwrap_or(u32::MAX));
    pool.get(at as usize).copied()
}

/// The name of the familiar that is actor `id` in the run of `seed`: one draw on
/// `companion/name/<id>` (a kept beast's name stream) from the familiars' names.
pub fn name_for(content: &Content, seed: u64, id: ActorId) -> String {
    let Some(rules) = content.familiars() else {
        return String::new();
    };
    let mut rng = Rng::stream(seed, &format!("companion/name/{}", id.0));
    let at = rng.below(u32::try_from(rules.names.len()).unwrap_or(u32::MAX));
    rules.names.get(at as usize).cloned().unwrap_or_default()
}

impl World {
    /// The player's familiar, if they have one: their one ally of source `Familiar`, on any
    /// level or in the retinue.
    pub fn familiar(&self) -> Option<ActorId> {
        (self.allies_of(self.state.player).into_iter())
            .find(|id| self.state.actors[id].familiar().is_some())
    }

    /// Familiar's beast as the run begins (the module docs): drawn, made beside the player (into
    /// the retinue on the overworld), named, and said: "Tibb, your cat, winds about your
    /// ankles." Nothing without the talent, the data, the ally core, a character, or with a
    /// familiar already.
    pub(super) fn bring_familiar(&mut self) {
        let content = self.content.clone();
        if self.familiar().is_some() || content.allies().is_none() {
            return;
        }
        let Some(character) = self.state.character.clone() else {
            return;
        };
        if !self.state.player().talent_mods(&content).familiar {
            return;
        }
        let Some(rules) = content.familiars() else {
            return;
        };
        let Some(creature) = draw(&content, self.state.seed, &character.spec) else {
            return;
        };
        let level = self.state.current_level.clone();
        let overworld = self.state.current().kind == LevelKind::Overworld;
        let at = match overworld {
            true => self.player_pos(),
            false => self.free_cell_near(&level, self.player_pos()),
        };
        let own = content.creature(creature).level;
        let master = self.player_level();
        let grade = (content.companions()).map_or(master.max(own), |r| r.growth.grade(master, own));
        let data = CompanionData {
            name: String::new(),
            grade,
            scent: None,
            quarry: None,
        };
        let id = self.make_ally(creature, at, AllySource::Familiar(data), None);
        let name = name_for(&content, self.state.seed, id);
        if let Some(actor) = self.state.actors.get_mut(&id) {
            if let Some(data) = actor.kept_mut() {
                data.name = name.clone();
            }
            // Its grade raised its maximum: it starts whole.
            actor.hp = actor.max_hp(&content);
            if overworld && let Some(ally) = actor.ally.as_mut() {
                ally.retinue = true;
            }
        }
        let kind = &content.creature(creature).name;
        let line = rules.arrives(creature).unwrap_or("keeps to your heel");
        self.say(format!("{name}, your {kind}, {line}."));
    }

    /// The touch (the module docs): `attacker`'s melee hit landed on `target`, who lives. When
    /// the attacker is a familiar, its master's hex (`[familiar] touch`) is laid on the target
    /// at the degrees they would cast it now, with them as the giver, and every other hex of
    /// theirs on it that stacks by refreshing starts afresh at its own degree. Nothing for
    /// anyone else, for a target that is the master, or without the data.
    pub(super) fn familiar_touch(
        &mut self,
        attacker: ActorId,
        target: ActorId,
        kind: AttackKind,
        events: &mut Vec<Event>,
    ) {
        if kind != AttackKind::Melee {
            return;
        }
        let Some(master) = (self.state.actors.get(&attacker))
            .filter(|a| a.familiar().is_some())
            .and_then(Actor::master)
        else {
            return;
        };
        let content = self.content.clone();
        let Some(rules) = content.familiars() else {
            return;
        };
        if target == master || !self.state.actors.get(&target).is_some_and(|a| a.hp > 0) {
            return;
        }
        let spell = content.spell(rules.touch);
        let Some((status, _)) = spell.effect.status else {
            return;
        };
        let Some(degrees) =
            (self.state.actors.get(&master)).and_then(|m| m.spell_degrees(&content, spell))
        else {
            return;
        };
        // Her other hexes on it, each at its own degree, before the touch's own is laid.
        let hexes = content.hex_statuses();
        let others: Vec<_> = (self.state.actors[&target].effects.iter())
            .filter(|e| e.by == Some(master) && e.status != status && hexes.contains(&e.status))
            .filter(|e| content.status(e.status).stacking == Stacking::Refresh)
            .map(|e| (e.status, e.severity))
            .collect();
        for (other, severity) in others {
            self.afflict(target, other, severity, Some(master), events);
        }
        self.afflict(target, status, degrees, Some(master), events);
    }

    /// What ally `id` finds to fight under `wanted` (`ally.rs::ally_plan`): a familiar, the
    /// nearest creature it sees under a hex of its master's first (the harrying); else, and for
    /// every other ally, the nearest it senses (`ally.rs::senses`).
    pub(super) fn quarry(
        &mut self,
        id: ActorId,
        wanted: &dyn Fn(&WorldState, &Content, &Actor, &Actor) -> bool,
    ) -> Option<(ActorId, Pos, bool)> {
        let master = (self.state.actors.get(&id))
            .filter(|a| a.familiar().is_some())
            .and_then(Actor::master);
        if let Some(master) = master {
            let hexes = self.content.hex_statuses().to_vec();
            let hexed = |s: &WorldState, c: &Content, me: &Actor, a: &Actor| {
                wanted(s, c, me, a)
                    && (a.effects.iter()).any(|e| e.by == Some(master) && hexes.contains(&e.status))
            };
            if let Some((target, at)) = self.nearest_in_sight(id, &hexed) {
                return Some((target, at, false));
            }
        }
        self.senses(id, wanted)
    }
}

#[cfg(test)]
#[path = "familiar_tests.rs"]
mod tests;
