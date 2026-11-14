//! Making out a scroll (spec §8 "Scrolls"; #1024, the skill uses design §6.6): whether the
//! reading the player has asked for, with nothing left to refuse it, goes ahead.
//!
//! A scroll of a spell the reader knows reads at once: no draw, and [`Use::Decipher`] still marks
//! (a trained skill only, as any mark; a scroll they wrote themselves too). Failing that, one
//! trained in Decipher script whose learning (`Actor::learning`, #1270) is at least the scroll's
//! spell level reads it at once: no draw, no check's mark, no words (a read used up marks
//! `read`, #1270). A scroll with no spell behind it counts as level 0. Everyone else makes a
//! check, never a refusal: 2d12 + the Intelligence modifier + what
//! talents add to its checks + Decipher script's points ([`Effect::Decipher`]; an untrained
//! reader has none, whatever they wear that raises the skill), against
//! [`DecipherRules::difficulty`](crate::content::DecipherRules::difficulty). Two draws of the
//! gameplay stream, only when asked, and [`Use::Decipher`] marks whatever comes of it.
//!
//! Failed: "The writing swims before your eyes." The caller spends the read's time and stops
//! there: the scroll stays in the pack and a label stays unknown, with no draw for Scroll savant
//! and no Intelligence mark for a read (`use_up` has both). Made: the scroll reads as any other.
//!
//! The Decipher ability (#1025, design §6.6 D2; [`Action::Decipher`], `decipher:b`): the same
//! check on a scroll of a type not yet identified ([`undeciphered`]), without reading it.
//! `data/decipher.toml`'s `seconds` of turns of the `decipher` action ([`World::keep_at`], as a
//! forage); a creature coming into view or a blow stops it and nothing comes of it. Done, the
//! check is [`World::deciphers`] with the scroll's spell and level; made, the type is learned
//! (every scroll of it shows by its name from now on) and the scroll is kept, unread. Failed,
//! the time is gone and the scroll stays a label: try again (no limit). The mark is the
//! check's own. Refused, no time, with a stack that is not such a scroll.
//!
//! Only reading asks. Studying a scroll has its own check (`learn.rs`), writing one reads
//! nothing (`scribe.rs`), and casting a spell one knows needs no scroll.

use super::rest::Ended;
use super::{Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, Content, Reading, SpellId};
use crate::object::Stack;
use crate::scroll_label::Lore;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::time::Duration;

/// What the player is told when they fail to make a scroll out.
const SWIMS: &str = "The writing swims before your eyes.";

impl World {
    /// Whether the player makes out a scroll of `spell` (`None`: no spell behind it, level 0) of
    /// spell level `level` (module docs); `false` has said so.
    pub(super) fn deciphers(&mut self, spell: Option<SpellId>, level: u8) -> bool {
        let content = self.content.clone();
        let Some(rules) = content.decipher() else {
            return true;
        };
        let me = self.state.player();
        if spell.is_some_and(|s| me.knows_spell(&content, s)) {
            self.train(Use::Decipher);
            return true;
        }
        if me.learning(&content).is_some_and(|l| l >= i32::from(level)) {
            return true;
        }
        // Untrained, the skill does nothing: not even what an item adds to it.
        let skill = match me.skills.trains_effect(&content, Effect::Decipher) {
            true => me.skill_effect(&content, Effect::Decipher),
            false => 0,
        };
        let bonus = me.effective_stats(&content).modifier(Stat::Intelligence)
            + me.talent_mods(&content).check(Stat::Intelligence)
            + skill;
        let made = crate::check::check(&mut self.state.rng, bonus, rules.difficulty(level));
        self.train(Use::Decipher);
        if !made {
            self.say(SWIMS);
        }
        made
    }
}

/// One try at making out a scroll: which stack of the pack, for how long.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deciphering {
    pub item: usize,
    pub seconds: u32,
}

/// Whether `stack` is a scroll of a type the player has not identified (the Decipher ability's
/// list): it has a label still.
pub fn undeciphered(content: &Content, lore: &Lore<'_>, stack: &Stack) -> bool {
    content.object(stack.object).read.is_some() && lore.label(stack.object).is_some()
}

impl World {
    /// The player sets to making out a scroll of the pack (module docs); refused, taking no
    /// time, for a stack that is not an unidentified scroll.
    pub(super) fn start_decipher(&mut self, item: usize) -> Spent {
        let content = self.content.clone();
        let Some(rules) = content.decipher() else {
            return Spent::Nothing;
        };
        let Some(stack) = self.state.player().inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        if !undeciphered(&content, &self.lore(), stack) {
            self.say("There is nothing there to make out.");
            return Spent::Nothing;
        }
        self.say("You pore over the writing.");
        Spent::Decipher(Deciphering {
            item,
            seconds: rules.seconds,
        })
    }

    /// Pores over the scroll turn after turn until the time is up or something stops it.
    pub(super) fn decipher(&mut self, work: Deciphering, events: &mut Vec<Event>) {
        let on = self.work_on(&[Some(work.item)]);
        let until = self.state.time + Duration::seconds(u64::from(work.seconds));
        let turn = self.price_of(self.state.player, ActionKind::Decipher);
        let ended = self.keep_at(
            events,
            |w| w.state.time < until,
            |w, _| turn.min(w.price_until(until)),
        );
        match ended {
            Ended::Died => {}
            _ if self.state.time >= until => match self.worked_on_one(&on) {
                Some(item) => self.deciphered(item),
                None => self.say("You have nothing left to make out."),
            },
            Ended::Came(came) => {
                let text = format!("{}; you put the scroll away.", self.came_line(came));
                self.say(text);
            }
            Ended::Said | Ended::Done => self.say("You put the scroll away."),
        }
    }

    /// The time is up: the check, and the type learned if it was made.
    fn deciphered(&mut self, item: usize) {
        let content = self.content.clone();
        let Some(stack) = self.state.player().inventory.get(item) else {
            return;
        };
        let object = stack.object;
        if !undeciphered(&content, &self.lore(), stack) {
            self.say("You have nothing left to make out.");
            return;
        }
        let (spell, level) = match content.object(object).read.as_ref() {
            Some(Reading::Cast(scroll)) => (Some(scroll.spell), content.spell(scroll.spell).level),
            _ => (None, 0),
        };
        if !self.deciphers(spell, level) {
            return;
        }
        self.learn_scroll(object);
        let name = self.lore().describe_object(object, 1);
        self.say(format!("You make out the writing: {name}."));
    }
}
