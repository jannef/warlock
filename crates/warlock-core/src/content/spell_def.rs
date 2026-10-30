//! Spells as data (spec §8 "Spells", the magic design `docs/design/magic.md` §3, §5, §6, the
//! spell scaling design `docs/design/spell-scaling.md` §5): the cost table
//! (`data/spells.toml`, [`SpellRules`]) and the spells (`data/spells/`, one `[[spell]]` file
//! each, [`SpellData`]), cast by the one cast path (`world/cast.rs`). Not the bursts and bolts
//! scrolls and traps cast (`content/spell.rs`), whose code the cast path shares.
//!
//! - **Spell levels**: a spell has a level, 0 to [`MAX_SPELL_LEVEL`]: its base mana (at least
//!   one) and the caster level at which a scroll of it teaches for certain. A spell of level 0
//!   or 1 is a cantrip. Nothing says how much a spell of a level may do (decision 16).
//! - **Tags**: a spell's school, kind, shape, and damage type ([`SpellData::tags`]); talents,
//!   abilities, and items will key on them.
//! - **The lift** ([`SpellRules::lift`]; the combat progression design §7, until the ladders
//!   replace it): the stats a spell's `[spell.scales]` names for `damage` lift its dice by a
//!   percent of its most ([`SpellDice::lifted`]).

use super::effect::EffectShape;
use super::spell::{Burst, BurstIce, SpellFields, check_effect, default_verb};
use super::spell_scales::{self as scales, Has, ScalesSpec, SpellScales};
use super::trap::TrapNoise;
use super::{Content, ContentError, DamageTypeId, MAX_LEVEL, MAX_SEVERITY, StatusId, duplicate_id};
use crate::dice::Dice;
use crate::noise::MAX_LOUDNESS;
use crate::stats::{PerStat, Stat};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The highest level a spell may have (the spell scaling design §5.1): 0 (a cantrip) to 70.
pub const MAX_SPELL_LEVEL: u8 = 70;
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
/// The most mana a spell's explicit `mana` may ask.
pub const MAX_SPELL_MANA: u16 = 500;
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

/// What a resolution does to a spell's cost, in percent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ratio {
    /// What it multiplies the base cost by (an area spell's cost is its shape's).
    pub cost: u16,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesFile {
    shapes: ShapesSpec,
    resolution: ResolutionSpec,
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

/// The cost table (`data/spells.toml`): what a spell's shape and resolution do to its mana.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellRules {
    /// The single targets' and the areas' cost multipliers, in percent, by [`Shape::ALL`]
    /// order; a burst's is [`SpellRules::burst`]'s.
    shapes: [u16; 6],
    /// A burst's cost multiplier by radius, radius 1 first.
    pub burst: Vec<u16>,
    /// Each resolution's cost, by [`Resolution::ALL`] order.
    resolutions: [Ratio; 4],
}

impl SpellRules {
    pub(super) fn parse(path: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let file: RulesFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let ascending = |v: &[u16]| v.windows(2).all(|w| w[0] < w[1]);
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
        if !(resolutions.iter()).all(|x| PERCENTS.contains(&x.cost)) {
            return Err(err("resolution: `cost` is 1 to 1000 percent".into()));
        }
        Ok(SpellRules {
            shapes,
            burst: s.burst.clone(),
            resolutions,
        })
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

    /// What `resolution` does to a spell's cost.
    pub fn ratio(&self, resolution: Resolution) -> Ratio {
        let i = (Resolution::ALL.iter()).position(|&r| r == resolution);
        self.resolutions[i.unwrap_or(0)]
    }

    /// What `spell` costs a caster wearing armour that adds `armour` percent (the spell scaling
    /// design §5.3), before its talents: its base mana (its level, at least 1, unless it names
    /// `mana`) × its shape's percent (a burst's at its own `radius`) × its resolution's `cost`
    /// percent × (100 + `armour`) %, rounded to the nearest once at the end, at least 1. No
    /// caster level counts: a known spell costs the same whoever casts it.
    pub fn mana(&self, spell: &SpellData, armour: u32) -> i32 {
        // An explicit cost (Identify's) stands in for the level's, the shape's, and the
        // resolution's: only the armour is added.
        if let Some(mana) = spell.mana {
            let num = u64::from(mana) * (100 + u64::from(armour));
            return i32::try_from((num + 50) / 100).unwrap_or(i32::MAX).max(1);
        }
        let radius = spell.radius.unwrap_or(1);
        let factors = [
            u64::from(self.shape_cost(spell.shape, radius)),
            u64::from(self.ratio(spell.resolution).cost),
            100 + u64::from(armour),
        ];
        let num = factors.iter().product::<u64>() * u64::from(spell.level.max(1));
        let den = 100u64.pow(3);
        i32::try_from((num + den / 2) / den)
            .unwrap_or(i32::MAX)
            .max(1)
    }

    /// A reach of `base` cells grown by `more` (its `reach` scales' cells; magic design §3),
    /// never past `cap`.
    pub fn grown(base: i32, cap: i32, more: i32) -> i32 {
        (base + more.max(0)).min(cap.max(base))
    }

    /// The degrees of a buff's or a hex's status, `severity` lengthened by `more` (its
    /// `duration` scales' degrees; the status's own `max` holds it when it is given).
    pub fn lengthened(severity: u16, more: i32) -> u16 {
        let more = u16::try_from(more.max(0)).unwrap_or(u16::MAX);
        severity.saturating_add(more).min(MAX_SEVERITY)
    }

    /// The percent a caster's stats lift a spell's dice by: `raised` (its `damage` scales'
    /// sum, [`super::Weights::sum`]) never below 0, and `more` (Keen mind's for its school or
    /// element, Potent's) on top. Nothing bounds it (the spell scaling design, decision 16);
    /// the ladders replace it (S2).
    pub fn lift(raised: i32, more: i32) -> i32 {
        raised.max(0) + more.max(0)
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
    /// 0 to [`MAX_SPELL_LEVEL`]; an `Option` so a file without it is told what it lacks.
    level: Option<u8>,
    /// Refused: the spell scaling design replaced tiers with levels (§5).
    tier: Option<toml::Value>,
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
    mana: Option<u16>,
    damage: Option<DiceSpec>,
    effect: Option<EffectSpec>,
    cast: Option<CastSpec>,
    scroll: Option<ScrollSpec>,
    scales: Option<ScalesSpec>,
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
    #[serde(default)]
    identify: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScrollSpec {
    level: Option<u8>,
}

/// What a spell rolls, harm or healing: dice that grow with caster level up to `max_dice`
/// (until the ladders replace them, the spell scaling design S2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellDice {
    /// Dice at the caster level of the spell's level ([`SpellData::grows_from`]).
    pub count: u8,
    /// Their sides.
    pub sides: u8,
    /// One die more per this many caster levels above that one (0: never more).
    pub per_levels: u8,
    /// Never more dice than this by caster level.
    pub max_dice: u8,
    /// A static bonus, rare.
    pub bonus: u8,
}

impl SpellDice {
    /// The most its dice can roll by caster level: every die of `max_dice` at its highest,
    /// plus the bonus.
    pub const fn max(&self) -> i32 {
        self.max_dice as i32 * self.sides as i32 + self.bonus as i32
    }

    /// What it rolls at `caster_level` for a spell whose dice grow from caster level `from`
    /// ([`SpellData::grows_from`]): its dice, one more per `per_levels` levels above that,
    /// never more than `max_dice`.
    pub fn at(&self, caster_level: i32, from: u8) -> Dice {
        self.of(self.count_at(caster_level, from))
    }

    /// [`SpellDice::at`] for a caster whose stats lift the spell by `lift` percent
    /// ([`SpellRules::lift`]): `max_dice × lift / 100` dice more, rounded down; no lift, no
    /// change.
    pub fn lifted(&self, caster_level: i32, from: u8, lift: i32) -> Dice {
        let more = i32::from(self.max_dice) * lift.max(0) / 100;
        self.of(self.count_at(caster_level, from) + more)
    }

    /// Its dice at `caster_level`: one more per `per_levels` above `from`, never more than
    /// `max_dice`.
    fn count_at(&self, caster_level: i32, from: u8) -> i32 {
        let above = (caster_level - i32::from(from)).max(0);
        let more = match self.per_levels {
            0 => 0,
            n => above / i32::from(n),
        };
        (i32::from(self.count) + more).min(i32::from(self.max_dice))
    }

    /// `count` of its dice, with its bonus.
    fn of(&self, count: i32) -> Dice {
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
    /// Whether a `utility` spell identifies one stack the caster carries (Identify).
    pub identify: bool,
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
    /// Its spell level, 0 to [`MAX_SPELL_LEVEL`] (0 and 1: a cantrip): its base mana, and the
    /// caster level at which a scroll of it teaches for certain.
    pub level: u8,
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
    /// Its base mana cost given outright (Identify), in place of its level's: no shape or
    /// resolution counts; armour and the talents still do.
    pub mana: Option<u16>,
    /// How it shows and sounds.
    pub look: SpellLook,
    /// A `damage` spell's damage.
    pub damage: Option<SpellDice>,
    /// What a spell of another kind does; a `damage` spell's `status` is suffered by everyone
    /// it harms.
    pub effect: SpellEffect,
    /// The caster level a read of its scroll casts it at, if it comes on a scroll.
    pub scroll_level: Option<u8>,
    /// What it grows with (`[spell.scales]`; the defaults where it is silent).
    pub scales: SpellScales,
}

impl SpellData {
    /// The caster level its dice grow from, and a scroll or a wand of it casts at by default:
    /// its level, at least 1 (until the ladders replace both, the spell scaling design S2).
    pub fn grows_from(&self) -> u8 {
        self.level.max(1)
    }

    /// Whether it rolls to hit (spec §8 "Spells"): a damage spell, a ray, a bolt, or a touch,
    /// resolved by the attack roll. Such a cast trains the spell attack skill and a caster
    /// style (spec §8 "Weapon skills", "Fighting styles").
    pub fn rolls_to_hit(&self) -> bool {
        self.kind == SpellPurpose::Damage
            && matches!(self.resolution, Resolution::Attack | Resolution::Touch)
            && matches!(self.shape, Shape::Ray | Shape::Bolt | Shape::Touch)
    }

    /// Whether it asks for a cell when cast: anything but a spell on oneself.
    pub fn aimed(&self) -> bool {
        self.shape != Shape::Myself
    }

    /// Whether it asks for a stack the caster carries when cast (Identify).
    pub fn aims_item(&self) -> bool {
        self.effect.identify
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

    /// The stats a cast of it trains (spec §8 "Stats", the `spell` use): those its damage
    /// grows with (Intelligence when it names none), those its reach grows with if it has a
    /// reach, and those its status's duration grows with if it is a buff or a hex. Each once.
    pub fn trained_stats(&self) -> PerStat<bool> {
        let mut trained = PerStat::splat(false);
        let reach = !matches!(self.shape, Shape::Touch | Shape::Myself);
        let status = matches!(self.kind, SpellPurpose::Buff | SpellPurpose::Hex);
        let dims = [
            (&self.scales.damage, true),
            (&self.scales.reach, reach),
            (&self.scales.duration, status),
        ];
        for (weights, has) in dims {
            for (stat, _) in weights.iter().filter(|_| has) {
                trained[stat] = true;
            }
        }
        if self.scales.damage.is_empty() {
            trained[Stat::Intelligence] = true;
        }
        trained
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

/// Reads `data/spells.toml` and the spells of `data/spells/` (`files`, in path order, the cost
/// table's own file left out): none without the cost table, and a spell without it is an error.
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

/// Checks a dice table's own shape. Nothing checks how much it rolls (decision 16).
fn dice(spec: &DiceSpec) -> Result<SpellDice, String> {
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
    Ok(SpellDice {
        count,
        sides,
        per_levels: spec.per_levels,
        max_dice,
        bonus: spec.bonus,
    })
}

/// One spell, checked.
fn resolve(spec: SpellSpec, rules: &SpellRules, content: &Content) -> Result<SpellData, String> {
    if spec.tier.is_some() {
        return Err(format!(
            "`tier` is gone: use `level`, 0 to {MAX_SPELL_LEVEL} (docs/design/spell-scaling.md §5)"
        ));
    }
    let level = spec
        .level
        .ok_or_else(|| format!("a spell needs a `level`, 0 to {MAX_SPELL_LEVEL}"))?;
    if level > MAX_SPELL_LEVEL {
        return Err(format!("`level` is 0 to {MAX_SPELL_LEVEL}"));
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
    if spec
        .mana
        .is_some_and(|m| !(1..=MAX_SPELL_MANA).contains(&m))
    {
        return Err(format!("`mana` is 1 to {MAX_SPELL_MANA}"));
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
        (Some(d), SpellPurpose::Damage) => Some(dice(d)?),
        (None, SpellPurpose::Damage) => return Err("a `damage` spell needs [spell.damage]".into()),
        (Some(_), _) => return Err("only a `damage` spell has [spell.damage]".into()),
        (None, _) => None,
    };
    if spec.kind == SpellPurpose::Damage && damage_type.is_none() {
        return Err("a `damage` spell needs a `damage_type`".into());
    }
    let effect = match spec.effect {
        Some(e) => effect(e, spec.kind, content)?,
        None => SpellEffect::default(),
    };
    let needs = match spec.kind {
        SpellPurpose::Heal => effect.heal.is_none().then_some("`heal`"),
        SpellPurpose::Buff | SpellPurpose::Hex => effect.status.is_none().then_some("`status`"),
        SpellPurpose::Cleanse => {
            (effect.cures.is_empty() && !effect.diseases).then_some("`cures` or `diseases`")
        }
        SpellPurpose::Utility => (!effect.identify).then_some("`identify`"),
        SpellPurpose::Damage => None,
    };
    if effect.identify && shape != Shape::Myself {
        return Err(
            "an `identify` spell is cast on `self`: it asks for a stack, not a cell".into(),
        );
    }
    if let Some(what) = needs {
        return Err(format!(
            "a `{}` spell needs {what} in [spell.effect]",
            spec.kind.id()
        ));
    }
    let scroll_level = match spec.scroll {
        None => None,
        Some(s) => {
            let at = s.level.unwrap_or(level.max(1));
            if !(1..=MAX_LEVEL).contains(&at) {
                return Err(format!("scroll: `level` is 1 to {MAX_LEVEL}"));
            }
            Some(at)
        }
    };
    let has = Has {
        roll: damage.is_some() || effect.heal.is_some(),
        attack: spec.kind == SpellPurpose::Damage
            && matches!(spec.resolution, Resolution::Attack | Resolution::Touch),
        reach: !matches!(shape, Shape::Touch | Shape::Myself),
        status: matches!(spec.kind, SpellPurpose::Buff | SpellPurpose::Hex),
    };
    let scales = scales::resolve(spec.scales, &has)?;
    let look = look(spec.cast, shape, radius, damage_type, content)?;
    let look = SpellLook {
        noun: look.noun.unwrap_or_else(|| spec.name.to_lowercase()),
        ..look.look
    };
    Ok(SpellData {
        id: spec.id,
        name: spec.name,
        description: spec.description,
        level,
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
        mana: spec.mana,
        look,
        damage,
        effect,
        scroll_level,
        scales,
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
fn effect(e: EffectSpec, kind: SpellPurpose, content: &Content) -> Result<SpellEffect, String> {
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
    only("identify", e.identify, &[SpellPurpose::Utility])?;
    let status = |name: &str| {
        content
            .status_id(name)
            .ok_or_else(|| format!("unknown status `{name}`"))
    };
    let heal = match &e.heal {
        Some(h) => Some(dice(h)?),
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
        identify: e.identify,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHIPPED: &str = include_str!("../../../../assets/data/spells.toml");

    fn rules() -> SpellRules {
        SpellRules::parse("data/spells.toml", SHIPPED).expect("valid")
    }

    /// Parses spell files `text` against the shipped cost table and the built-in content.
    fn spells(text: &str) -> Result<Vec<SpellData>, ContentError> {
        let content = Content::builtin().expect("content");
        let (_, list) = parse(Some(("data/spells.toml", SHIPPED)), [("s", text)], &content)?;
        Ok(list)
    }

    /// The magic design's first spell list (§7), as data, at the spell scaling design's levels.
    const FIRST_LIST: &str = r#"
[[spell]]
id = "ray_of_frost"
name = "Ray of frost"
description = "A thin beam of cold."
level = 1
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
level = 1
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
level = 1
school = "restoration"
kind = "heal"
shape = "touch"
resolution = "touch"
effect = { heal = { dice = "1d3" } }

[[spell]]
id = "magic_missile"
name = "Magic missile"
description = "A dart of force that never misses."
level = 3
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
level = 4
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
level = 4
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
level = 3
school = "restoration"
kind = "heal"
shape = "touch"
resolution = "touch"
effect = { heal = { dice = "2d6" } }

[[spell]]
id = "bless"
name = "Bless"
description = "A boon."
level = 3
school = "abjuration"
kind = "buff"
shape = "self"
resolution = "auto"
effect = { status = "blessed" }

[[spell]]
id = "scorching_ray"
name = "Scorching ray"
description = "Rays of fire."
level = 7
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
level = 11
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
level = 11
school = "evocation"
kind = "damage"
shape = "line"
resolution = "area"
damage_type = "electricity"
max_length = 12
damage = { dice = "2d6", per_levels = 4, max_dice = 3 }
"#;

    #[test]
    fn the_shipped_cost_table_reads_as_the_design_has_it() {
        let r = rules();
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
        assert_eq!(r.ratio(Resolution::Touch), Ratio { cost: 70 });
    }

    /// The magic design's §12 check 5 (#401), its cost half: an area is dearer than a single
    /// target, a touch cheaper. Nothing says how much any of them may do (decision 16).
    #[test]
    fn the_shipped_table_makes_areas_dearer_and_touches_cheaper() {
        let r = rules();
        let single = r.ratio(Resolution::Attack);
        for &shape in Shape::ALL {
            let radii = if shape == Shape::Burst {
                1..=r.max_radius()
            } else {
                0..=0
            };
            for radius in radii {
                let cost = r.shape_cost(shape, radius);
                if shape.is_area() {
                    assert!(cost > single.cost, "{shape:?} {radius} costs no more");
                } else {
                    assert_eq!(cost, 100, "{shape:?} is a single target");
                }
            }
        }
        assert!(r.ratio(Resolution::Touch).cost < single.cost);
    }

    #[test]
    fn the_cost_table_is_refused_when_malformed() {
        for (from, to, why) in [
            (
                "burst = [180, 240, 300]",
                "burst = [180, 300, 240]",
                "one ascending value per radius",
            ),
            (
                "touch = { cost = 70 }",
                "touch = { cost = 0 }",
                "resolution",
            ),
            // The resolutions' damage column went with the cap (decision 16).
            (
                "touch = { cost = 70 }",
                "touch = { damage = 120, cost = 70 }",
                "unknown field `damage`",
            ),
            // The tier table and the overreach surcharge are gone (the spell scaling design §5).
            (
                "\n[shapes]\n",
                "\noverreach_cost = 50\n\n[shapes]\n",
                "unknown field `overreach_cost`",
            ),
            (
                "\n[shapes]\n",
                "\n[tiers]\ncost = [1]\n\n[shapes]\n",
                "unknown field `tiers`",
            ),
        ] {
            let text = SHIPPED.replace(from, to);
            assert_ne!(text, SHIPPED, "{from}");
            let e = SpellRules::parse("x", &text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }

    #[test]
    fn the_designs_first_spell_list_parses() {
        let list = spells(FIRST_LIST).expect("every spell");
        assert_eq!(list.len(), 11);
        let content = Content::builtin().expect("content");
        let fireball = list.iter().find(|s| s.id == "fireball").expect("fireball");
        assert_eq!(
            fireball.tags(&content),
            ["evocation", "damage", "burst", "fire"]
        );
        assert_eq!(fireball.level, 11);
        assert_eq!(fireball.damage.expect("damage").max(), 18);
        assert_eq!(
            list[0].scroll_level,
            Some(1),
            "the spell's level, at least 1"
        );
        assert_eq!(list[1].scroll_level, None, "no scroll block");
        let bless = list.iter().find(|s| s.id == "bless").expect("bless");
        assert_eq!(
            bless.effect.status,
            Some((content.status_id("blessed").expect("blessed"), 1))
        );
    }

    /// Decision 16: nothing holds how much a spell does. Dice, a bonus, and a heal of any size
    /// load at any level; the author writes the strength they intend.
    #[test]
    fn no_check_holds_how_much_a_spell_does() {
        let fireball = "max_dice = 3 }\n\n[[spell]]\nid = \"lightning_bolt\"";
        for more in ["20 }", "3, bonus = 40 }"] {
            let strong = FIRST_LIST.replace(fireball, &fireball.replace("3 }", more));
            assert_ne!(strong, FIRST_LIST);
            assert!(spells(&strong).is_ok(), "{more}");
        }
        let strong = FIRST_LIST.replace("heal = { dice = \"2d6\" }", "heal = { dice = \"20d6\" }");
        assert_ne!(strong, FIRST_LIST);
        assert!(spells(&strong).is_ok());
        let cantrip = FIRST_LIST.replace(
            "damage = { dice = \"1d4\" }\n",
            "damage = { dice = \"10d12\", bonus = 50 }\n",
        );
        assert_ne!(cantrip, FIRST_LIST);
        assert!(
            spells(&cantrip).is_ok(),
            "a cantrip as strong as its author writes it"
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
        let good = "level = 11\nshape = \"burst\"\nresolution = \"area\"\ndamage_type = \"fire\"\n\
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
            ("level = 11", "level = 71", "`level` is 0 to 70"),
            ("level = 11\n", "", "a spell needs a `level`, 0 to 70"),
            (
                "level = 11",
                "tier = 3",
                "`tier` is gone: use `level`, 0 to 70",
            ),
            (
                "level = 11",
                "level = 11\ntier = 3",
                "`tier` is gone: use `level`",
            ),
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
            ("level = 11", "level = 11\ncasts = 0", "`casts` is 1 to 10"),
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
        assert!(spells(&fireball(&good.replace("level = 11", "level = 70"))).is_ok());
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
    fn dice_grow_with_caster_level_from_the_spells_level_up_to_max_dice() {
        let d = SpellDice {
            count: 2,
            sides: 6,
            per_levels: 4,
            max_dice: 3,
            bonus: 0,
        };
        // Fireball, level 11: 2d6 up to caster level 14, 3d6 from 15 on, never more.
        let at = |level| d.at(level, 11).to_string();
        assert_eq!(
            [at(1), at(11), at(14), at(15), at(60)],
            ["2d6", "2d6", "2d6", "3d6", "3d6"]
        );
        let fixed = SpellDice { per_levels: 0, ..d };
        assert_eq!(fixed.at(60, 11).to_string(), "2d6");
    }

    #[test]
    fn a_lift_adds_its_percent_of_max_dice_and_no_lift_changes_nothing() {
        let d = SpellDice {
            count: 2,
            sides: 6,
            per_levels: 4,
            max_dice: 3,
            bonus: 0,
        };
        // Fireball (three dice at most by level): lifted 50 % a die more, 70 % two.
        let lifted = |level, lift| d.lifted(level, 11, lift).to_string();
        assert_eq!(
            [
                lifted(11, 0),
                lifted(11, 50),
                lifted(15, 50),
                lifted(60, 70)
            ],
            ["2d6", "3d6", "4d6", "5d6"]
        );
        for level in [1, 11, 15, 60] {
            assert_eq!(d.lifted(level, 11, 0), d.at(level, 11), "no lift");
            assert_eq!(d.lifted(level, 11, -40), d.at(level, 11), "never below");
        }
        // Nothing bounds it: a lift of 400 % is twelve dice more.
        assert_eq!(lifted(15, 400), "15d6");
        let b = SpellDice {
            count: 1,
            sides: 6,
            per_levels: 0,
            max_dice: 1,
            bonus: 4,
        };
        assert_eq!(b.lifted(1, 1, 99).to_string(), "1d6+4");
        assert_eq!(b.lifted(1, 1, 100).to_string(), "2d6+4");
    }

    #[test]
    fn the_lift_is_never_below_0_and_has_no_most() {
        assert_eq!(
            [
                SpellRules::lift(-20, 0),
                SpellRules::lift(50, 0),
                SpellRules::lift(110, 0)
            ],
            [0, 50, 110]
        );
        assert_eq!(SpellRules::lift(110, 10), 120, "Keen mind's 10 points more");
        assert_eq!(SpellRules::lift(-20, 10), 10);
    }

    #[test]
    fn scales_default_to_the_old_steps_and_are_checked() {
        let list = spells(FIRST_LIST).expect("valid");
        assert_eq!(
            list[0].scales,
            SpellScales::default(),
            "silent: the defaults"
        );
        // The defaults are the retired steps: a cell or a degree per two points of a positive
        // modifier, to-hit by Dexterity, no lift.
        let d = SpellScales::default();
        for score in 1..=40u8 {
            let mut scores = crate::stats::PerStat::splat(10);
            scores[crate::stats::Stat::Willpower] = score;
            scores[crate::stats::Stat::Presence] = score;
            scores[crate::stats::Stat::Dexterity] = score;
            let stats = crate::stats::Stats::new(scores, crate::stats::PerStat::splat(40));
            let m = crate::stats::modifier(i32::from(score));
            assert_eq!(d.reach.steps(&stats), m.max(0) / 2, "reach at {score}");
            assert_eq!(d.duration.steps(&stats), m.max(0) / 2, "degrees at {score}");
            assert_eq!(d.to_hit.signed(&stats), m, "to-hit at {score}");
            assert_eq!(d.damage.sum(&stats), 0);
        }
        let ray = |scales: &str| {
            format!(
                "[[spell]]\nid = \"ray\"\nname = \"Ray\"\ndescription = \"x\"\nlevel = 6\n\
                 school = \"evocation\"\nkind = \"damage\"\nshape = \"ray\"\n\
                 resolution = \"attack\"\ndamage_type = \"fire\"\nmax_range = 8\n\
                 damage = {{ dice = \"3d6\" }}\n[spell.scales]\n{scales}\n"
            )
        };
        let ok = spells(&ray(
            "damage = { intelligence = 10, willpower = 5 }\nto_hit = {}",
        ))
        .expect("valid");
        let stats: Vec<_> = ok[0].scales.damage.iter().collect();
        use crate::stats::Stat;
        assert_eq!(stats, [(Stat::Intelligence, 10), (Stat::Willpower, 5)]);
        assert!(ok[0].scales.to_hit.is_empty(), "`{{}}`: no stat aims it");
        assert_eq!(ok[0].scales.reach, SpellScales::default().reach, "left out");
        for (scales, why) in [
            ("damage = { wits = 10 }", "unknown stat `wits`"),
            ("damage = { intelligence = 0 }", "weights are 1 to 200"),
            ("reach = { willpower = 201 }", "weights are 1 to 200"),
            ("duration = { presence = 50 }", "no `duration`"),
            ("speed = { presence = 50 }", "unknown field `speed`"),
        ] {
            let e = spells(&ray(scales)).unwrap_err();
            assert!(e.message.contains(why), "{scales}: {}", e.message);
        }
        // An auto spell has no attack roll to aim; a touch has no reach to grow.
        let spark = FIRST_LIST.replace(
            "damage = { dice = \"1d4\" }\n",
            "damage = { dice = \"1d4\" }\nscales = { to_hit = { intelligence = 100 } }\n",
        );
        assert_ne!(spark, FIRST_LIST);
        let e = spells(&spark).unwrap_err();
        assert!(e.message.contains("no `to_hit`"), "{}", e.message);
    }

    #[test]
    fn a_cast_costs_by_level_shape_resolution_and_armour() {
        let r = rules();
        let list = spells(FIRST_LIST).expect("the first list");
        let spell = |id: &str| list.iter().find(|s| s.id == id).expect("a spell").clone();
        let at = |id: &str, armour: u32| r.mana(&spell(id), armour);
        // (spell, armour %, mana): the spell scaling design §5.3 and §5.8.
        let table = [
            ("spark", 0, 1),           // level 1, single target
            ("magic_missile", 0, 3),   // level 3
            ("scorching_ray", 0, 7),   // level 7
            ("burning_hands", 0, 6),   // a cone: 4 × 1.6 = 6.4
            ("cause_wounds", 0, 3),    // touch: 4 × 0.7 = 2.8
            ("fireball", 0, 26),       // a burst of radius 2: 11 × 2.4 = 26.4
            ("lightning_bolt", 0, 20), // a line: 11 × 1.8 = 19.8
            ("magic_missile", 40, 4),  // plate: 3 × 1.4 = 4.2
            ("fireball", 30, 34),      // banded mail: 26.4 × 1.3 = 34.32
        ];
        for (id, armour, mana) in table {
            assert_eq!(at(id, armour), mana, "{id} in {armour} %");
        }
        // Area is dearer than a single target of its level.
        let ball = spell("fireball");
        assert!(r.mana(&ball, 0) > i32::from(ball.level));
        // A level-0 spell still costs 1.
        let light = SpellData {
            level: 0,
            ..spell("spark")
        };
        assert_eq!(r.mana(&light, 0), 1);
    }

    #[test]
    fn reach_grows_to_the_cap_and_degrees_add_up() {
        // A range of 6 capped at 8, grown by the cells the scales give.
        let grown: Vec<i32> = [-1, 0, 1, 2, 4]
            .iter()
            .map(|&more| SpellRules::grown(6, 8, more))
            .collect();
        assert_eq!(grown, [6, 6, 7, 8, 8]);
        assert_eq!(
            SpellRules::grown(3, 3, 4),
            3,
            "no cap above the base: no growth"
        );
        let degrees: Vec<u16> = [-1, 0, 1, 2]
            .iter()
            .map(|&more| SpellRules::lengthened(1, more))
            .collect();
        assert_eq!(degrees, [1, 1, 2, 3]);
    }

    #[test]
    fn the_shipped_spells_are_at_the_designs_levels_and_costs() {
        let content = Content::builtin().expect("content");
        let r = content.spell_rules().expect("the cost table");
        // (spell, level, its dice at the caster level of its level, mana, whether a class
        // ability gives it; every other is learned from scrolls only): the spell scaling design
        // §5.8.
        let table = [
            ("arcane_bolt", 6, Some("2d6"), 11, true),
            ("barkskin", 3, None, 3, true),
            ("blessing", 3, None, 3, true),
            ("blight", 7, Some("2d6"), 17, true),
            ("burning_hands", 4, Some("1d4"), 6, true),
            ("call_lightning", 11, Some("2d6"), 20, true),
            ("cause_wounds", 4, Some("1d6"), 3, true),
            ("cure_wounds", 3, None, 2, true),
            ("curse_of_sloth", 3, None, 3, true),
            ("evil_eye", 0, None, 1, true),
            ("fireball", 11, Some("2d6"), 26, false),
            ("flame", 12, Some("3d6"), 29, true),
            ("frost", 6, Some("2d6"), 14, true),
            ("haste", 11, None, 11, false),
            ("hex_of_weakness", 3, None, 3, true),
            ("light", 0, None, 1, false),
            ("lightning_bolt", 11, Some("2d6"), 20, false),
            ("magic_missile", 3, Some("1d4"), 3, true),
            ("mend", 1, None, 1, true),
            ("ray_of_frost", 1, Some("1d6"), 1, true),
            ("identify", 6, None, 60, false),
            ("remove_disease", 11, None, 11, true),
            ("scorching_ray", 7, Some("1d6"), 7, false),
            ("sleep", 6, None, 11, false),
            ("spark", 1, Some("1d4"), 1, true),
            ("web", 7, None, 13, false),
        ];
        for (id, level, dice, mana, given) in table {
            let at = content.spell_id(id).expect("shipped");
            let spell = content.spell(at);
            assert_eq!(spell.level, level, "{id}");
            assert_eq!(r.mana(spell, 0), mana, "{id}");
            let from = spell.grows_from();
            let rolled = spell
                .damage
                .map(|d| d.at(i32::from(from), from).to_string());
            assert_eq!(rolled.as_deref(), dice, "{id}");
            assert_eq!(
                content.class_spell(at).is_some(),
                given,
                "{id} given by a class"
            );
        }
        assert_eq!(content.spells().len(), table.len());
    }
}
