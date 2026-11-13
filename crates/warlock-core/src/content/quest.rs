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
//! - `escort`: `target` a level id, `escort` a unique character (an npc id with a dialogue):
//!   chatted with while the quest is active, they fall in behind the player saying `joins`
//!   (`world/escort.rs`); the quest is done when they stand on the target level with the
//!   player: they say `thanks` (optional), and the giver, if on that level, else the escort,
//!   says `done` and pays. `waits` places the escort as the quest is given
//!   (`world/escort_waits.rs`). Their death fails it.
//! - `lure`: `target` a unique creature type, `level` the hand-made level it is lured to,
//!   `waits` the place the quest makes it on as it is given and again whenever the player comes
//!   to that level and it is nowhere (killed, it forms anew), and `[quest.binding]`: what becomes
//!   of it once it stands on `level` with the player and the giver (`world/lure.rs`).
//!
//! Asking the giver about a quest **gives** it (there is no accepting or declining, #422): the
//! giver says `offer`, then `accept` (optional) in the same reply, and it is active at once.
//! `reminder` is what they say asked again; `done` is said by whoever the quest is handed in to (the recipient of a bring with `to` or a delivery,
//! else the giver). The `reward` is `gold` (gold coins), `items` (placement strings, `"apple*2"`),
//! `xp`, and `skills` (by skill id: `start`, the dice an untrained skill is trained at, `marks`,
//! the training marks a trained one gets, and `taught`/`more`, what is said of each), each
//! optional. `requires` names a quest, or a list of them (loaded before this
//! one), that must all be done before this one is offered, and `requires_alignment` the bands the player must be in for it
//! to be offered (spec §8 "Alignment"); `min_level` the character level the player must have
//! reached for it to be given, with `early`, what the giver says asked about it before then (the
//! topic is listed; the quest is not given); `alignment` moves their score as it is completed;
//! `repeatable` quests are offered again once done (never a reach). `reveals` names a hidden
//! location of the overworld revealed as the quest is given (the default) or done (spec §7
//! "Hidden locations"), and `where` says where its goal lies after the objective.
//!
//! A kill or bring quest may draw its target as it is given instead of naming it: a `pick` or
//! `[[quest.variant]]`s ([`super::quest_roll`]), its texts naming the roll through placeholders.

use super::creature::CreatureId;
use super::dialogue::{DialogueId, ID_SHAPE, Speaker, is_id};
use super::object::{ObjectId, parse_stacks};
use super::quest_roll::{self, PickSpec, Rolled, Variant, VariantSpec};
use super::tag::{TAG_TARGET, spoken};
use super::{Content, ContentError};
use super::{FactionId, StatusId};
use crate::alignment::{MAX as ALIGNMENT_MAX, Requirement};
use crate::dice::Dice;
use crate::geom::Pos;
use crate::object::Stack;
use crate::skills::SkillId;
use serde::Deserialize;
use std::collections::BTreeMap;

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
    Escort,
    Lure,
}

impl QuestKind {
    /// Its `type` in the file.
    pub fn id(self) -> &'static str {
        match self {
            QuestKind::Kill => "kill",
            QuestKind::Bring => "bring",
            QuestKind::Reach => "reach",
            QuestKind::Deliver => "deliver",
            QuestKind::Escort => "escort",
            QuestKind::Lure => "lure",
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
    escort: Option<String>,
    joins: Option<String>,
    thanks: Option<String>,
    waits: Option<WaitsSpec>,
    binding: Option<BindingSpec>,
    #[serde(default)]
    reward: RewardSpec,
    offer: Option<String>,
    accept: Option<String>,
    reminder: Option<String>,
    done: Option<String>,
    requires: Option<Requires>,
    requires_alignment: Option<Requirement>,
    min_level: Option<u8>,
    early: Option<String>,
    #[serde(default)]
    alignment: i32,
    #[serde(default)]
    repeatable: bool,
    reveals: Option<RevealSpec>,
    #[serde(rename = "where")]
    where_: Option<String>,
}

/// A quest's `requires`: one quest id, or a list of them (`["barrow_rats", "barrow_heart"]`).
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

/// An escort quest's `waits = { creature = "cottar", level = "aelfham:3", at = "the moss bed" }`;
/// a lure quest's names no `creature` (it is the quest's `target`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WaitsSpec {
    creature: Option<String>,
    level: String,
    at: String,
}

/// A lure quest's `[quest.binding]` (`world/lure.rs`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingSpec {
    status: String,
    faction: String,
    post: [i32; 2],
    words: Vec<String>,
    shatters: Option<String>,
    reforms: Option<String>,
    boon: Option<BoonSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BoonSpec {
    skill: String,
    bonus: i32,
}

/// What becomes of a lure quest's creature once it is brought home (`world/lure.rs`): bound by
/// the giver, it carries `status` for good (a circled one: its spell circle shows it), joins
/// `faction`, walks to `post` on the quest's level and stays there, and gives its `boon`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    /// The status it carries from then on (no `eases`: it lasts until cured, and nothing cures
    /// it).
    pub status: StatusId,
    /// The faction it joins (one at peace with the player: `peaceful`).
    pub faction: FactionId,
    /// The cell of the quest's level it walks to and keeps.
    pub post: Pos,
    /// What the player is told as it is bound, a message each, before the giver's `done`.
    pub words: Vec<String>,
    /// What the player is told as it dies unbound, while the quest is active.
    pub shatters: Option<String>,
    /// What the player is told as it forms anew on its level after a death.
    pub reforms: Option<String>,
    /// What it does for the player's checks while it stands bound on their level.
    pub boon: Option<Boon>,
}

/// A bound creature's help (`world/lure.rs`): `bonus` to the player's checks of skill `skill`'s
/// recipes (Jewelcraft's Cut and Set) made on its level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Boon {
    pub skill: SkillId,
    pub bonus: i32,
}

/// Where an escort quest places its escort as it is given (`world/escort.rs`, [`Waits`]): a
/// creature of type `creature` that is the escort, on the named place `at` of level `level`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Waits {
    pub creature: CreatureId,
    pub level: String,
    /// A named place of the level (a legend's `name`: a hand-made level's, or a vault's the
    /// level stamps for certain).
    pub at: String,
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
    /// Skills taught, by skill id, in id order.
    #[serde(default)]
    skills: BTreeMap<String, SkillRewardSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillRewardSpec {
    start: Option<Dice>,
    #[serde(default)]
    marks: u32,
    taught: Option<String>,
    more: Option<String>,
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
    /// Bring `escort`, a unique character, to `level` (`world/escort.rs`): chatted with while
    /// the quest is active, they say `joins` and follow the player.
    Escort {
        escort: DialogueId,
        level: String,
        joins: String,
        /// What the escort says as they come home, before whoever thanks the player says
        /// `done` (`world/escort.rs`).
        thanks: Option<String>,
        /// Where the quest places the escort as it is given; `None`: a level places them.
        waits: Option<Waits>,
    },
    /// Lure `creature`, a unique type the quest makes on `waits`, to `level`, a hand-made level,
    /// where the giver binds it (`world/lure.rs`).
    Lure {
        creature: CreatureId,
        level: String,
        waits: Waits,
        binding: Binding,
    },
}

/// What completing a quest gives.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reward {
    /// Gold coins.
    pub gold: u32,
    pub xp: u32,
    pub items: Vec<Stack>,
    /// The skills it teaches (`world/quest.rs`): a skill the player is untrained in is trained
    /// at `start`, a trained one gets `marks`.
    pub skills: Vec<SkillReward>,
}

/// A skill a quest's reward teaches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillReward {
    pub skill: SkillId,
    /// The roll an untrained skill starts at; `None`: an untrained one is not taught.
    pub start: Option<Dice>,
    /// The training marks a trained skill gets.
    pub marks: u32,
    /// What is said as it is taught (untrained); `None`: the default words.
    pub taught: Option<String>,
    /// What is said as a trained one gets its marks; `None`: the default words.
    pub more: Option<String>,
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
    /// The character level the player must have reached for it to be given, and what the giver
    /// says, asked about it before then (`early`): the topic is listed, the quest not given.
    pub min_level: Option<(u8, String)>,
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
            Goal::Escort { .. } => QuestKind::Escort,
            Goal::Lure { .. } => QuestKind::Lure,
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

    /// The levels or branches it names: a kill's `level`, a reach's or an escort's target, a
    /// lure's level.
    pub fn places(&self) -> impl Iterator<Item = &str> {
        let place = match &self.goal {
            Goal::Kill { level, .. } => level.as_deref(),
            Goal::Reach { level } | Goal::Escort { level, .. } | Goal::Lure { level, .. } => {
                Some(level.as_str())
            }
            _ => None,
        };
        place.into_iter()
    }

    /// A lure quest's creature, its level, where it waits, and its binding (`world/lure.rs`).
    pub fn lure(&self) -> Option<(CreatureId, &str, &Waits, &Binding)> {
        match &self.goal {
            Goal::Lure {
                creature,
                level,
                waits,
                binding,
            } => Some((*creature, level.as_str(), waits, binding)),
            _ => None,
        }
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

    /// Everyone it names: the giver, whoever it is handed in to, and an escort.
    pub fn speakers(&self) -> impl Iterator<Item = DialogueId> {
        [self.giver, self.turn_in()]
            .into_iter()
            .chain(self.escort())
    }

    /// The unique character an escort quest brings home (`world/escort.rs`).
    pub fn escort(&self) -> Option<DialogueId> {
        match self.goal {
            Goal::Escort { escort, .. } => Some(escort),
            _ => None,
        }
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
                return Err(bad(ID_SHAPE.into()));
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
    // A dialogue's lines after a quest name one that exists.
    let after = (content.dialogues.iter()).flat_map(|d| d.after.iter().map(move |a| (d, a)));
    for (d, a) in after {
        if !quests.iter().any(|q| q.id == a.quest) {
            let who = match &d.speaker {
                Speaker::Creature(c) => content.creature(*c).id.clone(),
                Speaker::Npc(npc) => npc.clone(),
            };
            return Err(err(format!(
                "the dialogue of `{who}` has lines after `{}`, which is no quest",
                a.quest
            )));
        }
    }
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
    let speaker = |name: &str| {
        content.speaker(name).ok_or_else(|| {
            format!("`{name}` is no one who talks (a creature or npc with a dialogue)")
        })
    };
    let giver = speaker(&spec.giver)?;
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
    if kind != QuestKind::Escort {
        unused("escort", spec.escort.is_some())?;
        unused("joins", spec.joins.is_some())?;
        unused("thanks", spec.thanks.is_some())?;
    }
    if !matches!(kind, QuestKind::Escort | QuestKind::Lure) {
        unused("waits", spec.waits.is_some())?;
    }
    if kind != QuestKind::Lure {
        unused("binding", spec.binding.is_some())?;
    }
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
            let count = spec.count.unwrap_or(1);
            if let Some(level) = &spec.level
                && !names_level(content, campaign, level)
                && content.dungeon(level).is_none_or(|d| d.portal.is_some())
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
                count: spec.count.unwrap_or(1),
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
            // An endless branch's floor may be a target (#1119): it is forgotten on leaving, but
            // the arrival (`World::reach`) marks the quest, which is kept.
            if !names_level(content, campaign, &target) {
                return Err(format!("unknown level `{target}`"));
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
            Goal::Deliver {
                object: object(&target)?,
                to,
            }
        }
        QuestKind::Escort => {
            unused("what", spec.what.is_some())?;
            unused("level", spec.level.is_some())?;
            unused("to", spec.to.is_some())?;
            unused("count", spec.count.is_some())?;
            if roll.is_some() {
                return Err("an escort quest is not rolled".into());
            }
            if !names_level(content, campaign, &target) {
                return Err(format!("unknown level `{target}`"));
            }
            let name = spec
                .escort
                .as_deref()
                .ok_or("an escort quest needs `escort`")?;
            let escort = speaker(name)?;
            if !matches!(content.dialogue(escort).speaker, Speaker::Npc(_)) {
                return Err(format!(
                    "`escort`: `{name}` is no unique character (an npc id)"
                ));
            }
            let joins = spec.joins.clone().ok_or("an escort quest needs `joins`")?;
            let waits = (spec.waits.as_ref())
                .map(|w| {
                    let creature = (w.creature.as_deref()).ok_or("`waits` needs `creature`")?;
                    waits(content, campaign, creature, w)
                })
                .transpose()?;
            Goal::Escort {
                escort,
                level: target,
                joins,
                thanks: spec.thanks.clone(),
                waits,
            }
        }
        QuestKind::Lure => {
            unused("what", spec.what.is_some())?;
            unused("to", spec.to.is_some())?;
            unused("count", spec.count.is_some())?;
            if roll.is_some() {
                return Err("a lure quest is not rolled".into());
            }
            lure(content, campaign, &target, &spec)?
        }
    };
    let reward = &spec.reward;
    if reward.gold > 0 && content.object_id(GOLD).is_none() {
        return Err(format!("reward: gold needs the object `{GOLD}`"));
    }
    let items = parse_stacks(&reward.items, content).map_err(|e| format!("reward: {e}"))?;
    let mut skills = Vec::new();
    for (name, s) in &reward.skills {
        let skill =
            (content.skill_id(name)).ok_or_else(|| format!("reward: unknown skill `{name}`"))?;
        if s.start.is_none() && s.marks == 0 {
            return Err(format!("reward: skill `{name}` needs `start` or `marks`"));
        }
        skills.push(SkillReward {
            skill,
            start: s.start,
            marks: s.marks,
            taught: s.taught.clone(),
            more: s.more.clone(),
        });
    }
    let min_level = match (spec.min_level, &spec.early) {
        (None, None) => None,
        (Some(level), Some(early)) => Some((level, early.clone())),
        (Some(_), None) => return Err("`min_level` needs `early`, the giver's refusal".into()),
        (None, Some(_)) => return Err("`early` is for a quest with a `min_level`".into()),
    };
    // The shift is multiplied by the scale (`Score::shifted`): past it, it would overflow.
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
            skills,
        },
        texts,
        requires,
        requires_alignment: spec.requires_alignment,
        min_level,
        alignment: spec.alignment,
        repeatable: spec.repeatable,
        campaign: None,
        reveals,
        where_: spec.where_,
        roll,
    })
}

/// Whether a quest of campaign `campaign` may name level `id`: one it can reach, and no Pit's
/// floor (spec §7 "The Pits": nothing leads into a Pit but a portal).
fn names_level(content: &Content, campaign: &str, id: &str) -> bool {
    content.has_level(campaign, id) && !content.is_portal_floor(id)
}

/// A lure quest's goal, checked: `target` a unique creature type, `level` a hand-made level of
/// the campaign, `waits` (with no `creature` of its own) a place for it, and a `binding` whose
/// status lasts, whose faction is known, and whose post is a walkable cell of `level`.
fn lure(content: &Content, campaign: &str, target: &str, spec: &QuestSpec) -> Result<Goal, String> {
    let creature = (content.creature_id(target))
        .filter(|&c| target != "player" && content.creature(c).unique)
        .ok_or_else(|| format!("`target`: `{target}` is no unique creature type"))?;
    let level = spec.level.as_deref().ok_or("a lure quest needs `level`")?;
    let template = (content.level(level))
        .filter(|_| names_level(content, campaign, level))
        .ok_or_else(|| format!("`level`: `{level}` is no hand-made level of the campaign"))?;
    let w = spec.waits.as_ref().ok_or("a lure quest needs `waits`")?;
    if w.creature.is_some() {
        return Err("a lure quest's `waits` has no `creature`: it is the `target`".into());
    }
    let waits = waits(content, campaign, target, w)?;
    let b = spec
        .binding
        .as_ref()
        .ok_or("a lure quest needs `binding`")?;
    let bad = |m: String| format!("`binding`: {m}");
    let status = (content.status_id(&b.status))
        .ok_or_else(|| bad(format!("unknown status `{}`", b.status)))?;
    if content.status(status).eases.is_some() {
        return Err(bad(format!(
            "status `{}` eases away: it must last",
            b.status
        )));
    }
    let faction = (content.factions().id(&b.faction))
        .ok_or_else(|| bad(format!("unknown faction `{}`", b.faction)))?;
    let post = Pos::new(b.post[0], b.post[1]);
    let walkable = (template.terrain.get(post)).is_some_and(|&t| content.terrain(t).walkable);
    if !walkable {
        return Err(bad(format!(
            "`post` {post} is no walkable cell of `{level}`"
        )));
    }
    if b.words.is_empty() {
        return Err(bad("`words` needs at least one line".into()));
    }
    let boon = match &b.boon {
        Some(boon) => Some(Boon {
            skill: (content.skill_id(&boon.skill))
                .ok_or_else(|| bad(format!("boon: unknown skill `{}`", boon.skill)))?,
            bonus: boon.bonus,
        }),
        None => None,
    };
    Ok(Goal::Lure {
        creature,
        level: level.to_string(),
        waits,
        binding: Binding {
            status,
            faction,
            post,
            words: b.words.clone(),
            shatters: b.shatters.clone(),
            reforms: b.reforms.clone(),
            boon,
        },
    })
}

/// An escort quest's `waits` (or a lure quest's, `creature` its target), checked: a known
/// creature type, a level of the campaign, and a place of that name on it (a hand-made level's,
/// or a vault's its branch level stamps for certain), exactly one.
fn waits(
    content: &Content,
    campaign: &str,
    creature: &str,
    w: &WaitsSpec,
) -> Result<Waits, String> {
    let creature = (content.creature_id(creature))
        .filter(|_| creature != "player")
        .ok_or_else(|| format!("`waits`: unknown creature `{creature}`"))?;
    if !names_level(content, campaign, &w.level) {
        return Err(format!("`waits`: unknown level `{}`", w.level));
    }
    let named = |places: &std::collections::BTreeMap<crate::geom::Pos, super::Place>| {
        places.values().filter(|p| p.name == w.at).count()
    };
    let found = match (content.level(&w.level), content.branch_level(&w.level)) {
        (Some(level), _) => named(&level.places),
        (None, Some((dungeon, depth))) => (dungeon.vaults(depth).iter())
            .filter_map(|id| content.vaults().iter().find(|v| v.id() == id))
            .map(|v| named(&v.template.places))
            .sum(),
        (None, None) => 0,
    };
    if found != 1 {
        return Err(format!(
            "`waits`: level `{}` has {found} places named `{}`, not one",
            w.level, w.at
        ));
    }
    Ok(Waits {
        creature,
        level: w.level.clone(),
        at: w.at.clone(),
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

/// A `reveals` of a quest of campaign `campaign`, checked: it names a hidden location of one of
/// the campaign's levels (its overworld's, or a hand-made level's).
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
            "`reveals`: the campaign has no hidden location `{}`",
            r.location
        ));
    }
    Ok(Reveal {
        location: r.location,
        when: r.when,
    })
}

/// The tag a kill quest of campaign `owner` (`None`: outside every campaign) names, checked:
/// carried by a creature type a run of the campaign can meet (a typo is carried by none;
/// another campaign's types do not count).
pub(super) fn check_tag(
    content: &Content,
    owner: Option<&str>,
    tag: &str,
) -> Result<String, String> {
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
