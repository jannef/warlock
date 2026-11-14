//! Lure quests (#1446; spec §8 "Quests": "Lure quests"; `docs/dev/quests.md`): a unique creature
//! the player draws after them, stair by stair, to the quest's level, where the giver binds it
//! (`type = "lure"`, [`Goal::Lure`]). The Ascmearc's one is the crystal guardian of Meyn Bew,
//! bound to Gimmwīc's gem trade by Ysella the Binder.
//!
//! - **Where it waits** ([`World::lure_places`], from `World::travel` on every arrival and
//!   `quest.rs::give` as the quest is given): while the quest is active and no creature of its
//!   type is anywhere in the run, one is made on the quest's `waits` place (or the nearest free
//!   cell, `escort_waits::spot`), awake. Killed before it is bound, it forms there anew the next
//!   time the player comes to that level (the binding's `reforms` line, said then; its
//!   `shatters` line as it dies): no anti-farming limit, so the quest can always be tried again.
//! - **The way home** is the stairs' own rule (`stair_followers.rs`): it is hostile, so beside the
//!   player as they take a staircase it follows them, through Gimmwīc's portal too (a staircase
//!   down by its terrain, `crystal_portal`). Left behind, it stays on its floor, hunting, until
//!   fetched. Nothing else moves it between levels.
//! - **The binding** ([`World::lure_home`], from `World::take_stairs` after the followers
//!   arrive): the creature on the quest's level with the player, unbound, and the giver there too:
//!   the binding's `words` are told, then it is bound: its faction is the binding's (at peace),
//!   its mind calm, its stance neutral, its post the binding's `post`, where `ai.rs` walks it and
//!   leaves it, and it carries the binding's `status` (no `eases`: for good; circled in
//!   `data/circles.toml`, so its spell circle shows it). Then the giver says `done` and pays
//!   (`complete_saying`). The giver dead or elsewhere: no binding (it hunts the player there as
//!   anywhere).
//! - **The boon** ([`World::binding_boon`], one term in `cut.rs` and `set.rs`): while a bound
//!   creature stands on the player's level, the binding's `boon.bonus` adds to their checks of
//!   recipes of `boon.skill` (Jewelcraft's Cut and Set in Gimmwīc). Killed, it is gone, and so is
//!   the boon; the quest stays done.
//!
//! Nothing here draws from any stream; what is saved is the actor itself (its status, faction,
//! and post) and the quest's record.

use super::escort_waits::spot;
use crate::content::{CombatStance, CreatureSpawn, QuestId};
use crate::equipment::Equipment;
use crate::skills::SkillId;
use crate::world::levels::travel::make_creature;
use crate::world::{Actor, ActorId, Event, Spent, World};

impl World {
    /// The player is on level `level`: each active lure quest whose creature waits there and is
    /// nowhere in the run gets it made on its place (the module docs).
    pub(in crate::world) fn lure_places(&mut self, level: &str) {
        if !self.state.levels.contains_key(level) {
            return;
        }
        let content = self.content.clone();
        for def in content.quests() {
            let Some((creature, _, waits, binding)) = def.lure() else {
                continue;
            };
            if waits.level != level || !self.state.quests.is_active(def) {
                continue;
            }
            if (self.state.actors.values()).any(|a| a.creature == creature) {
                continue;
            }
            let Some(pos) = spot(&self.state, &content, waits) else {
                continue;
            };
            make_creature(
                &mut self.state,
                &content,
                level,
                &waiting_spawn(creature, pos),
            );
            // Killed before, it has formed anew (`quests.slain` keeps every unique's death).
            let slain = &content.creature(creature).id;
            if self.state.current_level == level
                && self.state.quests.slain_set().contains(slain)
                && let Some(line) = &binding.reforms
            {
                self.say(line.clone());
            }
        }
    }

    /// The player has arrived by the stairs, the followers beside them: on a lure quest's level,
    /// its creature there unbound and its giver there too, the giver binds it and the quest is
    /// done (the module docs).
    pub(in crate::world) fn lure_home(&mut self, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let here = self.state.current_level.clone();
        for (i, def) in content.quests().iter().enumerate() {
            let Some((creature, level, _, binding)) = def.lure() else {
                continue;
            };
            if level != here || !self.state.quests.is_active(def) {
                continue;
            }
            let on_level = |a: &&Actor| a.level == here && !a.in_retinue();
            let lured = (self.state.actors.values())
                .filter(on_level)
                .find(|a| a.creature == creature && !a.effects.has(binding.status))
                .map(|a| a.id);
            let giver = (self.state.actors.values())
                .filter(on_level)
                .find(|a| self.dialogue_of(a) == Some(def.giver))
                .map(|a| a.id);
            let (Some(lured), Some(giver)) = (lured, giver) else {
                continue;
            };
            for line in &binding.words {
                self.say(line.clone());
            }
            self.bind(lured, binding, events);
            self.complete_saying(giver, QuestId(i as u16), None);
        }
    }

    /// Creature `id` is bound by `binding`: at peace, calm, its post the binding's, and the
    /// binding's status on it for good.
    fn bind(&mut self, id: ActorId, binding: &crate::content::Binding, events: &mut Vec<Event>) {
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.faction_override = Some(binding.faction);
            actor.stance = CombatStance::Neutral;
            actor.patrol = None;
            actor.mind = Default::default();
            actor.mind.post = Some(binding.post);
        }
        self.afflict_one(id, binding.status, 1, None, events);
    }

    /// The dev action `lure:ID` ([`crate::action::Action::Lure`]): active lure quest `id`'s
    /// creature comes beside the player, aware of them, made if it is nowhere. Takes no time.
    pub(in crate::world) fn lure_dev(&mut self, id: &str) -> Spent {
        let content = self.content.clone();
        let def = (content.quest_id(id)).map(|q| content.quest(q));
        let Some((def, (creature, ..))) = def.and_then(|d| Some((d, d.lure()?))) else {
            self.say(format!("There is no lure quest `{id}`."));
            return Spent::Nothing;
        };
        if !self.state.quests.is_active(def) {
            self.say(format!("{} is not under way.", def.name));
            return Spent::Nothing;
        }
        let here = self.state.current_level.clone();
        let at = self.player_pos();
        let Some(cell) = self.free_cell_within(&here, at, 1, |_| true) else {
            self.say("There is no room beside you.");
            return Spent::Nothing;
        };
        let found = (self.state.actors.values())
            .find(|a| a.creature == creature)
            .map(|a| a.id);
        let lured = match found {
            Some(lured) => {
                self.cross_to(lured, &here, cell);
                lured
            }
            None => make_creature(
                &mut self.state,
                &content,
                &here,
                &waiting_spawn(creature, cell),
            ),
        };
        if let Some(a) = self.state.actors.get_mut(&lured) {
            a.patrol = None;
            a.mind.post = None;
            a.mind.last_seen = Some(at);
        }
        let name = crate::content::the(&content.creature(creature).name);
        self.say(format!("You lure {name} to your side (dev)."));
        Spent::NoTime
    }

    /// `actor` died: a lure quest's creature, unbound while the quest is active, has its
    /// `shatters` line told (it forms anew: the module docs). The lines to say.
    pub(in crate::world) fn lured_dies(&self, actor: &Actor) -> Vec<String> {
        (self.content.quests().iter())
            .filter_map(|def| {
                let (creature, _, _, binding) = def.lure()?;
                let unbound = actor.creature == creature && !actor.effects.has(binding.status);
                (unbound && self.state.quests.is_active(def))
                    .then(|| binding.shatters.clone())
                    .flatten()
            })
            .collect()
    }

    /// What bound creatures on the player's level add to their checks of skill `skill`'s
    /// recipes (the module docs): each lure quest's boon of that skill whose creature stands
    /// there bound.
    pub(in crate::world) fn binding_boon(&self, skill: SkillId) -> i32 {
        let here = &self.state.current_level;
        (self.content.quests().iter())
            .filter_map(|def| {
                let (creature, _, _, binding) = def.lure()?;
                let boon = binding.boon.filter(|b| b.skill == skill)?;
                let bound = (self.state.actors.values()).any(|a| {
                    a.level == *here && a.creature == creature && a.effects.has(binding.status)
                });
                bound.then_some(boon.bonus)
            })
            .sum()
    }
}

/// A lure quest's creature as it is made on cell `pos`: awake, of its type's own faction, with
/// no route and no gear of its own.
fn waiting_spawn(creature: crate::content::CreatureId, pos: crate::geom::Pos) -> CreatureSpawn {
    CreatureSpawn {
        creature,
        faction: None,
        alignment: None,
        npc: None,
        pos,
        route: Vec::new(),
        carry: Vec::new(),
        equipment: Equipment::default(),
        rolls_gear: false,
        asleep: false,
        status: Vec::new(),
    }
}
