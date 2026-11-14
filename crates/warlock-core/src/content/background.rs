//! Background stories' data (spec §8 "Background story"; docs/dev/background.md): the settings
//! in `data/background.toml` (`[tiers]`, the odds per mille of the luckier tiers, and
//! `[points]`, what each kind of effect is worth for the load-time tier check and the wiki) and
//! the chapters, one file each: `data/background/NN-<chapter>.toml`, then a campaign's own in
//! `campaigns/<id>/data/background/` (after the shared ones; a campaign never redefines a
//! shared chapter). A chapter is a `[chapter]` table (`id`, `name`, `paragraph` 1 to 4, slot
//! `defaults`) and its `[[line]]`s (`id`, `tier`, `weight`, `text`, `when`, `only`, `effects`,
//! `tags`, `set`). [`crate::background`] tells a story from them.
//!
//! Checked here: ids; every id a `when` names exists; a tag asked was set by an earlier
//! chapter's line; slots are the known ones and have a default by then; kit lines parse as kits
//! do; a stat change within ±2 a line; a line's worth within its tier's band, a rare or fateful
//! line without a bane; an `only` line common; a plain line (common, no conditions, no effects)
//! in every chapter; paragraphs non-decreasing in chapter order. Content without
//! `data/background.toml` tells no story.

use super::kit::{self, KitLineSpec};
use super::{Content, ContentError, CreatureId, KitItem, WeaponSkillId, campaign};
use crate::alignment::Requirement;
use crate::character::Gender;
use crate::dice::Dice;
use crate::magic::LootGrade;
use crate::skills::SkillId;
use crate::stats::Stat;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

/// The settings file.
pub const SETTINGS: &str = "data/background.toml";

/// How rare a line is: drawn first for every chapter ([`Tiers`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Common,
    Uncommon,
    Rare,
    Fateful,
}

impl Tier {
    /// Every tier, the commonest first.
    pub const ALL: [Tier; 4] = [Tier::Common, Tier::Uncommon, Tier::Rare, Tier::Fateful];

    pub const fn id(self) -> &'static str {
        match self {
            Tier::Common => "common",
            Tier::Uncommon => "uncommon",
            Tier::Rare => "rare",
            Tier::Fateful => "fateful",
        }
    }

    /// The tier below, the one a draw falls back to when no line of this one fits.
    pub const fn below(self) -> Option<Tier> {
        match self {
            Tier::Common => None,
            Tier::Uncommon => Some(Tier::Common),
            Tier::Rare => Some(Tier::Uncommon),
            Tier::Fateful => Some(Tier::Rare),
        }
    }

    /// The net worth in points a line of this tier has (design §7), and whether it may cost
    /// anything (a bane).
    pub const fn band(self) -> (i32, i32, bool) {
        match self {
            Tier::Common => (-4, 2, true),
            Tier::Uncommon => (1, 3, true),
            Tier::Rare => (3, 5, false),
            Tier::Fateful => (5, 8, false),
        }
    }
}

/// The odds of the luckier tiers, per mille, for every chapter; common takes the rest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tiers {
    pub uncommon: u32,
    pub rare: u32,
    pub fateful: u32,
}

impl Tiers {
    /// The tier a draw of `roll` (0 to 999) gives.
    pub fn of(&self, roll: u32) -> Tier {
        let fateful = self.fateful;
        let rare = fateful + self.rare;
        let uncommon = rare + self.uncommon;
        match roll {
            r if r < fateful => Tier::Fateful,
            r if r < rare => Tier::Rare,
            r if r < uncommon => Tier::Uncommon,
            _ => Tier::Common,
        }
    }

    /// The odds of `tier`, per mille, as written.
    pub fn per_mille(&self, tier: Tier) -> u32 {
        match tier {
            Tier::Common => 1000 - self.uncommon - self.rare - self.fateful,
            Tier::Uncommon => self.uncommon,
            Tier::Rare => self.rare,
            Tier::Fateful => self.fateful,
        }
    }
}

/// What each kind of effect is worth, in points: for the tier check of every line at load and
/// for the wiki. Nothing counts a story's total.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Points {
    /// Per point of score, either way.
    pub stat: i32,
    /// A skill start or raise, whatever the dice.
    pub skill: i32,
    /// Marks in a weapon skill, whatever the count.
    pub weapon_marks: i32,
    /// One point per this many coins of the dice's mean, rounded up (a bane when taken).
    pub gold_per: i32,
    /// A plain item.
    pub item: i32,
    /// An item with `magic = "any"`, `"magic"`, `"rare"`.
    pub magic_any: i32,
    pub magic: i32,
    pub magic_rare: i32,
    /// A talent granted, a flaw granted (B5).
    pub talent: i32,
    pub flaw: i32,
    /// An alignment nudge.
    pub alignment: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsFile {
    tiers: Tiers,
    points: Points,
}

/// The word slots a line's `set` may fill and a later text may use.
pub const SET_SLOTS: [&str; 3] = ["guardian", "home", "trade"];

/// Every slot a text may use: the name, the gender's pronouns, and [`SET_SLOTS`]; each also
/// capitalised for a sentence start (`{They}`).
pub const SLOTS: [&str; 7] = ["name", "they", "them", "their", "guardian", "home", "trade"];

/// The slots `text` uses, as written (`They`, `guardian`); an error for an unknown one or a
/// brace left open.
pub fn slots_of(text: &str) -> Result<Vec<&str>, String> {
    let mut used = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let close = after
            .find('}')
            .ok_or_else(|| "a `{` without its `}`".to_string())?;
        let slot = &after[..close];
        if !SLOTS.contains(&slot.to_lowercase().as_str()) {
            return Err(format!(
                "unknown slot `{{{slot}}}` (known: {})",
                SLOTS.join(", ")
            ));
        }
        used.push(slot);
        rest = &after[close + 1..];
    }
    if rest.contains('}') {
        return Err("a `}` without its `{`".into());
    }
    Ok(used)
}

/// An age range at the start of the run, in whole years.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeRange {
    #[serde(default)]
    pub min: Option<u32>,
    #[serde(default)]
    pub max: Option<u32>,
}

impl AgeRange {
    pub fn holds(&self, age: u32) -> bool {
        self.min.is_none_or(|m| age >= m) && self.max.is_none_or(|m| age <= m)
    }
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct WhenSpec {
    race: Vec<String>,
    class: Vec<String>,
    gender: Vec<Gender>,
    animal: Vec<String>,
    element: Vec<String>,
    lucky: Option<bool>,
    band: Option<Requirement>,
    talents: Vec<String>,
    not_talents: Vec<String>,
    skills: Vec<String>,
    tags: Vec<String>,
    not_tags: Vec<String>,
    age: Option<AgeRange>,
    campaign: Vec<String>,
    companion: Vec<String>,
    familiar: Vec<String>,
}

/// What must hold of the witness for a whole chapter to be told (`[chapter] when`): a chapter
/// whose `when` does not fit is skipped before its stream is drawn.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct ChapterWhen {
    /// With a kept beast, a companion or a familiar, as the run begins (`true`), or without one
    /// (`false`).
    pub companion: Option<bool>,
}

impl ChapterWhen {
    /// Whether the chapter is told to a witness with (`has`) a kept beast or without.
    pub fn fits(&self, has_kept: bool) -> bool {
        self.companion.is_none_or(|c| c == has_kept)
    }
}

/// What must hold for a line to be told: every field given (a list: any of its ids; `talents`,
/// `skills`, and `tags`: every one). [`crate::background::Witness`] answers it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct When {
    pub race: Vec<String>,
    pub class: Vec<String>,
    pub gender: Vec<Gender>,
    pub animal: Vec<String>,
    pub element: Vec<String>,
    /// Born on a lucky day (`true`), or not (`false`).
    pub lucky: Option<bool>,
    /// The starting band, before the story's own nudges.
    pub band: Option<Requirement>,
    /// Talents held (every one), by id: the creation picks, the sign's gifts, the class's level
    /// 1 grants.
    pub talents: Vec<String>,
    pub not_talents: Vec<String>,
    /// Skills trained (every one).
    pub skills: Vec<SkillId>,
    /// Tags earlier lines set (every one), and tags none set.
    pub tags: Vec<String>,
    pub not_tags: Vec<String>,
    /// The age at the start of the run.
    pub age: Option<AgeRange>,
    /// The run's campaign (any of them).
    pub campaign: Vec<String>,
    /// A companion of one of these types as the run begins (any of them; spec §8 "Allies").
    pub companion: Vec<CreatureId>,
    /// A familiar of one of these types as the run begins (any of them; the witch's).
    pub familiar: Vec<CreatureId>,
}

impl When {
    /// Whether it asks for nothing.
    pub fn is_empty(&self) -> bool {
        *self == When::default()
    }
}

/// Which weapon skill a line's marks go to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marks {
    Skill(WeaponSkillId),
    /// The weapon skill (not a style) held at the highest rank, ties by data order; nothing
    /// when none has a rank.
    Best,
}

/// One effect of a line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// The score and its potential, together (`Stats::gift`).
    Stat(Stat, i32),
    /// Gold coins given (or, negative, taken; never below none).
    Gold(Dice),
    /// An item as a kit line gives it; a magic one is identified.
    Item(KitItem),
    /// A skill started (or raised) by the dice.
    Skill(SkillId, Dice),
    /// Marks toward the next rank, never a rank.
    WeaponMarks(Marks, u32),
    /// The starting alignment score moved.
    Alignment(i32),
    /// A talent (by id, with its choice) given at rank 1 with what it hands out and kept among
    /// the boons: never a granted, class, or flaw talent.
    Talent(String, Option<String>),
    /// A birth sign's flaw talent (by id), given likewise and kept among the boons.
    Flaw(String),
}

impl Effect {
    /// What it is worth by `points`: boons and banes, each at least 0.
    pub fn worth(&self, points: &Points) -> (i32, i32) {
        let signed = |n: i32| if n >= 0 { (n, 0) } else { (0, -n) };
        match self {
            Effect::Stat(_, by) => signed(by * points.stat),
            Effect::Gold(dice) => {
                // The mean is half of `min + max`: a point per `gold_per` coins of it, rounded
                // up.
                let twice = dice.min() + dice.max();
                let per = 2 * points.gold_per.max(1);
                let worth = (twice.abs() + per - 1) / per;
                signed(worth * twice.signum())
            }
            Effect::Item(item) => (
                match item.magic {
                    None => points.item,
                    Some(LootGrade::Any) => points.magic_any,
                    Some(LootGrade::Magic) => points.magic,
                    Some(LootGrade::Rare) => points.magic_rare,
                },
                0,
            ),
            Effect::Skill(..) => (points.skill, 0),
            Effect::WeaponMarks(..) => (points.weapon_marks, 0),
            Effect::Alignment(_) => (points.alignment, 0),
            Effect::Talent(..) => (points.talent, 0),
            Effect::Flaw(_) => (0, points.flaw),
        }
    }
}

/// One effect as written: a table naming one kind.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectSpec {
    stat: Option<BTreeMap<String, i32>>,
    gold: Option<Dice>,
    item: Option<KitLineSpec>,
    skill: Option<BTreeMap<String, Dice>>,
    weapon_marks: Option<BTreeMap<String, u32>>,
    alignment: Option<i32>,
    talent: Option<String>,
    flaw: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LineSpec {
    id: String,
    tier: Tier,
    weight: u32,
    text: String,
    #[serde(default)]
    when: WhenSpec,
    #[serde(default)]
    only: bool,
    #[serde(default)]
    effects: Vec<EffectSpec>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    set: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChapterHead {
    id: String,
    name: String,
    paragraph: u8,
    #[serde(default)]
    defaults: BTreeMap<String, String>,
    #[serde(default)]
    when: ChapterWhen,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChapterFile {
    chapter: ChapterHead,
    #[serde(default)]
    line: Vec<LineSpec>,
}

/// A line of a chapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// Unique within its chapter; saves name it (`chapter/line`), so it never changes.
    pub id: String,
    pub tier: Tier,
    /// Among the fitting lines of its tier, 1 to 20.
    pub weight: u32,
    /// The sentence, with slots; second person, past tense.
    pub text: String,
    pub when: When,
    /// When it fits, the chapter's pool is the fitting `only` lines alone.
    pub only: bool,
    /// In order; their dice are rolled in this order.
    pub effects: Vec<Effect>,
    pub tags: Vec<String>,
    /// Slots it fills for later texts (and its own), in slot order.
    pub set: BTreeMap<String, String>,
}

impl Line {
    /// Whether it is a plain line: common, asking nothing, giving nothing.
    pub fn is_plain(&self) -> bool {
        self.tier == Tier::Common && !self.only && self.when.is_empty() && self.effects.is_empty()
    }

    /// Its boons and banes in points.
    pub fn worth(&self, points: &Points) -> (i32, i32) {
        (self.effects.iter()).fold((0, 0), |(b, c), e| {
            let (boon, bane) = e.worth(points);
            (b + boon, c + bane)
        })
    }
}

/// A chapter of a story.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chapter {
    pub id: String,
    pub name: String,
    /// The paragraph of the story it is told in, 1 to 4.
    pub paragraph: u8,
    /// Slot defaults for texts of this chapter and later ones that use a slot nothing set.
    pub defaults: BTreeMap<String, String>,
    /// The campaign it belongs to (`None`: shared, told in every run).
    pub campaign: Option<String>,
    /// What must hold of the witness for the chapter to be told at all.
    pub when: ChapterWhen,
    pub lines: Vec<Line>,
}

impl Chapter {
    pub fn line(&self, id: &str) -> Option<&Line> {
        self.lines.iter().find(|l| l.id == id)
    }
}

/// The background stories' data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Background {
    pub tiers: Tiers,
    pub points: Points,
    /// The shared chapters in file order, then each campaign's.
    pub chapters: Vec<Chapter>,
}

impl Background {
    /// The chapters of a story told in `campaign`, in order: the shared ones, then its own.
    pub fn chapters_for<'a>(&'a self, campaign: &'a str) -> impl Iterator<Item = &'a Chapter> {
        (self.chapters.iter()).filter(move |c| c.campaign.as_deref().is_none_or(|o| o == campaign))
    }

    /// Chapter `id` of a story told in `campaign`.
    pub fn chapter<'a>(&'a self, campaign: &'a str, id: &str) -> Option<&'a Chapter> {
        self.chapters_for(campaign).find(|c| c.id == id)
    }
}

impl Content {
    /// The background stories' data (`data/background.toml`, `data/background/`); `None` without
    /// the settings file: no story is told.
    pub fn background(&self) -> Option<&Background> {
        self.background.as_ref()
    }

    /// Loads the background stories once races, classes, skills, weapon skills, talents,
    /// objects, slots, the horoscope, and the campaigns are known.
    pub(super) fn load_background(
        &mut self,
        files: &super::ContentFiles,
    ) -> Result<(), ContentError> {
        let Some(text) = files.get_optional(SETTINGS) else {
            return Ok(());
        };
        let settings: SettingsFile =
            toml::from_str(text).map_err(|e| ContentError::new(SETTINGS, e.to_string()))?;
        let total = settings.tiers.uncommon + settings.tiers.rare + settings.tiers.fateful;
        // The tiers' draw is out of 1000, the common tier the rest.
        if total > 1000 {
            return Err(ContentError::new(
                SETTINGS,
                format!("`[tiers]` add up to {total} per mille, more than 1000"),
            ));
        }
        let mut chapters = Vec::new();
        for (path, text) in files.table("background").filter(|(p, _)| *p != SETTINGS) {
            let file: ChapterFile =
                toml::from_str(text).map_err(|e| ContentError::new(path, e.to_string()))?;
            let owner = campaign::of_path(path).map(str::to_string);
            let chapter = self
                .story_chapter(file, owner)
                .map_err(|m| ContentError::new(path, m))?;
            chapters.push((path, chapter));
        }
        check_ids(&chapters)?;
        self.background = Some(Background {
            tiers: settings.tiers,
            points: settings.points,
            chapters: chapters.into_iter().map(|(_, c)| c).collect(),
        });
        Ok(())
    }

    /// One chapter resolved: its lines' ids against the content.
    fn story_chapter(
        &self,
        file: ChapterFile,
        campaign: Option<String>,
    ) -> Result<Chapter, String> {
        let head = file.chapter;
        if let Some(slot) = head
            .defaults
            .keys()
            .find(|k| !SET_SLOTS.contains(&k.as_str()))
        {
            return Err(format!(
                "chapter `{}`: `defaults`: `{slot}` is not a slot a line sets ({})",
                head.id,
                SET_SLOTS.join(", ")
            ));
        }
        let mut lines: Vec<Line> = Vec::new();
        for spec in file.line {
            let at = |m: String| format!("chapter `{}`, line `{}`: {m}", head.id, spec.id);
            if lines.iter().any(|l| l.id == spec.id) {
                return Err(at("the id is used twice in the chapter".into()));
            }
            let line = self.story_line(spec, &head.id).map_err(|m| m.to_string());
            lines.push(line?);
        }
        // A draw sums the weights of lines of the chapter.
        if crate::rng::weight_sum(lines.iter().map(|l| l.weight)).is_none() {
            let m = crate::rng::WEIGHTS_OVERFLOW;
            return Err(format!("chapter `{}`: {m}", head.id));
        }
        Ok(Chapter {
            id: head.id,
            name: head.name,
            paragraph: head.paragraph,
            defaults: head.defaults,
            campaign,
            when: head.when,
            lines,
        })
    }

    /// One line resolved.
    fn story_line(&self, spec: LineSpec, chapter: &str) -> Result<Line, String> {
        let at = |m: String| format!("chapter `{chapter}`, line `{}`: {m}", spec.id);
        slots_of(&spec.text).map_err(at)?;
        if let Some(slot) = spec.set.keys().find(|k| !SET_SLOTS.contains(&k.as_str())) {
            return Err(at(format!(
                "`set`: `{slot}` is not a slot a line sets ({})",
                SET_SLOTS.join(", ")
            )));
        }
        let when = self.story_when(spec.when).map_err(at)?;
        let mut effects = Vec::new();
        for effect in spec.effects {
            effects.extend(self.story_effect(effect).map_err(at)?);
        }
        Ok(Line {
            id: spec.id,
            tier: spec.tier,
            weight: spec.weight,
            text: spec.text,
            when,
            only: spec.only,
            effects,
            tags: spec.tags,
            set: spec.set,
        })
    }

    /// A line's `when`, its ids checked.
    fn story_when(&self, spec: WhenSpec) -> Result<When, String> {
        for race in &spec.race {
            self.race(race)
                .ok_or_else(|| format!("`when.race`: unknown race `{race}`"))?;
        }
        for class in &spec.class {
            self.class(class)
                .ok_or_else(|| format!("`when.class`: unknown class `{class}`"))?;
        }
        let parts = |field: &str, ids: &[String], elements: bool| -> Result<(), String> {
            let Some(h) = self.horoscope() else {
                return match ids.first() {
                    Some(id) => Err(format!("`when.{field}`: `{id}`, but there is no horoscope")),
                    None => Ok(()),
                };
            };
            let known = if elements { &h.elements } else { &h.animals };
            match ids.iter().find(|id| !known.iter().any(|p| &p.id == *id)) {
                Some(id) => Err(format!("`when.{field}`: unknown `{id}`")),
                None => Ok(()),
            }
        };
        parts("animal", &spec.animal, false)?;
        parts("element", &spec.element, true)?;
        for talent in spec.talents.iter().chain(&spec.not_talents) {
            self.talents()
                .id(talent)
                .ok_or_else(|| format!("`when`: unknown talent `{talent}`"))?;
        }
        let skills = (spec.skills.iter())
            .map(|s| {
                self.skill_id(s)
                    .ok_or_else(|| format!("`when.skills`: unknown skill `{s}`"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if spec.band.is_some() && self.alignment().is_none() {
            return Err("`when.band`, but there is no alignment".into());
        }
        for c in &spec.campaign {
            self.campaign(c)
                .ok_or_else(|| format!("`when.campaign`: unknown campaign `{c}`"))?;
        }
        let companion = (spec.companion.iter())
            .map(|c| {
                (self.creature_id(c))
                    .ok_or_else(|| format!("`when.companion`: unknown creature `{c}`"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let familiar = (spec.familiar.iter())
            .map(|c| {
                (self.creature_id(c))
                    .filter(|_| c != "player")
                    .ok_or_else(|| format!("`when.familiar`: unknown creature `{c}`"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(When {
            race: spec.race,
            class: spec.class,
            gender: spec.gender,
            animal: spec.animal,
            element: spec.element,
            lucky: spec.lucky,
            band: spec.band,
            talents: spec.talents,
            not_talents: spec.not_talents,
            skills,
            tags: spec.tags,
            not_tags: spec.not_tags,
            age: spec.age,
            campaign: spec.campaign,
            companion,
            familiar,
        })
    }

    /// One effect as written: one kind, perhaps several stats or skills (each an effect, in
    /// the order written).
    fn story_effect(&self, spec: EffectSpec) -> Result<Vec<Effect>, String> {
        let kinds = [
            spec.stat.is_some(),
            spec.gold.is_some(),
            spec.item.is_some(),
            spec.skill.is_some(),
            spec.weapon_marks.is_some(),
            spec.alignment.is_some(),
            spec.talent.is_some(),
            spec.flaw.is_some(),
        ];
        if kinds.iter().filter(|&&k| k).count() != 1 {
            return Err(
                "an effect names one kind: `stat`, `gold`, `item`, `skill`, \
                        `weapon_marks`, `alignment`, `talent`, or `flaw`"
                    .into(),
            );
        }
        if let Some(stats) = spec.stat {
            return (stats.into_iter())
                .map(|(name, by)| {
                    let stat = (Stat::ALL.into_iter().find(|s| s.id() == name))
                        .ok_or_else(|| format!("`stat`: unknown stat `{name}`"))?;
                    Ok(Effect::Stat(stat, by))
                })
                .collect();
        }
        if let Some(gold) = spec.gold {
            return Ok(vec![Effect::Gold(gold)]);
        }
        if let Some(line) = spec.item {
            let kit = kit::resolve(std::slice::from_ref(&line), self)
                .map_err(|m| format!("`item`: {m}"))?;
            return Ok(kit.0.into_iter().map(Effect::Item).collect());
        }
        if let Some(skills) = spec.skill {
            return (skills.into_iter())
                .map(|(name, dice)| {
                    let id = (self.skill_id(&name))
                        .ok_or_else(|| format!("`skill`: unknown skill `{name}`"))?;
                    Ok(Effect::Skill(id, dice))
                })
                .collect();
        }
        if let Some(marks) = spec.weapon_marks {
            return (marks.into_iter())
                .map(|(name, count)| {
                    let to = match name.as_str() {
                        "best" => Marks::Best,
                        _ => Marks::Skill((self.weapon_skills().id(&name)).ok_or_else(|| {
                            format!("`weapon_marks`: unknown weapon skill `{name}`")
                        })?),
                    };
                    Ok(Effect::WeaponMarks(to, count))
                })
                .collect();
        }
        if let Some(by) = spec.alignment {
            return Ok(vec![Effect::Alignment(by)]);
        }
        if let Some(text) = spec.talent {
            let (id, choice) = match text.split_once(':') {
                Some((id, choice)) => (id, Some(choice.to_string())),
                None => (text.as_str(), None),
            };
            let book = self.talents();
            let def = (book.id(id))
                .map(|t| book.get(t))
                .ok_or_else(|| format!("`talent`: unknown talent `{id}`"))?;
            // The choice keys the talent held.
            match (def.choice, &choice) {
                (None, None) => {}
                (None, Some(c)) => {
                    return Err(format!("`talent`: `{id}` takes no choice (`{c}` given)"));
                }
                (Some(kind), None) => {
                    return Err(format!("`talent`: `{id}` is chosen for {}", kind.words()));
                }
                (Some(kind), Some(c)) if !kind.options(self).contains(c) => {
                    return Err(format!("`talent`: `{c}` is no choice of `{id}`"));
                }
                (Some(_), Some(_)) => {}
            }
            return Ok(vec![Effect::Talent(id.to_string(), choice)]);
        }
        let id = spec.flaw.expect("one kind is named");
        if self.talents().id(&id).is_none() {
            return Err(format!("`flaw`: unknown talent `{id}`"));
        }
        Ok(vec![Effect::Flaw(id)])
    }
}

/// Checks the chapter ids across files: a story's chapters are found by id
/// ([`Background::chapter`]), so a campaign's never takes a shared one's, nor any its own twice.
fn check_ids(chapters: &[(&str, Chapter)]) -> Result<(), ContentError> {
    let shared: BTreeSet<&str> = (chapters.iter())
        .filter(|(_, c)| c.campaign.is_none())
        .map(|(_, c)| c.id.as_str())
        .collect();
    let mut seen: BTreeSet<(Option<&str>, &str)> = BTreeSet::new();
    for (path, chapter) in chapters {
        let taken = chapter.campaign.is_some() && shared.contains(chapter.id.as_str());
        if taken || !seen.insert((chapter.campaign.as_deref(), &chapter.id)) {
            return Err(ContentError::new(
                path,
                format!("chapter `{}`: the id is used twice", chapter.id),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "background_tests.rs"]
mod tests;
