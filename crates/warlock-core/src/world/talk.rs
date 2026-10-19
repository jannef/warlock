//! Talk (spec §8 "Dialogue", #146) and making way (spec §8 "Displacement", #147).
//!
//! **Chatting** ([`Action::Chat`](crate::Action::Chat)) with a creature the player sees next to
//! them takes the time of `chat`. A creature with a dialogue (`data/dialogue.toml`, by its `npc`
//! id or its type) answers: its greeting the first time, then its one-liners in turn
//! ([`rotation`]): `The gardener says: "…"`. One with topics opens the dialogue screen
//! ([`Event::Talked`] carries what it said and its topics), where asking about a topic
//! ([`Action::Ask`](crate::Action::Ask)) takes no time. A creature with nothing to say, or one
//! hostile to the player, gives a non-answer. Quests (#148, `quest.rs`) add topics of their own
//! in [`World::topics`], answered in [`World::ask`].
//!
//! **Muttering**: a creature whose dialogue has a `mutter` may mutter one of its lines (in turn,
//! like the one-liners) with each action, by its `chance`, while it goes about its routine and
//! the player is within [`MUTTER_REACH`]. It is a noise ([`Act::Mutter`]): the player gets the
//! words only if they hear it, with the speaker's name if they see it (`The gardener grumbles:
//! "…"`), else as a voice from where they place the sound (`You hear a voice to the north:
//! "…"`; a faint one without the words). Mutters are only logged: they never stop a rest. The
//! roll hashes the seed, the creature, and the moment, so it draws nothing from the gameplay
//! stream.
//!
//! **Making way**: moving into a creature that is not hostile swaps places with it ([`World::
//! refusal`] says when it will not), taking the player's step; the creature remembers where it
//! stood as its post and walks back there (waiting beside it while someone stands on it), then
//! carries on with its routine.

use super::{Actor, ActorId, ActorKind, Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, Budge, Dialogue, DialogueId, Stance};
use crate::geom::{Dir, Pos};
use crate::noise::{self, Act, Heard, SoundKind, Volume};
use serde::{Deserialize, Serialize};

/// How near the player a creature mutters, in cells (Chebyshev): as far as its voice carries.
pub const MUTTER_REACH: i32 = (noise::VOICE_NOISE / noise::STEP_LOSS) as i32;

/// How far an actor's talk has got (saved; left out while nothing has been said).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Talk {
    /// How many one-liners it has said: the next is the [`rotation`]'s line for this count.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub said: u32,
    /// How many lines it has muttered.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub muttered: u32,
    /// Whether the player has chatted with it: it greets them once.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub met: bool,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl Talk {
    /// Whether nothing has been said yet.
    pub fn is_new(&self) -> bool {
        *self == Self::default()
    }
}

/// A topic the player may ask about, as the dialogue screen lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopicView {
    /// Its id in replays (`ask:E:rooms`).
    pub id: String,
    /// What the player picks: "The rooms".
    pub name: String,
}

/// A creature's answer to the player (an [`Event::Talked`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversation {
    /// Who answered.
    pub with: ActorId,
    /// What they said.
    pub line: String,
    /// What the player may ask them about; none for a creature of one-liners.
    pub topics: Vec<TopicView>,
    /// The topic this answers, if the player asked about one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asked: Option<String>,
}

/// A line muttered during the current turn (not saved): said when perception runs, if the
/// player heard it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Mutter {
    actor: ActorId,
    at: Pos,
    verb: String,
    line: String,
}

/// Why a creature does not make way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// It will not or cannot move: it keeps its post, never gives way, or cannot move at all.
    Budge,
    /// It is too big to squeeze past: two sizes or more larger than the player.
    TooBig,
}

/// Which of `n` lines a speaker keyed `key` says `k`-th (from 0): every line once a round, each
/// round in an order of its own (a shuffle keyed by the speaker and the round), and never the
/// same line twice in a row, across rounds too.
pub fn rotation(key: u64, k: u32, n: usize) -> usize {
    match n {
        0 | 1 => 0,
        // Two lines can only take turns.
        2 => (shuffle(key, 0, 2)[0] + k as usize) % 2,
        _ => {
            let round = k / n as u32;
            let mut order = shuffle(key, round, n);
            // A round that would start with the line the last one ended on starts with its
            // second instead (its last line is not moved, so the next round is not changed).
            if round > 0 && order[0] == shuffle(key, round - 1, n)[n - 1] {
                order.swap(0, 1);
            }
            order[(k % n as u32) as usize]
        }
    }
}

/// `0..n` shuffled (Fisher–Yates) by draws keyed by `key` and `round`.
fn shuffle(key: u64, round: u32, n: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let label = [b"line".as_slice(), &round.to_le_bytes()].concat();
        let j = crate::weather::draw(key, &label, i as i64) % (i as u64 + 1);
        order.swap(i, j as usize);
    }
    order
}

impl World {
    /// The key of `actor`'s rotation of `what` (its one-liners or its mutters): the run's seed
    /// and the actor, so each speaker has an order of its own.
    fn rotation_key(&self, actor: ActorId, what: &[u8]) -> u64 {
        let label = [what, &actor.0.to_le_bytes()].concat();
        crate::weather::draw(self.state.seed, &label, 0)
    }

    /// What `actor` is called in messages: "the gardener". Shopkeepers (#145) will have names.
    pub(super) fn called(&self, actor: &Actor) -> String {
        format!("the {}", self.content.creature(actor.creature).name)
    }

    /// The dialogue `actor` speaks: its npc's, else its type's.
    fn dialogue_of(&self, actor: &Actor) -> Option<DialogueId> {
        self.content
            .dialogue_of(actor.creature, actor.npc.as_deref())
    }

    /// The creature the player sees on `at`, if any.
    fn seen_creature_at(&self, at: Pos) -> Option<ActorId> {
        let level = &self.state.current_level;
        let actor = self.state.actor_at(level, at)?;
        (actor.kind == ActorKind::Creature && self.perception.sees_actors_at(at))
            .then_some(actor.id)
    }

    /// Whether `actor` stops a rest by coming into view (spec §8 "Rest"): any creature but one
    /// who talks (has a dialogue) and fights no one (its faction is hostile to nobody): the
    /// showcase's people go about their business, and the player sees who they are.
    pub(super) fn disturbs_rest(&self, actor: &Actor) -> bool {
        let fights =
            (actor.faction(&self.content)).is_some_and(|f| self.content.factions().has_enemies(f));
        actor.kind == ActorKind::Creature && (fights || self.dialogue_of(actor).is_none())
    }

    /// What the player may ask `actor` about: its quest topics first (`quest.rs`), then its
    /// dialogue's topics. The one place topics come from.
    pub fn topics(&self, actor: ActorId) -> Vec<TopicView> {
        let Some(actor) = self.state.actors.get(&actor) else {
            return Vec::new();
        };
        let Some(id) = self.dialogue_of(actor) else {
            return Vec::new();
        };
        let mut topics = self.quest_topics(id);
        topics.extend(self.content.dialogue(id).topics.iter().map(|t| TopicView {
            id: t.id.clone(),
            name: t.name.clone(),
        }));
        topics
    }

    /// Chats with the creature in `dir` (see the module docs). Refused, taking no time, when the
    /// player sees no one there.
    pub(super) fn chat(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let at = self.player_pos().step(dir);
        let Some(id) = self.seen_creature_at(at) else {
            self.say("You see no one there to talk to.");
            return Spent::Nothing;
        };
        let price = self.price_of(self.state.player, ActionKind::Chat);
        let Some(dialogue) = self.listens(id) else {
            return Spent::Turn(price);
        };
        let actor = &self.state.actors[&id];
        let d = self.content.dialogue(dialogue);
        let talk = actor.talk;
        let line = match &d.greet {
            Some(greet) if !talk.met => greet.clone(),
            _ => self.one_liner(id, d, talk.said),
        };
        if let Some(actor) = self.state.actors.get_mut(&id) {
            if actor.talk.met || d.greet.is_none() {
                actor.talk.said += 1;
            }
            actor.talk.met = true;
        }
        self.answer(id, line, None, events);
        Spent::Turn(price)
    }

    /// Asks the creature in `dir` about `topic` (see the module docs; quest topics are answered
    /// in `quest.rs`). Takes no time; refused when the player sees no one there or it knows
    /// nothing of the topic.
    pub(super) fn ask(&mut self, dir: Dir, topic: &str, events: &mut Vec<Event>) -> Spent {
        let at = self.player_pos().step(dir);
        let Some(id) = self.seen_creature_at(at) else {
            self.say("You see no one there to talk to.");
            return Spent::Nothing;
        };
        let Some(dialogue) = self.listens(id) else {
            return Spent::Nothing;
        };
        if let Some(spent) = self.ask_quest(id, dialogue, topic, events) {
            return spent;
        }
        let reply = (self.content.dialogue(dialogue).topic(topic)).map(|t| t.reply.clone());
        match reply {
            Some(reply) => self.answer(id, reply, Some(topic.to_string()), events),
            None => {
                let who = capitalized(&self.called(&self.state.actors[&id]));
                self.say(format!("{who} knows nothing about that."));
            }
        }
        Spent::Nothing
    }

    /// The dialogue of creature `id` if it will talk to the player; if not, says why: it has
    /// nothing to say, or it is hostile to them.
    fn listens(&mut self, id: ActorId) -> Option<DialogueId> {
        let actor = &self.state.actors[&id];
        let who = capitalized(&self.called(actor));
        let dialogue = self.dialogue_of(actor);
        let hostile = self.stance(self.state.player, id) == Stance::Hostile;
        match dialogue {
            None => self.say(format!("{who} doesn't answer.")),
            Some(_) if hostile => self.say(format!("{who} is in no mood to talk.")),
            Some(d) => return Some(d),
        }
        None
    }

    /// Says what creature `id` answered and records the conversation for presentation.
    pub(super) fn answer(
        &mut self,
        id: ActorId,
        line: String,
        asked: Option<String>,
        events: &mut Vec<Event>,
    ) {
        let who = capitalized(&self.called(&self.state.actors[&id]));
        self.say(format!("{who} says: \"{line}\""));
        events.push(Event::Talked(Conversation {
            with: id,
            line,
            topics: self.topics(id),
            asked,
        }));
    }

    /// The `k`-th one-liner of `d` for creature `id`.
    fn one_liner(&self, id: ActorId, d: &Dialogue, k: u32) -> String {
        let key = self.rotation_key(id, b"lines");
        d.lines[rotation(key, k, d.lines.len())].clone()
    }

    /// Creature `id`, having acted, may mutter (see the module docs).
    pub(super) fn mutter(&mut self, id: ActorId) {
        let Some(actor) = self.state.actors.get(&id) else {
            return;
        };
        let near = actor.pos.chebyshev(self.player_pos()) <= MUTTER_REACH;
        if !near || !actor.mind.is_calm() {
            return;
        }
        let Some(dialogue) = self.dialogue_of(actor) else {
            return;
        };
        let Some(mutter) = &self.content.dialogue(dialogue).mutter else {
            return;
        };
        let moment = i64::try_from(self.state.time.seconds()).unwrap_or(i64::MAX);
        let label = [b"mutter".as_slice(), &id.0.to_le_bytes()].concat();
        if crate::weather::draw(self.state.seed, &label, moment) % 100 >= u64::from(mutter.chance) {
            return;
        }
        let key = self.rotation_key(id, b"mutters");
        let line = mutter.lines[rotation(key, actor.talk.muttered, mutter.lines.len())].clone();
        let (at, verb) = (actor.pos, mutter.verb.clone());
        if let Some(actor) = self.state.actors.get_mut(&id) {
            actor.talk.muttered += 1;
        }
        self.make_noise(id, Act::Mutter);
        self.mutters.push(Mutter {
            actor: id,
            at,
            verb,
            line,
        });
    }

    /// Says the lines muttered this turn that the player heard: with the speaker's name if they
    /// see it, else as a voice from where they place it (the words lost if it was faint). Only
    /// logged: a mutter never stops a rest.
    pub(super) fn say_mutters(&mut self) {
        let mutters = std::mem::take(&mut self.mutters);
        let player = self.state.player();
        let viewer = player.pos;
        let threshold = player.hearing(&self.content);
        let voices: Vec<Heard> = (player.heard.iter())
            .filter(|h| h.kind == SoundKind::Voice)
            .copied()
            .collect();
        for m in mutters {
            let heard = voices.iter().find(|h| h.at == m.at).zip(threshold);
            let Some((heard, threshold)) = heard else {
                continue;
            };
            let text = match self.state.actors.get(&m.actor) {
                Some(actor) if self.perception.sees_actors_at(m.at) => {
                    let who = capitalized(&self.called(actor));
                    format!("{who} {}: \"{}\"", m.verb, m.line)
                }
                _ => {
                    let volume = Volume::of(heard.loudness, threshold);
                    let level = self.state.current();
                    let turn = self.state.turn;
                    let hidden = |p: Pos| self.perception.hides(p);
                    let place =
                        noise::rough_place(level, &self.content, m.at, volume, turn, hidden);
                    let from = Dir::toward(viewer, place)
                        .map_or_else(String::new, |d| format!(" to the {}", d.compass_name()));
                    match volume {
                        Volume::Faint => format!("You faintly hear a voice{from}."),
                        Volume::Clear | Volume::Loud => {
                            format!("You hear a voice{from}: \"{}\"", m.line)
                        }
                    }
                }
            };
            self.note(text);
        }
    }

    /// Why creature `id` would not make way for the player, if it would not (spec §8
    /// "Displacement"): it keeps its post or never gives way (its type's `budge`), it cannot
    /// move (overloaded), or it is two sizes or more larger than the player.
    pub fn refusal(&self, id: ActorId) -> Option<Refusal> {
        let actor = self.state.actors.get(&id)?;
        let budges = match self.content.creature(actor.creature).budge {
            Budge::Always => true,
            Budge::OffPost => !actor.mind.is_calm(),
            Budge::Never => false,
        };
        if !budges || !self.load_of(id).penalty().can_move() {
            return Some(Refusal::Budge);
        }
        let player = self.state.player();
        let size = |a: &Actor| self.state.size_of(&self.content, a);
        (size(actor) >= size(player) + 2).then_some(Refusal::TooBig)
    }

    /// What the player is told when creature `id` will not make way: why, if they see it.
    pub(super) fn refused(&mut self, id: ActorId, why: Refusal, seen: bool) {
        let text = match (seen, why) {
            (false, _) => "Something is in the way.".to_string(),
            (true, Refusal::Budge) => {
                let who = capitalized(&self.called(&self.state.actors[&id]));
                format!("{who} doesn't budge.")
            }
            (true, Refusal::TooBig) => {
                let who = capitalized(&self.called(&self.state.actors[&id]));
                format!("{who} is too big to squeeze past.")
            }
        };
        self.say(text);
    }

    /// Creature `id` steps from `from` onto `to`, where the player stood, making way for them:
    /// it remembers where it stood (unless it already has a post to return to), which it walks
    /// back to before it carries on. Its own turn is unaffected. Says so, and makes its noise.
    pub(super) fn swap_places(
        &mut self,
        id: ActorId,
        from: Pos,
        to: Pos,
        seen: bool,
        events: &mut Vec<Event>,
    ) {
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return;
        };
        actor.pos = to;
        actor.mind.post.get_or_insert(from);
        let who = if seen {
            self.called(&self.state.actors[&id])
        } else {
            "something".to_string()
        };
        events.push(Event::ActorMoved {
            actor: id,
            from,
            to,
        });
        self.make_noise(id, Act::Step(to));
        self.say(format!("You swap places with {who}."));
    }
}

/// `text` with its first letter in upper case.
pub(super) fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_says_every_line_once_a_round_and_never_twice_in_a_row() {
        for n in 1..=9 {
            for key in [0, 1, 42, u64::MAX] {
                let said: Vec<usize> = (0..n as u32 * 12).map(|k| rotation(key, k, n)).collect();
                for round in said.chunks(n) {
                    let mut lines = round.to_vec();
                    lines.sort_unstable();
                    assert_eq!(lines, (0..n).collect::<Vec<_>>(), "n {n} key {key}");
                }
                if n > 1 {
                    assert!(said.windows(2).all(|w| w[0] != w[1]), "n {n}: {said:?}");
                }
                // Deterministic.
                let again: Vec<usize> = (0..n as u32 * 12).map(|k| rotation(key, k, n)).collect();
                assert_eq!(said, again);
            }
        }
        // Rounds are shuffled differently, and speakers differ.
        let order = |key, round: u32| -> Vec<usize> {
            (round * 6..round * 6 + 6)
                .map(|k| rotation(key, k, 6))
                .collect()
        };
        assert!((1..8).any(|r| order(7, r) != order(7, 0)));
        assert!((1..8).any(|key| order(key, 0) != order(0, 0)));
    }
}
