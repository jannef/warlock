//! Animated objects (spec §8 "Animated objects"; docs/dev/animated.md): `data/animated.toml` (the
//! tiers, the drop rule, what every animated type is) and `data/animated/` (the household bodies,
//! one file each). Each body becomes an ordinary creature type when content loads,
//! `animated_<body>`, at the body's level, and each tier without bodies one type per level of
//! its range, `animated_<tier>_<level>` (a family: `animated_blade_5` … `animated_blade_20`),
//! their numbers by the tier's formulas: every rule reads them as it reads a kobold. Generation
//! is a pure function of the content (no draws), so the ids, their order, and the fingerprint
//! are the same on every load.

use super::creature::CreatureSpec;
use super::{Content, ContentError, ContentFiles, CreatureId, ItemClassId, ObjectId, SlotId};
use crate::stats::Stat;
use serde::Deserialize;
use std::collections::BTreeMap;

/// The rules file.
pub(super) const RULES: &str = "data/animated.toml";

/// The prefix of a generated type's id (`animated_candlestick`).
pub const ID_PREFIX: &str = "animated_";

/// The dice the formula picks from, by average × 10, smallest first: the roster's ladder (#383)
/// with 1d2 below it, for the household tier's two steps under the curve.
const LADDER: &[(&str, u32)] = &[
    ("1d2", 15),
    ("1d3", 20),
    ("1d4", 25),
    ("1d6", 35),
    ("1d8", 45),
    ("1d10", 55),
    ("2d6", 70),
    ("2d8", 90),
    ("3d6", 105),
    ("3d8", 135),
    ("4d6", 140),
    ("3d10", 165),
    ("4d8", 180),
    ("3d12", 195),
    ("4d10", 220),
    ("4d12", 260),
    ("5d10", 275),
    ("5d12", 325),
];

/// Animated objects as content defines them; empty without `data/animated.toml`.
#[derive(Clone, Debug, Default)]
pub struct Animated {
    pub drops: Drops,
    pub creature: Common,
    /// What a household body's base material gives its type, by the material's group.
    pub groups: Vec<Group>,
    /// In file order.
    pub tiers: Vec<Tier>,
    /// The household bodies, in path order (their types' order).
    pub bodies: Vec<Body>,
    /// Each tier's slot lines, resolved (by tier index).
    pub lines: Vec<Vec<Line>>,
}

/// What becomes of what an animated thing is made of when it falls (spec §8 "Animated objects").
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Drops {
    /// The chance in percent that a component falls whole; else nothing of it falls (A1).
    pub intact: u8,
}

/// What every animated type is.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Common {
    pub faction: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub vision: String,
    pub sight: i32,
    pub hearing: u16,
    /// Status ids it never suffers (`diseased`: every disease).
    #[serde(default)]
    pub immune: Vec<String>,
}

/// What a household body's base material gives its type, by the material's group.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    /// The material group (`metal`, `wood`, ...; data/materials/).
    pub id: String,
    /// Resistances in percent by damage type.
    #[serde(default)]
    pub resist: BTreeMap<String, i32>,
    /// Natural Protection, its hide named by the group.
    #[serde(default)]
    pub protection: i32,
    /// The verb of its death line ("splinters").
    pub dies: String,
}

/// A tier of animated things: the household tier (one type per body file), or a family (one
/// type per level of `levels`: blades).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tier {
    pub id: String,
    /// What the wiki calls it.
    pub name: String,
    /// One type per body file (the household tier); else one type per level of `levels`.
    #[serde(default)]
    pub bodies: bool,
    /// The levels its types lie within, least first.
    pub levels: [u8; 2],
    /// What it adds to the base stats.
    #[serde(default)]
    pub stats: BTreeMap<Stat, i32>,
    /// The factor on `base_hp` (2.5 × L × it).
    pub hp_factor: f64,
    /// Steps along the dice ladder from the curve's nearest.
    #[serde(default)]
    pub dice_steps: i32,
    /// Its types' size (data/sizes.toml).
    pub size: String,
    /// How loud its steps are.
    #[serde(default)]
    pub footsteps: u16,
    /// The death line's verb of a body whose base material has no group, and of every type of
    /// a family.
    pub dies: String,
    /// A family's name, and every member's ("animated blade").
    pub creature: Option<String>,
    /// A family's own glyph and colour (`"#rrggbb"`): what shows without the piece it is shown
    /// by.
    pub glyph: Option<char>,
    pub fg: Option<String>,
    /// A family: the slot whose piece it is shown by (in its material's colour; with
    /// `item_glyph`, by its glyph too).
    pub shown_by: Option<String>,
    #[serde(default)]
    pub item_glyph: bool,
    /// A family's rarity (`data/creature_rarities.toml`); none: common.
    pub rarity: Option<String>,
    /// A family's speed by level; none: the default.
    pub speed: Option<Ramp>,
    /// A family's natural Defence by level, and its hide's name.
    pub defence: Option<Ramp>,
    pub hide: Option<String>,
    /// A family's resistances in percent by damage type.
    #[serde(default)]
    pub resist: BTreeMap<String, i32>,
    /// What a family's creature is made of: one line per slot, tried in order as it spawns.
    #[serde(default)]
    pub slots: Vec<SlotLine>,
    /// The chance in percent by level that a piece is at least magic, and that it is rare.
    pub magic: Option<Ramp>,
    pub rare: Option<Ramp>,
    /// Its ranks by level in the weapon skill of what it wields.
    pub weapon_rank: Option<Ramp>,
    /// A family's id (`animated_blade`): what populations, kill quests, and first kills name.
    #[serde(skip)]
    pub family: String,
}

impl Tier {
    /// Whether it is a family: one type per level, no bodies.
    pub fn is_family(&self) -> bool {
        !self.bodies
    }
}

/// A number by level L: nothing below `from`, else `base + per_level × (L − from)` rounded
/// down, at most `most`.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ramp {
    #[serde(default)]
    pub from: u8,
    #[serde(default)]
    pub base: f64,
    #[serde(default)]
    pub per_level: f64,
    pub most: Option<f64>,
}

impl Ramp {
    /// Its value at level `level`.
    pub fn at(&self, level: u8) -> i32 {
        if level < self.from {
            return 0;
        }
        let v = self.base + self.per_level * f64::from(level - self.from);
        let v = self.most.map_or(v, |m| v.min(m));
        // A hair over, so that 0.1 × 30 is 3 and never 2.999….
        (v + 1e-9).floor() as i32
    }
}

/// One line of what a family's creature is made of: a piece for `slot`, drawn among the base
/// items of `classes`, by `chance` in percent.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotLine {
    pub slot: String,
    pub classes: Vec<String>,
    #[serde(default = "always")]
    pub chance: u8,
}

fn always() -> u8 {
    100
}

/// A [`SlotLine`] resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub slot: SlotId,
    pub classes: Vec<ItemClassId>,
    pub chance: u8,
}

/// A household body's blow, as its file writes it: its dice come from the tier's formula.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyAttack {
    pub damage_type: String,
    pub name: String,
    pub verb: String,
}

/// A household body (`data/animated/<id>.toml`): a real object and how it moves and hits.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodySpec {
    pub id: String,
    pub object: String,
    pub level: u8,
    pub rarity: Option<String>,
    pub speed: Option<u32>,
    pub attack: BodyAttack,
    #[serde(default)]
    pub on_hit: Vec<toml::Value>,
    pub name: Option<String>,
    pub plural: Option<String>,
    pub count: Option<[u32; 2]>,
    pub moves: Option<String>,
    pub dies: Option<String>,
    pub light: Option<String>,
}

/// A household body, its object resolved.
#[derive(Clone, Debug)]
pub struct Body {
    pub spec: BodySpec,
    /// The object it is made of.
    pub object: ObjectId,
    /// How many of its object it is, least and most (a clatter of cutlery: 3 to 6).
    pub count: (u32, u32),
    /// The tier it is of.
    pub tier: usize,
    /// The file it came from.
    pub path: String,
}

/// What makes a creature type an animated one (`CreatureDef::animated`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimatedDef {
    /// Its tier, an index into [`Animated::tiers`].
    pub tier: usize,
    /// The first type of its family: a household type is its own.
    pub family: CreatureId,
    /// Its place in its family, 0 for the first (its level less the tier's least).
    pub member: u8,
    /// Its household body, an index into [`Animated::bodies`]; none in a family.
    pub body: Option<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesFile {
    drops: Drops,
    creature: Common,
    #[serde(default)]
    group: Vec<Group>,
    #[serde(default)]
    tier: Vec<Tier>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BodyFile {
    #[serde(default)]
    body: Vec<BodySpec>,
}

impl Animated {
    /// Reads `data/animated.toml` and the bodies of `data/animated/` (path order), checking them
    /// against the objects, materials, and lights loaded before. None without the rules file;
    /// bodies without it are an error.
    pub(super) fn parse(files: &ContentFiles, content: &Content) -> Result<Self, ContentError> {
        let bodies: Vec<(&str, &str)> = files
            .table("animated")
            .filter(|(p, _)| *p != RULES)
            .collect();
        let Some(text) = files.get_optional(RULES) else {
            return match bodies.iter().find(|(_, t)| has_body(t)) {
                Some((path, _)) => Err(ContentError::new(path, format!("bodies need {RULES}"))),
                None => Ok(Self::default()),
            };
        };
        let err = |m: String| ContentError::new(RULES, m);
        let rules: RulesFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if rules.drops.intact > 100 {
            return Err(err(
                "drops: `intact` is a chance in percent, 0 to 100".into()
            ));
        }
        let mut lines = Vec::new();
        let mut tiers = rules.tier;
        for tier in &mut tiers {
            let [from, to] = tier.levels;
            if tier.bodies && (!tier.slots.is_empty() || tier.creature.is_some()) {
                return Err(err(format!(
                    "tier `{}`: a tier of bodies has no `creature` or `slots`",
                    tier.id
                )));
            }
            lines.push(if tier.is_family() {
                tier.family = format!("{ID_PREFIX}{}", tier.id);
                family_lines(tier, content).map_err(err)?
            } else {
                Vec::new()
            });
            if from == 0 || from > to || to > super::MAX_LEVEL {
                return Err(err(format!(
                    "tier `{}`: `levels` must be 1 to {} and least first",
                    tier.id,
                    super::MAX_LEVEL
                )));
            }
            if !(0.0..=10.0).contains(&tier.hp_factor) {
                return Err(err(format!("tier `{}`: `hp_factor` is 0 to 10", tier.id)));
            }
        }
        if tiers.iter().filter(|t| t.bodies).count() > 1 {
            return Err(err("one tier of bodies at most".into()));
        }
        let household = (tiers.iter()).position(|t| t.bodies);
        let mut out = Self {
            drops: rules.drops,
            creature: rules.creature,
            groups: rules.group,
            tiers,
            bodies: Vec::new(),
            lines,
        };
        let mut seen: BTreeMap<String, String> = BTreeMap::new();
        for (path, text) in bodies {
            let err = |m: String| ContentError::new(path, m);
            if super::campaign::of_path(path).is_some() {
                return Err(err("animated bodies are shared: data/animated/ only".into()));
            }
            let file: BodyFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
            for spec in file.body {
                let tier = household.ok_or_else(|| err("no tier of bodies".into()))?;
                if let Some(first) = seen.insert(spec.id.clone(), path.to_string()) {
                    return Err(super::duplicate_id("animated body", &spec.id, path, &first));
                }
                let body = out.resolve(spec, tier, path, content).map_err(err)?;
                out.bodies.push(body);
            }
        }
        Ok(out)
    }

    fn resolve(
        &self,
        spec: BodySpec,
        tier: usize,
        path: &str,
        content: &Content,
    ) -> Result<Body, String> {
        let id = &spec.id;
        let object = (content.object_id(&spec.object))
            .ok_or_else(|| format!("body `{id}`: unknown object `{}`", spec.object))?;
        let def = content.object(object);
        if content.item_class(def.class).id != "household" {
            return Err(format!(
                "body `{id}`: object `{}` is not of class `household`",
                spec.object
            ));
        }
        let [from, to] = self.tiers[tier].levels;
        if !(from..=to).contains(&spec.level) {
            return Err(format!(
                "body `{id}`: level {} is outside its tier's {from} to {to}",
                spec.level
            ));
        }
        let count = match spec.count {
            None => (1, 1),
            Some([least, most]) if least >= 1 && least <= most && def.stackable => (least, most),
            Some(_) => {
                return Err(format!(
                    "body `{id}`: `count` is [least, most], 1 or more, of a stackable object"
                ));
            }
        };
        if spec.name.is_none() && spec.plural.is_some() {
            return Err(format!("body `{id}`: `plural` only with `name`"));
        }
        if let Some(light) = &spec.light
            && content.light_id(light).is_none()
        {
            return Err(format!("body `{id}`: unknown light `{light}`"));
        }
        Ok(Body {
            spec,
            object,
            count,
            tier,
            path: path.to_string(),
        })
    }

    /// The creature types the tiers generate, tiers in file order: the bodies' in body order, a
    /// family's by level, least first. Each comes with the file it came from and what makes it
    /// animated (its family filled in once it is numbered). Their numbers are the tier's
    /// formulas at the type's level ([`Numbers`]).
    pub(super) fn specs(
        &self,
        content: &Content,
    ) -> Result<Vec<(String, CreatureSpec)>, ContentError> {
        let mut out = Vec::new();
        for (t, tier) in self.tiers.iter().enumerate() {
            if tier.is_family() {
                for level in tier.levels[0]..=tier.levels[1] {
                    let err = |m: String| {
                        ContentError::new(RULES, format!("tier `{}` at {level}: {m}", tier.id))
                    };
                    let spec = self.member(t, level, content).map_err(err)?;
                    out.push((RULES.to_string(), spec));
                }
                continue;
            }
            for (index, body) in self.bodies.iter().enumerate() {
                let err = |m: String| {
                    ContentError::new(&body.path, format!("body `{}`: {m}", body.spec.id))
                };
                let spec = self.spec(index, body, content).map_err(err)?;
                out.push((body.path.clone(), spec));
            }
        }
        Ok(out)
    }

    /// The type of family tier `t` at `level`: `animated_<tier>_<level>`.
    fn member(&self, t: usize, level: u8, content: &Content) -> Result<CreatureSpec, String> {
        let tier = &self.tiers[t];
        let numbers = Numbers::at(level, tier);
        let mut table = self.common(tier, &numbers, level, content);
        let s = |v: &str| toml::Value::String(v.to_string());
        let int = |v: i64| toml::Value::Integer(v);
        table.insert("id".into(), s(&format!("{}_{level}", tier.family)));
        table.insert(
            "name".into(),
            s(tier.creature.as_deref().unwrap_or_default()),
        );
        table.insert("glyph".into(), s(&tier.glyph.unwrap_or('?').to_string()));
        table.insert("fg".into(), s(tier.fg.as_deref().unwrap_or_default()));
        if let Some(speed) = tier.speed {
            table.insert("speed".into(), int(i64::from(speed.at(level))));
        }
        if let Some(rarity) = &tier.rarity {
            table.insert("rarity".into(), s(rarity));
        }
        if let Some(defence) = tier.defence.map(|d| d.at(level)).filter(|&d| d > 0) {
            let mut hide = toml::Table::new();
            hide.insert("defence".into(), int(i64::from(defence)));
            hide.insert("name".into(), s(tier.hide.as_deref().unwrap_or("edge")));
            table.insert("natural".into(), toml::Value::Table(hide));
        }
        let resist = (tier.resist.iter())
            .map(|(k, &v)| (k.clone(), int(i64::from(v))))
            .collect();
        table.insert("resist".into(), toml::Value::Table(resist));
        table.insert("dies".into(), s(&tier.dies));
        let mut spec: CreatureSpec = toml::Value::Table(table)
            .try_into()
            .map_err(|e: toml::de::Error| e.to_string())?;
        spec.animated = Some(AnimatedDef {
            tier: t,
            family: CreatureId(0),
            member: level - tier.levels[0],
            body: None,
        });
        Ok(spec)
    }

    /// What every generated type of `tier` at `level` holds: its weight, size, faction, tags,
    /// senses, footsteps, stats, hit points, level, experience, no corpse and no blood, and its
    /// immunities.
    fn common(&self, tier: &Tier, numbers: &Numbers, level: u8, content: &Content) -> toml::Table {
        let mut t = toml::Table::new();
        let s = |v: &str| toml::Value::String(v.to_string());
        let int = |v: i64| toml::Value::Integer(v);
        // It weighs what it is made of, which it carries: of its own, nothing to speak of.
        t.insert("weight".into(), int(1));
        t.insert("size".into(), s(&tier.size));
        t.insert("faction".into(), s(&self.creature.faction));
        let tags = (self.creature.tags.iter()).map(|x| s(x)).collect();
        t.insert("tags".into(), toml::Value::Array(tags));
        t.insert("vision".into(), s(&self.creature.vision));
        t.insert("sight".into(), int(i64::from(self.creature.sight)));
        t.insert("hearing".into(), int(i64::from(self.creature.hearing)));
        t.insert("footsteps".into(), int(i64::from(tier.footsteps)));
        let mut stats = toml::Table::new();
        for (stat, score) in &numbers.stats {
            stats.insert(stat.id().into(), int(i64::from(*score)));
        }
        t.insert("stats".into(), toml::Value::Table(stats));
        t.insert("base_hp".into(), int(i64::from(numbers.base_hp)));
        t.insert("level".into(), int(i64::from(level)));
        let xp = content.experience().guideline().xp(level);
        t.insert("xp".into(), int(i64::from(xp)));
        t.insert("corpse".into(), toml::Value::Boolean(false));
        t.insert("blood".into(), s("none"));
        let immune = (self.creature.immune.iter()).map(|x| s(x)).collect();
        t.insert("immune".into(), toml::Value::Array(immune));
        t
    }

    /// The family `tier` belongs to, by its index; `None` for the tier of bodies.
    pub fn family_tier(&self, def: Option<&AnimatedDef>) -> Option<&Tier> {
        let tier = self.tiers.get(def?.tier)?;
        tier.is_family().then_some(tier)
    }

    /// The families' ids with their first members' (`animated_blade`, `animated_blade_5`).
    pub(super) fn families(&self) -> impl Iterator<Item = (&str, String)> {
        (self.tiers.iter())
            .filter(|t| t.is_family())
            .map(|t| (t.family.as_str(), format!("{}_{}", t.family, t.levels[0])))
    }

    fn spec(&self, index: usize, body: &Body, content: &Content) -> Result<CreatureSpec, String> {
        let tier = &self.tiers[body.tier];
        let object = content.object(body.object);
        let material = content.material(object.material());
        let group =
            (material.group.as_deref()).and_then(|g| self.groups.iter().find(|x| x.id == g));
        let level = body.spec.level;
        let numbers = Numbers::at(level, tier);
        let mut t = self.common(tier, &numbers, level, content);
        let s = |v: &str| toml::Value::String(v.to_string());
        let int = |v: i64| toml::Value::Integer(v);
        t.insert("id".into(), s(&format!("{ID_PREFIX}{}", body.spec.id)));
        let name = (body.spec.name.clone()).unwrap_or_else(|| format!("animated {}", object.name));
        t.insert("name".into(), s(&name));
        if let Some(plural) = &body.spec.plural {
            t.insert("plural".into(), s(plural));
        }
        t.insert("glyph".into(), s(&object.glyph.to_string()));
        t.insert("fg".into(), s(&object.fg.to_string()));
        if let Some(speed) = body.spec.speed {
            t.insert("speed".into(), int(i64::from(speed)));
        }
        if let Some(rarity) = &body.spec.rarity {
            t.insert("rarity".into(), s(rarity));
        }
        let mut attack = toml::Table::new();
        attack.insert("damage".into(), s(numbers.dice));
        attack.insert("damage_type".into(), s(&body.spec.attack.damage_type));
        attack.insert("name".into(), s(&body.spec.attack.name));
        attack.insert("verb".into(), s(&body.spec.attack.verb));
        if !body.spec.on_hit.is_empty() {
            attack.insert(
                "on_hit".into(),
                toml::Value::Array(body.spec.on_hit.clone()),
            );
        }
        t.insert("attack".into(), toml::Value::Table(attack));
        if let Some(group) = group {
            if group.protection > 0 {
                let mut hide = toml::Table::new();
                hide.insert("protection".into(), int(i64::from(group.protection)));
                hide.insert("name".into(), s(&material.name));
                t.insert("natural".into(), toml::Value::Table(hide));
            }
            let resist = (group.resist.iter())
                .map(|(k, &v)| (k.clone(), int(i64::from(v))))
                .collect();
            t.insert("resist".into(), toml::Value::Table(resist));
        }
        if let Some(light) = &body.spec.light {
            t.insert("light".into(), s(light));
        }
        let dies = (body.spec.dies.as_deref())
            .or(group.map(|g| g.dies.as_str()))
            .unwrap_or(&tier.dies);
        t.insert("dies".into(), s(dies));
        let mut spec: CreatureSpec = toml::Value::Table(t)
            .try_into()
            .map_err(|e: toml::de::Error| e.to_string())?;
        spec.animated = Some(AnimatedDef {
            tier: body.tier,
            family: CreatureId(0),
            member: 0,
            body: Some(index),
        });
        Ok(spec)
    }

    /// The body of animated type `def`, if it has one.
    pub fn body_of(&self, def: Option<&AnimatedDef>) -> Option<&Body> {
        def.and_then(|a| self.bodies.get(a.body?))
    }
}

/// The slot lines of family `tier`, resolved, after checking what a family must have: a
/// `creature` name, its glyph and colour, its slots and their classes, and a `shown_by` slot
/// among them.
fn family_lines(tier: &Tier, content: &Content) -> Result<Vec<Line>, String> {
    let id = &tier.id;
    if tier.creature.as_deref().is_none_or(str::is_empty) || tier.glyph.is_none() {
        return Err(format!(
            "tier `{id}`: a family needs `creature` and `glyph`"
        ));
    }
    if tier.fg.is_none() || tier.slots.is_empty() {
        return Err(format!("tier `{id}`: a family needs `fg` and `slots`"));
    }
    let mut lines = Vec::new();
    for line in &tier.slots {
        let slot = (content.slot_id(&line.slot))
            .ok_or_else(|| format!("tier `{id}`: unknown slot `{}`", line.slot))?;
        let classes = (line.classes.iter())
            .map(|c| {
                (content.item_class_id(c))
                    .ok_or_else(|| format!("tier `{id}`: unknown class `{c}`"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if line.chance > 100 || classes.is_empty() {
            return Err(format!(
                "tier `{id}`: slot `{}`: `chance` is 0 to 100, and `classes` not empty",
                line.slot
            ));
        }
        lines.push(Line {
            slot,
            classes,
            chance: line.chance,
        });
    }
    if let Some(shown) = &tier.shown_by
        && !tier.slots.iter().any(|l| l.slot == *shown)
    {
        return Err(format!(
            "tier `{id}`: `shown_by` names no slot of its `slots`"
        ));
    }
    Ok(lines)
}

impl Content {
    /// The first type of creature type `creature`'s family (spec §8 "Animated objects"): the
    /// first of an animated tier's run of levels (`animated_blade_5` for every blade); any other
    /// type is its own.
    pub fn family(&self, creature: CreatureId) -> CreatureId {
        self.creature(creature)
            .animated
            .map_or(creature, |a| a.family)
    }

    /// The id first kills, kill quests, and the kill tally know creature type `creature` by:
    /// its family's (`animated_blade`), else its own.
    pub fn family_key(&self, creature: CreatureId) -> &str {
        let def = self.creature(creature);
        (self.animated.family_tier(def.animated.as_ref())).map_or(def.id.as_str(), |t| &t.family)
    }

    /// The levels of creature type `creature`'s family, least and most, if it is one (its
    /// members are numbered in a row from [`Content::family`]).
    pub fn family_levels(&self, creature: CreatureId) -> Option<(u8, u8)> {
        let def = self.creature(creature);
        (self.animated.family_tier(def.animated.as_ref())).map(|t| (t.levels[0], t.levels[1]))
    }

    /// Names each family by its id (`animated_blade`) for its first member, so that populations
    /// and quests can name it; a creature file of that id is an error.
    pub(super) fn name_families(&mut self) -> Result<(), ContentError> {
        let named: Vec<(String, String)> = (self.animated.families())
            .map(|(f, first)| (f.to_string(), first))
            .collect();
        for (family, first) in named {
            if self.creature_ids.contains_key(&family) {
                return Err(ContentError::new(
                    RULES,
                    format!("family `{family}`: a creature has that id"),
                ));
            }
            if let Some(&id) = self.creature_ids.get(&first) {
                self.creature_ids.insert(family, id);
            }
        }
        Ok(())
    }
}

/// Whether a body file's text defines anything (not a file of notes only).
fn has_body(text: &str) -> bool {
    toml::from_str::<BodyFile>(text).is_ok_and(|f| !f.body.is_empty())
}

/// An animated type's numbers at level L by its tier (the undead roster's formulas, #383, rounded
/// down part by part): the wiki prints them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Numbers {
    pub stats: BTreeMap<Stat, u8>,
    pub base_hp: u16,
    pub dice: &'static str,
}

impl Numbers {
    pub fn at(level: u8, tier: &Tier) -> Self {
        let l = i32::from(level);
        let base = [
            (Stat::Strength, 11 + 4 * l / 10),
            (Stat::Toughness, 12 + 4 * l / 10),
            (Stat::Dexterity, 10 + l / 4),
            (Stat::Willpower, 16 + 35 * l / 100),
            (Stat::Intelligence, 2),
            (Stat::Presence, 2),
        ];
        let stats = (base.into_iter())
            .map(|(stat, score)| {
                let score = score + tier.stats.get(&stat).copied().unwrap_or(0);
                (stat, u8::try_from(score.clamp(1, 40)).unwrap_or(1))
            })
            .collect();
        // 2.5 × L × factor, the factor to a thousandth.
        let factor = (tier.hp_factor * 1000.0).round() as i64;
        let base_hp = u16::try_from(25 * i64::from(level) * factor / 10_000).unwrap_or(u16::MAX);
        // The curve's target, in tenths: 2 + 0.55 L to L20, then 13 + 0.25 (L − 20).
        let target = if level <= 20 {
            20 + 55 * l / 10
        } else {
            130 + 25 * (l - 20) / 10
        };
        let nearest = (0..LADDER.len())
            .min_by_key(|&i| (i32::try_from(LADDER[i].1).unwrap_or(0) - target).abs())
            .unwrap_or(0);
        let last = i32::try_from(LADDER.len()).unwrap_or(1) - 1;
        let at = (i32::try_from(nearest).unwrap_or(0) + tier.dice_steps).clamp(0, last);
        Self {
            stats,
            base_hp,
            dice: LADDER[usize::try_from(at).unwrap_or(0)].0,
        }
    }
}

// Tests: `content/animated_tests.rs`.
