//! Rolled quests (spec §8 "Quests", docs/design/adventurers-guild.md §5, #422): a `[[quest]]`
//! whose target is drawn as the quest is given, from a **`pick`** (a pool drawn by a rule from
//! data that already exists) or a list of authored **`[[quest.variant]]`**s. The quest stays one
//! template in data; what was drawn is a [`Roll`], kept in the quest's record (saved), and
//! [`QuestDef::rolled`] makes the quest as given from the two. Texts name the roll through
//! placeholders (`{target}`, `{count}`, `{members}`, ...), filled from it.
//!
//! - `pick = { creatures = ["cealdhlaw", ...] }`: a creature type that is no unique, from the
//!   populations of those branches the player can enter (a kill quest).
//! - `pick = { tags = ["goblin", ...] }`: a tag carried by a type in the population of a branch
//!   of the quest's campaign the player can enter (a kill quest).
//! - `pick = { objects = { class = "scrolls", loot = true } }`: an object type of that item class,
//!   with `loot` only those loot can hold (a bring quest).
//! - `[[quest.variant]]` (`id`, `target`, `weight`, `reveals`, `where`, and any of the texts): an
//!   authored option, drawn by weight; what it gives replaces the quest's own.
//!
//! The pool is in content order (creature, object, and tag order; variants in file order), never
//! a hash order; the draw is the world's (`world/quest_roll.rs`).

use super::Content;
use super::creature::CreatureId;
use super::dialogue::{MAX_REPLY, check_lines, is_id};
use super::item_class::ItemClassId;
use super::object::{ObjectId, article};
use super::quest::{Goal, KillTarget, QuestDef, QuestKind, QuestTexts, Reveal, RevealSpec};
use super::tag::TAG_TARGET;
use serde::{Deserialize, Serialize};

/// The most variants a quest has.
pub const MAX_VARIANTS: usize = 32;
/// The heaviest a variant's `weight` is.
pub const MAX_WEIGHT: u32 = 1000;

/// What a rolled quest drew as it was given (saved in its record): a creature type's, a tag's
/// (`tag:goblin`), or an object type's id, the variant drawn, if any, and how many it asks for.
/// Ids, not indices, so adding content keeps saves.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Roll {
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    pub count: u32,
}

/// A pool a quest's target is drawn from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    /// The creature types that are no uniques of these branches' populations (branch ids).
    Creatures(Vec<String>),
    /// These tags.
    Tags(Vec<String>),
    /// The object types of an item class; with `loot`, those loot can hold only.
    Objects { class: ItemClassId, loot: bool },
}

/// An authored option of a rolled quest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variant {
    pub id: String,
    /// What it asks for, as a quest's `target` is written (`wystan`, `tag:goblin`, an object).
    pub target: String,
    /// How likely it is drawn, against the others' weights.
    pub weight: u32,
    /// The hidden location it reveals, instead of the quest's.
    pub reveals: Option<Reveal>,
    /// Where its goal lies, instead of the quest's `where`.
    pub where_: Option<String>,
    /// Its own words, each instead of the quest's.
    pub offer: Option<String>,
    pub accept: Option<String>,
    pub reminder: Option<String>,
    pub done: Option<String>,
}

/// How a quest's target is drawn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rolled {
    Pick(Pick),
    Variants(Vec<Variant>),
}

/// A quest's `pick` as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PickSpec {
    creatures: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    objects: Option<ObjectsSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectsSpec {
    class: String,
    #[serde(default)]
    loot: bool,
}

/// A `[[quest.variant]]` as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VariantSpec {
    id: String,
    target: String,
    weight: Option<u32>,
    reveals: Option<RevealSpec>,
    #[serde(rename = "where")]
    where_: Option<String>,
    offer: Option<String>,
    accept: Option<String>,
    reminder: Option<String>,
    done: Option<String>,
}

/// One target a roll may draw: its id as a [`Roll`] keeps it, the variant (by index) it comes
/// from, and its weight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate {
    pub target: String,
    pub variant: Option<usize>,
    pub weight: u32,
}

impl Rolled {
    /// Every target a roll may draw now, for a quest of campaign `campaign`, in content order:
    /// of the branches `open` says the player can enter, for a pick of creatures or tags. Empty
    /// when nothing is open.
    pub fn candidates(
        &self,
        content: &Content,
        campaign: Option<&str>,
        open: &dyn Fn(&str) -> bool,
    ) -> Vec<Candidate> {
        let one = |target: String| Candidate {
            target,
            variant: None,
            weight: 1,
        };
        match self {
            Rolled::Pick(Pick::Creatures(branches)) => {
                let open: Vec<&String> = branches.iter().filter(|b| open(b)).collect();
                (0..content.creature_defs().len())
                    .map(|i| CreatureId(i as u16))
                    .filter(|&c| !content.creature(c).unique)
                    .filter(|&c| open.iter().any(|b| content.lives_in(b, c)))
                    .map(|c| one(content.creature(c).id.clone()))
                    .collect()
            }
            Rolled::Pick(Pick::Tags(tags)) => {
                let open: Vec<&str> = (content.branches_of(campaign))
                    .filter(|b| open(b))
                    .collect();
                let mut tags: Vec<&String> = tags.iter().collect();
                tags.sort();
                tags.dedup();
                (tags.into_iter())
                    .filter(|tag| {
                        (content.tagged_in(tag, campaign))
                            .any(|c| open.iter().any(|b| content.lives_in(b, c)))
                    })
                    .map(|tag| one(format!("{TAG_TARGET}{tag}")))
                    .collect()
            }
            Rolled::Pick(Pick::Objects { class, loot }) => (content.object_defs().iter())
                .filter(|o| o.class == *class && (!loot || o.frequency > 0))
                .map(|o| one(o.id.clone()))
                .collect(),
            Rolled::Variants(variants) => (variants.iter().enumerate())
                .map(|(i, v)| Candidate {
                    target: v.target.clone(),
                    variant: Some(i),
                    weight: v.weight,
                })
                .collect(),
        }
    }
}

impl Content {
    /// The dungeon branches of campaign `campaign`'s runs (its own and those outside every
    /// campaign), in table order.
    fn branches_of<'a>(&'a self, campaign: Option<&'a str>) -> impl Iterator<Item = &'a str> + 'a {
        (self.dungeons())
            .filter(move |d| {
                d.campaign.is_none() || campaign.is_none() || d.campaign.as_deref() == campaign
            })
            .map(|d| d.id.as_str())
    }

    /// Whether branch `branch`'s population holds creature type `creature`.
    fn lives_in(&self, branch: &str, creature: CreatureId) -> bool {
        (self.dungeon(branch).and_then(|d| d.spawning.as_ref()))
            .is_some_and(|s| s.population.contains(&creature))
    }
}

impl QuestDef {
    /// Whether its target is drawn as it is given.
    pub fn is_rolled(&self) -> bool {
        self.roll.is_some()
    }

    /// Every target a roll of it may draw now ([`Rolled::candidates`]); empty for a quest that
    /// is not rolled.
    pub fn candidates(&self, content: &Content, open: &dyn Fn(&str) -> bool) -> Vec<Candidate> {
        (self.roll.as_ref()).map_or_else(Vec::new, |r| {
            r.candidates(content, self.campaign.as_deref(), open)
        })
    }

    /// Its variant `id`, by index.
    pub fn variant(&self, id: &str) -> Option<(usize, &Variant)> {
        match &self.roll {
            Some(Rolled::Variants(variants)) => {
                variants.iter().enumerate().find(|(_, v)| v.id == id)
            }
            _ => None,
        }
    }

    /// The quest as given with `roll`: its goal after the drawn target, a variant's texts,
    /// `reveals`, and `where` instead of its own, and every text's placeholders filled. A roll
    /// that names nothing of the content (a guard: saves are checked as they load) leaves it as
    /// it is.
    pub fn rolled(&self, content: &Content, roll: &Roll) -> QuestDef {
        let mut def = self.clone();
        let Ok(goal) = goal_for(content, self, &roll.target, roll.count) else {
            return def;
        };
        def.goal = goal;
        if let Some((_, v)) = roll.variant.as_deref().and_then(|id| self.variant(id)) {
            let pick =
                |own: &Option<String>, quest: &String| own.clone().unwrap_or_else(|| quest.clone());
            def.texts = QuestTexts {
                offer: pick(&v.offer, &self.texts.offer),
                accept: v.accept.clone().or_else(|| self.texts.accept.clone()),
                reminder: pick(&v.reminder, &self.texts.reminder),
                done: pick(&v.done, &self.texts.done),
            };
            if v.reveals.is_some() {
                def.reveals = v.reveals.clone();
            }
            if v.where_.is_some() {
                def.where_ = v.where_.clone();
            }
        }
        let words = Words::of(content, &def, &roll.target);
        let fill = |text: &str| fill(text, &words).unwrap_or_else(|_| text.to_string());
        def.where_ = def.where_.as_deref().map(fill);
        let filled = Words {
            where_: def.where_.clone().unwrap_or_default(),
            ..words
        };
        let fill = |text: &str| self::fill(text, &filled).unwrap_or_else(|_| text.to_string());
        def.texts = QuestTexts {
            offer: fill(&def.texts.offer),
            accept: def.texts.accept.as_deref().map(fill),
            reminder: fill(&def.texts.reminder),
            done: fill(&def.texts.done),
        };
        // As given, it is drawn: nothing about it is rolled any more.
        def.roll = None;
        def
    }
}

/// The goal of `def` (a template) after target `target` and count `count`.
fn goal_for(content: &Content, def: &QuestDef, target: &str, count: u32) -> Result<Goal, String> {
    Ok(match &def.goal {
        Goal::Kill { level, what, .. } => Goal::Kill {
            target: kill_target(content, target)?,
            count,
            level: level.clone(),
            what: what.clone(),
        },
        Goal::Bring { to, .. } => Goal::Bring {
            object: object(content, target)?,
            count,
            to: *to,
        },
        other => other.clone(),
    })
}

/// A kill target as a quest writes it: a creature type, or `tag:` and a tag some type carries.
fn kill_target(content: &Content, target: &str) -> Result<KillTarget, String> {
    match target.strip_prefix(TAG_TARGET) {
        Some(tag) if content.tagged(tag).next().is_some() => Ok(KillTarget::Tag(tag.into())),
        Some(tag) => Err(format!("tag `{tag}`: no creature carries it")),
        None => (content.creature_id(target))
            .filter(|_| target != "player")
            .map(KillTarget::Creature)
            .ok_or_else(|| format!("unknown creature `{target}`")),
    }
}

fn object(content: &Content, target: &str) -> Result<ObjectId, String> {
    content
        .object_id(target)
        .ok_or_else(|| format!("unknown object `{target}`"))
}

/// Whether roll `roll` names only what `content` has, for quest `def` (a guard for saves).
pub(crate) fn roll_known(content: &Content, def: &QuestDef, roll: &Roll) -> bool {
    let variant = roll
        .variant
        .as_deref()
        .is_none_or(|v| def.variant(v).is_some());
    variant && goal_for(content, def, &roll.target, roll.count).is_ok()
}

/// What a quest's placeholders say, for one roll.
#[derive(Clone, Debug, Default)]
struct Words {
    target: String,
    targets: String,
    a_target: String,
    count: String,
    members: String,
    branch: String,
    where_: String,
}

impl Words {
    /// The words of quest `def` (with its goal after the roll) for target `target`.
    fn of(content: &Content, def: &QuestDef, target: &str) -> Words {
        let mut words = Words::default();
        match &def.goal {
            Goal::Kill {
                target: kill,
                count,
                what,
                ..
            } => {
                let (one, many) = match what {
                    Some(what) => (what.clone(), what.clone()),
                    None => (kill.name(content, true), kill.name(content, false)),
                };
                let unique = matches!(kill, KillTarget::Creature(c) if content.creature(*c).unique);
                words.a_target = one_of(&one, unique);
                words.target = one;
                words.targets = many;
                words.count = count.to_string();
                if let KillTarget::Tag(tag) = kill {
                    words.members = def.members(content, tag);
                }
                words.branch = branch_of(content, def, kill).unwrap_or_default();
            }
            Goal::Bring { object, count, .. } => {
                let o = content.object(*object);
                words.target = o.name.clone();
                words.targets = o.plural.clone();
                words.a_target = o.describe(1);
                words.count = count.to_string();
            }
            _ => words.target = target.to_string(),
        }
        words.where_ = def.where_.clone().unwrap_or_default();
        words
    }

    fn get(&self, key: &str) -> Option<&str> {
        Some(match key {
            "target" => &self.target,
            "targets" => &self.targets,
            "a_target" => &self.a_target,
            "count" => &self.count,
            "members" => &self.members,
            "branch" => &self.branch,
            "where" => &self.where_,
            _ => return None,
        })
    }
}

/// The name of the first branch the kill target `kill` lives in: of the pick's, in its order,
/// for a pick of creatures; else of the quest's campaign's, in table order.
fn branch_of(content: &Content, def: &QuestDef, kill: &KillTarget) -> Option<String> {
    let lives = |b: &str| match kill {
        KillTarget::Creature(c) => content.lives_in(b, *c),
        KillTarget::Tag(tag) => {
            (content.tagged_in(tag, def.campaign.as_deref())).any(|c| content.lives_in(b, c))
        }
    };
    let found = match &def.roll {
        Some(Rolled::Pick(Pick::Creatures(branches))) => {
            branches.iter().map(String::as_str).find(|b| lives(b))
        }
        _ => content
            .branches_of(def.campaign.as_deref())
            .find(|b| lives(b)),
    };
    found.map(|b| content.place_name(b))
}

/// `text` with its placeholders filled from `words`: `{target}`, `{targets}`, `{a_target}`,
/// `{count}`, `{members}`, `{branch}`, and `{where}`, each also capitalised (`{A_target}`,
/// `{Where}`). An unknown or unclosed placeholder is an error naming it.
fn fill(text: &str, words: &Words) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let close = after
            .find('}')
            .ok_or_else(|| format!("an unclosed placeholder in \"{text}\""))?;
        let key = &after[..close];
        let lower = key.to_lowercase();
        let value = words
            .get(&lower)
            .ok_or_else(|| format!("unknown placeholder `{{{key}}}`"))?;
        if key == lower {
            out.push_str(value);
        } else {
            out.push_str(&capitalized(value));
        }
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Whether `text` has a placeholder.
pub(super) fn has_placeholder(text: &str) -> bool {
    text.contains('{') || text.contains('}')
}

/// Parses a quest's `pick` (for a quest of kind `kind` in campaign `campaign`).
pub(super) fn parse_pick(
    spec: PickSpec,
    kind: QuestKind,
    content: &Content,
    campaign: Option<&str>,
) -> Result<Pick, String> {
    let want = |k: QuestKind, what: &str| {
        if kind == k {
            Ok(())
        } else {
            Err(format!("a pick of {what} is for a {} quest", k.id()))
        }
    };
    match (spec.creatures, spec.tags, spec.objects) {
        (Some(branches), None, None) => {
            want(QuestKind::Kill, "creatures")?;
            if branches.is_empty() {
                return Err("`pick.creatures` names no branch".into());
            }
            for (i, b) in branches.iter().enumerate() {
                let d = (content.dungeon(b))
                    .ok_or_else(|| format!("`pick`: unknown dungeon branch `{b}`"))?;
                if campaign.is_some() && d.campaign.is_some() && d.campaign.as_deref() != campaign {
                    return Err(format!("`pick`: branch `{b}` is another campaign's"));
                }
                if branches[..i].contains(b) {
                    return Err(format!("`pick` names branch `{b}` twice"));
                }
            }
            Ok(Pick::Creatures(branches))
        }
        (None, Some(tags), None) => {
            want(QuestKind::Kill, "tags")?;
            if tags.is_empty() {
                return Err("`pick.tags` names no tag".into());
            }
            for (i, tag) in tags.iter().enumerate() {
                super::quest::check_tag(content, campaign, tag)?;
                if tags[..i].contains(tag) {
                    return Err(format!("`pick` names tag `{tag}` twice"));
                }
            }
            Ok(Pick::Tags(tags))
        }
        (None, None, Some(objects)) => {
            want(QuestKind::Bring, "objects")?;
            let class = (content.item_class_id(&objects.class))
                .ok_or_else(|| format!("`pick`: unknown item class `{}`", objects.class))?;
            Ok(Pick::Objects {
                class,
                loot: objects.loot,
            })
        }
        _ => Err("`pick` has exactly one of `creatures`, `tags`, and `objects`".into()),
    }
}

/// Parses a quest's variants (for a quest of kind `kind` in campaign `campaign`).
pub(super) fn parse_variants(
    specs: Vec<VariantSpec>,
    kind: QuestKind,
    content: &Content,
    campaign: Option<&str>,
) -> Result<Vec<Variant>, String> {
    if specs.len() > MAX_VARIANTS {
        return Err(format!("a quest has at most {MAX_VARIANTS} variants"));
    }
    let mut variants: Vec<Variant> = Vec::new();
    for spec in specs {
        let bad = |m: String| format!("variant `{}`: {m}", spec.id);
        if !is_id(&spec.id) {
            return Err(bad(
                "ids are lower-case letters, digits, and underscores".into()
            ));
        }
        if variants.iter().any(|v| v.id == spec.id) {
            return Err(bad("a second variant of that id".into()));
        }
        match kind {
            QuestKind::Kill => {
                if let Some(tag) = spec.target.strip_prefix(TAG_TARGET) {
                    super::quest::check_tag(content, campaign, tag).map_err(bad)?;
                } else {
                    kill_target(content, &spec.target).map_err(bad)?;
                }
            }
            QuestKind::Bring => _ = object(content, &spec.target).map_err(bad)?,
            _ => return Err(format!("a {} quest has no variants", kind.id())),
        }
        let weight = match spec.weight.unwrap_or(1) {
            w @ 1..=MAX_WEIGHT => w,
            _ => return Err(bad(format!("`weight` is 1 to {MAX_WEIGHT}"))),
        };
        let reveals = match spec.reveals {
            Some(r) => Some(super::quest::reveal(content, campaign, r).map_err(bad)?),
            None => None,
        };
        variants.push(Variant {
            id: spec.id,
            target: spec.target,
            weight,
            reveals,
            where_: spec.where_,
            offer: spec.offer,
            accept: spec.accept,
            reminder: spec.reminder,
            done: spec.done,
        });
    }
    if variants.is_empty() {
        return Err("a quest's variants are at least one".into());
    }
    Ok(variants)
}

/// The checks of a rolled quest `def`, made for every target its pool can hold (whatever the
/// player can reach): its pool is not empty, a kill of a unique asks for one, and every text,
/// filled, is one line of at most [`MAX_REPLY`] characters with no unknown placeholder.
pub(super) fn check(content: &Content, def: &QuestDef) -> Result<(), String> {
    let all = def.candidates(content, &|_| true);
    if all.is_empty() {
        return Err("`pick`: the pool is empty".into());
    }
    for c in &all {
        let variant = c.variant.and_then(|i| match &def.roll {
            Some(Rolled::Variants(v)) => Some(v[i].id.clone()),
            _ => None,
        });
        let roll = Roll {
            target: c.target.clone(),
            variant,
            count: def.count(),
        };
        check_roll(content, def, &roll).map_err(|e| format!("drawing `{}`: {e}", c.target))?;
    }
    Ok(())
}

/// Checks one roll of `def`: as [`check`] says.
fn check_roll(content: &Content, def: &QuestDef, roll: &Roll) -> Result<(), String> {
    let goal = goal_for(content, def, &roll.target, roll.count)?;
    if let Goal::Kill {
        target: KillTarget::Creature(c),
        count,
        ..
    } = goal
        && content.creature(c).unique
        && count != 1
    {
        return Err("a unique is killed once: `count` is 1".into());
    }
    // Fill as `rolled` does, but strictly, so a bad placeholder is an error.
    let given = def.rolled(content, roll);
    let mut raw = def.clone();
    raw.goal = goal;
    let variant = roll
        .variant
        .as_deref()
        .and_then(|v| def.variant(v))
        .map(|(_, v)| v);
    let texts = [
        variant
            .and_then(|v| v.offer.clone())
            .unwrap_or_else(|| def.texts.offer.clone()),
        variant
            .and_then(|v| v.reminder.clone())
            .unwrap_or_else(|| def.texts.reminder.clone()),
        variant
            .and_then(|v| v.done.clone())
            .unwrap_or_else(|| def.texts.done.clone()),
    ];
    let accept = variant
        .and_then(|v| v.accept.clone())
        .or_else(|| def.texts.accept.clone());
    let where_ = variant
        .and_then(|v| v.where_.clone())
        .or_else(|| def.where_.clone());
    raw.where_ = where_.clone();
    let words = Words::of(content, &raw, &roll.target);
    if let Some(w) = &where_ {
        if w.to_lowercase().contains("{where}") {
            return Err("`where` cannot name `{where}`".into());
        }
        fill(w, &words)?;
    }
    for text in texts.iter().chain(accept.iter()) {
        fill(text, &words)?;
    }
    let mut said: Vec<&String> = vec![&given.texts.offer, &given.texts.reminder, &given.texts.done];
    said.extend(given.texts.accept.iter());
    said.extend(given.where_.iter());
    check_lines(said, MAX_REPLY)?;
    Ok(())
}

/// One creature called `name` with its article: "a goblin chief"; a unique's ("the fire
/// drake"), or a unique's proper name alone ("Black Wystan").
pub fn one_of(name: &str, unique: bool) -> String {
    match (unique, name.chars().next()) {
        (true, Some(c)) if c.is_uppercase() => name.to_string(),
        (true, _) => format!("the {name}"),
        (false, _) => format!("{} {name}", article(name)),
    }
}

/// `text` with its first letter upper-case.
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
