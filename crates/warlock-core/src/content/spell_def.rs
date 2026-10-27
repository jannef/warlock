//! Spells as data (spec §8 "Spells", the magic design `docs/design/magic.md` §3, §5, §6): the
//! tier table (`data/spells.toml`, [`SpellRules`]) and the spells (`data/spells/`, one
//! `[[spell]]` file each, [`SpellData`]), cast by the one cast path (`world/cast.rs`). Not the
//! bursts and bolts scrolls and traps cast (`content/spell.rs`), whose code the cast path shares.
//!
//! - **Tiers**: a spell has a tier, 0 (a cantrip) to 9; each tier has a comfortable caster level,
//!   a base mana cost, and a damage ceiling, all ascending (checked).
//! - **Tags**: a spell's school, kind, shape, and damage type ([`SpellData::tags`]); talents,
//!   abilities, and items will key on them.
//! - **The cap** ([`SpellRules::cap`]): no spell may roll more in one cast than its tier's
//!   ceiling times its resolution's `damage` percent, harm or healing, nor carry a static bonus
//!   above a quarter of the ceiling. A spell over it is refused when the content loads, so no
//!   spell can be written past its cap; nothing in the rules lifts one.

use super::effect::EffectShape;
use super::spell::{Burst, BurstIce, SpellFields, check_effect, default_verb};
use super::trap::TrapNoise;
use super::{Content, ContentError, DamageTypeId, MAX_LEVEL, MAX_SEVERITY, StatusId, duplicate_id};
use crate::dice::Dice;
use crate::noise::MAX_LOUDNESS;
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
/// The most `casts` a spell may take.
pub const MAX_CASTS: u8 = 10;
/// A stat step's default and its largest (`reach_step`, `presence_step`).
const DEFAULT_STEP: u8 = 2;
const MAX_STEP: u8 = 10;
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
    /// Every school, in the design's order (§6).
    pub const ALL: [School; 7] = [
        School::Evocation,
        School::Necromancy,
        School::Abjuration,
        School::Restoration,
        School::Transmutation,
        School::Enchantment,
        School::Divination,
    ];

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
    reach_step: Option<u8>,
    presence_step: Option<u8>,
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
    wand_charges: Vec<u8>,
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
    /// Willpower modifier points per cell a spell reaches farther, or its area grows (magic
    /// design §3), never past the spell's own caps.
    pub reach_step: u8,
    /// Presence modifier points per degree more of a buff's or a hex's status.
    pub presence_step: u8,
    /// Each tier's comfortable caster level, tier 0 first: ascending, the first 1.
    pub comfortable: [u8; TIERS],
    /// Each tier's base mana cost: ascending.
    pub cost: [u16; TIERS],
    /// Each tier's damage ceiling for a single-target spell at its highest roll: ascending.
    pub ceiling: [u16; TIERS],
    /// The charges a wand of each tier's spell is made with (spec §8 "Magic items": "Wands"):
    /// 1 to [`super::MAX_WAND_CHARGES`], never more than the tier below's.
    pub wand_charges: [u8; TIERS],
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
        ten("wand_charges", t.wand_charges.len())?;
        let charges = &t.wand_charges;
        if !(charges.iter()).all(|c| (1..=super::MAX_WAND_CHARGES).contains(c))
            || charges.windows(2).any(|w| w[0] < w[1])
        {
            return Err(err(format!(
                "tiers: `wand_charges` are 1 to {}, never more than the tier below's",
                super::MAX_WAND_CHARGES
            )));
        }
        if !PERCENTS.contains(&file.overreach_cost) {
            return Err(err("overreach_cost is 1 to 1000 percent".into()));
        }
        let (reach_step, presence_step) = (
            file.reach_step.unwrap_or(DEFAULT_STEP),
            file.presence_step.unwrap_or(DEFAULT_STEP),
        );
        if !(1..=MAX_STEP).contains(&reach_step) || !(1..=MAX_STEP).contains(&presence_step) {
            return Err(err(format!(
                "`reach_step` and `presence_step` are 1 to {MAX_STEP}"
            )));
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
            reach_step,
            presence_step,
            comfortable: t.comfortable.clone().try_into().expect("checked length"),
            cost: array(&t.cost),
            ceiling: array(&t.ceiling),
            wand_charges: t.wand_charges.clone().try_into().expect("checked length"),
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

    /// What `spell` costs a caster at `caster_level` wearing armour that adds `armour` percent
    /// (magic design §5), before its talents: the tier's base cost × overreach (1 +
    /// `overreach_cost` % per tier above the caster level's) × its shape's percent (a burst's
    /// at its own `radius`) × its resolution's `cost` percent × (100 + `armour`) %, rounded to
    /// the nearest once at the end, at least 1.
    pub fn mana(&self, spell: &SpellData, caster_level: i32, armour: u32) -> i32 {
        self.mana_with(spell, caster_level, armour, 0)
    }

    /// [`SpellRules::mana`] for a caster whose talents make overreach cost `overreach_percent`
    /// percent more (negative: less; Overreach): the `overreach_cost` per tier scaled by it,
    /// never below 0.
    pub fn mana_with(
        &self,
        spell: &SpellData,
        caster_level: i32,
        armour: u32,
        overreach_percent: i32,
    ) -> i32 {
        let tier = usize::from(spell.tier).min(TIERS - 1);
        let over = u64::from(spell.tier.saturating_sub(self.tier_of(caster_level)));
        let radius = spell.radius.unwrap_or(1);
        let per_tier = i64::from(self.overreach_cost) * i64::from(100 + overreach_percent).max(0);
        let factors = [
            10_000 + u64::try_from(per_tier).unwrap_or(0) * over,
            u64::from(self.shape_cost(spell.shape, radius)),
            u64::from(self.ratio(spell.resolution).cost),
            100 + u64::from(armour),
        ];
        let num = factors.iter().product::<u64>() * u64::from(self.cost[tier]);
        let den = 100u64.pow(5);
        i32::try_from((num + den / 2) / den)
            .unwrap_or(i32::MAX)
            .max(1)
    }

    /// A reach of `base` cells grown by a Willpower `modifier` (magic design §3): a cell per
    /// `reach_step` points of a positive modifier, never past `cap`.
    pub fn grown(&self, base: i32, cap: i32, modifier: i32) -> i32 {
        (base + modifier.max(0) / i32::from(self.reach_step)).min(cap.max(base))
    }

    /// The degrees of a buff's or a hex's status, `severity` lengthened by a Presence
    /// `modifier`: a degree per `presence_step` points of a positive modifier (the status's own
    /// `max` holds it when it is given).
    pub fn lengthened(&self, severity: u16, modifier: i32) -> u16 {
        let more = u16::try_from(modifier.max(0) / i32::from(self.presence_step)).unwrap_or(0);
        severity.saturating_add(more).min(MAX_SEVERITY)
    }

    /// The most `spell` may do to one target in one cast: its tier's cap for its resolution.
    /// Whatever rolls it (caster level, talents, a critical), nothing does more.
    pub fn cap_of(&self, spell: &SpellData) -> i32 {
        self.cap(spell.tier, spell.resolution)
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
    range: Option<u8>,
    radius: Option<u8>,
    length: Option<u8>,
    casts: Option<u8>,
    #[serde(default)]
    open_sky: bool,
    damage: Option<DiceSpec>,
    effect: Option<EffectSpec>,
    cast: Option<CastSpec>,
    scroll: Option<ScrollSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CastSpec {
    effect: Option<String>,
    noun: Option<String>,
    verb: Option<String>,
    message: Option<String>,
    noise: Option<TrapNoise>,
    blast: Option<u16>,
    #[serde(default)]
    heat: bool,
    frost: Option<u16>,
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

/// How a spell shows and sounds, and what its area does to walls and ice (`[spell.cast]`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpellLook {
    /// The effect presentation shows (`data/effects.toml`): a burst's for a burst or a cone,
    /// a bolt's for a ray, a bolt, or a line. None: nothing is drawn.
    pub effect: Option<String>,
    /// What hits ("fireball") and what it does ("burns"), for messages.
    pub noun: String,
    pub verb: String,
    /// Said as it is cast.
    pub message: Option<String>,
    /// The noise it makes where it goes off (a burst's centre, else the caster).
    pub noise: Option<TrapNoise>,
    /// A burst's blast against walls (0: none).
    pub blast: u16,
    /// What a burst does to ice and water.
    pub ice: Option<BurstIce>,
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
    /// Its reach before Willpower (magic design §3): a ray's, bolt's, or burst's range, a
    /// burst's radius, a cone's or a line's length; each at most its cap above (default it).
    pub range: Option<u8>,
    pub radius: Option<u8>,
    pub length: Option<u8>,
    /// Its time, in `cast` actions (1 to [`MAX_CASTS`]).
    pub casts: u8,
    /// It is cast only under the open sky (call lightning).
    pub open_sky: bool,
    /// How it shows and sounds.
    pub look: SpellLook,
    /// A `damage` spell's damage.
    pub damage: Option<SpellDice>,
    /// What a spell of another kind does; a `damage` spell's `status` is suffered by everyone
    /// it harms.
    pub effect: SpellEffect,
    /// The caster level a read of its scroll casts it at, if it comes on a scroll.
    pub scroll_level: Option<u8>,
}

impl SpellData {
    /// Whether it asks for a cell when cast: anything but a spell on oneself.
    pub fn aimed(&self) -> bool {
        self.shape != Shape::Myself
    }

    /// How far a cell it is aimed at may be, before Willpower: a ray's, bolt's, or burst's
    /// range, a cone's or a line's length, a touch's 1; `None` on oneself.
    pub fn base_reach(&self) -> Option<i32> {
        match self.shape {
            Shape::Touch => Some(1),
            Shape::Myself => None,
            Shape::Cone | Shape::Line => self.length.map(i32::from),
            Shape::Ray | Shape::Bolt | Shape::Burst => self.range.map(i32::from),
        }
    }

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
    // Its reach before Willpower: only where it has the cap, at most the cap, default it.
    let base = |name: &str, value: Option<u8>, cap: Option<u8>| match (value, cap) {
        (None, cap) => Ok(cap),
        (Some(_), None) => Err(format!("a `{}` has no `{name}`", spec.shape.id())),
        (Some(v), Some(cap)) if (1..=cap).contains(&v) => Ok(Some(v)),
        (Some(_), Some(cap)) => Err(format!("`{name}` is 1 to its cap, {cap}")),
    };
    let range = base("range", spec.range, spec.max_range)?;
    let radius = base("radius", spec.radius, spec.max_radius)?;
    let length = base("length", spec.length, spec.max_length)?;
    let casts = spec.casts.unwrap_or(1);
    if !(1..=MAX_CASTS).contains(&casts) {
        return Err(format!("`casts` is 1 to {MAX_CASTS}"));
    }
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
    let look = look(spec.cast, shape, radius, damage_type, content)?;
    let look = SpellLook {
        noun: look.noun.unwrap_or_else(|| spec.name.to_lowercase()),
        ..look.look
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
        range,
        radius,
        length,
        casts,
        open_sky: spec.open_sky,
        look,
        damage,
        effect,
        scroll_level,
    })
}

/// A spell's `[spell.cast]` checked, but its noun when it gives none (the caller's default).
struct Looked {
    look: SpellLook,
    noun: Option<String>,
}

/// A spell's `[spell.cast]`, checked: an `effect` of the shape that draws it (only for a spell
/// cast at a distance: burst and cone a burst's, ray, bolt, and line a bolt's; none: nothing is
/// drawn), blast
/// and heat or frost for a burst, a noise as loud as a trap's may be, no empty words. Its
/// effect is checked against `data/effects.toml` once effects are loaded
/// ([`check_effects`]).
fn look(
    spec: Option<CastSpec>,
    shape: Shape,
    radius: Option<u8>,
    damage_type: Option<DamageTypeId>,
    content: &Content,
) -> Result<Looked, String> {
    let verb = |given: Option<String>| given.unwrap_or_else(|| default_verb(content, damage_type));
    let Some(c) = spec else {
        return Ok(Looked {
            look: SpellLook {
                verb: verb(None),
                ..SpellLook::default()
            },
            noun: None,
        });
    };
    let words = [&c.noun, &c.verb, &c.message];
    if words
        .iter()
        .any(|w| w.as_ref().is_some_and(|w| w.trim().is_empty()))
    {
        return Err("cast: `noun`, `verb`, and `message` must not be empty".into());
    }
    if let Some(noise) = c.noise
        && !(1..=MAX_LOUDNESS).contains(&noise.loudness)
    {
        return Err(format!("cast: `noise` loudness is 1 to {MAX_LOUDNESS}"));
    }
    let far = !matches!(shape, Shape::Touch | Shape::Myself);
    if c.effect.is_some() && !far {
        return Err(format!("cast: a `{}` has no `effect`", shape.id()));
    }
    let burst = shape == Shape::Burst;
    if !burst && (c.blast.is_some() || c.heat || c.frost.is_some()) {
        return Err("cast: `blast`, `heat`, and `frost` are a burst's".into());
    }
    let (blast, ice) = if burst {
        // The scrolls' and traps' checks of the same fields.
        let fields = SpellFields {
            spell: Some(c.effect.clone().unwrap_or_else(|| "-".into())),
            damage: Some(crate::dice::Dice::of(1, 6)),
            kind: None,
            radius: radius.map(i32::from),
            blast: c.blast,
            heat: c.heat,
            frost: c.frost,
            length: None,
        };
        let b = Burst::new(&fields).map_err(|m| format!("cast: {m}"))?;
        (b.blast, b.ice)
    } else {
        (0, None)
    };
    Ok(Looked {
        look: SpellLook {
            effect: c.effect,
            noun: String::new(),
            verb: verb(c.verb),
            message: c.message,
            noise: c.noise,
            blast,
            ice,
        },
        noun: c.noun,
    })
}

/// With spell effects in content, every spell's `effect` names an effect of its shape
/// (checked once effects are loaded, after the objects).
pub(super) fn check_effects(content: &Content) -> Result<(), String> {
    for spell in content.spells() {
        let Some(effect) = &spell.look.effect else {
            continue;
        };
        let shape = match spell.shape {
            Shape::Burst | Shape::Cone => EffectShape::Burst,
            _ => EffectShape::Bolt,
        };
        check_effect(content, effect, shape).map_err(|m| format!("spell `{}`: {m}", spell.id))?;
    }
    Ok(())
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
    // A damage spell's status falls on everyone it harms (blight's poison).
    let marks = [SpellPurpose::Buff, SpellPurpose::Hex, SpellPurpose::Damage];
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
            let id = status(name)?;
            // Only the caster may be hastened (a buff): creatures' steps are never quicker
            // than their type's (a rest's batching counts on it).
            if kind != SpellPurpose::Buff && content.status(id).cost < 100 {
                return Err(format!(
                    "a `{}` spell's status `{name}` hastens: only a buff's may",
                    kind.id()
                ));
            }
            Some((id, severity))
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
            // Only a buff on its caster may hasten (creatures never step quicker).
            (
                "damage = { dice = \"2d6\" }",
                "damage = { dice = \"2d6\" }\neffect = { status = \"hasted\" }",
                "status `hasted` hastens: only a buff's may",
            ),
            (
                "damage = { dice = \"2d6\" }",
                "damage = { dice = \"2d6\" }\neffect = { diseases = true }",
                "a `damage` spell has no `diseases`",
            ),
            (
                "max_range = 8",
                "max_range = 8\nrange = 9",
                "`range` is 1 to its cap, 8",
            ),
            (
                "max_radius = 2",
                "max_radius = 2\nlength = 2",
                "a `burst` has no `length`",
            ),
            ("tier = 3", "tier = 3\ncasts = 0", "`casts` is 1 to 10"),
            (
                "damage = { dice = \"2d6\" }",
                "damage = { dice = \"2d6\" }\ncast = { effect = \"fireball\", frost = 10, heat = true }",
                "`heat` or `frost`, not both",
            ),
            (
                "damage = { dice = \"2d6\" }",
                "damage = { dice = \"2d6\" }\ncast = { effect = \"fireball\", noun = \" \" }",
                "must not be empty",
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

    #[test]
    fn a_cast_costs_by_tier_overreach_shape_resolution_and_armour() {
        let r = rules();
        let list = spells(FIRST_LIST).expect("the first list");
        let spell = |id: &str| list.iter().find(|s| s.id == id).expect("a spell").clone();
        let at = |id: &str, level: i32, armour: u32| r.mana(&spell(id), level, armour);
        // (spell, caster level, armour %, mana): the design's §5 and §7.
        let table = [
            ("spark", 1, 0, 1),            // tier 0, single target
            ("magic_missile", 3, 0, 3),    // tier 1 at its comfortable level
            ("magic_missile", 1, 0, 5),    // one tier over: 3 × 1.5 = 4.5, rounded up
            ("scorching_ray", 1, 0, 12),   // two tiers over: 6 × 2
            ("scorching_ray", 30, 0, 6),   // far above it: never cheaper than the base
            ("burning_hands", 3, 0, 5),    // a cone: 3 × 1.6 = 4.8
            ("cause_wounds", 3, 0, 2),     // touch: 3 × 0.7 = 2.1
            ("fireball", 12, 0, 26),       // a burst of radius 2: 11 × 2.4 = 26.4
            ("lightning_bolt", 12, 0, 20), // a line: 11 × 1.8 = 19.8
            ("fireball", 6, 0, 40),        // one tier over, then the burst: 11 × 1.5 × 2.4
            ("magic_missile", 3, 40, 4),   // plate: 3 × 1.4 = 4.2
            ("fireball", 12, 30, 34),      // banded mail: 26.4 × 1.3 = 34.32
        ];
        for (id, level, armour, mana) in table {
            assert_eq!(at(id, level, armour), mana, "{id} at {level} in {armour} %");
        }
        // Area is dearer than the tier's single target, and weaker per target (§12.5).
        let (ball, ray) = (spell("fireball"), spell("magic_missile"));
        assert!(r.mana(&ball, 12, 0) > i32::from(r.cost[3]));
        assert!(r.cap(3, Resolution::Area) < r.cap(3, Resolution::Attack));
        assert!(r.mana(&ray, 3, 0) == i32::from(r.cost[1]));
    }

    #[test]
    fn willpower_grows_reach_to_the_cap_and_presence_adds_degrees() {
        let r = rules();
        assert_eq!(r.reach_step, 2);
        // A range of 6 capped at 8: a cell per two points of a positive modifier.
        let grown: Vec<i32> = [-3, 0, 1, 2, 4, 9]
            .iter()
            .map(|&m| r.grown(6, 8, m))
            .collect();
        assert_eq!(grown, [6, 6, 6, 7, 8, 8]);
        assert_eq!(r.grown(3, 3, 9), 3, "no cap above the base: no growth");
        let degrees: Vec<u16> = [-2, 1, 2, 5].iter().map(|&m| r.lengthened(1, m)).collect();
        assert_eq!(degrees, [1, 1, 2, 3]);
    }

    #[test]
    fn the_shipped_spells_are_the_first_list_within_their_tiers() {
        let content = Content::builtin().expect("content");
        let r = content.spell_rules().expect("the tier table");
        // (spell, tier, its dice at the tier's comfortable level, mana there, whether a class
        // ability gives it; every other is learned from scrolls only).
        let table = [
            ("arcane_bolt", 2, Some("2d6"), 11, true),
            ("barkskin", 1, None, 3, true),
            ("blessing", 1, None, 3, true),
            ("blight", 2, Some("2d6"), 14, true),
            ("burning_hands", 1, Some("1d4"), 5, true),
            ("call_lightning", 3, Some("2d6"), 20, true),
            ("cause_wounds", 1, Some("1d6"), 2, true),
            ("cure_wounds", 1, None, 2, true),
            ("curse_of_sloth", 1, None, 3, true),
            ("evil_eye", 0, None, 1, true),
            ("fireball", 3, Some("2d6"), 26, false),
            ("flame", 3, Some("3d6"), 26, true),
            ("frost", 2, Some("2d6"), 14, true),
            ("haste", 3, None, 11, false),
            ("hex_of_weakness", 1, None, 3, true),
            ("lightning_bolt", 3, Some("2d6"), 20, false),
            ("magic_missile", 1, Some("1d4"), 3, true),
            ("mend", 0, None, 1, true),
            ("ray_of_frost", 0, Some("1d6"), 1, true),
            ("remove_disease", 3, None, 11, true),
            ("scorching_ray", 2, Some("1d6"), 6, false),
            ("sleep", 2, None, 11, false),
            ("spark", 0, Some("1d4"), 1, true),
            ("web", 2, None, 11, false),
        ];
        for (id, tier, dice, mana, given) in table {
            let at = content.spell_id(id).expect("shipped");
            let spell = content.spell(at);
            assert_eq!(spell.tier, tier, "{id}");
            let level = r.comfortable[usize::from(tier)];
            assert_eq!(r.mana(spell, i32::from(level), 0), mana, "{id}");
            let rolled = spell
                .damage
                .map(|d| d.at(i32::from(level), level).to_string());
            assert_eq!(rolled.as_deref(), dice, "{id}");
            if let Some(d) = spell.damage.or(spell.effect.heal) {
                assert!(d.max() <= r.cap_of(spell), "{id} under its cap");
            }
            assert_eq!(
                content.class_spell(at).is_some(),
                given,
                "{id} given by a class"
            );
        }
        assert_eq!(content.spells().len(), table.len());
    }
}
