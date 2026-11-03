//! Writing scrolls (spec §8 "Scrolls"; the witch and wizard design §6.1): the wizard's Scribe,
//! [`Action::Scribe`](crate::Action::Scribe) (`scribe:fireball`), from the abilities menu's
//! Class group through the scribe list.
//!
//! The player writes a scroll of a spell they know on the first blank scroll they carry
//! ([`Content::blank_scroll`]); what they write is the ordinary scroll of that spell
//! ([`Content::scroll_of`]), at its own `rungs` (the magic is the item's: Scroll adept adds his
//! rung when it is read, as to any scroll):
//!
//! - **Refusals** (no time, nothing spent): without the ability, a spell they do not know or no
//!   scroll holds, no blank scroll ("You have nothing to write on."), a status that blocks
//!   casting (Raging), and too little mana for the price ("You lack the mana to write it.").
//!   Nothing about light or place: a scroll is read in the dark, and written the same.
//! - **The price** ([`Actor::scribe_base`]): the spell's mana before Frugal caster
//!   ([`Actor::spell_base`]: the cost table, the armour worn, School focus, Empower, Archmage)
//!   times the talent's `mana_times` (3), spent through [`World::spend_mana`] (Frugal caster's
//!   cut, the Willpower and Concentration marks) **at the end**.
//! - **Time** ([`scribe_turns`]): turn after turn of the `scribe` action ([`World::keep_at`], as
//!   a study), as many as a study of the spell takes (`data/study.toml`), Quick study's cut not
//!   applied. It stops early as a study does (a creature coming into view, a message said);
//!   then the time is lost and the blank and the mana kept.
//! - **The end** ([`World::finish_scribe`]): the mana spent, one blank used up, the scroll put
//!   in the pack (the equipment's letters followed), its type known
//!   ([`World::learn_scroll`]), and an Intelligence mark (`scribe`, [`StatUse::Scribe`]).
//!
//! Determinism: nothing here draws at random (the price is derived, the turns counted, the
//! scroll made from a known id); the spend's own training chances are the only draws.

use super::{Actor, Ended, Spent, World};
use crate::action::{Event, Positions};
use crate::content::{ActionKind, ClassActive, Content, ObjectId, SpellData, SpellId, StatUse};
use crate::object::{self, Stack};
use std::cell::Cell;

/// A scroll being written: the spell, the scroll object it makes, and the turns it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Writing {
    pub(super) spell: SpellId,
    pub(super) scroll: ObjectId,
    pub(super) turns: u32,
}

/// How many turns of the `scribe` action writing a scroll of `spell` takes: a study's
/// (`data/study.toml`), without Quick study's cut; `None` without the study rules.
pub fn scribe_turns(content: &Content, spell: &SpellData) -> Option<u32> {
    let rules = content.study_rules()?;
    Some(super::study_turns(rules, spell.level, 0))
}

impl Actor {
    /// What writing a scroll of `spell` asks of it before Frugal caster: the spell's mana
    /// ([`Actor::spell_base`]) times its Scribe's `mana_times`; `None` without Scribe.
    pub fn scribe_base(&self, content: &Content, spell: &SpellData) -> Option<i32> {
        let times = self.talent_mods(content).scribe?;
        Some(
            self.spell_base(content, spell)
                .saturating_mul(i32::from(times)),
        )
    }

    /// What writing a scroll of `spell` costs it in mana: [`Actor::scribe_base`] through
    /// [`Actor::mana_cost`], what the end of the writing spends.
    pub fn scribe_cost(&self, content: &Content, spell: &SpellData) -> Option<i32> {
        Some(self.mana_cost(content, self.scribe_base(content, spell)?))
    }
}

impl World {
    /// `scribe:SPELL`: the player begins to write a scroll of the spell of id `name` (module
    /// docs), or is refused.
    pub(super) fn start_scribe(&mut self, name: &str) -> Spent {
        if !self.may_use_class(ClassActive::Scribe) {
            return Spent::Nothing;
        }
        let content = self.content.clone();
        let Some(spell) = content.spell_id(name) else {
            self.say(format!("There is no spell `{name}`."));
            return Spent::Nothing;
        };
        let me = self.state.player();
        if !me.knows_spell(&content, spell) {
            self.say("You do not know that spell.");
            return Spent::Nothing;
        }
        let def = content.spell(spell);
        let Some(scroll) = content.scroll_of(spell) else {
            self.say(format!("No scroll can hold {}.", def.name));
            return Spent::Nothing;
        };
        let blank = content.blank_scroll();
        if !(me.inventory.iter()).any(|s| Some(s.object) == blank) {
            self.say("You have nothing to write on.");
            return Spent::Nothing;
        }
        let (Some(cost), Some(turns)) =
            (me.scribe_cost(&content, def), scribe_turns(&content, def))
        else {
            self.say("You cannot write it.");
            return Spent::Nothing;
        };
        if me.mana < cost {
            self.say("You lack the mana to write it.");
            return Spent::Nothing;
        }
        self.say("You begin to write.");
        Spent::Scribe(Writing {
            spell,
            scroll,
            turns,
        })
    }

    /// Writes turn after turn until the scroll is done or something interrupts (module docs).
    pub(super) fn scribe(&mut self, writing: Writing, events: &mut Vec<Event>) {
        let left = Cell::new(writing.turns);
        let ended = self.keep_at(
            events,
            |_| left.get() > 0,
            |w, _| {
                left.set(left.get() - 1);
                w.price_of(w.state.player, ActionKind::Scribe)
            },
        );
        match ended {
            Ended::Died => {}
            _ if left.get() == 0 => self.finish_scribe(writing),
            Ended::Came(id) => {
                let text = format!(
                    "{} comes into view; you stop writing.",
                    self.creature_called(id)
                );
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You stop writing."),
        }
    }

    /// The last turn of a writing is done: the mana spent, a blank used up, and the scroll made
    /// (module docs). A blank gone or the mana short by then leaves nothing written.
    fn finish_scribe(&mut self, writing: Writing) {
        let content = self.content.clone();
        let player = self.state.player;
        let blank = content.blank_scroll();
        let me = self.state.player();
        let Some(at) = (me.inventory.iter()).position(|s| Some(s.object) == blank) else {
            self.say("The blank scroll is gone.");
            return;
        };
        let base = me.scribe_base(&content, content.spell(writing.spell));
        if !base.is_some_and(|base| self.spend_mana(player, base)) {
            self.say("You lack the mana to write it.");
            return;
        }
        let me = self.player_mut();
        me.inventory[at].count -= 1;
        if me.inventory[at].count == 0 {
            me.inventory.remove(at);
            me.equipment.after_removal(&Positions::of([at]));
        }
        object::add_to_inventory(
            self.state.inventory_mut(),
            Stack::one(writing.scroll),
            &content,
        );
        self.learn_scroll(writing.scroll);
        self.train_stat(StatUse::Scribe);
        let what = content.object(writing.scroll).describe(1);
        self.say(format!("You write {what}."));
    }
}
