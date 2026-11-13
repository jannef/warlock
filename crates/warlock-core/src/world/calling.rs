//! The calling (spec §8 "Prestige classes"): the one place a prestige class is given. A
//! calling (`content/calling.rs`, a campaign's `data/callings/`) is **open** to the player when
//! they have no prestige class yet, their class is one it is `from`, and their level is at
//! least its `level` ([`World::calling_open`]). [`World::call`] walks them through it: the
//! character's `prestige` is set, the calling's `message` said, and they are given **exactly
//! the experience the next level costs** if their bank is short of it (the owner: "enough
//! experience to immediately receive level up to the first level of the new class"), so the
//! level-up to the first prestige level comes at once, with every level a fuller bank buys
//! besides, each with its hit points, rolls, picks, and class abilities by the class it falls
//! in (`World::rise_from`). The experience is added as it is, never scaled by a talent
//! (`World::add_xp`'s rule, not `grant_xp`'s), and nothing is drawn: the level-up's stat checks
//! draw from the levels stream as any level-up's do.
//!
//! Who calls: a dialogue topic's `calls` ([`World::ask_calling`], offered only while the calling
//! is open: the Strange Spirit's oath in the Ascmearc, the hermit's in the test world), and the dev action
//! `call:ID` ([`crate::action::Action::Call`]) here, whatever the level (for tests and captures).
//! An encounter entry's `when = { calling = "ID" }` holds while it is open.
//! There is no `from_state`: a restore grants both tables' abilities (`World::grant_class`).

use super::talk::capitalized;
use super::{ActorId, Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, CallingDef, DialogueId};

/// Where the player stands with a calling, for the dev `progress` dump.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallingStatus {
    /// Open: it would be made if its topic were asked now.
    Open,
    /// Their class is none it is `from`.
    NotFrom,
    /// The level it asks is not met.
    LevelNotMet,
    /// They have been called already (into this class or another).
    Called,
}

impl World {
    /// Where the player stands with `calling`.
    pub fn calling_status(&self, calling: &CallingDef) -> CallingStatus {
        let Some(character) = &self.state.character else {
            return CallingStatus::NotFrom;
        };
        if character.prestige.is_some() {
            return CallingStatus::Called;
        }
        if !calling.is_from(&character.spec.class) {
            return CallingStatus::NotFrom;
        }
        if self.player_level() < calling.level {
            return CallingStatus::LevelNotMet;
        }
        CallingStatus::Open
    }

    /// Whether the calling into prestige class `class` is open to the player now: a calling of
    /// the run's campaign (or none's), no prestige class yet, their class among its `from`, and
    /// its level met.
    pub fn calling_open(&self, class: &str) -> bool {
        (self.content.calling(self.campaign(), class))
            .is_some_and(|c| self.calling_status(c) == CallingStatus::Open)
    }

    /// The player is called into `calling`'s prestige class (module docs): refused, with
    /// nothing changed, unless their class is one it is `from` and they hold no prestige class
    /// yet (the level is the caller's to check: the dev action ignores it). Whether they were.
    pub(super) fn call(&mut self, calling: &CallingDef) -> bool {
        let content = self.content.clone();
        let Some(character) = self.state.character.as_ref() else {
            return false;
        };
        if character.prestige.is_some() || !calling.is_from(&character.spec.class) {
            return false;
        }
        let before = self.player_level();
        let cap = character.class_cap(&content);
        if let Some(character) = self.state.character.as_mut() {
            character.prestige = Some(calling.class.clone());
        }
        self.say(calling.message.clone());
        // Exactly the experience the first prestige level costs, if the bank is short of it.
        let factor = self.experience_factor();
        let need = content
            .experience()
            .threshold_for(cap.saturating_add(1), factor);
        if self.state.experience < need {
            self.state.experience = need;
        }
        self.rise_from(before);
        true
    }

    /// The player asks creature `id`, speaking dialogue `speaker`, about `topic`, if it is a
    /// calling's (`calls`): its reply, then the calling made, then the speaker's departure if the
    /// topic `departs` (said as a message only, so the farewell is heard and no screen opens on
    /// one gone). Takes a chat's time. Not open (the topic is not offered): it knows nothing of
    /// it, and no time passes. `None` for any other topic.
    pub(super) fn ask_calling(
        &mut self,
        id: ActorId,
        speaker: DialogueId,
        topic: &str,
        events: &mut Vec<Event>,
    ) -> Option<Spent> {
        let content = self.content.clone();
        let t = content.dialogue(speaker).topic(topic)?;
        let class = t.calls.as_ref()?;
        let calling = (content.calling(self.campaign(), class))
            .filter(|c| self.calling_status(c) == CallingStatus::Open);
        let who = capitalized(&self.called(&self.state.actors[&id]));
        let Some(calling) = calling else {
            self.say(format!("{who} knows nothing about that."));
            return Some(Spent::Nothing);
        };
        if t.departs {
            self.say(format!("{who} says: \"{}\"", t.reply));
        } else {
            self.answer(id, t.reply.clone(), Some(topic.into()), events);
        }
        self.call(calling);
        if t.departs {
            self.departs(id, events);
        }
        Some(Spent::Turn(
            self.price_of(self.state.player, ActionKind::Chat),
        ))
    }

    /// The dev action `call:ID` ([`crate::action::Action::Call`]): the player is called into
    /// prestige class `id` now, whatever their level, through the calling of the run's campaign
    /// that opens it. Refused with no prestige class of that id, one held already, or no calling
    /// of this campaign open to their class. Takes no time.
    pub(super) fn call_dev(&mut self, id: &str) -> Spent {
        let content = self.content.clone();
        if content.prestige(id).is_none() {
            self.say(format!("There is no prestige class `{id}` (dev)."));
            return Spent::Nothing;
        }
        let Some(character) = &self.state.character else {
            self.say("There is no character to call (dev).");
            return Spent::Nothing;
        };
        if let Some(held) = &character.prestige {
            self.say(format!("You are a {held} already (dev)."));
            return Spent::Nothing;
        }
        let class = character.spec.class.clone();
        let Some(calling) = (content.calling(self.campaign(), id)).filter(|c| c.is_from(&class))
        else {
            self.say(format!(
                "No calling of this world opens `{id}` to {} {class} (dev).",
                crate::content::article(&class)
            ));
            return Spent::Nothing;
        };
        let calling = calling.clone();
        self.call(&calling);
        Spent::NoTime
    }
}
