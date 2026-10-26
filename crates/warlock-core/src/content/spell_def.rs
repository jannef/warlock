//! Spells as data (spec §8 "Spells", the magic design `docs/design/magic.md` §3, §5, §6): the
//! tier table (`data/spells.toml`, [`SpellRules`]) and the spells (`data/spells/`, one
//! `[[spell]]` file each, [`SpellData`]). Not the casters' placeholder spells
//! ([`super::ClassSpell`]) nor the bursts and bolts scrolls and traps cast (`content/spell.rs`):
//! nothing casts these yet (#395 brings the one cast path).
//!
//! - **Tiers**: a spell has a tier, 0 (a cantrip) to 9; each tier has a comfortable caster level,
//!   a base mana cost, and a damage ceiling, all ascending (checked).
//! - **Tags**: a spell's school, kind, shape, and damage type ([`SpellData::tags`]); talents,
//!   abilities, and items will key on them.
//! - **The cap** ([`SpellRules::cap`]): no spell may roll more in one cast than its tier's
//!   ceiling times its resolution's `damage` percent, harm or healing, nor carry a static bonus
//!   above a quarter of the ceiling. A spell over it is refused when the content loads, so no
//!   spell can be written past its cap; nothing in the rules lifts one.

use super::{Content, ContentError, DamageTypeId, MAX_LEVEL, MAX_SEVERITY, StatusId, duplicate_id};
use crate::dice::Dice;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// How many tiers there are: 0 (cantrips) to 9.
pub const TIERS: usize = 10;
/// The farthest a ray, bolt, or burst may reach and a line may run, in cells.
pub const MAX_REACH: u8 = 20;
/// The longest a cone may be, in cells.
pub const MAX_CONE: u8 = 5;
/// The most dice a spell may roll.
pub const MAX_DICE: u8 = 20;
/// The most caster levels a spell's `per_levels` may ask for one die more.
pub const MAX_PER_LEVELS: u8 = 20;
/// The percents a shape's or a resolution's multiplier may be.
const PERCENTS: std::ops::RangeInclusive<u16> = 1..=1000;

/// Index of a spell in [`Content::spells`], in path order (in the content fingerprint).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SpellId(pub u16);

/// A spell's school (magic design §6): a theme, not a tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum School {
    Evocation,
    Necromancy,
    Abjuration,
    Restoration,
    Transmutation,
    Enchantment,
    Divination,
}

impl School {
    /// Its id in data files and as a tag.
    pub const fn id(self) -> &'static str {
        match self {
            School::Evocation => "evocation",
            School::Necromancy => "necromancy",
            School::Abjuration => "abjuration",
            School::Restoration => "restoration",
            School::Transmutation => "transmutation",
            School::Enchantment => "enchantment",
            School::Divination => "divination",
        }
    }
}

/// What a spell is for (its `kind`; magic design §6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpellPurpose {
    Damage,
    Heal,
    Buff,
    Hex,
    Cleanse,
    Utility,
}

impl SpellPurpose {
    /// Its id in data files and as a tag.
    pub const fn id(self) -> &'static str {
        match self {
            SpellPurpose::Damage => "damage",
            SpellPurpose::Heal => "heal",
            SpellPurpose::Buff => "buff",
            SpellPurpose::Hex => "hex",
            SpellPurpose::Cleanse => "cleanse",
            SpellPurpose::Utility => "utility",
        }
    }
}

/// A spell's shape (magic design §5, §6): a single target or an area.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    Ray,
    Bolt,
    Touch,
    #[serde(rename = "self")]
    Myself,
    Cone,
    Line,
    Burst,
}

impl Shape {
    /// Every shape, single targets first, in the order the wiki lists them.
    pub const ALL: &[Shape] = &[
        Shape::Ray,
        Shape::Bolt,
        Shape::Touch,
        Shape::Myself,
        Shape::Cone,
        Shape::Line,
        Shape::Burst,
    ];

    /// Its id in data files and as a tag.
    pub const fn id(self) -> &'static str {
        match self {
            Shape::Ray => "ray",
            Shape::Bolt => "bolt",
            Shape::Touch => "touch",
            Shape::Myself => "self",
            Shape::Cone => "cone",
            Shape::Line => "line",
            Shape::Burst => "burst",
        }
    }

    /// Whether it is an area: everyone in it is a target.
    pub const fn is_area(self) -> bool {
        matches!(self, Shape::Cone | Shape::Line | Shape::Burst)
    }
}

/// How a spell finds its targets (magic design §5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    /// The 2d12 attack roll against Defence: full damage, can critical.
    Attack,
    /// No roll, always hits: less damage.
    Auto,
    /// Everyone in the shape, no roll: less damage per target.
    Area,
    /// An attack roll at adjacency only: more damage, cheaper.
    Touch,
}

impl Resolution {
    /// Every resolution, in the order the wiki lists them.
    pub const ALL: &[Resolution] = &[
        Resolution::Attack,
        Resolution::Auto,
        Resolution::Area,
        Resolution::Touch,
    ];

    /// Its id in data files.
    pub const fn id(self) -> &'static str {
        match self {
            Resolution::Attack => "attack",
            Resolution::Auto => "auto",
            Resolution::Area => "area",
            Resolution::Touch => "touch",
        }
    }

    /// Whether a spell of `shape` may resolve this way: `area` for the area shapes and only
    /// them, `touch` only with the touch shape, `attack` with a ray or a bolt, `auto` with any
    /// single target.
    pub const fn fits(self, shape: Shape) -> bool {
        match self {
            Resolution::Area => shape.is_area(),
            Resolution::Touch => matches!(shape, Shape::Touch),
            Resolution::Attack => matches!(shape, Shape::Ray | Shape::Bolt),
            Resolution::Auto => !shape.is_area(),
        }
    }
}

/// What a resolution does to a spell's cap and cost, in percent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ratio {
    /// The most it may do to one target, of the tier's ceiling.
    pub damage: u16,
    /// What it multiplies the base cost by (an area spell's cost is its shape's).
    pub cost: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesFile {
    overreach_cost: u16,
    tiers: TiersSpec,
    shapes: ShapesSpec,
    resolution: ResolutionSpec,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TiersSpec {
    comfortable: Vec<u8>,
    cost: Vec<u16>,
    ceiling: Vec<u16>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShapesSpec {
    ray: u16,
    bolt: u16,
    touch: u16,
    #[serde(rename = "self")]
    myself: u16,
    cone: u16,
    line: u16,
    burst: Vec<u16>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolutionSpec {
    attack: Ratio,
    auto: Ratio,
    area: Ratio,
    touch: Ratio,
}

/// The tier table (`data/spells.toml`): what every spell is measured by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellRules {
    /// Percent more mana per tier of overreach.
    pub overreach_cost: u16,
    /// Each tier's comfortable caster level, tier 0 first: ascending, the first 1.
    pub comfortable: [u8; TIERS],
    /// Each tier's base mana cost: ascending.
    pub cost: [u16; TIERS],
    /// Each tier's damage ceiling for a single-target spell at its highest roll: ascending.
    pub ceiling: [u16; TIERS],
    /// The single targets' and the areas' cost multipliers, in percent, by [`Shape::ALL`]
    /// order; a burst's is [`SpellRules::burst`]'s.
    shapes: [u16; 6],
    /// A burst's cost multiplier by radius, radius 1 first.
    pub burst: Vec<u16>,
    /// Each resolution's cap and cost, by [`Resolution::ALL`] order.
    resolutions: [Ratio; 4],
}

impl SpellRules {
    pub(super) fn parse(path: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let file: RulesFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let ten = |name: &str, n: usize| {
            (n == TIERS)
                .then_some(())
                .ok_or_else(|| err(format!("tiers: `{name}` has one value per tier, {TIERS}")))
        };
        let t = &file.tiers;
        ten("comfortable", t.comfortable.len())?;
        ten("cost", t.cost.len())?;
        ten("ceiling", t.ceiling.len())?;
        let ascending = |v: &[u16]| v.windows(2).all(|w| w[0] < w[1]);
        let levels: Vec<u16> = t.comfortable.iter().map(|&l| u16::from(l)).collect();
        if levels[0] != 1 || !ascending(&levels) || levels[TIERS - 1] > u16::from(MAX_LEVEL) {
            return Err(err(format!(
                "tiers: `comfortable` levels ascend from 1, at most {MAX_LEVEL}"
            )));
        }
        if t.cost[0] < 1 || !ascending(&t.cost) {
            return Err(err("tiers: `cost` ascends from at least 1".into()));
        }
        if t.ceiling[0] < 1 || !ascending(&t.ceiling) {
            return Err(err("tiers: `ceiling` ascends from at least 1".into()));
        }
        if !PERCENTS.contains(&file.overreach_cost) {
            return Err(err("overreach_cost is 1 to 1000 percent".into()));
        }
        let s = &file.shapes;
        let shapes = [s.ray, s.bolt, s.touch, s.myself, s.cone, s.line];
        let burst_ok = (1..=crate::content::BURST_RADII.end().unsigned_abs() as usize)
            .contains(&s.burst.len());
        if !shapes.iter().chain(&s.burst).all(|p| PERCENTS.contains(p))
            || !burst_ok
            || !ascending(&s.burst)
        {
            return Err(err(format!(
                "shapes: each is 1 to 1000 percent; `burst` one ascending value per radius, 1 \
                 to {}",
                crate::content::BURST_RADII.end()
            )));
        }
        let r = &file.resolution;
        let resolutions = [r.attack, r.auto, r.area, r.touch];
        if !(resolutions.iter()).all(|x| PERCENTS.contains(&x.damage) && PERCENTS.contains(&x.cost))
        {
            return Err(err(
                "resolution: `damage` and `cost` are 1 to 1000 percent".into()
            ));
        }
        let array = |v: &[u16]| -> [u16; TIERS] { v.try_into().expect("checked length") };
        Ok(SpellRules {
            overreach_cost: file.overreach_cost,
            comfortable: t.comfortable.clone().try_into().expect("checked length"),
            cost: array(&t.cost),
            ceiling: array(&t.ceiling),
            shapes,
            burst: s.burst.clone(),
            resolutions,
        })
    }

    /// The tier a caster level is at: the highest whose comfortable level it has reached (tier
    /// 0 below the first). A spell above it is overreach.
    pub fn tier_of(&self, caster_level: i32) -> u8 {
        let reached = (self.comfortable.iter())
            .take_while(|&&l| i32::from(l) <= caster_level)
            .count();
        u8::try_from(reached.saturating_sub(1)).unwrap_or(0)
    }

    /// What `shape` multiplies the base cost by, in percent; a burst's at `radius` (its largest
    /// beyond the table).
    pub fn shape_cost(&self, shape: Shape, radius: u8) -> u16 {
        match shape {
            Shape::Burst => {
                let i = usize::from(radius.max(1)) - 1;
                self.burst[i.min(self.burst.len() - 1)]
            }
            _ => {
                let i = Shape::ALL.iter().position(|&s| s == shape).unwrap_or(0);
                self.shapes[i]
            }
        }
    }

    /// The largest radius a burst may have.
    pub fn max_radius(&self) -> u8 {
        u8::try_from(self.burst.len()).unwrap_or(u8::MAX)
    }

    /// What `resolution` does to a spell's cap and cost.
    pub fn ratio(&self, resolution: Resolution) -> Ratio {
        let i = (Resolution::ALL.iter()).position(|&r| r == resolution);
        self.resolutions[i.unwrap_or(0)]
    }

    /// The most a spell of `tier` that resolves as `resolution` may roll in one cast against
    /// one target: the tier's ceiling times the resolution's `damage` percent, rounded down.
    pub fn cap(&self, tier: u8, resolution: Resolution) -> i32 {
        let ceiling = i32::from(self.ceiling[usize::from(tier).min(TIERS - 1)]);
        ceiling * i32::from(self.ratio(resolution).damage) / 100
    }

    /// The largest static bonus a spell of `tier` may carry: a quarter of its ceiling.
    pub fn max_bonus(&self, tier: u8) -> i32 {
        i32::from(self.ceiling[usize::from(tier).min(TIERS - 1)]) / 4
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpellFile {
    #[serde(default)]
    spell: Vec<SpellSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpellSpec {
    id: String,
    name: String,
    description: String,
    tier: u8,
    school: School,
    kind: SpellPurpose,
    shape: Shape,
    resolution: Resolution,
    damage_type: Option<String>,
    max_range: Option<u8>,
    max_radius: Option<u8>,
    max_length: Option<u8>,
    damage: Option<DiceSpec>,
    effect: Option<EffectSpec>,
    scroll: Option<ScrollSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiceSpec {
    dice: Dice,
    #[serde(default)]
    per_levels: u8,
    max_dice: Option<u8>,
    #[serde(default)]
    bonus: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EffectSpec {
    heal: Option<DiceSpec>,
    status: Option<String>,
    severity: Option<u16>,
    #[serde(default)]
    cures: Vec<String>,
    #[serde(default)]
    diseases: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScrollSpec {
    level: Option<u8>,
}

/// What a spell rolls, harm or healing: dice that grow with caster level up to a cap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellDice {
    /// Dice at the tier's comfortable caster level.
    pub count: u8,
    /// Their sides.
    pub sides: u8,
    /// One die more per this many caster levels above the comfortable one (0: never more).
    pub per_levels: u8,
    /// The cap: never more dice than this.
    pub max_dice: u8,
    /// A static bonus, rare.
    pub bonus: u8,
}

impl SpellDice {
    /// The most it can roll: every die of the cap at its highest, plus the bonus.
    pub const fn max(&self) -> i32 {
        self.max_dice as i32 * self.sides as i32 + self.bonus as i32
    }

    /// What it rolls at `caster_level` for a spell whose tier is comfortable at `comfortable`:
    /// its dice, one more per `per_levels` levels above that, never more than `max_dice`.
    pub fn at(&self, caster_level: i32, comfortable: u8) -> Dice {
        let above = (caster_level - i32::from(comfortable)).max(0);
        let more = match self.per_levels {
            0 => 0,
            n => above / i32::from(n),
        };
        let count = (i32::from(self.count) + more).min(i32::from(self.max_dice));
        Dice::of(count.unsigned_abs(), u32::from(self.sides)).plus(i32::from(self.bonus))
    }
}

/// What a spell of a kind other than `damage` does (`[spell.effect]`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpellEffect {
    /// A `heal`'s hit points.
    pub heal: Option<SpellDice>,
    /// A `buff`'s status on the caster, a `hex`'s on its target, with its severity.
    pub status: Option<(StatusId, u16)>,
    /// The statuses a `cleanse` ends.
    pub cures: Vec<StatusId>,
    /// Whether a `cleanse` ends every disease.
    pub diseases: bool,
}

/// A spell (`data/spells/`), in path order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellData {
    pub id: String,
    pub name: String,
    /// One line.
    pub description: String,
    /// 0 (a cantrip) to 9.
    pub tier: u8,
    pub school: School,
    pub kind: SpellPurpose,
    pub shape: Shape,
    pub resolution: Resolution,
    /// The kind of harm a `damage` spell does (a tag too).
    pub damage_type: Option<DamageTypeId>,
    /// The farthest a ray, bolt, or burst reaches (a burst: its centre).
    pub max_range: Option<u8>,
    /// A burst's largest radius.
    pub max_radius: Option<u8>,
    /// How far a cone or a line reaches from the caster.
    pub max_length: Option<u8>,
    /// A `damage` spell's damage.
    pub damage: Option<SpellDice>,
    /// What a spell of another kind does.
    pub effect: SpellEffect,
    /// The caster level a read of its scroll casts it at, if it comes on a scroll.
    pub scroll_level: Option<u8>,
}

impl SpellData {
    /// Its tags (magic design §6): its school, kind, shape, and damage type, if any.
    pub fn tags<'c>(&self, content: &'c Content) -> Vec<&'c str> {
        let mut tags = vec![self.school.id(), self.kind.id(), self.shape.id()];
        if let Some(kind) = self.damage_type {
            tags.push(content.damage_type(kind).id.as_str());
        }
        tags
    }
}

/// Reads `data/spells.toml` and the spells of `data/spells/` (`files`, the table in path order,
/// the tier table's own file left out), checking each spell against the tier table: none
/// without the tier table, and a spell without it is an error.
pub(super) fn parse<'a>(
    rules: Option<(&str, &str)>,
    files: impl IntoIterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<(Option<SpellRules>, Vec<SpellData>), ContentError> {
    let rules = match rules {
        Some((path, text)) => Some(SpellRules::parse(path, text)?),
        None => None,
    };
    let mut list = Vec::new();
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for (file, text) in files {
        let err = |m: String| ContentError::new(file, m);
        let parsed: SpellFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for spec in parsed.spell {
            let Some(rules) = &rules else {
                return Err(err("spells need data/spells.toml".into()));
            };
            if let Some(first) = seen.insert(spec.id.clone(), file) {
                return Err(duplicate_id("spell", &spec.id, file, first));
            }
            let id = spec.id.clone();
            list.push(
                resolve(spec, rules, content).map_err(|m| err(format!("spell `{id}`: {m}")))?,
            );
        }
    }
    if list.len() > usize::from(u16::MAX) {
        return Err(ContentError::new("data/spells/", "too many spells"));
    }
    Ok((rules, list))
}

/// Checks a dice table of a spell of `tier` resolving as `resolution` against the cap.
fn dice(
    spec: &DiceSpec,
    tier: u8,
    resolution: Resolution,
    rules: &SpellRules,
) -> Result<SpellDice, String> {
    let d = spec.dice;
    if d.is_fixed() || d.is_negative() || d.bonus() != 0 {
        return Err(format!("`dice` is dice only (`1d6`), not `{d}`"));
    }
    let count = u8::try_from(d.count()).ok().filter(|c| *c <= MAX_DICE);
    let sides = u8::try_from(d.sides()).ok();
    let (Some(count), Some(sides)) = (count, sides) else {
        return Err(format!("at most {MAX_DICE} dice of at most 255 sides"));
    };
    let max_dice = spec.max_dice.unwrap_or(count);
    if !(count..=MAX_DICE).contains(&max_dice) {
        return Err(format!("`max_dice` is {count} (its dice) to {MAX_DICE}"));
    }
    if spec.per_levels > MAX_PER_LEVELS {
        return Err(format!("`per_levels` is 0 to {MAX_PER_LEVELS}"));
    }
    let rolled = SpellDice {
        count,
        sides,
        per_levels: spec.per_levels,
        max_dice,
        bonus: spec.bonus,
    };
    let most_bonus = rules.max_bonus(tier);
    if i32::from(spec.bonus) > most_bonus {
        return Err(format!(
            "a bonus of {} is more than a quarter of tier {tier}'s ceiling ({most_bonus})",
            spec.bonus
        ));
    }
    let cap = rules.cap(tier, resolution);
    if rolled.max() > cap {
        return Err(format!(
            "it may roll {} in one cast, over tier {tier}'s cap of {cap} for `{}` (the \
             ceiling {} × {} %)",
            rolled.max(),
            resolution.id(),
            rules.ceiling[usize::from(tier)],
            rules.ratio(resolution).damage
        ));
    }
    Ok(rolled)
}

/// One spell, checked.
fn resolve(spec: SpellSpec, rules: &SpellRules, content: &Content) -> Result<SpellData, String> {
    let tier = spec.tier;
    if usize::from(tier) >= TIERS {
        return Err(format!("tier is 0 to {}", TIERS - 1));
    }
    if !spec.resolution.fits(spec.shape) {
        return Err(format!(
            "a `{}` cannot resolve as `{}`",
            spec.shape.id(),
            spec.resolution.id()
        ));
    }
    // The caps of its reach: each shape has exactly its own.
    let reach = |name: &str, value: Option<u8>, wanted: bool, most: u8| match (value, wanted) {
        (Some(v), true) if (1..=most).contains(&v) => Ok(()),
        (Some(_), true) => Err(format!("`{name}` is 1 to {most}")),
        (None, true) => Err(format!("a `{}` needs `{name}`", spec.shape.id())),
        (Some(_), false) => Err(format!("a `{}` has no `{name}`", spec.shape.id())),
        (None, false) => Ok(()),
    };
    let shape = spec.shape;
    let ranged = matches!(shape, Shape::Ray | Shape::Bolt | Shape::Burst);
    reach("max_range", spec.max_range, ranged, MAX_REACH)?;
    let burst = shape == Shape::Burst;
    reach("max_radius", spec.max_radius, burst, rules.max_radius())?;
    let long = matches!(shape, Shape::Cone | Shape::Line);
    let most = if shape == Shape::Cone {
        MAX_CONE
    } else {
        MAX_REACH
    };
    reach("max_length", spec.max_length, long, most)?;
    let damage_type = match &spec.damage_type {
        None => None,
        Some(name) => Some(
            content
                .damage_type_id(name)
                .ok_or_else(|| format!("unknown damage type `{name}`"))?,
        ),
    };
    let damage = match (&spec.damage, spec.kind) {
        (Some(d), SpellPurpose::Damage) => Some(dice(d, tier, spec.resolution, rules)?),
        (None, SpellPurpose::Damage) => return Err("a `damage` spell needs [spell.damage]".into()),
        (Some(_), _) => return Err("only a `damage` spell has [spell.damage]".into()),
        (None, _) => None,
    };
    if spec.kind == SpellPurpose::Damage && damage_type.is_none() {
        return Err("a `damage` spell needs a `damage_type`".into());
    }
    let effect = match spec.effect {
        Some(_) if spec.kind == SpellPurpose::Damage => {
            return Err("a `damage` spell has no [spell.effect]".into());
        }
        Some(e) => effect(e, tier, (spec.kind, spec.resolution), rules, content)?,
        None => SpellEffect::default(),
    };
    let needs = match spec.kind {
        SpellPurpose::Heal => effect.heal.is_none().then_some("`heal`"),
        SpellPurpose::Buff | SpellPurpose::Hex => effect.status.is_none().then_some("`status`"),
        SpellPurpose::Cleanse => {
            (effect.cures.is_empty() && !effect.diseases).then_some("`cures` or `diseases`")
        }
        SpellPurpose::Damage | SpellPurpose::Utility => None,
    };
    if let Some(what) = needs {
        return Err(format!(
            "a `{}` spell needs {what} in [spell.effect]",
            spec.kind.id()
        ));
    }
    let scroll_level = match spec.scroll {
        None => None,
        Some(s) => {
            let level = s.level.unwrap_or(rules.comfortable[usize::from(tier)]);
            if !(1..=MAX_LEVEL).contains(&level) {
                return Err(format!("scroll: `level` is 1 to {MAX_LEVEL}"));
            }
            Some(level)
        }
    };
    Ok(SpellData {
        id: spec.id,
        name: spec.name,
        description: spec.description,
        tier,
        school: spec.school,
        kind: spec.kind,
        shape,
        resolution: spec.resolution,
        damage_type,
        max_range: spec.max_range,
        max_radius: spec.max_radius,
        max_length: spec.max_length,
        damage,
        effect,
        scroll_level,
    })
}

/// A spell's `[spell.effect]`, checked: only what its kind uses, and a heal under the cap.
fn effect(
    e: EffectSpec,
    tier: u8,
    (kind, resolution): (SpellPurpose, Resolution),
    rules: &SpellRules,
    content: &Content,
) -> Result<SpellEffect, String> {
    let only = |field: &str, present: bool, kinds: &[SpellPurpose]| {
        if present && !kinds.contains(&kind) {
            Err(format!("a `{}` spell has no `{field}`", kind.id()))
        } else {
            Ok(())
        }
    };
    only("heal", e.heal.is_some(), &[SpellPurpose::Heal])?;
    let marks = [SpellPurpose::Buff, SpellPurpose::Hex];
    only("status", e.status.is_some(), &marks)?;
    only("severity", e.severity.is_some(), &marks)?;
    only("cures", !e.cures.is_empty(), &[SpellPurpose::Cleanse])?;
    only("diseases", e.diseases, &[SpellPurpose::Cleanse])?;
    let status = |name: &str| {
        content
            .status_id(name)
            .ok_or_else(|| format!("unknown status `{name}`"))
    };
    let heal = match &e.heal {
        Some(h) => Some(dice(h, tier, resolution, rules)?),
        None => None,
    };
    let marked = match &e.status {
        Some(name) => {
            let severity = e.severity.unwrap_or(1);
            if !(1..=MAX_SEVERITY).contains(&severity) {
                return Err(format!("`severity` is 1 to {MAX_SEVERITY}"));
            }
            Some((status(name)?, severity))
        }
        None => None,
    };
    let cures = (e.cures.iter())
        .map(|name| status(name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SpellEffect {
        heal,
        status: marked,
        cures,
        diseases: e.diseases,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHIPPED: &str = include_str!("../../../../assets/data/spells.toml");

    fn rules() -> SpellRules {
        SpellRules::parse("data/spells.toml", SHIPPED).expect("valid")
    }

    /// Parses spell files `text` against the shipped tier table and the built-in content.
    fn spells(text: &str) -> Result<Vec<SpellData>, ContentError> {
        let content = Content::builtin().expect("content");
        let (_, list) = parse(Some(("data/spells.toml", SHIPPED)), [("s", text)], &content)?;
        Ok(list)
    }

    /// The magic design's first spell list (§7), as data: every one passes the cap check.
    const FIRST_LIST: &str = r#"
[[spell]]
id = "ray_of_frost"
name = "Ray of frost"
description = "A thin beam of cold."
tier = 0
school = "evocation"
kind = "damage"
shape = "ray"
resolution = "attack"
damage_type = "cold"
max_range = 6
damage = { dice = "1d4", per_levels = 3, max_dice = 1 }
scroll = {}

[[spell]]
id = "spark"
name = "Spark"
description = "A snap of lightning."
tier = 0
school = "evocation"
kind = "damage"
shape = "bolt"
resolution = "auto"
damage_type = "electricity"
max_range = 5
damage = { dice = "1d4" }

[[spell]]
id = "mend"
name = "Mend"
description = "Closes a scratch."
tier = 0
school = "restoration"
kind = "heal"
shape = "touch"
resolution = "touch"
effect = { heal = { dice = "1d3" } }

[[spell]]
id = "magic_missile"
name = "Magic missile"
description = "A dart of force that never misses."
tier = 1
school = "evocation"
kind = "damage"
shape = "bolt"
resolution = "auto"
damage_type = "force"
max_range = 8
damage = { dice = "1d4", per_levels = 3, max_dice = 2 }

[[spell]]
id = "burning_hands"
name = "Burning hands"
description = "A fan of flame."
tier = 1
school = "evocation"
kind = "damage"
shape = "cone"
resolution = "area"
damage_type = "fire"
max_length = 3
damage = { dice = "1d4", per_levels = 3, max_dice = 2 }

[[spell]]
id = "cause_wounds"
name = "Cause wounds"
description = "A touch that opens wounds."
tier = 1
school = "necromancy"
kind = "damage"
shape = "touch"
resolution = "touch"
damage_type = "necrotic"
damage = { dice = "1d6", per_levels = 4, max_dice = 2 }

[[spell]]
id = "cure_wounds"
name = "Cure wounds"
description = "Heals."
tier = 1
school = "restoration"
kind = "heal"
shape = "touch"
resolution = "touch"
effect = { heal = { dice = "2d6" } }

[[spell]]
id = "bless"
name = "Bless"
description = "A boon."
tier = 1
school = "abjuration"
kind = "buff"
shape = "self"
resolution = "auto"
effect = { status = "blessed" }

[[spell]]
id = "scorching_ray"
name = "Scorching ray"
description = "Rays of fire."
tier = 2
school = "evocation"
kind = "damage"
shape = "ray"
resolution = "attack"
damage_type = "fire"
max_range = 8
damage = { dice = "1d6", per_levels = 6, max_dice = 3 }

[[spell]]
id = "fireball"
name = "Fireball"
description = "A ball of fire."
tier = 3
school = "evocation"
kind = "damage"
shape = "burst"
resolution = "area"
damage_type = "fire"
max_range = 8
max_radius = 2
damage = { dice = "2d6", per_levels = 4, max_dice = 3 }

[[spell]]
id = "lightning_bolt"
name = "Lightning bolt"
description = "A stroke of lightning."
tier = 3
school = "evocation"
kind = "damage"
shape = "line"
resolution = "area"
damage_type = "electricity"
max_length = 12
damage = { dice = "2d6", per_levels = 4, max_dice = 3 }
"#;

    #[test]
    fn the_shipped_tier_table_reads_as_the_design_has_it() {
        let r = rules();
        assert_eq!(r.comfortable, [1, 3, 6, 12, 18, 25, 32, 40, 50, 60]);
        assert_eq!(r.cost, [1, 3, 6, 11, 16, 22, 29, 37, 46, 56]);
        assert_eq!(r.ceiling, [6, 12, 18, 24, 32, 40, 50, 62, 76, 90]);
        assert_eq!(r.overreach_cost, 50);
        assert_eq!(
            (r.shape_cost(Shape::Bolt, 0), r.shape_cost(Shape::Cone, 0)),
            (100, 160)
        );
        assert_eq!(
            (1..=4)
                .map(|n| r.shape_cost(Shape::Burst, n))
                .collect::<Vec<_>>(),
            [180, 240, 300, 300],
            "the largest beyond the table"
        );
        assert_eq!(r.max_radius(), 3);
        assert_eq!(
            r.ratio(Resolution::Touch),
            Ratio {
                damage: 120,
                cost: 70
            }
        );
        // A tier-3 burst may do at most 19 (24 × 80 %), a tier-1 auto 9 (12 × 75 %).
        assert_eq!(r.cap(3, Resolution::Area), 19);
        assert_eq!(r.cap(1, Resolution::Auto), 9);
        assert_eq!(r.max_bonus(9), 22);
    }

    #[test]
    fn a_caster_level_is_at_the_highest_tier_it_has_reached() {
        let r = rules();
        let tiers: Vec<u8> = [0, 1, 2, 3, 5, 6, 12, 59, 60, 99]
            .iter()
            .map(|&l| r.tier_of(l))
            .collect();
        assert_eq!(tiers, [0, 0, 0, 1, 1, 2, 3, 8, 9, 9]);
    }

    #[test]
    fn the_tier_table_is_refused_unless_it_ascends() {
        for (from, to, why) in [
            (
                "ceiling = [6, 12, 18, 24, 32, 40, 50, 62, 76, 90]",
                "ceiling = [6, 12, 18, 24, 32, 30, 50, 62, 76, 90]",
                "`ceiling` ascends",
            ),
            (
                "cost = [1, 3, 6, 11, 16, 22, 29, 37, 46, 56]",
                "cost = [1, 3, 6, 11, 16, 22, 29, 37, 46]",
                "one value per tier",
            ),
            (
                "comfortable = [1, 3, 6, 12, 18, 25, 32, 40, 50, 60]",
                "comfortable = [2, 3, 6, 12, 18, 25, 32, 40, 50, 60]",
                "ascend from 1",
            ),
            (
                "burst = [180, 240, 300]",
                "burst = [180, 300, 240]",
                "one ascending value per radius",
            ),
            (
                "overreach_cost = 50",
                "overreach_cost = 0",
                "overreach_cost",
            ),
            (
                "touch = { damage = 120, cost = 70 }",
                "touch = { damage = 120, cost = 0 }",
                "resolution",
            ),
        ] {
            let text = SHIPPED.replace(from, to);
            assert_ne!(text, SHIPPED, "{from}");
            let e = SpellRules::parse("x", &text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }

    #[test]
    fn the_designs_first_spell_list_passes_the_cap_check() {
        let list = spells(FIRST_LIST).expect("every spell under its cap");
        assert_eq!(list.len(), 11);
        let content = Content::builtin().expect("content");
        let fireball = list.iter().find(|s| s.id == "fireball").expect("fireball");
        assert_eq!(
            fireball.tags(&content),
            ["evocation", "damage", "burst", "fire"]
        );
        assert_eq!(fireball.damage.expect("damage").max(), 18);
        assert_eq!(
            list[0].scroll_level,
            Some(1),
            "the tier's comfortable level"
        );
        assert_eq!(list[1].scroll_level, None, "no scroll block");
        let bless = list.iter().find(|s| s.id == "bless").expect("bless");
        assert_eq!(
            bless.effect.status,
            Some((content.status_id("blessed").expect("blessed"), 1))
        );
    }

    #[test]
    fn a_spell_over_its_cap_is_refused() {
        let fireball = "max_dice = 3 }\n\n[[spell]]\nid = \"lightning_bolt\"";
        // A tier-3 fireball of 4d6 could roll 24, over the area cap of 19.
        let over = FIRST_LIST.replace(fireball, &fireball.replace("3 }", "4 }"));
        assert_ne!(over, FIRST_LIST);
        let e = spells(&over).unwrap_err();
        assert!(
            e.message
                .contains("spell `fireball`: it may roll 24 in one cast, over tier 3's cap of 19"),
            "{}",
            e.message
        );
        // The same 3d6 with a bonus of 2 is 20: over, though the bonus itself is small.
        let over = FIRST_LIST.replace(fireball, &fireball.replace("3 }", "3, bonus = 2 }"));
        let e = spells(&over).unwrap_err();
        assert!(e.message.contains("may roll 20"), "{}", e.message);
        // Healing obeys the cap too: 3d6 is over a tier-1 touch's 14.
        let over = FIRST_LIST.replace("heal = { dice = \"2d6\" }", "heal = { dice = \"3d6\" }");
        let e = spells(&over).unwrap_err();
        assert!(e.message.contains("spell `cure_wounds`"), "{}", e.message);
    }

    #[test]
    fn a_static_bonus_above_a_quarter_of_the_ceiling_is_refused() {
        // Tier 9: ceiling 90, so a bonus of 22 is allowed and 23 is not, whatever the dice.
        let spell = |bonus: u8| {
            format!(
                "[[spell]]\nid = \"doom\"\nname = \"Doom\"\ndescription = \"x\"\ntier = 9\n\
                 school = \"evocation\"\nkind = \"damage\"\nshape = \"ray\"\n\
                 resolution = \"attack\"\ndamage_type = \"fire\"\nmax_range = 8\n\
                 damage = {{ dice = \"1d4\", bonus = {bonus} }}\n"
            )
        };
        assert!(spells(&spell(22)).is_ok());
        let e = spells(&spell(23)).unwrap_err();
        assert!(
            e.message
                .contains("a bonus of 23 is more than a quarter of tier 9's ceiling (22)"),
            "{}",
            e.message
        );
    }

    #[test]
    fn malformed_spells_are_refused() {
        let fireball = |body: &str| {
            format!(
                "[[spell]]\nid = \"fireball\"\nname = \"Fireball\"\ndescription = \"x\"\n\
                 school = \"evocation\"\nkind = \"damage\"\n{body}\n"
            )
        };
        let good = "tier = 3\nshape = \"burst\"\nresolution = \"area\"\ndamage_type = \"fire\"\n\
                    max_range = 8\nmax_radius = 2\ndamage = { dice = \"2d6\" }";
        assert!(spells(&fireball(good)).is_ok());
        for (from, to, why) in [
            (
                "resolution = \"area\"",
                "resolution = \"attack\"",
                "cannot resolve as",
            ),
            ("max_radius = 2", "max_radius = 4", "`max_radius` is 1 to 3"),
            ("max_range = 8\n", "", "needs `max_range`"),
            (
                "max_radius = 2",
                "max_radius = 2\nmax_length = 3",
                "has no `max_length`",
            ),
            ("damage_type = \"fire\"\n", "", "needs a `damage_type`"),
            (
                "damage_type = \"fire\"",
                "damage_type = \"holy\"",
                "unknown damage type",
            ),
            ("\"2d6\"", "\"2d6+1\"", "dice only"),
            (
                "{ dice = \"2d6\" }",
                "{ dice = \"2d6\", max_dice = 1 }",
                "`max_dice` is 2",
            ),
            ("tier = 3", "tier = 10", "tier is 0 to 9"),
            (
                "damage = { dice = \"2d6\" }",
                "damage = { dice = \"2d6\" }\neffect = { diseases = true }",
                "no [spell.effect]",
            ),
        ] {
            let body = good.replace(from, to);
            assert_ne!(body, good, "{from}");
            let e = spells(&fireball(&body)).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
        let e = spells(&format!("{}{}", fireball(good), fireball(good))).unwrap_err();
        assert!(
            e.message.contains("duplicate spell id `fireball`"),
            "{}",
            e.message
        );
        let content = Content::builtin().expect("content");
        let e = parse(None, [("s", fireball(good).as_str())], &content).unwrap_err();
        assert!(e.message.contains("need data/spells.toml"), "{}", e.message);
    }

    #[test]
    fn dice_grow_with_caster_level_and_stop_at_the_cap() {
        let d = SpellDice {
            count: 2,
            sides: 6,
            per_levels: 4,
            max_dice: 3,
            bonus: 0,
        };
        // Fireball, comfortable at 12: 2d6 up to 15, 3d6 from 16 on, never more.
        let at = |level| d.at(level, 12).to_string();
        assert_eq!(
            [at(1), at(12), at(15), at(16), at(60)],
            ["2d6", "2d6", "2d6", "3d6", "3d6"]
        );
        let fixed = SpellDice { per_levels: 0, ..d };
        assert_eq!(fixed.at(60, 12).to_string(), "2d6");
    }
}
