//! Animated objects (spec §8 "Animated objects"; docs/dev/animated.md): `data/animated.toml` (the
//! tiers, the drop rule, what every animated type is) and `data/animated/` (the household bodies,
//! one file each). Each body becomes an ordinary creature type when content loads,
//! `animated_<body>`, at the body's level, its numbers by the tier's formulas: every rule reads
//! it as it reads a kobold. Generation is a pure function of the content (no draws), so the ids,
//! their order, and the fingerprint are the same on every load.

use super::creature::CreatureSpec;
use super::{Content, ContentError, ContentFiles, CreatureId, ObjectId};
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
}

/// What becomes of what an animated thing is made of when it falls (spec §8 "Animated objects").
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Drops {
    /// The chance in percent that a component falls whole; else it breaks into its parts.
    pub intact: u8,
    /// The percent of a broken part's weight that falls as pieces of its material
    /// ([`crate::piece::break_up`]).
    pub pieces: u8,
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

/// A tier of animated things (A1: the household tier, one type per body).
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tier {
    pub id: String,
    /// What the wiki calls it.
    pub name: String,
    /// One type per body file (the household tier); every tier has bodies until blades (A3).
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
    /// The death line's verb of a body whose base material has no group.
    pub dies: String,
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
    /// Its household body, an index into [`Animated::bodies`].
    pub body: usize,
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
        if rules.drops.pieces > 100 {
            return Err(err("drops: `pieces` is a percent, 0 to 100".into()));
        }
        for tier in &rules.tier {
            let [from, to] = tier.levels;
            if !tier.bodies {
                return Err(err(format!(
                    "tier `{}`: only tiers of bodies so far",
                    tier.id
                )));
            }
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
        let household = (rules.tier.iter()).position(|t| t.bodies);
        let mut out = Self {
            drops: rules.drops,
            creature: rules.creature,
            groups: rules.group,
            tiers: rules.tier,
            bodies: Vec::new(),
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

    /// The creature types the bodies generate, in body order, each with the file it came from
    /// and what makes it animated (its family filled in once it is numbered). Their numbers are
    /// the tier's formulas at the body's level ([`Numbers`]).
    pub(super) fn specs(
        &self,
        content: &Content,
    ) -> Result<Vec<(String, CreatureSpec)>, ContentError> {
        let mut out = Vec::new();
        for (index, body) in self.bodies.iter().enumerate() {
            let err =
                |m: String| ContentError::new(&body.path, format!("body `{}`: {m}", body.spec.id));
            let spec = self.spec(index, body, content).map_err(err)?;
            out.push((body.path.clone(), spec));
        }
        Ok(out)
    }

    fn spec(&self, index: usize, body: &Body, content: &Content) -> Result<CreatureSpec, String> {
        let tier = &self.tiers[body.tier];
        let object = content.object(body.object);
        let material = content.material(object.material());
        let group =
            (material.group.as_deref()).and_then(|g| self.groups.iter().find(|x| x.id == g));
        let level = body.spec.level;
        let numbers = Numbers::at(level, tier);
        let mut t = toml::Table::new();
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
        // It weighs what its body weighs, which it carries: of its own, nothing to speak of.
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
        for (stat, score) in numbers.stats {
            stats.insert(stat.id().into(), int(i64::from(score)));
        }
        t.insert("stats".into(), toml::Value::Table(stats));
        t.insert("base_hp".into(), int(i64::from(numbers.base_hp)));
        t.insert("level".into(), int(i64::from(level)));
        let xp = content.experience().guideline().xp(level);
        t.insert("xp".into(), int(i64::from(xp)));
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
        t.insert("corpse".into(), toml::Value::Boolean(false));
        t.insert("blood".into(), s("none"));
        if let Some(light) = &body.spec.light {
            t.insert("light".into(), s(light));
        }
        let immune = (self.creature.immune.iter()).map(|x| s(x)).collect();
        t.insert("immune".into(), toml::Value::Array(immune));
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
            body: index,
        });
        Ok(spec)
    }

    /// The body of animated type `def`, if it has one.
    pub fn body_of(&self, def: Option<&AnimatedDef>) -> Option<&Body> {
        def.and_then(|a| self.bodies.get(a.body))
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
