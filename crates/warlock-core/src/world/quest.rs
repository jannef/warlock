//! Quests (spec §8 "Quests", #148): simple, ADOM-like errands people give the player through
//! their dialogue topics ([`World::topics`], answered in [`World::ask`]).
//!
//! **Flow**: a giver lists each quest it has for the player as a topic (`quest-ID`, named after
//! the quest). Asking it hears the offer, which then waits for an answer: `accept-ID` or
//! `decline-ID` (one offer per giver at a time). An active quest stays a topic of its giver
//! (the reminder) and, once its goal is met, of whoever it is handed in to; asking that one
//! completes it: what it asks for leaves the pack, they say their thanks, and the reward
//! follows (gold coins and items into the pack, or at the player's feet if it is full; XP).
//! A repeatable quest is offered again once done; a quest that `requires` others is offered
//! once they are all done. There is no abandoning a quest and no cap on how many are active (the
//! owner's decision).
//!
//! **Goals**, and how far they have got ([`progress`]):
//! - kill: the player's own kills of the creature type (on its level or branch, if it names
//!   one), counted from a tally of the kills of every creature type (so kills
//!   made before the quest was taken count; a repeat counts only those since);
//! - bring: how many of the object the player carries;
//! - reach: whether the player has been to the level (before taking the quest too);
//! - deliver: whether the player carries the object the giver handed over. Dropping it blocks
//!   the quest until they pick it up again.
//!
//! Progress the player's action made is said at once ("Quest updated: 2 / 3 giant rats."). The
//! quest log reads [`PerceptionView::quests`]: the player knows their quests.

use super::talk::capitalized;
use super::{ActorId, Deed, Spent, TopicView, World};
use crate::action::{Event, Positions};
use crate::content::StatUse;
use crate::content::{Content, DialogueId, Goal, QuestDef, QuestId, QuestKind, RevealWhen};
use crate::object::{self, Stack};
use crate::perception::PerceptionView;
use crate::talent::TalentMods;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The topic of a quest: its offer, reminder, or hand-in (`quest-rats`).
const QUEST: &str = "quest-";
/// The topic that accepts an offer (`accept-rats`).
const ACCEPT: &str = "accept-";
/// The topic that declines an offer (`decline-rats`).
const DECLINE: &str = "decline-";

/// Where a quest the player has heard of stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    /// Offered, waiting for the player to accept or decline.
    Offered,
    /// Taken up.
    Active,
    /// Done (a repeatable one may be offered again).
    Done,
}

/// How far a quest has got (saved).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuestRecord {
    pub stage: Stage,
    /// How many times the player has completed it.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub completed: u32,
    /// A repeated kill quest: the kills already counted when it was taken up again.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub counted: u32,
    /// A reach quest: the player has been there.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reached: bool,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// The player's quests and the kills they count (saved and hashed; left out of the state while
/// empty).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quests {
    /// The quests the player has heard of, by id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    records: BTreeMap<String, QuestRecord>,
    /// The player's kills of every creature type: by level id, then by creature type id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    kills: BTreeMap<String, BTreeMap<String, u32>>,
    /// The creature type the player has killed most, the first to get there of those tied
    /// (the ranger's Favoured prey, spec §8 "Class abilities"); `None` before the first kill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    prey: Option<String>,
}

impl Quests {
    pub fn is_empty(&self) -> bool {
        self.records.is_empty() && self.kills.is_empty() && self.prey.is_none()
    }

    /// How many creatures of type `id` the player has killed, on every level.
    pub fn killed(&self, id: &str) -> u32 {
        self.kills.values().filter_map(|k| k.get(id)).sum()
    }

    /// The creature type the player has killed most (ties: the first to get there).
    pub fn prey(&self) -> Option<&str> {
        self.prey.as_deref()
    }

    /// Where quest `id` stands, if the player has heard of it.
    pub fn record(&self, id: &str) -> Option<&QuestRecord> {
        self.records.get(id)
    }

    /// The quest records, by id.
    pub fn records(&self) -> impl Iterator<Item = (&str, &QuestRecord)> {
        self.records.iter().map(|(id, r)| (id.as_str(), r))
    }

    /// The tallied kills, by level and creature type id.
    pub fn kills(&self) -> impl Iterator<Item = (&str, &str, u32)> {
        (self.kills.iter())
            .flat_map(|(level, k)| k.iter().map(move |(c, n)| (level.as_str(), c.as_str(), *n)))
    }

    /// A quest or creature type these records name that `content` does not have (a guard for
    /// saves).
    pub(crate) fn unknown_to(&self, content: &Content) -> Option<String> {
        let quest = (self.records.keys()).find(|id| content.quest_id(id).is_none());
        let creature = (self.kills.values())
            .flat_map(BTreeMap::keys)
            .chain(self.prey.iter())
            .find(|id| content.creature_id(id).is_none());
        quest.or(creature).cloned()
    }

    fn stage(&self, def: &QuestDef) -> Option<Stage> {
        self.records.get(&def.id).map(|r| r.stage)
    }

    /// Whether quest `def` is active.
    fn is_active(&self, def: &QuestDef) -> bool {
        self.stage(def) == Some(Stage::Active)
    }

    /// Whether `def` may be offered: what it requires is done, and it is new to the player (or
    /// done and repeatable).
    fn available(&self, content: &Content, def: &QuestDef) -> bool {
        let required = def.requires.iter().all(|&r| {
            let id = &content.quest(r).id;
            self.records.get(id).is_some_and(|r| r.completed > 0)
        });
        required
            && match self.records.get(&def.id) {
                None => true,
                Some(r) => r.stage == Stage::Done && def.repeatable,
            }
    }

    /// The player's kills that count toward kill quest `def`, all of them.
    fn kills_for(&self, content: &Content, def: &QuestDef) -> u32 {
        let Goal::Kill { creature, .. } = def.goal else {
            return 0;
        };
        let id = &content.creature(creature).id;
        (self.kills.iter())
            .filter(|(level, _)| def.counts_kill(content, creature, level))
            .filter_map(|(_, kills)| kills.get(id))
            .sum()
    }

    /// The quest goes back to before it was offered: forgotten if never done, else done.
    fn withdraw(&mut self, def: &QuestDef) {
        match self.records.get_mut(&def.id) {
            Some(r) if r.completed > 0 => r.stage = Stage::Done,
            _ => {
                self.records.remove(&def.id);
            }
        }
    }
}

/// How far active quest `def` has got: what counts toward it, at most what it needs, and what
/// it needs. `inventory` is the player's.
pub(crate) fn progress(
    content: &Content,
    quests: &Quests,
    inventory: &[Stack],
    def: &QuestDef,
) -> (u32, u32) {
    let record = quests.records.get(&def.id);
    let carried = |object| -> u32 {
        (inventory.iter())
            .filter(|s| s.object == object)
            .map(|s| s.count)
            .sum()
    };
    let (have, need) = match def.goal {
        Goal::Kill { count, .. } => {
            let counted = record.map_or(0, |r| r.counted);
            (
                quests.kills_for(content, def).saturating_sub(counted),
                count,
            )
        }
        Goal::Bring { object, count, .. } => (carried(object), count),
        Goal::Reach { .. } => (u32::from(record.is_some_and(|r| r.reached)), 1),
        Goal::Deliver { object, .. } => (carried(object), 1),
    };
    (have.min(need), need)
}

/// A quest as the quest log shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestView {
    pub id: String,
    pub name: String,
    pub kind: QuestKind,
    /// Taken up and not yet done; else done.
    pub active: bool,
    /// How many times the player has completed it.
    pub completed: u32,
    /// Who gave it: "the old hermit".
    pub giver: String,
    /// What the giver asked, in their words.
    pub offer: String,
    /// What it asks: "Kill 3 giant rats in the Arena."
    pub objective: String,
    /// How far it has got: "2 / 3 killed.", "Return to the old hermit.", "Done."
    pub status: String,
}

impl PerceptionView<'_> {
    /// The player's quests (spec §8 "Quests"): the active ones, then the done ones, each in the
    /// order of `data/quests.toml`. Offers not yet answered are not here.
    pub fn quests(&self) -> Vec<QuestView> {
        let content = self.content;
        let view = |def: &QuestDef, record: &QuestRecord| {
            let active = record.stage == Stage::Active;
            let status = if active {
                status(content, self.quests, &self.player.inventory, def)
            } else if record.completed > 1 {
                format!("Done ({} times).", record.completed)
            } else {
                "Done.".to_string()
            };
            QuestView {
                id: def.id.clone(),
                name: def.name.clone(),
                kind: def.kind(),
                active,
                completed: record.completed,
                giver: speaker_called(content, def.giver),
                offer: def.texts.offer.clone(),
                objective: objective(content, def),
                status,
            }
        };
        let of = |active: bool| {
            (content.quests().iter()).filter_map(move |def| {
                let record = self.quests.records.get(&def.id)?;
                let shown = match record.stage {
                    Stage::Active => active,
                    // A repeatable quest offered again was done before.
                    Stage::Done | Stage::Offered => !active && record.completed > 0,
                };
                shown.then(|| view(def, record))
            })
        };
        of(true).chain(of(false)).collect()
    }
}

/// What whoever speaks dialogue `id` is called in messages: "the old hermit", "Sister Hild".
fn speaker_called(content: &Content, id: DialogueId) -> String {
    content.speaker_called(id)
}

/// `name` as it reads inside a sentence: "The Arena" becomes "the Arena".
fn in_sentence(name: &str) -> String {
    match name.strip_prefix("The ") {
        Some(rest) => format!("the {rest}"),
        None => name.to_string(),
    }
}

/// `n` creatures of type `creature`: "a kobold shaman", "3 giant rats".
fn creatures(content: &Content, creature: crate::content::CreatureId, n: u32) -> String {
    let def = content.creature(creature);
    if n == 1 {
        format!("{} {}", crate::content::article(&def.name), def.name)
    } else {
        format!("{n} {}", def.plural)
    }
}

/// What quest `def` asks, as the log says it: "Kill 3 giant rats in the Arena.", then where,
/// if the quest says so in words ("Reach the Depths. The way in lies in the Barrowdowns.").
fn objective(content: &Content, def: &QuestDef) -> String {
    let asks = asks(content, def);
    match &def.where_ {
        Some(place) => format!("{asks} {place}"),
        None => asks,
    }
}

/// What quest `def` asks, without where ("Kill 5 giant rats in Cealdhlāw."): the log's
/// objective, and the wiki's goal.
pub fn asks(content: &Content, def: &QuestDef) -> String {
    match &def.goal {
        Goal::Kill {
            creature,
            count,
            level,
        } => {
            let place = level.as_deref().map_or_else(String::new, |l| {
                format!(" in {}", in_sentence(&content.place_name(l)))
            });
            format!("Kill {}{place}.", creatures(content, *creature, *count))
        }
        Goal::Bring { object, count, .. } => format!(
            "Bring {} to {}.",
            content.object(*object).describe(*count),
            speaker_called(content, def.turn_in())
        ),
        Goal::Reach { level } => format!("Reach {}.", in_sentence(&content.place_name(level))),
        Goal::Deliver { object, to } => format!(
            "Deliver {} to {}.",
            content.object(*object).describe(1),
            speaker_called(content, *to)
        ),
    }
}

/// Where active quest `def` stands, as the log says it.
fn status(content: &Content, quests: &Quests, inventory: &[Stack], def: &QuestDef) -> String {
    let (have, need) = progress(content, quests, inventory, def);
    if have >= need {
        return format!("{}.", capitalized(&next_step(content, def)));
    }
    match def.goal {
        Goal::Kill { .. } => format!("{have} / {need} killed."),
        Goal::Bring { .. } => format!("You carry {have} / {need}."),
        Goal::Reach { .. } => "Not reached yet.".into(),
        Goal::Deliver { object, .. } => {
            format!("You no longer carry the {}.", content.object(object).name)
        }
    }
}

/// What the player does once quest `def`'s goal is met: "return to the old hermit".
fn next_step(content: &Content, def: &QuestDef) -> String {
    let to = speaker_called(content, def.turn_in());
    match def.goal {
        Goal::Deliver { object, .. } => {
            format!("take the {} to {to}", content.object(object).name)
        }
        Goal::Bring { .. } if def.turn_in() != def.giver => format!("take them to {to}"),
        _ => format!("return to {to}"),
    }
}

/// The message for progress on quest `def`, now `have` of `need`.
fn progress_news(content: &Content, def: &QuestDef, (have, need): (u32, u32)) -> String {
    let done = have >= need;
    let mut text = match def.goal {
        Goal::Kill { creature, .. } => {
            let what = content.creature(creature);
            let name = if need == 1 { &what.name } else { &what.plural };
            format!("Quest updated: {have} / {need} {name}.")
        }
        Goal::Bring { object, .. } => {
            let what = content.object(object);
            let name = if need == 1 { &what.name } else { &what.plural };
            format!("Quest updated: {have} / {need} {name}.")
        }
        Goal::Reach { ref level } => format!(
            "Quest updated: you have reached {}.",
            in_sentence(&content.place_name(level))
        ),
        Goal::Deliver { .. } => "Quest updated:".into(),
    };
    if done {
        text.push(' ');
        let step = next_step(content, def);
        match def.goal {
            Goal::Deliver { .. } => text.push_str(&format!("{step}.")),
            _ => text.push_str(&format!("{}.", capitalized(&step))),
        }
    }
    text
}

/// What a quest topic asks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ask {
    /// The offer, the reminder, or handing it in.
    Quest,
    Accept,
    Decline,
}

/// The quest a topic id is about, and what it asks; `None` for any other topic.
fn quest_topic(topic: &str) -> Option<(Ask, &str)> {
    [
        (QUEST, Ask::Quest),
        (ACCEPT, Ask::Accept),
        (DECLINE, Ask::Decline),
    ]
    .into_iter()
    .find_map(|(prefix, ask)| topic.strip_prefix(prefix).map(|id| (ask, id)))
}

impl World {
    /// The quest topics of whoever speaks dialogue `speaker`, first among their topics: each
    /// quest they may offer (or the accept and decline of the one they offered; one they
    /// [`World::offers`] the player as they stand toward them, `scorned` or not), each of theirs
    /// the player has taken up (for the reminder, or to hand it in), and each the player may hand
    /// in to them.
    pub(super) fn quest_topics(&self, speaker: DialogueId, scorned: bool) -> Vec<TopicView> {
        let quests = &self.state.quests;
        let mut topics = Vec::new();
        for def in self.content.quests() {
            let topic = |prefix: &str, name: String| TopicView {
                id: format!("{prefix}{}", def.id),
                name,
                price: None,
            };
            let stage = quests.stage(def);
            if def.giver == speaker {
                if stage == Some(Stage::Offered) {
                    topics.push(topic(ACCEPT, format!("Accept: {}", def.name)));
                    topics.push(topic(DECLINE, format!("Decline: {}", def.name)));
                } else if stage == Some(Stage::Active)
                    || (quests.available(&self.content, def) && self.offers(def, scorned))
                {
                    topics.push(topic(QUEST, def.name.clone()));
                }
            } else if def.turn_in() == speaker && stage == Some(Stage::Active) && self.ready(def) {
                topics.push(topic(QUEST, def.name.clone()));
            }
        }
        topics
    }

    /// Whether active quest `def`'s goal is met.
    fn ready(&self, def: &QuestDef) -> bool {
        let (have, need) = self.progress_of(def);
        have >= need
    }

    fn progress_of(&self, def: &QuestDef) -> (u32, u32) {
        let inventory = &self.state.player().inventory;
        progress(&self.content, &self.state.quests, inventory, def)
    }

    /// Answers quest topic `topic` asked of creature `id`, who speaks `speaker`; `None` if it is
    /// no quest topic. A topic they do not have now is refused.
    pub(super) fn ask_quest(
        &mut self,
        id: ActorId,
        speaker: DialogueId,
        topic: &str,
        events: &mut Vec<Event>,
    ) -> Option<Spent> {
        let (ask, quest) = quest_topic(topic)?;
        let scorned = self.scorns(&self.state.actors[&id]);
        let listed = (self.quest_topics(speaker, scorned).iter()).any(|t| t.id == topic);
        let Some(q) = self.content.quest_id(quest).filter(|_| listed) else {
            let who = capitalized(&self.called(&self.state.actors[&id]));
            self.say(format!("{who} knows nothing about that."));
            return Some(Spent::Nothing);
        };
        let content = self.content.clone();
        let def = content.quest(q);
        Some(match ask {
            Ask::Quest if self.state.quests.is_active(def) => {
                if speaker == def.turn_in() && self.ready(def) {
                    self.complete(id, q, topic, events);
                    Spent::NoTime
                } else {
                    self.answer(id, def.texts.reminder.clone(), Some(topic.into()), events);
                    Spent::Nothing
                }
            }
            Ask::Quest => {
                self.offer(q);
                self.answer(id, def.texts.offer.clone(), Some(topic.into()), events);
                self.reveal_at(id, def, RevealWhen::Offered);
                Spent::NoTime
            }
            Ask::Accept => self.accept(id, q, topic, events),
            Ask::Decline => {
                self.state.quests.withdraw(def);
                self.answer(id, def.texts.decline.clone(), Some(topic.into()), events);
                Spent::NoTime
            }
        })
    }

    /// Quest `def` reached stage `when`: if it reveals its hidden location then (spec §7
    /// "Hidden locations"), creature `id`, who speaks for it, marks the location on the player's
    /// map. A location revealed before is not marked again.
    fn reveal_at(&mut self, id: ActorId, def: &QuestDef, when: RevealWhen) {
        let Some(reveal) = def.reveals.as_ref().filter(|r| r.when == when) else {
            return;
        };
        if !self.reveal_location(&reveal.location) {
            return;
        }
        let Some((_, location)) = self
            .content
            .hidden_location(&self.state.campaign, &reveal.location)
        else {
            return;
        };
        let place = in_sentence(&location.name);
        let who = capitalized(&self.called(&self.state.actors[&id]));
        self.say(format!("{who} marks {place} on your map."));
    }

    /// Quest `q` is offered: its giver withdraws any other offer still waiting for an answer.
    fn offer(&mut self, q: QuestId) {
        let content = self.content.clone();
        let def = content.quest(q);
        for other in content.quests() {
            let waiting = self.state.quests.stage(other) == Some(Stage::Offered);
            if other.giver == def.giver && other.id != def.id && waiting {
                self.state.quests.withdraw(other);
            }
        }
        let quests = &mut self.state.quests.records;
        let record = quests.entry(def.id.clone()).or_insert(QuestRecord {
            stage: Stage::Offered,
            completed: 0,
            counted: 0,
            reached: false,
        });
        record.stage = Stage::Offered;
    }

    /// The player accepts quest `q` from creature `id`: it becomes active, the giver hands over
    /// what is to be delivered, and if its goal is met already, the player is told.
    fn accept(&mut self, id: ActorId, q: QuestId, topic: &str, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let def = content.quest(q);
        let given = match def.goal {
            Goal::Deliver { object, .. } => {
                let mut letter = Stack::one(object);
                crate::rot::made_at([&mut letter], &content, self.state.time);
                object::add_to_inventory(self.state.inventory_mut(), letter.clone(), &content);
                Some(letter)
            }
            _ => None,
        };
        let kills = self.state.quests.kills_for(&content, def);
        let reached = match &def.goal {
            Goal::Reach { level } => self.state.levels.contains_key(level),
            _ => false,
        };
        if let Some(record) = self.state.quests.records.get_mut(&def.id) {
            record.stage = Stage::Active;
            // A repeat counts only the kills from now on.
            record.counted = if record.completed > 0 { kills } else { 0 };
            record.reached = reached;
        }
        self.answer(id, def.texts.accept.clone(), Some(topic.into()), events);
        self.reveal_at(id, def, RevealWhen::Accepted);
        if let Some(letter) = given {
            let who = capitalized(&self.called(&self.state.actors[&id]));
            self.say(format!("{who} gives you {}.", letter.describe(&content)));
        }
        let progress = self.progress_of(def);
        if def.kind() != QuestKind::Deliver && progress.0 >= progress.1 {
            self.say(progress_news(&content, def, progress));
        }
        Spent::NoTime
    }

    /// Creature `id` takes quest `q` in: what it asks for leaves the pack, they say their thanks,
    /// and the reward follows.
    fn complete(&mut self, id: ActorId, q: QuestId, topic: &str, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let def = content.quest(q);
        if let Some((object, count)) = def.hand_in() {
            let handed = self.take_from_pack(object, count);
            let what = object::phrase(&handed, &content);
            let who = self.called(&self.state.actors[&id]);
            self.say(format!("You give {who} {what}."));
        }
        if let Some(record) = self.state.quests.records.get_mut(&def.id) {
            record.stage = Stage::Done;
            record.completed += 1;
            record.counted = 0;
        }
        self.answer(id, def.texts.done.clone(), Some(topic.into()), events);
        self.reveal_at(id, def, RevealWhen::Done);
        self.say(format!("Quest complete: {}.", def.name));
        self.train_stat(StatUse::Quest);
        let who = capitalized(&self.called(&self.state.actors[&id]));
        // Class abilities (spec §8 "Class abilities": Well dressed, Folk hero, Magnate) make a
        // quest pay more gold and experience.
        let talents = self.state.player().talent_mods(&content);
        let mut stacks = def.reward.stacks(&content);
        if let Some(gold) = (content.object_id(crate::content::GOLD))
            .and_then(|g| stacks.iter_mut().find(|s| s.object == g))
        {
            let more = TalentMods::scaled(u64::from(gold.count), talents.quest_gold_percent);
            gold.count = u32::try_from(more).unwrap_or(u32::MAX).max(1);
        }
        let xp = TalentMods::scaled(u64::from(def.reward.xp), talents.quest_xp_percent);
        let xp = u32::try_from(xp).unwrap_or(u32::MAX);
        crate::rot::made_at(&mut stacks, &content, self.state.time);
        if !stacks.is_empty() {
            self.say(format!(
                "{who} gives you {}.",
                object::phrase(&stacks, &content)
            ));
            self.give_player(&stacks);
        }
        if xp > 0 {
            self.say(format!("You gain {xp} experience."));
            self.grant_xp(xp);
            self.say_news();
        }
        self.shift_alignment(self.state.player, def.alignment, Deed::Quest);
    }

    /// Takes `count` of `object` out of the player's pack (stacks not worn or wielded first, the
    /// last first), keeping the letters of what is worn right. Returns what was taken.
    fn take_from_pack(&mut self, object: crate::content::ObjectId, count: u32) -> Vec<Stack> {
        let player = self.player_mut();
        let mut order: Vec<usize> = (0..player.inventory.len())
            .filter(|&i| player.inventory[i].object == object)
            .collect();
        order.sort_by_key(|&i| (player.equipment.holds(i), std::cmp::Reverse(i)));
        let (mut left, mut emptied) = (count, Positions::default());
        for i in order {
            let stack = &mut player.inventory[i];
            let take = stack.count.min(left);
            stack.count -= take;
            left -= take;
            if stack.count == 0 {
                emptied.insert(i);
            }
            if left == 0 {
                break;
            }
        }
        player.inventory.retain(|s| s.count > 0);
        player.equipment.after_removal(&emptied);
        vec![Stack::new(object, count - left)]
    }

    /// Puts `stacks` into the player's pack.
    fn give_player(&mut self, stacks: &[Stack]) {
        let content = self.content.clone();
        for stack in stacks {
            object::add_to_inventory(self.state.inventory_mut(), stack.clone(), &content);
        }
    }

    /// The player killed a creature of type `creature` on level `level`: tallied, whatever the
    /// type (the design's B6: kill quests and the ranger's Favoured prey read it). The type
    /// killed most becomes the favoured prey once it is killed more often than the one before.
    pub(super) fn tally_kill(&mut self, creature: crate::content::CreatureId, level: &str) {
        let id = self.content.creature(creature).id.clone();
        let quests = &mut self.state.quests;
        let kills = quests.kills.entry(level.to_string()).or_default();
        *kills.entry(id.clone()).or_default() += 1;
        let prey = quests.prey.as_deref().map_or(0, |p| quests.killed(p));
        if quests.killed(&id) > prey {
            quests.prey = Some(id);
        }
    }

    /// The player arrived on level `level`: active reach quests there are reached.
    pub(super) fn reach(&mut self, level: &str) {
        let content = self.content.clone();
        for def in content.quests() {
            let here = matches!(&def.goal, Goal::Reach { level: l } if l == level);
            if let Some(record) = self.state.quests.records.get_mut(&def.id)
                && here
                && record.stage == Stage::Active
            {
                record.reached = true;
            }
        }
    }

    /// How far each active quest has got, before an action of the player's.
    pub(super) fn quest_progress(&self) -> Vec<(QuestId, (u32, u32))> {
        (self.content.quests().iter().enumerate())
            .filter(|(_, def)| self.state.quests.is_active(def))
            .map(|(i, def)| (QuestId(i as u16), self.progress_of(def)))
            .collect()
    }

    /// Says the progress the player's action made on the quests active before it (`before`).
    pub(super) fn quest_news(&mut self, before: &[(QuestId, (u32, u32))]) {
        let content = self.content.clone();
        for &(q, (had, _)) in before {
            let def = content.quest(q);
            if !self.state.quests.is_active(def) {
                continue;
            }
            let now = self.progress_of(def);
            if now.0 > had {
                self.say(progress_news(&content, def, now));
            }
        }
    }
}
