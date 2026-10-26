//! Quests (`assets/data/quests.toml` and the campaigns' `data/quests/`, spec §7 "Quests", §8 "Quests", #148): simple, ADOM-like
//! errands people give the player through their dialogue topics.
//!
//! Each `[[quest]]` has an `id`, a `name` (the quest log's title and the topic it is asked
//! about), a `giver` (a speaker in `data/dialogue.toml`: a creature type with a dialogue, or an
//! npc id), a `type` and a `target`:
//!
//! - `kill`: `target` a creature type; `count` of them (default 1), optionally on one `level` (a
//!   level id, or a dungeon branch id for any of its levels). The player's own kills count.
//! - `bring`: `target` an object type; `count` of them (default 1), handed in from the pack to
//!   the giver, or to the speaker `to`.
//! - `reach`: `target` a level id (`depths:3` for a level of a branch): the player gets there.
//! - `deliver`: `target` an object type the giver hands over on accepting; carried to the
//!   speaker `to` and handed in there.
//!
//! The giver's words are `offer`, `accept`, `decline` (optional), and `reminder`; `done` is
//! said by whoever the quest is handed in to (the recipient of a bring with `to` or a delivery,
//! else the giver). The `reward` is `gold` (gold coins), `items` (placement strings, `"apple*2"`),
//! and `xp`, each optional. `requires` names a quest, or a list of them (loaded before this
//! one), that must all be done before this one is offered, and `requires_alignment` the bands the player must be in for it
//! to be offered (spec §8 "Alignment"); `alignment` moves their score as it is completed;
//! `repeatable` quests are offered again once done (never a reach). `reveals` names a hidden
//! location of the overworld revealed as the quest is offered, accepted (the default), or done
//! (spec §7 "Hidden locations"), and `where` says where its goal lies after the objective.

use super::creature::CreatureId;
use super::dialogue::{DialogueId, MAX_LINE, MAX_REPLY, MAX_TOPICS, Speaker, check_lines, is_id};
use super::object::{MAX_PLACED, ObjectId, parse_stacks};
use super::{Content, ContentError};
use crate::alignment::{MAX as ALIGNMENT_MAX, Requirement};
use crate::object::Stack;
use serde::Deserialize;

/// The longest quest name, in characters: it is a topic ("Accept: …") and a log title.
pub const MAX_NAME: usize = 32;
/// The most creatures a kill quest, or objects a bring quest, asks for.
pub const MAX_COUNT: u32 = 99;
/// The most experience a quest gives.
pub const MAX_QUEST_XP: u32 = 1_000_000;
/// The object a reward's `gold` is paid in.
pub const GOLD: &str = "gold_coin";
/// What a giver says when the player declines, unless the quest says otherwise.
pub const DEFAULT_DECLINE: &str = "Another time, then.";

/// Index of a quest in [`Content::quests`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QuestId(pub u16);

/// What kind of errand a quest is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestKind {
    Kill,
    Bring,
    Reach,
    Deliver,
}

impl QuestKind {
    /// Its `type` in the file.
    pub fn id(self) -> &'static str {
        match self {
            QuestKind::Kill => "kill",
            QuestKind::Bring => "bring",
            QuestKind::Reach => "reach",
            QuestKind::Deliver => "deliver",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QuestFile {
    #[serde(default)]
    quest: Vec<QuestSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QuestSpec {
    id: String,
    name: String,
    giver: String,
    #[serde(rename = "type")]
    kind: QuestKind,
    target: String,
    count: Option<u32>,
    level: Option<String>,
    to: Option<String>,
    #[serde(default)]
    reward: RewardSpec,
    offer: String,
    accept: String,
    decline: Option<String>,
    reminder: String,
    done: String,
    requires: Option<Requires>,
    requires_alignment: Option<Requirement>,
    #[serde(default)]
    alignment: i32,
    #[serde(default)]
    repeatable: bool,
    reveals: Option<RevealSpec>,
    #[serde(rename = "where")]
    where_: Option<String>,
}

/// A quest's `requires`: one quest id, or a list of them (`["ledger", "barrow_heart"]`).
#[derive(Deserialize)]
#[serde(untagged)]
enum Requires {
    One(String),
    Many(Vec<String>),
}

impl Requires {
    fn ids(&self) -> &[String] {
        match self {
            Requires::One(id) => std::slice::from_ref(id),
            Requires::Many(ids) => ids,
        }
    }
}

/// A quest's `reveals = { location = "ID", when = "accepted" }`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevealSpec {
    location: String,
    #[serde(default)]
    when: RevealWhen,
}

/// At which of a quest's stages its hidden location is revealed (spec §7 "Hidden locations").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevealWhen {
    /// As the giver offers it: where it is, as a tip.
    Offered,
    /// As the player accepts it.
    #[default]
    Accepted,
    /// As it is done.
    Done,
}

/// The hidden location a quest reveals, and when (spec §7 "Hidden locations").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reveal {
    /// A hidden location of the overworld, by id.
    pub location: String,
    pub when: RevealWhen,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct RewardSpec {
    #[serde(default)]
    gold: u32,
    #[serde(default)]
    xp: u32,
    #[serde(default)]
    items: Vec<String>,
}

/// What a quest asks of the player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Goal {
    /// Kill `count` creatures of a type, on `level` (a level id or a branch id) if given.
    Kill {
        creature: CreatureId,
        count: u32,
        level: Option<String>,
    },
    /// Hand in `count` objects of a type to the giver, or to `to`.
    Bring {
        object: ObjectId,
        count: u32,
        to: Option<DialogueId>,
    },
    /// Get to a level.
    Reach { level: String },
    /// Carry the object the giver hands over to `to`.
    Deliver { object: ObjectId, to: DialogueId },
}

/// What completing a quest gives.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reward {
    /// Gold coins.
    pub gold: u32,
    pub xp: u32,
    pub items: Vec<Stack>,
}

impl Reward {
    /// The objects it gives: the gold coins, then the items.
    pub fn stacks(&self, content: &Content) -> Vec<Stack> {
        let gold = (content.object_id(GOLD))
            .filter(|_| self.gold > 0)
            .map(|object| Stack::new(object, self.gold));
        gold.into_iter().chain(self.items.iter().cloned()).collect()
    }
}

/// What the people of a quest say.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestTexts {
    /// The giver, asked about the quest: what they want.
    pub offer: String,
    /// The giver, as the player accepts.
    pub accept: String,
    /// The giver, as the player declines.
    pub decline: String,
    /// The giver, asked about it before it is done.
    pub reminder: String,
    /// Whoever it is handed in to, as it is.
    pub done: String,
}

/// A quest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestDef {
    /// Its id in topics (`quest-rats`) and saves.
    pub id: String,
    /// Its title in the quest log, and the topic it is asked about.
    pub name: String,
    /// Who gives it: whoever speaks this dialogue.
    pub giver: DialogueId,
    pub goal: Goal,
    pub reward: Reward,
    pub texts: QuestTexts,
    /// The quests that must all be done before this one is offered.
    pub requires: Vec<QuestId>,
    /// The alignment the player must be of for it to be offered (spec §8 "Alignment").
    pub requires_alignment: Option<Requirement>,
    /// How far completing it moves the player's alignment score (spec §8 "Alignment"):
    /// helping quests good, dark ones evil.
    pub alignment: i32,
    /// Offered again once done.
    pub repeatable: bool,
    /// The campaign whose `data/quests/` it is in; `None` outside every campaign.
    pub campaign: Option<String>,
    /// The hidden location it reveals, and at which stage (spec §7 "Hidden locations").
    pub reveals: Option<Reveal>,
    /// Where its goal lies, in words, after the objective in the quest log ("The minster lies
    /// in Micelfenn, east of Fennhām.").
    pub where_: Option<String>,
}

impl QuestDef {
    pub fn kind(&self) -> QuestKind {
        match self.goal {
            Goal::Kill { .. } => QuestKind::Kill,
            Goal::Bring { .. } => QuestKind::Bring,
            Goal::Reach { .. } => QuestKind::Reach,
            Goal::Deliver { .. } => QuestKind::Deliver,
        }
    }

    /// Who it is handed in to, and who rewards it: the recipient of a bring with `to` or a
    /// delivery, else the giver.
    pub fn turn_in(&self) -> DialogueId {
        match self.goal {
            Goal::Bring { to: Some(to), .. } | Goal::Deliver { to, .. } => to,
            _ => self.giver,
        }
    }

    /// What is handed in from the pack to complete it, and how many (bring and deliver).
    pub fn hand_in(&self) -> Option<(ObjectId, u32)> {
        match self.goal {
            Goal::Bring { object, count, .. } => Some((object, count)),
            Goal::Deliver { object, .. } => Some((object, 1)),
            _ => None,
        }
    }

    /// Whether a kill of a `creature` on level `level` counts toward it.
    pub fn counts_kill(&self, content: &Content, creature: CreatureId, level: &str) -> bool {
        match &self.goal {
            Goal::Kill {
                creature: target,
                level: place,
                ..
            } => *target == creature && place.as_deref().is_none_or(|p| content.is_in(level, p)),
            _ => false,
        }
    }

    /// The levels or branches it names: a kill's `level`, a reach's target.
    pub fn places(&self) -> impl Iterator<Item = &str> {
        let place = match &self.goal {
            Goal::Kill { level, .. } => level.as_deref(),
            Goal::Reach { level } => Some(level.as_str()),
            _ => None,
        };
        place.into_iter()
    }

    /// The creature type a kill quest is after.
    pub fn kill_target(&self) -> Option<CreatureId> {
        match self.goal {
            Goal::Kill { creature, .. } => Some(creature),
            _ => None,
        }
    }

    /// Everyone it names: the giver, and whoever it is handed in to.
    pub fn speakers(&self) -> impl Iterator<Item = DialogueId> {
        [self.giver, self.turn_in()].into_iter()
    }
}

impl Content {
    /// The quests (`data/quests.toml`; none without it), in file order.
    pub fn quests(&self) -> &[QuestDef] {
        &self.quests
    }

    pub fn quest(&self, id: QuestId) -> &QuestDef {
        &self.quests[usize::from(id.0)]
    }

    /// The quest with id `id`.
    pub fn quest_id(&self, id: &str) -> Option<QuestId> {
        (self.quests.iter().position(|q| q.id == id)).map(|i| QuestId(i as u16))
    }

    /// Whether level `level` is `place`: the level itself, or a level of the branch `place`.
    pub fn is_in(&self, level: &str, place: &str) -> bool {
        level == place || (self.branch_level(level)).is_some_and(|(branch, _)| branch.id == place)
    }

    /// What `place` (a level or a branch id) is called: "The Arena", "The Depths".
    pub fn place_name(&self, place: &str) -> String {
        (self.level_display_name(place))
            .or_else(|| self.dungeon(place).map(|d| d.name.clone()))
            .unwrap_or_else(|| place.to_string())
    }

    /// What whoever speaks dialogue `id` is called, without an article: a unique npc's proper
    /// name ("Sister Hild"), its creature type's name ("old hermit"), or for a unique npc
    /// without one that of the creature a level makes it.
    pub fn speaker_name(&self, id: DialogueId) -> String {
        if let Some(name) = &self.dialogue(id).name {
            return name.clone();
        }
        let creature = match &self.dialogue(id).speaker {
            Speaker::Creature(c) => Some(*c),
            Speaker::Npc(npc) => (self.levels.values())
                .flat_map(|l| &l.creatures)
                .find(|s| s.npc.as_deref() == Some(npc))
                .map(|s| s.creature),
        };
        match (creature, &self.dialogue(id).speaker) {
            (Some(c), _) => self.creature(c).name.clone(),
            (None, Speaker::Npc(npc)) => npc.clone(),
            (None, Speaker::Creature(_)) => String::new(),
        }
    }

    /// The dialogue spoken by the speaker named `name`: a creature type with a dialogue, else an
    /// npc with one.
    fn speaker(&self, name: &str) -> Option<DialogueId> {
        let find = |s: &Speaker| self.dialogues.iter().position(|d| d.speaker == *s);
        let creature = self
            .creature_id(name)
            .and_then(|c| find(&Speaker::Creature(c)));
        creature
            .or_else(|| find(&Speaker::Npc(name.to_string())))
            .map(|i| DialogueId(i as u16))
    }
}

/// Parses the quest table (`data/quests.toml`, `data/quests/`, and each campaign's
/// `data/quests/`; [`super::ContentFiles::table`]) against the creatures, objects, dialogues,
/// levels, and dungeon branches loaded already.
pub(super) fn parse<'a>(
    table: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<QuestDef>, ContentError> {
    let mut quests: Vec<QuestDef> = Vec::new();
    let mut seen: std::collections::BTreeMap<String, &str> = std::collections::BTreeMap::new();
    let mut last = "data/quests.toml";
    for (file, text) in table {
        last = file;
        let err = |m: String| ContentError::new(file, m);
        let parsed: QuestFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for spec in parsed.quest {
            let id = spec.id.clone();
            let bad = |m: String| err(format!("quest `{id}`: {m}"));
            if !is_id(&spec.id) {
                return Err(bad(
                    "ids are lower-case letters, digits, and underscores".into()
                ));
            }
            if let Some(first) = seen.insert(spec.id.clone(), file) {
                return Err(super::duplicate_id("quest", &spec.id, file, first));
            }
            let campaign = super::campaign::of_path(file);
            let mut quest = resolve(spec, &quests, (content, campaign)).map_err(bad)?;
            quest.campaign = campaign.map(str::to_string);
            quests.push(quest);
        }
    }
    let err = |m: String| ContentError::new(last, m);
    if quests.len() > usize::from(u16::MAX) {
        return Err(err("too many quests".into()));
    }
    check_topics(&quests, content).map_err(err)?;
    Ok(quests)
}

/// One quest as written, checked against `content` and the quests before it.
/// `campaign` is the one whose table it is in (its local areas are the ones a quest may name).
fn resolve(
    spec: QuestSpec,
    before: &[QuestDef],
    (content, campaign): (&Content, Option<&str>),
) -> Result<QuestDef, String> {
    let campaign = campaign.unwrap_or_default();
    if spec.name.chars().count() > MAX_NAME {
        return Err(format!("`name` is at most {MAX_NAME} characters"));
    }
    check_lines([&spec.name], MAX_LINE)?;
    let speaker = |name: &str| {
        content.speaker(name).ok_or_else(|| {
            format!("`{name}` is no one who talks (a creature or npc with a dialogue)")
        })
    };
    let giver = speaker(&spec.giver)?;
    let count = |n: Option<u32>| match n.unwrap_or(1) {
        n @ 1..=MAX_COUNT => Ok(n),
        _ => Err(format!("`count` is 1 to {MAX_COUNT}")),
    };
    let object = |id: &str| (content.object_id(id)).ok_or_else(|| format!("unknown object `{id}`"));
    let kind = spec.kind;
    let unused = |field: &str, given: bool| {
        if given {
            Err(format!("a {} quest has no `{field}`", kind.id()))
        } else {
            Ok(())
        }
    };
    let goal = match kind {
        QuestKind::Kill => {
            unused("to", spec.to.is_some())?;
            let creature = (content.creature_id(&spec.target))
                .filter(|_| spec.target != "player")
                .ok_or_else(|| format!("unknown creature `{}`", spec.target))?;
            if let Some(level) = &spec.level
                && !content.has_level(campaign, level)
                && content.dungeon(level).is_none()
            {
                return Err(format!("unknown level or dungeon branch `{level}`"));
            }
            Goal::Kill {
                creature,
                count: count(spec.count)?,
                level: spec.level,
            }
        }
        QuestKind::Bring => {
            unused("level", spec.level.is_some())?;
            let to = spec.to.as_deref().map(speaker).transpose()?;
            Goal::Bring {
                object: object(&spec.target)?,
                count: count(spec.count)?,
                to: to.filter(|&to| to != giver),
            }
        }
        QuestKind::Reach => {
            unused("level", spec.level.is_some())?;
            unused("to", spec.to.is_some())?;
            unused("count", spec.count.is_some())?;
            if !content.has_level(campaign, &spec.target) {
                return Err(format!("unknown level `{}`", spec.target));
            }
            if spec.repeatable {
                return Err(
                    "a reach quest is not repeatable: a place once reached stays so".into(),
                );
            }
            Goal::Reach { level: spec.target }
        }
        QuestKind::Deliver => {
            unused("level", spec.level.is_some())?;
            unused("count", spec.count.is_some())?;
            let to = speaker(spec.to.as_deref().ok_or("a deliver quest needs `to`")?)?;
            if to == giver {
                return Err("a delivery goes to someone other than the giver".into());
            }
            Goal::Deliver {
                object: object(&spec.target)?,
                to,
            }
        }
    };
    let reward = &spec.reward;
    if reward.gold > MAX_PLACED {
        return Err(format!("reward: `gold` is at most {MAX_PLACED}"));
    }
    if reward.gold > 0 && content.object_id(GOLD).is_none() {
        return Err(format!("reward: gold needs the object `{GOLD}`"));
    }
    if reward.xp > MAX_QUEST_XP {
        return Err(format!("reward: `xp` is at most {MAX_QUEST_XP}"));
    }
    let items = parse_stacks(&reward.items, content).map_err(|e| format!("reward: {e}"))?;
    let decline = spec.decline.unwrap_or_else(|| DEFAULT_DECLINE.into());
    let said = [
        &spec.offer,
        &spec.accept,
        &decline,
        &spec.reminder,
        &spec.done,
    ];
    check_lines(said, MAX_REPLY)?;
    if let Some(r) = spec.requires_alignment {
        r.check()?;
    }
    if spec.alignment.abs() > ALIGNMENT_MAX {
        return Err(format!(
            "`alignment` is -{ALIGNMENT_MAX} to {ALIGNMENT_MAX}"
        ));
    }
    let mut requires = Vec::new();
    for id in spec.requires.as_ref().map_or(&[][..], Requires::ids) {
        let quest = (before.iter().position(|q| q.id == *id))
            .map(|i| QuestId(i as u16))
            .ok_or_else(|| format!("`requires`: no quest `{id}` before this one"))?;
        if requires.contains(&quest) {
            return Err(format!("`requires` names `{id}` twice"));
        }
        requires.push(quest);
    }
    let reveals = match spec.reveals {
        Some(r) if content.hidden_location(campaign, &r.location).is_none() => {
            return Err(format!(
                "`reveals`: the overworld has no hidden location `{}`",
                r.location
            ));
        }
        Some(r) => Some(Reveal {
            location: r.location,
            when: r.when,
        }),
        None => None,
    };
    if let Some(text) = &spec.where_ {
        check_lines([text], MAX_REPLY)?;
    }
    Ok(QuestDef {
        id: spec.id,
        name: spec.name,
        giver,
        goal,
        reward: Reward {
            gold: spec.reward.gold,
            xp: spec.reward.xp,
            items,
        },
        texts: QuestTexts {
            offer: spec.offer,
            accept: spec.accept,
            decline,
            reminder: spec.reminder,
            done: spec.done,
        },
        requires,
        requires_alignment: spec.requires_alignment,
        alignment: spec.alignment,
        repeatable: spec.repeatable,
        campaign: None,
        reveals,
        where_: spec.where_,
    })
}

/// Everyone's topics fit the dialogue screen: their dialogue's own, one per quest they give or
/// take, and one more while an offer of theirs waits for an answer (accept and decline).
fn check_topics(quests: &[QuestDef], content: &Content) -> Result<(), String> {
    for (i, dialogue) in content.dialogues().iter().enumerate() {
        let id = DialogueId(i as u16);
        let theirs = (quests.iter())
            .filter(|q| q.giver == id || q.turn_in() == id)
            .count();
        let gives = quests.iter().any(|q| q.giver == id);
        let topics = dialogue.topics.len() + theirs + usize::from(gives);
        if topics > MAX_TOPICS {
            return Err(format!(
                "`{}` has {topics} topics with its quests; at most {MAX_TOPICS} fit",
                content.speaker_name(id)
            ));
        }
    }
    Ok(())
}
