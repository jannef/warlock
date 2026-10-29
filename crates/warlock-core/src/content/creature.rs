//! Creature definitions (`assets/data/creatures/`, one file per creature; docs/dev/content.md).

use super::DamageTypeId;
use super::anim::GlyphAnim;
use super::animated::AnimatedDef;
use super::damage_type::TypeNames;
use super::disease::DiseaseId;
use super::faction::FactionId;
use super::gear::{Gear, GearLineSpec, GearSpec};
use super::light::LightId;
use super::object::MAX_BONUS;
use super::particles::ParticleEmitter;
use super::status::{OnHit, OnHitSpec, StatusId};
use super::summon::{Summon, SummonSpec};
use super::throw::{Throw, ThrowSpec};
use crate::alignment::Alignment;
use crate::color::Rgb;
use crate::combat::{Attack, CRITS, DEFAULT_CRIT, DEFAULT_THREAT, THREATS};
use crate::damage::Kinds;
use crate::dice::Dice;
use crate::skills::Skills;
use crate::stats::{Stat, Stats};
use crate::weapon_skills::WeaponSkills;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// The speed of a normal creature, and of the player (spec §8 "Speed").
pub const NORMAL_SPEED: u32 = 1000;

/// Whether a creature type's `name` is a proper name, said without an article: one that starts
/// upper-case ("Black Wystan", a named monster's), where every other is lower-case ("kobold").
pub fn is_proper(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
}

/// A creature called `name` as the player hears of one they know: "the kobold", or a proper
/// name alone ("Black Wystan").
pub fn the(name: &str) -> String {
    if is_proper(name) {
        name.to_string()
    } else {
        format!("the {name}")
    }
}

/// [`the`] at the start of a sentence: "The kobold", "Black Wystan".
pub fn the_start(name: &str) -> String {
    if is_proper(name) {
        name.to_string()
    } else {
        format!("The {name}")
    }
}
/// The speeds a creature type may have: a tenth of normal to ten times normal.
pub const SPEEDS: RangeInclusive<u32> = 100..=10_000;
/// How far a creature sees unless its type says otherwise, in cells (spec §5 "Creature sight").
pub const DEFAULT_SIGHT: i32 = 10;

fn normal_speed() -> u32 {
    NORMAL_SPEED
}

fn full_percent() -> u32 {
    100
}

fn first_level() -> u8 {
    1
}

fn default_sight() -> i32 {
    DEFAULT_SIGHT
}

fn normal_vision() -> String {
    "normal".into()
}

fn yes() -> bool {
    true
}

/// Index of a creature definition in [`super::Content::creatures`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CreatureId(pub u16);

/// How a creature fights (spec §8 "Creature behavior"): up close, shooting with the missile
/// weapon it has ready, or casting (spells are intents only: nothing is damaged).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Combat {
    /// Closes in and attacks from an adjacent cell.
    #[default]
    Melee,
    /// Shoots from up to its `range` away, with a clear line of fire.
    Ranged,
    /// Casts spells from up to its `range` away, with a clear line of fire.
    Caster,
}

/// Whether the player may swap places with a creature of a type that is not hostile to them
/// (spec §8 "Displacement", #147): moving into it swaps places unless it refuses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Budge {
    /// It gives way.
    #[default]
    Always,
    /// It keeps its post: while it goes about its routine (its mind is calm) it does not give
    /// way, as a shopkeeper keeps to the door of its shop; once it has left its post it does.
    OffPost,
    /// It never gives way: rooted to the spot.
    Never,
}

/// A creature type as written in content files.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CreatureSpec {
    pub id: String,
    pub name: String,
    /// Its name for more than one ("giant rats"); none: the name plus "s".
    pub plural: Option<String>,
    pub glyph: char,
    pub fg: Rgb,
    #[serde(default = "normal_speed")]
    pub speed: u32,
    /// Base weight in grams, without anything carried.
    pub weight: u32,
    /// Its size (`data/sizes.toml`); none: medium.
    pub size: Option<String>,
    /// The faction it belongs to (`data/factions.toml`).
    pub faction: Option<String>,
    /// Instead of `faction`: the factions one of its creatures may join, by weight
    /// (`{ seelie = 50, unseelie = 50 }`), each creature drawing its own when it is made (spec
    /// §7 "Factions": the fey courts).
    #[serde(default)]
    pub factions: BTreeMap<String, u32>,
    /// Its tags (spec §8 "Quests"): plain singular words a quest or a rule may ask for
    /// (`["goblin", "archer"]`); nothing declares them.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Where it stands between good and evil (spec §8 "Alignment"); neutral by default.
    #[serde(default)]
    pub alignment: Alignment,
    /// The race it is one of (`data/races.toml`), if any.
    pub race: Option<String>,
    /// Its eyes: a vision profile in `data/vision.toml`.
    #[serde(default = "normal_vision")]
    pub vision: String,
    /// How far it sees, in cells.
    #[serde(default = "default_sight")]
    pub sight: i32,
    #[serde(default)]
    pub combat: Combat,
    /// How far a `ranged` or `caster` creature reaches, in cells.
    pub range: Option<i32>,
    /// Whether it opens closed doors in its way.
    #[serde(default)]
    pub opens_doors: bool,
    /// Whether it lurks: waits where it stands for a hostile it sees to come beside it.
    #[serde(default)]
    pub lurks: bool,
    /// Whether it lets the player swap places with it.
    #[serde(default)]
    pub budge: Budge,
    /// What its attacks cost, in energy; none: the attack's cost in `data/actions.toml`.
    pub attack_cost: Option<u32>,
    /// Light preset the creature carries.
    pub light: Option<String>,
    /// How loud its steps are (spec §5 "Hearing"); none: it moves silently.
    #[serde(default)]
    pub footsteps: u16,
    /// The faintest loudness it hears; none: it is deaf.
    pub hearing: Option<u16>,
    /// Its scores by stat (spec §8 "Stats"); a missing one is average.
    #[serde(default)]
    pub stats: BTreeMap<Stat, u8>,
    /// Its potentials by stat; a missing one equals the score.
    #[serde(default)]
    pub potential: BTreeMap<Stat, u8>,
    /// Hit points beyond its Toughness (spec §8 "Combat").
    #[serde(default)]
    pub base_hp: u16,
    /// Mana beyond its mental stats' modifiers (spec §8 "Mana"); casters, later.
    #[serde(default)]
    pub mana: u16,
    /// Its level (spec §8 "Experience"), 1 unless it says otherwise.
    #[serde(default = "first_level")]
    pub level: u8,
    /// The experience a kill of it is worth at its own level; none: nothing.
    #[serde(default)]
    pub xp: u32,
    /// A hand-set modifier of `xp`, in percent (owner 2026-10-27): any whole number from 0 up;
    /// none: 100.
    #[serde(default = "full_percent")]
    pub xp_percent: u32,
    /// Whether at most one of it comes in a run (spec §7 "Danger levels"): a branch's
    /// population draws it once, never again.
    #[serde(default)]
    pub unique: bool,
    /// Whether it may come as a stray to a branch whose population does not name it (spec §7
    /// "Danger levels"); `false` for a type only ever placed by hand or by its own rules.
    #[serde(default = "yes")]
    pub stray: bool,
    /// How often a population or a spawn draws it (`data/creature_rarities.toml`, spec §7
    /// "Danger levels"); none: common.
    pub rarity: Option<String>,
    /// Its trained skills by skill id, with their values (spec §8 "Skills"); the rest untrained.
    #[serde(default)]
    pub skills: BTreeMap<String, u16>,
    /// The chance in percent that it drops a magic or rare item where it dies (spec §8 "Magic
    /// items"); default 0.
    #[serde(default)]
    pub loot: u8,
    /// Its natural attack, used when it wields no weapon.
    pub attack: Option<NaturalAttackSpec>,
    /// Its throw (spec §8 "Ranged combat": "Creatures' throws"): a giant's boulders.
    pub throw: Option<ThrowSpec>,
    /// Its summoning (spec §8 "Summoning"): the creatures it calls forth.
    pub summon: Option<SummonSpec>,
    /// Its natural armour: hide, shell, or scales.
    pub natural: Option<Hide>,
    /// Whether it passes through solid terrain, and its Defence inside it (spec §8
    /// "Incorporeal creatures").
    pub incorporeal: Option<Incorporeal>,
    /// Whether it has vitals to find (spec §8 "Surprise and sneak attacks"); `vitals = false`:
    /// immune to criticals and sneak attacks, as the incorporeal, the animated, and the types
    /// `surprise.toml`'s `no_vitals` tags name are by rule.
    #[serde(default = "yes")]
    pub vitals: bool,
    /// What it resists, in percent per damage type (spec §8 "Damage types"): `resist = { acid
    /// = 50 }`; 100 is immune, negative a vulnerability. Rules only: never shown.
    #[serde(default)]
    pub resist: BTreeMap<String, i32>,
    /// Whether it leaves a corpse when it dies (spec §8 "Hunger"; with `data/hunger.toml`).
    #[serde(default = "yes")]
    pub corpse: bool,
    /// What eating its corpse does, beyond feeding the eater.
    pub on_eat: Option<OnEat>,
    /// The colour of its blood (`"#rrggbb"`), or `"none"` for a bloodless creature; none:
    /// dark red ([`crate::blood::RED`]).
    pub blood: Option<String>,
    /// The status effects it never suffers, by id (`immune = ["poisoned", "bleeding"]`;
    /// `diseased` is every disease): spec §8 "Status effects".
    #[serde(default)]
    pub immune: Vec<String>,
    /// The verb of its death line ("The animated chair splinters."); none: "dies".
    pub dies: Option<String>,
    /// Its ranks in weapon skills (spec §8 "Weapon skills"), by skill id.
    #[serde(default)]
    pub weapon_skills: BTreeMap<String, u8>,
    /// The weapon its kind favours (spec §8 "Creature gear"): it carries one by a high chance.
    pub racial_weapon: Option<String>,
    /// What it may spawn with, line by line (spec §8 "Creature gear").
    #[serde(default)]
    pub gear: Vec<GearLineSpec>,
    pub anim: Option<GlyphAnim>,
    #[serde(default)]
    pub glow: f32,
    #[serde(default)]
    pub particles: Vec<ParticleEmitter>,
    /// What makes a generated type an animated one (`content/animated.rs`); never written.
    #[serde(skip)]
    pub animated: Option<AnimatedDef>,
    /// What makes a generated type a mimic (`content/mimic.rs`); never written.
    #[serde(skip)]
    pub mimic: Option<super::mimic::MimicDef>,
}

/// What eating a creature's fresh corpse does (spec §8 "Hunger"), beyond feeding the eater by
/// its size: `on_eat = { message = "...", train = { toughness = 30 }, widen = { toughness = 5 },
/// heal = "1d4", harm = "1d2", satiation = 500, vomit = 25 }`. None of it is shown before the
/// player tries; a rotten corpse does none of it. Once content has loaded, `train` and `widen`
/// are the resolved gifts (`Content::load_hunger`): per stat the largest its tags give
/// (`[corpses.by_tag]`, `data/hunger.toml`), then its own over them (0 switches a tag's gift
/// off), zeros left out.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OnEat {
    /// Said once it is eaten ("This kobold meat tastes terrible.").
    pub message: Option<String>,
    /// Sure stat training marks it gives, per stat (spec §8 "Stats"), 0 to [`MAX_TRAIN`].
    #[serde(default)]
    pub train: BTreeMap<Stat, u16>,
    /// The chance in percent, per stat, that the potential rises by 1 (never past the scores'
    /// range), 0 to [`MAX_WIDEN`]: potential points are rare.
    #[serde(default)]
    pub widen: BTreeMap<Stat, u8>,
    /// The old raise chances, only to refuse them by name ([`OnEat::check`]).
    #[serde(default, rename = "stat")]
    old_stat: Option<BTreeMap<String, i64>>,
    /// Hit points it gives back.
    pub heal: Option<Dice>,
    /// Hit points it costs.
    pub harm: Option<Dice>,
    /// Nutrition beyond what its size gives (less, if negative).
    #[serde(default)]
    pub satiation: i32,
    /// The chance in percent that it comes back up: the eater vomits, and it feeds nothing.
    #[serde(default)]
    pub vomit: u8,
}

/// The most marks a corpse's `train` gives a stat.
pub const MAX_TRAIN: u16 = 200;
/// The highest chance in percent of a corpse's `widen` (decision 11 of
/// `docs/design/stat-training.md`: potential points are rare).
pub const MAX_WIDEN: u8 = 20;

impl OnEat {
    /// Checks the chances (`vomit` 0 to 100, `widen` 0 to [`MAX_WIDEN`]), the marks (0 to
    /// [`MAX_TRAIN`]), the dice (rolling 0 to 1000), the extra nutrition, and the message, and
    /// refuses the old `stat`.
    pub(super) fn check(&self) -> Result<(), String> {
        if self.message.as_ref().is_some_and(|m| m.trim().is_empty()) {
            return Err("on_eat: `message` must not be empty".into());
        }
        if self.old_stat.is_some() {
            return Err(
                "on_eat: `stat` is gone: a corpse gives stat training marks, `train = \
                        { toughness = 30 }`, and rarely a chance to widen a potential, `widen = \
                        { toughness = 5 }`"
                    .into(),
            );
        }
        if self.vomit > 100 {
            return Err("on_eat: `vomit` is a chance in percent, 0 to 100".into());
        }
        if self.train.values().any(|&m| m > MAX_TRAIN) {
            return Err(format!(
                "on_eat: `train` gives 0 to {MAX_TRAIN} marks a stat"
            ));
        }
        if self.widen.values().any(|&c| c > MAX_WIDEN) {
            return Err(format!(
                "on_eat: `widen` is a chance in percent, 0 to {MAX_WIDEN} (potential points are \
                 rare)"
            ));
        }
        for (field, dice) in [("heal", self.heal), ("harm", self.harm)] {
            if dice.is_some_and(|d| d.min() < 0 || d.max() > 1000) {
                return Err(format!("on_eat: `{field}` must roll 0 to 1000"));
            }
        }
        let max = super::hunger::MAX_NUTRITION;
        if self.satiation.abs() > max {
            return Err(format!("on_eat: `satiation` must be within ±{max}"));
        }
        Ok(())
    }

    /// Lays its own gifts over `tags`' (the largest per stat of its tags' entries): its own
    /// per stat win, and zeros are left out. Then a stat with a `widen` needs a `train`.
    pub(super) fn resolve(&mut self, tags: &CorpseGifts) -> Result<(), String> {
        let mut train = tags.train.clone();
        train.extend(&self.train);
        train.retain(|_, m| *m > 0);
        let mut widen = tags.widen.clone();
        widen.extend(&self.widen);
        widen.retain(|_, c| *c > 0);
        if let Some(stat) = widen.keys().find(|s| !train.contains_key(s)) {
            return Err(format!(
                "on_eat: `widen` {} needs a `train` of the same stat",
                stat.id()
            ));
        }
        (self.train, self.widen) = (train, widen);
        Ok(())
    }

    /// Whether it does nothing beyond feeding.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// What a corpse gives the stats by default, per creature tag (`[corpses.by_tag]`,
/// `data/hunger.toml`): `giant = { train = { strength = 60 }, widen = { strength = 5 } }`. A
/// creature's corpse takes the largest of its tags' per stat ([`CorpseGifts::merge`]).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpseGifts {
    /// Sure marks per stat, 1 to [`MAX_TRAIN`].
    #[serde(default)]
    pub train: BTreeMap<Stat, u16>,
    /// The chance in percent per stat that the potential widens by 1, 1 to [`MAX_TAG_WIDEN`].
    #[serde(default)]
    pub widen: BTreeMap<Stat, u8>,
}

/// The highest `widen` a tag's entry may give: 2 to 5 % (decision 11 of
/// `docs/design/stat-training.md`).
pub const MAX_TAG_WIDEN: u8 = 5;

impl CorpseGifts {
    /// Checks the marks (1 to [`MAX_TRAIN`]) and the chances (1 to [`MAX_TAG_WIDEN`]), and that
    /// a stat with a `widen` has a `train`.
    pub(super) fn check(&self) -> Result<(), String> {
        if self.train.values().any(|&m| !(1..=MAX_TRAIN).contains(&m)) {
            return Err(format!("`train` gives 1 to {MAX_TRAIN} marks a stat"));
        }
        if self
            .widen
            .values()
            .any(|&c| !(1..=MAX_TAG_WIDEN).contains(&c))
        {
            return Err(format!(
                "`widen` is a chance in percent, 1 to {MAX_TAG_WIDEN} for a tag"
            ));
        }
        if let Some(stat) = self.widen.keys().find(|s| !self.train.contains_key(s)) {
            return Err(format!(
                "`widen` {} needs a `train` of the same stat",
                stat.id()
            ));
        }
        Ok(())
    }

    /// Takes the larger of its and `other`'s per stat.
    pub fn merge(&mut self, other: &CorpseGifts) {
        for (&stat, &m) in &other.train {
            let e = self.train.entry(stat).or_insert(0);
            *e = (*e).max(m);
        }
        for (&stat, &c) in &other.widen {
            let e = self.widen.entry(stat).or_insert(0);
            *e = (*e).max(c);
        }
    }
}

/// A natural attack as written in content files: `attack = { damage = "1d4", name = "bite",
/// verb = "bites" }`, with an optional `threat`, `crit`, and `to_hit` like a weapon's, and the
/// status effects and diseases a hit may cause (`on_hit`, `data/status.toml`,
/// `data/diseases.toml`).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NaturalAttackSpec {
    pub damage: Dice,
    /// The kind of damage it does (spec §8 "Damage types"): a bite piercing, a claw slashing,
    /// a slam bludgeoning.
    pub damage_type: Option<TypeNames>,
    pub name: String,
    pub verb: String,
    pub threat: Option<u8>,
    pub crit: Option<u8>,
    #[serde(default)]
    pub to_hit: i32,
    #[serde(default)]
    pub on_hit: Vec<OnHitSpec>,
}

/// A creature's natural attack (spec §8 "Combat"): what it does, and the words for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NaturalAttack {
    pub attack: Attack,
    /// What it is called: "bite" ("You dodge the giant rat's bite.").
    pub name: String,
    /// What it does to its target: "bites" ("The giant rat bites you.").
    pub verb: String,
    /// The status effects and diseases a hit with it may cause, in data order (spec §8 "Status
    /// effects", "Diseases"): a giant rat's bite may poison, and give filth fever.
    pub on_hit: Vec<OnHit>,
}

/// A creature's natural armour: `natural = { defence = 1, protection = 3, name = "shell" }`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hide {
    /// Its bonus to Defence (the Natural category).
    #[serde(default)]
    pub defence: i32,
    /// The damage it soaks off every hit (Protection, adding up with what the creature wears).
    #[serde(default)]
    pub protection: i32,
    /// What it is called: "shell" ("Your sword glances off the cave beetle's shell.").
    pub name: String,
}

impl Hide {
    /// Checks it as written in a creature's or a race's file: `defence` and `protection` each 0
    /// to [`super::object::MAX_BONUS`], not both 0, and a `name`.
    pub(super) fn check(&self) -> Result<(), String> {
        let bonus = 0..=super::object::MAX_BONUS;
        if !bonus.contains(&self.defence)
            || !bonus.contains(&self.protection)
            || self.defence + self.protection == 0
            || self.name.is_empty()
        {
            return Err(format!(
                "natural: `defence` and `protection` must be 0 to {}, not both 0, and `name` \
                 given",
                super::object::MAX_BONUS
            ));
        }
        Ok(())
    }
}

/// An incorporeal creature's nature (spec §8 "Incorporeal creatures"): `incorporeal = {
/// wall_defence = 4 }`. It passes through solid terrain (neither walkable nor transparent: walls,
/// rock, closed doors) off the level's edge, and may stand inside it, where it is harder to hit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Incorporeal {
    /// Its bonus to Defence while inside solid terrain (the Wall category).
    #[serde(default)]
    pub wall_defence: i32,
}

impl Incorporeal {
    /// Checks it as written: `wall_defence` 0 to [`super::object::MAX_BONUS`].
    pub(super) fn check(&self) -> Result<(), String> {
        if !(0..=super::object::MAX_BONUS).contains(&self.wall_defence) {
            return Err(format!(
                "incorporeal: `wall_defence` must be 0 to {}",
                super::object::MAX_BONUS
            ));
        }
        Ok(())
    }
}

/// What a creature type's `blood` says: its colour, or `None` for `"none"` (bloodless); dark
/// red when it says nothing.
pub(super) fn blood(spec: Option<&str>) -> Result<Option<Rgb>, String> {
    match spec {
        None => Ok(Some(crate::blood::RED)),
        Some("none") => Ok(None),
        Some(color) => color.parse().map(Some).map_err(|_| {
            format!("blood: expected a color like \"#rrggbb\" or \"none\", got `{color}`")
        }),
    }
}

impl NaturalAttackSpec {
    /// Checks the attack like a weapon's and resolves its defaults; `status` resolves a status
    /// id to the status and its `max`, `disease` a disease id.
    pub(super) fn resolve(
        self,
        status: impl Fn(&str) -> Option<(StatusId, u16, bool)>,
        disease: impl Fn(&str) -> Option<DiseaseId>,
        kinds: Kinds,
    ) -> Result<NaturalAttack, String> {
        let threat = self.threat.unwrap_or(DEFAULT_THREAT);
        let crit = self.crit.unwrap_or(DEFAULT_CRIT);
        if !THREATS.contains(&threat) || !CRITS.contains(&crit) {
            return Err(format!(
                "attack: `threat` must be {} to {} and `crit` {} to {}",
                THREATS.start(),
                THREATS.end(),
                CRITS.start(),
                CRITS.end()
            ));
        }
        if self.name.trim().is_empty() || self.verb.trim().is_empty() {
            return Err("attack: `name` and `verb` must not be empty".into());
        }
        if self.to_hit.abs() > MAX_BONUS {
            return Err(format!("attack: `to_hit` must be within ±{MAX_BONUS}"));
        }
        let on_hit = (self.on_hit.iter())
            .map(|o| {
                o.resolve(&status, &disease)
                    .map_err(|m| format!("attack: {m}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(NaturalAttack {
            attack: Attack {
                damage: self.damage,
                to_hit: self.to_hit,
                threat,
                crit,
                strength: None,
                kinds,
            },
            name: self.name,
            verb: self.verb,
            on_hit,
        })
    }
}

/// What a creature type's names resolve to when content loads.
pub(super) struct Resolved {
    pub blood: Option<Rgb>,
    pub rarity: usize,
    pub light: Option<LightId>,
    pub faction: Option<FactionId>,
    pub factions: Vec<(FactionId, u32)>,
    pub vision: usize,
    pub size: usize,
    pub stats: Stats,
    pub skills: Skills,
    pub attack: Option<NaturalAttack>,
    pub throw: Option<Throw>,
    pub weapon_skills: WeaponSkills,
    pub resist: Vec<(DamageTypeId, i32)>,
    pub immune: Vec<StatusId>,
}

impl CreatureSpec {
    pub(super) fn resolve(self, r: Resolved) -> CreatureDef {
        CreatureDef {
            // The loader fills it from the plural dictionary when content gives none.
            plural: self.plural.unwrap_or_else(|| format!("{}s", self.name)),
            id: self.id,
            name: self.name,
            glyph: self.glyph,
            fg: self.fg,
            speed: self.speed,
            weight: self.weight,
            size: r.size,
            faction: r.faction,
            factions: r.factions,
            tags: self.tags,
            alignment: self.alignment,
            race: self.race,
            vision: r.vision,
            sight: self.sight,
            combat: self.combat,
            range: self.range.unwrap_or(1),
            opens_doors: self.opens_doors,
            lurks: self.lurks,
            budge: self.budge,
            attack_cost: self.attack_cost,
            light: r.light,
            footsteps: self.footsteps,
            hearing: self.hearing,
            stats: r.stats,
            skills: r.skills,
            base_hp: i32::from(self.base_hp),
            base_mana: i32::from(self.mana),
            level: self.level,
            xp: self.xp,
            xp_percent: self.xp_percent,
            unique: self.unique,
            stray: self.stray,
            rarity: r.rarity,
            loot: self.loot,
            attack: r.attack,
            throw: r.throw,
            summon: None,
            summon_spec: self.summon,
            natural: self.natural,
            incorporeal: self.incorporeal,
            vitals: self.vitals,
            resist: r.resist,
            corpse: self.corpse,
            on_eat: self.on_eat,
            blood: r.blood,
            immune: r.immune,
            dies: self.dies,
            animated: self.animated,
            mimic: self.mimic,
            weapon_skills: r.weapon_skills,
            gear: Gear::default(),
            gear_spec: GearSpec {
                racial_weapon: self.racial_weapon,
                lines: self.gear,
            },
            anim: self.anim,
            glow: self.glow,
            particles: self.particles,
            campaign: None,
        }
    }
}

impl CreatureDef {
    /// The verb of its death line: its type's `dies`, else "dies".
    pub fn death_verb(&self) -> &str {
        self.dies.as_deref().unwrap_or("dies")
    }

    /// The experience a kill of it is worth at its own level: `xp` times its hand-set
    /// `xp_percent`, rounded (halves up). The one place the modifier is read: the level scaling
    /// and the first-kill bonus start from it.
    pub fn kill_xp(&self) -> u32 {
        let scaled = (u64::from(self.xp) * u64::from(self.xp_percent) + 50) / 100;
        u32::try_from(scaled).unwrap_or(u32::MAX)
    }
}

/// A creature type. The player is the creature with id `"player"`.
#[derive(Clone, Debug)]
pub struct CreatureDef {
    pub id: String,
    pub name: String,
    /// Its name for more than one ("giant rats", "deer"): quest messages count them.
    pub plural: String,
    pub glyph: char,
    pub fg: Rgb,
    /// How quickly it acts ([`NORMAL_SPEED`] is normal; twice that, twice as many actions in the
    /// same game time). Rules only: never shown to the player, nor put in the perception snapshot.
    pub speed: u32,
    /// Gameplay: the creature's own weight in grams, without what it carries (see
    /// [`crate::object::Weight`]). At least 1.
    pub weight: u32,
    /// Gameplay: its size, an index into [`super::Content::sizes`] (spec §8 "Burden"): how much
    /// it can carry before it is burdened. The player's comes from their race, if the run has a
    /// character.
    pub size: usize,
    /// The faction it belongs to; `None` when the content defines no factions. The player's
    /// creature belongs to [`super::PLAYER_FACTION`]. For a type with [`CreatureDef::factions`],
    /// the heaviest of them (ties: the first by id), which type-level answers read (the wiki,
    /// [`super::Content::stance`]); its creatures each carry their own.
    pub faction: Option<FactionId>,
    /// The factions each of its creatures draws one of when it is made, by weight, in id order
    /// (spec §7 "Factions"; `world/court.rs`); empty for a type of one faction.
    pub factions: Vec<(FactionId, u32)>,
    /// Its tags, as written (spec §8 "Quests"): a kill quest's `tag:goblin` counts every type
    /// that carries `goblin`. Rules only; the player never sees them.
    pub tags: Vec<String>,
    /// Gameplay: where it stands between good and evil (spec §8 "Alignment"), unless a level
    /// placed it elsewhere ([`crate::world::Actor::nature`]). Never shown to the player.
    pub alignment: Alignment,
    /// Gameplay: the race it is one of (an id in `data/races.toml`), if any: its corpse is kin
    /// to a character of that race (spec §8 "Alignment").
    pub race: Option<String>,
    /// Its eyes: an index into [`super::Content::vision_profiles`]. The player's eyes come from
    /// the scenario (and the dev vision switch) instead.
    pub vision: usize,
    /// How far it sees, in cells (a Euclidean radius, like every radius).
    pub sight: i32,
    /// How it fights. Rules only, like the rest of its behavior.
    pub combat: Combat,
    /// How far its attacks reach, in cells: 1 for melee.
    pub range: i32,
    /// Whether it opens closed doors in its way.
    pub opens_doors: bool,
    /// Whether it lurks (spec §8 "Creature behavior"; `ai.rs::fight`): a hostile it sees, it
    /// waits for where it stands until the hostile is beside it or it has been harmed. Rules only.
    pub lurks: bool,
    /// Gameplay: whether the player may swap places with it when it is not hostile (spec §8
    /// "Displacement").
    pub budge: Budge,
    /// What its attacks (melee, shooting, or casting) cost in energy, if its type says
    /// ([`super::Content::action_cost`]); `None`: the cost of the attack in `data/actions.toml`.
    /// Rules only.
    pub attack_cost: Option<u32>,
    /// Light the creature carries with it (e.g. a torch), if any.
    pub light: Option<LightId>,
    /// Gameplay: the loudness of its steps on terrain with a `step_noise` of 100
    /// ([`crate::noise::noise`]); 0 moves silently. At most [`crate::noise::MAX_LOUDNESS`].
    pub footsteps: u16,
    /// Gameplay: the faintest loudness it hears (1 to [`crate::noise::MAX_LOUDNESS`]); `None` is
    /// deaf.
    pub hearing: Option<u16>,
    /// Gameplay: the scores and potentials a creature of this type starts with (spec §8
    /// "Stats"). Rules only: a creature's stats are never shown to the player. The player's
    /// come from the scenario instead ([`super::ScenarioDef::player_stats`]); theirs are average.
    pub stats: Stats,
    /// Gameplay: the skills a creature of this type is trained in (spec §8 "Skills"); every
    /// other skill is untrained. Rules only: a creature's skills are never shown to the player.
    /// The player's come from their race and class.
    pub skills: Skills,
    /// Gameplay: hit points beyond its Toughness (spec §8 "Combat"); the player's come from the
    /// scenario instead.
    pub base_hp: i32,
    /// Gameplay: mana beyond its mental stats' modifiers (spec §8 "Mana"); the player's come
    /// from their race and class (the scenario's without a character). Never shown.
    pub base_mana: i32,
    /// Gameplay: its level (spec §8 "Experience"); the player's own is their experience's.
    pub level: u8,
    /// Gameplay: the experience a kill of it is worth at its own level, before `xp_percent`.
    pub xp: u32,
    /// Gameplay: the hand-set modifier of `xp` in percent (100: none); [`Self::kill_xp`] applies
    /// it.
    pub xp_percent: u32,
    /// Gameplay: whether at most one of it is drawn in a run by a population (spec §7 "Danger
    /// levels"; [`crate::world::Progress`] remembers those drawn).
    pub unique: bool,
    /// Gameplay: whether a branch may draw it as a stray without naming it in its population
    /// (spec §7 "Danger levels"; `Content::fill_generic`); `stray = false` in its file for a
    /// type only ever placed by hand or by its own rules.
    pub stray: bool,
    /// Gameplay: how often a population or a spawn draws it, an index into
    /// [`super::Content::creature_rarities`] (spec §7 "Danger levels"). Never shown.
    pub rarity: usize,
    /// Gameplay: the chance in percent that it drops a magic or rare item where it dies.
    pub loot: u8,
    /// Gameplay: its natural attack, used when it wields no weapon; `None`: bare hands (1d3).
    pub attack: Option<NaturalAttack>,
    /// Gameplay: its throw, a natural ranged attack with no ammunition (spec §8 "Ranged combat":
    /// "Creatures' throws"); `None`: it throws nothing. Rules only (the wiki states it).
    pub throw: Option<Throw>,
    /// Gameplay: its summoning, the creatures of its side it calls forth (spec §8
    /// "Summoning"); `None`: it summons nothing. Resolved once every creature type is known.
    /// Rules only (the wiki states it).
    pub summon: Option<Summon>,
    /// Its summoning as written, resolved by `Content::load_summons`.
    pub(super) summon_spec: Option<SummonSpec>,
    /// Gameplay: its natural armour, if any.
    pub natural: Option<Hide>,
    /// Gameplay: whether it passes through solid terrain and stands inside it, and its Defence
    /// there (spec §8 "Incorporeal creatures"); `None`: it does not. Rules only (the wiki
    /// states it).
    pub incorporeal: Option<Incorporeal>,
    /// Gameplay: whether its type says it has vitals (`vitals = false` says not); whether it
    /// has them is [`super::Content::has_vitals`], which adds the rules' kinds. Rules only (the
    /// wiki states it).
    pub vitals: bool,
    /// Gameplay: what it resists, in percent per damage type (spec §8 "Damage types"); 100 is
    /// immune, negative a vulnerability. Rules only: the player learns it by play.
    pub resist: Vec<(DamageTypeId, i32)>,
    /// Gameplay: whether it leaves a corpse when it dies (spec §8 "Hunger"; only with
    /// `data/hunger.toml`, which says how corpses rot and feed).
    pub corpse: bool,
    /// Gameplay: what eating its fresh corpse does, beyond feeding the eater. Hidden from the
    /// player until they try.
    pub on_eat: Option<OnEat>,
    /// Presentation-only: the colour of its blood (spec §8 "Blood"); `None`: it does not bleed.
    pub blood: Option<Rgb>,
    /// Gameplay: the status effects it never suffers (spec §8 "Status effects"): refused where
    /// they are given (`World::afflict`), and the diseases' status for every disease
    /// (`World::expose`). Rules only (the wiki states it).
    pub immune: Vec<StatusId>,
    /// Presentation-only: the verb of its death line ("splinters"); `None`: "dies".
    pub dies: Option<String>,
    /// Gameplay: what makes it an animated object (spec §8 "Animated objects"), for a type
    /// generated from `data/animated/`; `None` for every other.
    pub animated: Option<AnimatedDef>,
    /// Gameplay: what makes it a mimic (spec §8 "Mimics"), for a type generated from
    /// `data/mimics.toml`; `None` for every other.
    pub mimic: Option<super::mimic::MimicDef>,
    /// Gameplay: the ranks in weapon skills a creature of this type has (spec §8 "Weapon
    /// skills"), defaults included. Rules only: never shown. The player's come from their race
    /// and class.
    pub weapon_skills: WeaponSkills,
    /// Gameplay: what a creature of this type may spawn carrying, wielding, and wearing (spec §8
    /// "Creature gear"), unless a level gives it a fixed kit. Rules only (the wiki states it).
    pub gear: Gear,
    /// Its gear as written, resolved once objects are known.
    pub(super) gear_spec: GearSpec,
    /// Presentation-only.
    pub anim: Option<GlyphAnim>,
    /// Presentation-only: how much the glyph glows (bloom), 0 to 1.
    pub glow: f32,
    /// Presentation-only: glyph particle emitters that move with the creature.
    pub particles: Vec<ParticleEmitter>,
    /// The campaign whose own type it is (`campaigns/<id>/data/creatures/`, spec §3); `None`
    /// for a shared one. Only that campaign's files may name it.
    pub campaign: Option<String>,
}
