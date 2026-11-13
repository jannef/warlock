//! Escorts (#1139; spec §8 "Quests": "Escort quests"; `docs/dev/quests.md`, `docs/dev/allies.md`):
//! a unique character the player brings home for an escort quest (`type = "escort"`,
//! [`Goal::Escort`]).
//!
//! - **Joining** ([`World::escort_joins`], from `talk.rs::chat`): chatted with while the quest
//!   is active, the escort says the quest's `joins` line instead of its greeting and becomes the
//!   player's ally, [`AllySource::Escort`]`{ quest, faction }` (its `faction_override` put to the
//!   player's, the old one kept to give back), following, in the **coward** stance. From then on
//!   it is an ally by every rule of `ally.rs`: it follows across levels and in the retinue, takes
//!   Come to heel and Hold here, and its kills pay as an ally's. Without `data/allies.toml` no
//!   one joins (the chat is an ordinary chat).
//! - **No fighter**: its mind (`ally.rs::ally_plan`) strikes only at a hostile beside it, and
//!   Set on passes it by with a line ([`World::escorts_refuse`]).
//! - **Gifts** ([`World::give_escort`], the Give ability, `collar.rs`): anything that fits one
//!   of its slots ([`escort_slot`]: a free one first, else the first it fits) goes on at once,
//!   and what that slot (or both hands, for a two-handed thing) held falls on its cell. Priced
//!   as the slot's action. Refused, no time, for what fits no slot.
//! - **Home** ([`World::escorts_home`], from `World::travel` after the allies come along): the
//!   escort standing on the quest's target level with the player completes the quest: the
//!   escort says the quest's `thanks`, if any, then the giver, if on the level, else the escort,
//!   says `done` and pays the reward; the escort is no ally any more (its faction and stance put
//!   back, its post where it stands).
//! - **Death** ([`World::escort_dies`], from `fight.rs::creature_dies`): an escort quest whose
//!   escort dies, joined or not, fails ([`super::Stage::Failed`]). Nothing else is locked out.
//!
//! Nothing here draws from any stream.

use super::{Actor, ActorId, Ally, AllySource, Event, Order, Spent, Stage, World};
use crate::action::Positions;
use crate::content::{
    ActionKind, CombatStance, Content, DialogueId, Goal, ObjectId, QuestId, SlotId,
};
use crate::equipment::{self, Equipment};
use crate::object::{self, Stack};

/// The slot an escort wearing `equipment` puts `object` in: the first free one it fits (its own
/// kind of slot first), else the first it fits, taken; `None` if it fits none.
pub fn escort_slot(content: &Content, equipment: &Equipment, object: ObjectId) -> Option<SlotId> {
    let fitting: Vec<SlotId> = (content.slots_for(None))
        .map(|(id, _)| id)
        .filter(|&s| content.fits_properly(object, s))
        .collect();
    let own = |s: &&SlotId| content.object(object).slot.as_ref() == Some(&content.slot(**s).fits);
    (equipment::free_slot(content, equipment, object))
        .or_else(|| fitting.iter().find(own).copied())
        .or_else(|| fitting.first().copied())
}

/// Whether an escort could put `stack` on: what the Give list shows beside an escort.
pub fn fits_escort(content: &Content, stack: &Stack) -> bool {
    !stack.is_spirit() && escort_slot(content, &Equipment::default(), stack.object).is_some()
}

impl Actor {
    /// The escort quest it follows the player for, if it is an escort.
    pub fn escorting(&self) -> Option<&str> {
        match self.ally.as_ref().map(|a| &a.source) {
            Some(AllySource::Escort { quest, .. }) => Some(quest),
            _ => None,
        }
    }
}

impl World {
    /// The active escort quest whose escort speaks dialogue `dialogue`.
    fn escort_quest(&self, dialogue: DialogueId) -> Option<QuestId> {
        (self.content.quests().iter().enumerate())
            .find(|(_, def)| def.escort() == Some(dialogue) && self.state.quests.is_active(def))
            .map(|(i, _)| QuestId(i as u16))
    }

    /// Creature `id`, who speaks `dialogue`, is chatted with: if it is the escort of an active
    /// escort quest and not yet anyone's ally, it says the quest's `joins` line and falls in
    /// behind the player (the module docs). Whether it did: then `chat` says nothing more.
    pub(super) fn escort_joins(
        &mut self,
        id: ActorId,
        dialogue: DialogueId,
        events: &mut Vec<Event>,
    ) -> bool {
        if self.content.allies().is_none() || self.state.actors[&id].ally.is_some() {
            return false;
        }
        let Some(q) = self.escort_quest(dialogue) else {
            return false;
        };
        let content = self.content.clone();
        let def = content.quest(q);
        let Goal::Escort { joins, .. } = &def.goal else {
            return false;
        };
        let player = self.state.player;
        let faction = self.state.player().faction(&content);
        if let Some(actor) = self.state.actors.get_mut(&id) {
            let source = AllySource::Escort {
                quest: def.id.clone(),
                faction: actor.faction_override,
            };
            actor.faction_override = faction;
            actor.stance = CombatStance::Coward;
            actor.ally = Some(Ally {
                master: player,
                source,
                order: Order::Follow,
                retinue: false,
                until: None,
            });
        }
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.talk.met = true;
        }
        self.answer(id, joins.clone(), None, events);
        // Found already at home: home at once.
        self.escorts_home();
        true
    }

    /// The player stands on a level (the arrival, or an escort joining): each active escort
    /// quest whose target it is and whose escort is on it with them is done (the module docs).
    pub(super) fn escorts_home(&mut self) {
        let content = self.content.clone();
        let here = self.state.current_level.clone();
        let player = self.state.player;
        for (i, def) in content.quests().iter().enumerate() {
            let home = matches!(&def.goal, Goal::Escort { level, .. } if *level == here);
            if !home || !self.state.quests.is_active(def) {
                continue;
            }
            let escort = (self.state.actors.values())
                .find(|a| {
                    a.level == here
                        && !a.in_retinue()
                        && a.master() == Some(player)
                        && a.escorting() == Some(def.id.as_str())
                })
                .map(|a| a.id);
            let Some(escort) = escort else {
                continue;
            };
            self.escort_released(escort);
            if let Goal::Escort {
                thanks: Some(thanks),
                ..
            } = &def.goal
            {
                let who = super::talk::capitalized(&self.called(&self.state.actors[&escort]));
                self.say(format!("{who} says: \"{thanks}\""));
            }
            // The giver thanks the player, if here to; else the escort does.
            let giver = (self.state.actors.values())
                .find(|a| {
                    a.level == here
                        && a.id != escort
                        && !a.in_retinue()
                        && self.dialogue_of(a) == Some(def.giver)
                })
                .map(|a| a.id);
            self.complete_saying(giver.unwrap_or(escort), QuestId(i as u16), None);
        }
    }

    /// Escort `id` is home: no one's ally, its faction and stance as they were, its post where
    /// it stands.
    fn escort_released(&mut self, id: ActorId) {
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        if let Some(AllySource::Escort { faction, .. }) = actor.ally.take().map(|a| a.source) {
            actor.faction_override = faction;
        }
        actor.stance = CombatStance::Neutral;
        actor.mind = Default::default();
        actor.mind.post = Some(actor.pos);
    }

    /// `actor` died: an active escort quest whose escort it was fails. The lines to say.
    pub(super) fn escort_dies(&mut self, actor: &Actor) -> Vec<String> {
        let Some(dialogue) = self.dialogue_of(actor) else {
            return Vec::new();
        };
        let Some(q) = self.escort_quest(dialogue) else {
            return Vec::new();
        };
        let def = self.content.quest(q);
        let name = def.name.clone();
        if let Some(record) = self.state.quests.records.get_mut(&def.id) {
            record.stage = Stage::Failed;
        }
        vec![format!("Quest failed: {name}.")]
    }

    /// Set on is given to `allies`: the escorts among them are taken out, and what they say of
    /// it is returned, if any were (the module docs).
    pub(super) fn escorts_refuse(&self, allies: &mut Vec<ActorId>) -> Option<String> {
        let escorts: Vec<String> = (allies.iter())
            .filter(|id| self.state.actors[id].escorting().is_some())
            .map(|&id| self.ally_subject(id))
            .collect();
        if escorts.is_empty() {
            return None;
        }
        allies.retain(|id| self.state.actors[id].escorting().is_none());
        let names: Vec<&str> = escorts.iter().map(String::as_str).collect();
        let will = crate::content::and_list(&names);
        Some(format!("{will} will not go looking for a fight."))
    }

    /// The player gives one of the stack at `item` to escort `escort` beside them (the module
    /// docs).
    pub(super) fn give_escort(&mut self, item: usize, escort: ActorId) -> Spent {
        let content = self.content.clone();
        let name = self.ally_subject(escort);
        let Some(stack) = self.state.player().inventory.get(item).cloned() else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        let equipment = &self.state.actors[&escort].equipment;
        let Some(slot) =
            (escort_slot(&content, equipment, stack.object)).filter(|_| !stack.is_spirit())
        else {
            self.say(format!("{name} has no use for that."));
            return Spent::Nothing;
        };
        if !self.may_loose(item) {
            return Spent::Nothing;
        }
        let action = content.slot(slot).action;
        let (spent, one) = self.hand_over_one(item, action);
        let what = super::collar::definite(&self.lore().describe(&one));
        self.say(format!("You give {what} to {name}."));
        self.put_on_escort(escort, one, slot, &name, action);
        spent
    }

    /// `gift` goes on escort `escort` in `slot` (both hands for a two-handed thing), and what
    /// was there falls on its cell.
    fn put_on_escort(
        &mut self,
        escort: ActorId,
        gift: Stack,
        slot: SlotId,
        name: &str,
        action: ActionKind,
    ) {
        let content = self.content.clone();
        let taken = equipment::slots_taken(&content, gift.object, slot);
        let Some(actor) = self.state.actors.get_mut(&escort) else {
            return;
        };
        let at = actor.pos;
        let mut held: Vec<usize> = (taken.iter())
            .filter_map(|&s| actor.equipment.get(s))
            .collect();
        held.sort_unstable();
        held.dedup();
        for &i in &held {
            actor.equipment.take_off(i);
        }
        let old: Vec<Stack> = held
            .iter()
            .rev()
            .map(|&i| actor.inventory.remove(i))
            .collect();
        actor
            .equipment
            .after_removal(&Positions::of(held.iter().copied()));
        let what = self.lore().describe(&gift);
        actor_put(self, escort, gift, &taken);
        let verb = match action {
            ActionKind::Wield => "wields",
            ActionKind::Ready => "readies",
            _ => "puts on",
        };
        self.say(format!("{name} {verb} {}.", super::collar::definite(&what)));
        if old.is_empty() {
            return;
        }
        for stack in &old {
            let what = super::collar::definite(&self.lore().describe(stack));
            self.say(format!("{name} drops {what}."));
        }
        let level = self.state.current_mut();
        let mut pile = level.objects_at(at).to_vec();
        object::add_to_pile(&mut pile, &old, &content);
        level.set_pile(at, pile);
    }
}

/// `gift` at the end of actor `id`'s pack, on in `slots`.
fn actor_put(world: &mut World, id: ActorId, gift: Stack, slots: &[SlotId]) {
    if let Some(actor) = world.state.actors.get_mut(&id) {
        actor.inventory.push(gift);
        let on = actor.inventory.len() - 1;
        actor.equipment.put(slots, on);
    }
}
