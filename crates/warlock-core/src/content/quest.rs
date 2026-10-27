//! Quests (`assets/data/quests.toml` and the campaigns' `data/quests/`, spec §7 "Quests", §8 "Quests", #148): simple, ADOM-like
//! errands people give the player through their dialogue topics.
//!
//! Each `[[quest]]` has an `id`, a `name` (the quest log's title and the topic it is asked
//! about), a `giver` (a speaker in `data/dialogue.toml`: a creature type with a dialogue, or an
//! npc id), a `type` and a `target`:
//!
//! - `kill`: `target` a creature type, or a tag (`tag:goblin`, [`super::tag`]: any type that
//!   carries it); `count` of them (default 1), optionally on one `level` (a level id, or a
//!   dungeon branch id for any of its levels). The player's own kills count, on every level.
//!   `what` says what they are called instead of the type's or the tag's name ("greenskins").
//! - `bring`: `target` an object type; `count` of them (default 1), handed in from the pack to
//!   the giver, or to the speaker `to`.
//! - `reach`: `target` a level id (`depths:3` for a level of a branch): the player gets there.
//! - `deliver`: `target` an object type the giver hands over on accepting; carried to the
//!   speaker `to` and handed in there.
//!
//! Asking the giver about a quest **gives** it (there is no accepting or declining, #422): the
//! giver says `offer`, then `accept` (optional) in the same reply, and it is active at once.
//! `reminder` is what they say asked again; `done` is said by whoever the quest is handed in to (the recipient of a bring with `to` or a delivery,
//! else the giver). The `reward` is `gold` (gold coins), `items` (placement strings, `"apple*2"`),
//! and `xp`, each optional. `requires` names a quest, or a list of them (loaded before this
//! one), that must all be done before this one is offered, and `requires_alignment` the bands the player must be in for it
//! to be offered (spec §8 "Alignment"); `alignment` moves their score as it is completed;
//! `repeatable` quests are offered again once done (never a reach). `reveals` names a hidden
//! location of the overworld revealed as the quest is given (the default) or done (spec §7
//! "Hidden locations"), and `where` says where its goal lies after the objective.
//!
//! A kill or bring quest may draw its target as it is given instead of naming it: a `pick` or
//! `[[quest.variant]]`s ([`super::quest_roll`]), its texts naming the roll through placeholders.

use super::creature::CreatureId;
use super::dialogue::{DialogueId, MAX_LINE, MAX_REPLY, MAX_TOPICS, Speaker, check_lines, is_id};
use super::object::{MAX_PLACED, ObjectId, parse_stacks};
use super::quest_roll::{self, PickSpec, Rolled, Variant, VariantSpec};
use super::tag::{MAX_TAG, TAG_TARGET, spoken};
use super::{Content, ContentError};
use crate::alignment::{MAX as ALIGNMENT_MAX, Requirement};
use crate::object::Stack;
use serde::Deserialize;

/// The longest quest name, in characters: it is a topic ("Accept: …") and a log title.
pub const MAX_NAME: usize = 32;
/// The most creatures of a type a kill quest, or objects a bring quest, asks for (a tag's
/// kill quest may ask for up to [`MAX_TAG_COUNT`]).
pub const MAX_COUNT: u32 = 99;
/// The most kills a tag's kill quest asks for.
pub const MAX_TAG_COUNT: u32 = 999;
/// The most experience a quest gives.
pub const MAX_QUEST_XP: u32 = 1_000_000;
/// The object a reward's `gold` is paid in.
pub const GOLD: &str = "gold_coin";

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
    target: Option<String>,
    pick: Option<PickSpec>,
    #[serde(default)]
    variant: Vec<VariantSpec>,
    count: Option<u32>,
    level: Option<String>,
    to: Option<String>,
    what: Option<String>,
    #[serde(default)]
    reward: RewardSpec,
    offer: Option<String>,
    accept: Option<String>,
    reminder: Option<String>,
    done: Option<String>,
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

/// A quest's `reveals = { location = "ID", when = "given" }`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RevealSpec {
    location: String,
    #[serde(default)]
    when: RevealWhen,
}

/// At which of a quest's stages its hidden location is revealed (spec §7 "Hidden locations").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevealWhen {
    /// As the giver gives it.
    #[default]
    Given,
    /// As it is done.
    Done,
}

impl RevealWhen {
    /// Its `when` in the file.
    pub fn id(self) -> &'static str {
        match self {
            RevealWhen::Given => "given",
            RevealWhen::Done => "done",
        }
    }
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

/// What a kill quest is after.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KillTarget {
    /// A creature type.
    Creature(CreatureId),
    /// Any creature type that carries a tag (`goblin`).
    Tag(String),
}

impl KillTarget {
    /// Whether a kill of a `creature` is one of these: of the type, or of any member of its
    /// family (spec §8 "Animated objects": `animated_blade` counts every blade).
    pub fn has(&self, content: &Content, creature: CreatureId) -> bool {
        match self {
            KillTarget::Creature(c) => content.family(*c) == content.family(creature),
            KillTarget::Tag(tag) => content.has_tag(creature, tag),
        }
    }

    /// One of them ("goblin chief", "marsh thing") or several ("goblin chiefs", "marsh
    /// things": a tag's plural from the plural dictionary).
    pub fn name(&self, content: &Content, one: bool) -> String {
        match (self, one) {
            (KillTarget::Creature(c), true) => content.creature(*c).name.clone(),
            (KillTarget::Creature(c), false) => content.creature(*c).plural.clone(),
            (KillTarget::Tag(tag), true) => spoken(tag),
            (KillTarget::Tag(tag), false) => content.plural(&spoken(tag)),
        }
    }
}

/// What a quest asks of the player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Goal {
    /// Kill `count` creatures of a type or a tag, on `level` (a level id or a branch id) if
    /// given; `what` is what they are called, if not the type's or the tag's name.
    Kill {
        target: KillTarget,
        count: u32,
        level: Option<String>,
        what: Option<String>,
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
    /// The giver, asked about the quest: what they want, as they give it.
    pub offer: String,
    /// The giver, after the offer in the same reply, if they say more.
    pub accept: Option<String>,
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
    /// How its target is drawn as it is given ([`super::quest_roll`]); `None`: it names it.
    /// For a rolled quest `goal` is a template: its target is the pool's first, and only the
    /// goal of the quest as given ([`QuestDef::rolled`]) is read.
    pub roll: Option<Rolled>,
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
                target,
                level: place,
                ..
            } => {
                target.has(content, creature)
                    && place.as_deref().is_none_or(|p| content.is_in(level, p))
            }
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

    /// What a kill quest is after.
    pub fn kill_target(&self) -> Option<&KillTarget> {
        match &self.goal {
            Goal::Kill { target, .. } => Some(target),
            _ => None,
        }
    }

    /// How many it asks for: a kill's or a bring's `count`, else 1.
    pub fn count(&self) -> u32 {
        match self.goal {
            Goal::Kill { count, .. } | Goal::Bring { count, .. } => count,
            _ => 1,
        }
    }

    /// The types carrying tag `tag` that its run can meet, by their plurals: "goblins, goblin
    /// archers, and goblin chiefs". A family (animated blades) is named once.
    pub fn members(&self, content: &Content, tag: &str) -> String {
        let plurals: Vec<&str> = (content.tagged_in(tag, self.campaign.as_deref()))
            .filter(|&c| content.family(c) == c)
            .map(|c| content.creature(c).plural.as_str())
            .collect();
        super::and_list(&plurals)
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
    pub(super) fn speaker(&self, name: &str) -> Option<DialogueId> {
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
            if quest.is_rolled() {
                super::quest_roll::check(content, &quest).map_err(bad)?;
            }
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
    mut spec: QuestSpec,
    before: &[QuestDef],
    (content, campaign): (&Content, Option<&str>),
) -> Result<QuestDef, String> {
    let owner = campaign;
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
    let variants = std::mem::take(&mut spec.variant);
    let roll = match (&spec.target, spec.pick.take(), variants.is_empty()) {
        (Some(_), None, true) => None,
        (None, Some(pick), true) => Some(Rolled::Pick(quest_roll::parse_pick(
            pick, kind, content, owner,
        )?)),
        (None, None, false) => Some(Rolled::Variants(quest_roll::parse_variants(
            variants, kind, content, owner,
        )?)),
        _ => return Err("a quest has exactly one of `target`, `pick`, and variants".into()),
    };
    // A rolled quest's goal is a template: its target is the pool's first.
    let target = match &roll {
        None => spec.target.clone().unwrap_or_default(),
        Some(roll) => (roll
            .candidates(content, owner, &|_| true)
            .into_iter()
            .next())
        .map(|c| c.target)
        .ok_or("`pick`: the pool is empty")?,
    };
    let texts = texts(&spec, roll.as_ref())?;
    let goal = match kind {
        QuestKind::Kill => {
            unused("to", spec.to.is_some())?;
            let target = match target.strip_prefix(TAG_TARGET) {
                Some(tag) => KillTarget::Tag(check_tag(content, owner, tag)?),
                None => KillTarget::Creature(
                    (content.creature_id(&target))
                        .filter(|_| target != "player")
                        .ok_or_else(|| format!("unknown creature `{target}`"))?,
                ),
            };
            let most = match target {
                KillTarget::Creature(_) => MAX_COUNT,
                KillTarget::Tag(_) => MAX_TAG_COUNT,
            };
            if let Some(what) = &spec.what {
                let n = what.chars().count();
                if n == 0 || n > MAX_TAG {
                    return Err(format!("`what` is 1 to {MAX_TAG} characters"));
                }
                check_lines([what], MAX_LINE)?;
            }
            let count = match spec.count.unwrap_or(1) {
                n @ 1.. if n <= most => n,
                _ => return Err(format!("`count` is 1 to {most}")),
            };
            if let KillTarget::Creature(c) = target
                && content.creature(c).unique
                && count != 1
            {
                return Err("a unique is killed once: `count` is 1".into());
            }
            if let Some(level) = &spec.level
                && !content.has_level(campaign, level)
                && content.dungeon(level).is_none()
            {
                return Err(format!("unknown level or dungeon branch `{level}`"));
            }
            Goal::Kill {
                target,
                count,
                level: spec.level,
                what: spec.what,
            }
        }
        QuestKind::Bring => {
            unused("what", spec.what.is_some())?;
            unused("level", spec.level.is_some())?;
            let to = spec.to.as_deref().map(speaker).transpose()?;
            Goal::Bring {
                object: object(&target)?,
                count: count(spec.count)?,
                to: to.filter(|&to| to != giver),
            }
        }
        QuestKind::Reach => {
            unused("what", spec.what.is_some())?;
            unused("level", spec.level.is_some())?;
            unused("to", spec.to.is_some())?;
            unused("count", spec.count.is_some())?;
            if roll.is_some() {
                return Err("a reach quest is not rolled".into());
            }
            if !content.has_level(campaign, &target) {
                return Err(format!("unknown level `{target}`"));
            }
            if spec.repeatable {
                return Err(
                    "a reach quest is not repeatable: a place once reached stays so".into(),
                );
            }
            Goal::Reach { level: target }
        }
        QuestKind::Deliver => {
            unused("what", spec.what.is_some())?;
            unused("level", spec.level.is_some())?;
            unused("count", spec.count.is_some())?;
            if roll.is_some() {
                return Err("a deliver quest is not rolled".into());
            }
            let to = speaker(spec.to.as_deref().ok_or("a deliver quest needs `to`")?)?;
            if to == giver {
                return Err("a delivery goes to someone other than the giver".into());
            }
            Goal::Deliver {
                object: object(&target)?,
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
    if roll.is_none() {
        let mut said = vec![&texts.offer, &texts.reminder, &texts.done];
        said.extend(texts.accept.iter());
        said.extend(spec.where_.iter());
        if said.iter().any(|t| quest_roll::has_placeholder(t)) {
            return Err(
                "placeholders (`{target}`) are for a quest with a `pick` or variants".into(),
            );
        }
        check_lines(said, MAX_REPLY)?;
    }
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
    let reveals = spec
        .reveals
        .map(|r| reveal(content, owner, r))
        .transpose()?;
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
        texts,
        requires,
        requires_alignment: spec.requires_alignment,
        alignment: spec.alignment,
        repeatable: spec.repeatable,
        campaign: None,
        reveals,
        where_: spec.where_,
        roll,
    })
}

/// The texts of quest `spec`: each of `offer`, `reminder`, and `done` it must have, unless it is
/// rolled from variants that each say their own.
fn texts(spec: &QuestSpec, roll: Option<&Rolled>) -> Result<QuestTexts, String> {
    let variants = match roll {
        Some(Rolled::Variants(v)) => &v[..],
        _ => &[],
    };
    let text =
        |own: &Option<String>, field: &str, theirs: fn(&Variant) -> &Option<String>| match own {
            Some(text) => Ok(text.clone()),
            None if !variants.is_empty() && variants.iter().all(|v| theirs(v).is_some()) => {
                Ok(String::new())
            }
            None => Err(format!("a quest needs `{field}`")),
        };
    Ok(QuestTexts {
        offer: text(&spec.offer, "offer", |v| &v.offer)?,
        accept: spec.accept.clone(),
        reminder: text(&spec.reminder, "reminder", |v| &v.reminder)?,
        done: text(&spec.done, "done", |v| &v.done)?,
    })
}

/// A `reveals` of a quest of campaign `campaign`, checked: it names a hidden location of the
/// campaign's overworld.
pub(super) fn reveal(
    content: &Content,
    campaign: Option<&str>,
    r: RevealSpec,
) -> Result<Reveal, String> {
    if content
        .hidden_location(campaign.unwrap_or_default(), &r.location)
        .is_none()
    {
        return Err(format!(
            "`reveals`: the overworld has no hidden location `{}`",
            r.location
        ));
    }
    Ok(Reveal {
        location: r.location,
        when: r.when,
    })
}

/// The tag a kill quest of campaign `owner` (`None`: outside every campaign) names, checked:
/// written as a tag, and carried by a creature type a run of the campaign can meet (a typo is
/// carried by none; another campaign's types do not count).
pub(super) fn check_tag(
    content: &Content,
    owner: Option<&str>,
    tag: &str,
) -> Result<String, String> {
    super::tag::check(tag)?;
    if content.tagged_in(tag, owner).next().is_some() {
        return Ok(tag.to_string());
    }
    match content.tagged(tag).next() {
        Some(c) => Err(format!(
            "tag `{tag}`: only creatures of campaign `{}` carry it",
            content.creature(c).campaign.as_deref().unwrap_or_default()
        )),
        None => Err(format!("tag `{tag}`: no creature carries it")),
    }
}

/// Everyone's topics fit the dialogue screen: their dialogue's own, and one per quest they give
/// or take.
fn check_topics(quests: &[QuestDef], content: &Content) -> Result<(), String> {
    for (i, dialogue) in content.dialogues().iter().enumerate() {
        let id = DialogueId(i as u16);
        let theirs = (quests.iter())
            .filter(|q| q.giver == id || q.turn_in() == id)
            .count();
        let topics = dialogue.topics.len() + theirs;
        if topics > MAX_TOPICS {
            return Err(format!(
                "`{}` has {topics} topics with its quests; at most {MAX_TOPICS} fit",
                content.speaker_name(id)
            ));
        }
    }
    Ok(())
}
