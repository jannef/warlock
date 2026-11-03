//! Saying words (spec §8 "Dialogue", "Saying words"; docs/design/prestige-classes.md §6.2): the
//! one place words are said. The player says something ([`Action::Say`](crate::Action::Say)) to
//! the creature beside them; if its dialogue knows the words (`[[dialogue.word]]`, matched
//! normalised: `words::normalise`), it replies with the word's lines and does what the word
//! `does` (a calling made, a quest given, the speaker departing); if not, it says its `unheard` line. Either way
//! it takes a chat's time: no count of tries and no lock-out. Nothing is drawn, nothing is
//! trained, and nothing new is saved.

use super::talk::capitalized;
use super::{ActorId, Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, Word};
use crate::geom::Dir;
use crate::words::normalise;

impl World {
    /// The player says `words` to the creature in `dir` (see the module docs). Refused, taking
    /// no time, as a chat is: no one seen there, or nothing said.
    pub(super) fn say_to(&mut self, dir: Dir, words: &str, events: &mut Vec<Event>) -> Spent {
        let words = normalise(words);
        if words.is_empty() {
            return Spent::Nothing;
        }
        let at = self.player_pos().step(dir);
        let Some(id) = self.seen_creature_at(at) else {
            self.say("You see no one there to talk to.");
            return Spent::Nothing;
        };
        let price = self.price_of(self.state.player, ActionKind::Chat);
        // No dialogue, a hostile, or someone outside whose `speaks_to` the player is: `listens`
        // says so, and the time is spent as a chat's.
        let Some(dialogue) = self.listens(id) else {
            return Spent::Turn(price);
        };
        let content = self.content.clone();
        let d = content.dialogue(dialogue);
        let heard = (d.words.iter()).find(|w| w.said == words && self.can_do(w));
        match heard {
            Some(word) => self.do_word(id, word, events),
            None => {
                match &d.unheard {
                    Some(line) => self.answer(id, line.clone(), None, events),
                    None => {
                        // Silence: told plainly, and the screen's line an ellipsis.
                        let who = capitalized(&self.called(&self.state.actors[&id]));
                        self.say(format!("{who} says nothing."));
                        self.converse(id, "…".into(), None, events);
                    }
                }
            }
        }
        Spent::Turn(price)
    }

    /// Whether what `word` does can be done now: a word whose calling is not open to the player
    /// is not heard (a warrior saying the oath hears the `unheard` line).
    fn can_do(&self, word: &Word) -> bool {
        (word.does.calls.as_ref()).is_none_or(|class| self.calling_open(class))
    }

    /// Creature `id` heard `word`: its reply, a message a line, the last also the dialogue
    /// screen's line (unless it departs); then the calling made, if the word calls (`can_do`
    /// saw it open); then the quest given, if the word gives one that could be given now; then
    /// its departure, last, so the farewell is heard.
    fn do_word(&mut self, id: ActorId, word: &Word, events: &mut Vec<Event>) {
        let departs = word.does.departs;
        let (last, before) = word.reply.split_last().expect("a reply has a line");
        let who = capitalized(&self.called(&self.state.actors[&id]));
        for line in before {
            self.say(format!("{who} says: \"{line}\""));
        }
        if departs {
            self.say(format!("{who} says: \"{last}\""));
        } else {
            self.answer(id, last.clone(), None, events);
        }
        if let Some(class) = &word.does.calls
            && let Some(calling) = self.content.calling(self.campaign(), class).cloned()
        {
            self.call(&calling);
        }
        if let Some(quest) = &word.does.quest {
            self.word_quest(id, quest, events);
        }
        if departs {
            self.departs(id, events);
        }
    }

    /// Gives quest `quest` through creature `id` as asking its topic would, if it could be
    /// asked for now: not active, offered (its requirements met, the player's alignment in its
    /// range, something to roll), and the player not too green for it. Else nothing: the reply
    /// was the answer.
    fn word_quest(&mut self, id: ActorId, quest: &str, events: &mut Vec<Event>) {
        let Some(q) = self.content.quest_id(quest) else {
            return;
        };
        let content = self.content.clone();
        let def = content.quest(q);
        let topic = format!("{}{quest}", super::quest::QUEST);
        let giver = def.giver;
        let offered = (self.quest_topics(giver).iter()).any(|t| t.id == topic);
        let green = (def.min_level.as_ref()).is_some_and(|(level, _)| self.player_level() < *level);
        if offered && !self.state.quests.is_active(def) && !green {
            self.give(id, q, &topic, events);
        }
    }
}
