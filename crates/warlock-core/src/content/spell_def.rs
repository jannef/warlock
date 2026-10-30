//! Spells as data (spec §8 "Spells", the magic design `docs/design/magic.md` §3, §5, §6, the
//! spell scaling design `docs/design/spell-scaling.md` §4 to §6): the cost table
//! (`data/spells.toml`, [`SpellRules`]) and the spells (`data/spells/`, one `[[spell]]` file
//! each, [`SpellData`]), cast by the one cast path (`world/cast.rs`). Not the bursts and bolts
//! traps cast (`content/spell.rs`), whose code the cast path shares.
//!
//! - **Spell levels**: a spell has a level, 0 to [`MAX_SPELL_LEVEL`]: its base mana (at least
//!   one) and the caster level at which a scroll of it teaches for certain. A spell of level 0
//!   or 1 is a cantrip. Nothing says how much a spell of a level may do (decision 16).
//! - **The base and the ladders**: a spell's own roll and reach are its **base**, what anyone
//!   who casts it does; `[spell.grows]` may give each dimension a ladder of the caster's
//!   scores (`content/spell_ladder.rs`, [`Grows`]). No check reads how much a rung does.
//! - **Tags**: a spell's school, kind, shape, and damage type ([`SpellData::tags`]); talents,
//!   abilities, and items key on them.

use super::effect::EffectShape;
use super::spell::{Burst, BurstIce, SpellFields, check_effect, default_verb};
use super::spell_ladder::{Ladder, RungValue, Unit};
use super::trap::TrapNoise;
use super::{Content, ContentError, DamageTypeId, MAX_SEVERITY, StatusId, duplicate_id};
use crate::dice::Roll;
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
    /// Every kind, in the order the design lists them (§6).
    pub const ALL: [SpellPurpose; 6] = [
        SpellPurpose::Damage,
        SpellPurpose::Heal,
        SpellPurpose::Buff,
        SpellPurpose::Hex,
        SpellPurpose::Cleanse,
        SpellPurpose::Utility,
    ];

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
    /// Refused, each naming its replacement (the spell scaling design §6.1).
    max_range: Option<toml::Value>,
    max_radius: Option<toml::Value>,
    max_length: Option<toml::Value>,
    scales: Option<toml::Value>,
    scroll: Option<toml::Value>,
    range: Option<u8>,
    radius: Option<u8>,
    length: Option<u8>,
    casts: Option<u8>,
    #[serde(default)]
    open_sky: bool,
    mana: Option<u16>,
    /// A roll (`"2d6"`); the old table form is refused naming it.
    damage: Option<toml::Value>,
    aim: Option<String>,
    effect: Option<EffectSpec>,
    cast: Option<CastSpec>,
    grows: Option<GrowsSpec>,
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
struct EffectSpec {
    /// A roll (`"2d6"`); the old table form is refused naming it.
    heal: Option<toml::Value>,
    status: Option<String>,
    severity: Option<u16>,
    #[serde(default)]
    cures: Vec<String>,
    #[serde(default)]
    diseases: bool,
    #[serde(default)]
    identify: bool,
}

/// `[spell.grows]` as written: one ladder string a dimension, each optional.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GrowsSpec {
    damage: Option<String>,
    heal: Option<String>,
    range: Option<String>,
    length: Option<String>,
    radius: Option<String>,
    degrees: Option<String>,
}

/// What a spell's dimensions grow with (`[spell.grows]`): a ladder each, or none (it never
/// grows by stats). Checked against the spell's base and the engine's limits as it loads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Grows {
    /// A `damage` spell's roll.
    pub damage: Option<Ladder>,
    /// A `heal` spell's roll.
    pub heal: Option<Ladder>,
    /// A ray's, bolt's, or burst's range.
    pub range: Option<Ladder>,
    /// A cone's or a line's length.
    pub length: Option<Ladder>,
    /// A burst's radius.
    pub radius: Option<Ladder>,
    /// A buff's or a hex's degrees of its status.
    pub degrees: Option<Ladder>,
}

impl Grows {
    /// The ladder of its roll, harm or healing.
    pub fn roll(&self) -> Option<&Ladder> {
        self.damage.as_ref().or(self.heal.as_ref())
    }

    /// The ladder of its reach: the range's, else the length's.
    pub fn reach(&self) -> Option<&Ladder> {
        self.range.as_ref().or(self.length.as_ref())
    }

    /// Every ladder it has, by dimension name, in the order the wiki prints them.
    pub fn named(&self) -> Vec<(&'static str, &Ladder)> {
        let all = [
            ("damage", &self.damage),
            ("heal", &self.heal),
            ("range", &self.range),
            ("length", &self.length),
            ("radius", &self.radius),
            ("degrees", &self.degrees),
        ];
        (all.into_iter())
            .filter_map(|(name, l)| l.as_ref().map(|l| (name, l)))
            .collect()
    }
}

/// What a spell of a kind other than `damage` does (`[spell.effect]`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpellEffect {
    /// A `heal`'s hit points: its base roll.
    pub heal: Option<Roll>,
    /// A `buff`'s status on the caster, a `hex`'s on its target, with its severity (its base
    /// degrees).
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
    /// Its base reach: a ray's, bolt's, or burst's range, a burst's radius, a cone's or a
    /// line's length (each exactly where its shape has it); its ladders grow them.
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
    /// A `damage` spell's base roll.
    pub damage: Option<Roll>,
    /// What a spell of another kind does; a `damage` spell's `status` is suffered by everyone
    /// it harms.
    pub effect: SpellEffect,
    /// Its ladders (`[spell.grows]`).
    pub grows: Grows,
    /// The stat whose modifier aims an `attack` or `touch` spell of its caster's own (default
    /// Dexterity; a scroll's or a wand's always Dexterity).
    pub aim: Stat,
}

impl SpellData {
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

    /// Its base roll, harm or healing.
    pub fn roll(&self) -> Option<Roll> {
        self.damage.or(self.effect.heal)
    }

    /// Its roll at rung `rung` of its damage or heal ladder, before any hook: the rung's value,
    /// the base at rung 0 (or without a ladder), the last rung's past it.
    pub fn roll_at(&self, rung: usize) -> Option<Roll> {
        let base = self.roll()?;
        let Some(ladder) = self.grows.roll() else {
            return Some(base);
        };
        let rung = rung.min(ladder.len());
        Some(
            (ladder.value(rung))
                .and_then(RungValue::roll)
                .unwrap_or(base),
        )
    }

    /// How far a cell it is aimed at may be at its base: a ray's, bolt's, or burst's range, a
    /// cone's or a line's length, a touch's 1; `None` on oneself.
    pub fn base_range(&self) -> Option<i32> {
        match self.shape {
            Shape::Touch => Some(1),
            Shape::Myself => None,
            Shape::Cone | Shape::Line => self.length.map(i32::from),
            Shape::Ray | Shape::Bolt | Shape::Burst => self.range.map(i32::from),
        }
    }

    /// The stats a cast of it trains (spec §8 "Stats", the `spell` use): its ladders' stats
    /// (Intelligence when its roll has none), each once.
    pub fn trained_stats(&self) -> PerStat<bool> {
        let mut trained = PerStat::splat(false);
        for (_, ladder) in self.grows.named() {
            trained[ladder.stat] = true;
        }
        if self.grows.roll().is_none() {
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

/// A base roll (`damage`, `heal`) as written: a [`Roll`] that can do harm (at least 0, at most
/// 1 or more). Nothing checks how much it rolls (decision 16). The old table form is refused,
/// naming the new one.
fn base_roll(name: &str, value: &toml::Value) -> Result<Roll, String> {
    let Some(text) = value.as_str() else {
        return Err(format!(
            "`{name}` is a roll now, `{name} = \"2d6\"`: `dice`, `per_levels`, `max_dice`, and \
             `bonus` are gone; a `[spell.grows] {name}` ladder grows it (docs/design/spell-scaling.md \
             §6.1)"
        ));
    };
    let roll: Roll = text.parse()?;
    if roll.min() < 0 || roll.max() < 1 {
        return Err(format!(
            "`{name}` must be a roll that can do something, not `{roll}`"
        ));
    }
    Ok(roll)
}

/// The retired fields, each refused naming its replacement (the spell scaling design §6.1).
fn retired(spec: &SpellSpec) -> Result<(), String> {
    if spec.tier.is_some() {
        return Err(format!(
            "`tier` is gone: use `level`, 0 to {MAX_SPELL_LEVEL} (docs/design/spell-scaling.md §5)"
        ));
    }
    let caps = [
        ("max_range", spec.max_range.is_some(), "range"),
        ("max_length", spec.max_length.is_some(), "length"),
        ("max_radius", spec.max_radius.is_some(), "radius"),
    ];
    if let Some((old, _, new)) = caps.iter().find(|(_, given, _)| *given) {
        return Err(format!(
            "`{old}` is gone: the base `{new}` is what anyone casts, and a `[spell.grows] {new}` \
             ladder grows it (docs/design/spell-scaling.md §6.1)"
        ));
    }
    if spec.scales.is_some() {
        return Err(
            "`[spell.scales]` is gone: write ladders in `[spell.grows]` (`damage`, \
                    `heal`, `range`, `length`, `radius`, `degrees`), and `aim` for the stat \
                    that aims an attack (docs/design/spell-scaling.md §6.1)"
                .into(),
        );
    }
    if spec.scroll.is_some() {
        return Err(
            "`[spell.scroll] level` is gone: a spell scroll casts at its own `rungs`, on \
                    the scroll's `read` table (docs/design/spell-scaling.md §7.4)"
                .into(),
        );
    }
    Ok(())
}

/// One spell, checked.
fn resolve(spec: SpellSpec, rules: &SpellRules, content: &Content) -> Result<SpellData, String> {
    retired(&spec)?;
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
    // Its base reach: each shape has exactly its own, within the engine's limits.
    let shape = spec.shape;
    let reach = |name: &str, value: Option<u8>, wanted: bool, most: u8| match (value, wanted) {
        (Some(v), true) if (1..=most).contains(&v) => Ok(Some(v)),
        (Some(_), true) => Err(format!("`{name}` is 1 to {most}")),
        (None, true) => Err(format!("a `{}` needs `{name}`", shape.id())),
        (Some(_), false) => Err(format!("a `{}` has no `{name}`", shape.id())),
        (None, false) => Ok(None),
    };
    let ranged = matches!(shape, Shape::Ray | Shape::Bolt | Shape::Burst);
    let range = reach("range", spec.range, ranged, MAX_REACH)?;
    let burst = shape == Shape::Burst;
    let radius = reach("radius", spec.radius, burst, rules.max_radius())?;
    let long = matches!(shape, Shape::Cone | Shape::Line);
    let longest = if shape == Shape::Cone {
        MAX_CONE
    } else {
        MAX_REACH
    };
    let length = reach("length", spec.length, long, longest)?;
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
        (Some(d), SpellPurpose::Damage) => Some(base_roll("damage", d)?),
        (None, SpellPurpose::Damage) => return Err("a `damage` spell needs `damage`".into()),
        (Some(_), _) => return Err("only a `damage` spell has `damage`".into()),
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
    let attack = spec.kind == SpellPurpose::Damage
        && matches!(spec.resolution, Resolution::Attack | Resolution::Touch);
    let aim = match &spec.aim {
        None => Stat::Dexterity,
        Some(_) if !attack => {
            return Err(
                "`aim` is an `attack` or `touch` damage spell's: nothing else rolls to \
                        hit"
                .into(),
            );
        }
        Some(id) => Stat::from_id(id).ok_or_else(|| format!("`aim`: unknown stat `{id}`"))?,
    };
    let bases = Bases {
        damage,
        heal: effect.heal,
        range,
        length,
        radius,
        degrees: matches!(spec.kind, SpellPurpose::Buff | SpellPurpose::Hex)
            .then_some(effect.status)
            .flatten(),
        shape,
        burst_most: rules.max_radius(),
    };
    let grows = match spec.grows {
        Some(g) => grows(g, &bases, content)?,
        None => Grows::default(),
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
        level,
        school: spec.school,
        kind: spec.kind,
        shape,
        resolution: spec.resolution,
        damage_type,
        range,
        radius,
        length,
        casts,
        open_sky: spec.open_sky,
        mana: spec.mana,
        look,
        damage,
        effect,
        grows,
        aim,
    })
}

/// A spell's base values, what its ladders are checked against.
struct Bases {
    damage: Option<Roll>,
    heal: Option<Roll>,
    range: Option<u8>,
    length: Option<u8>,
    radius: Option<u8>,
    /// A buff's or a hex's status and base degrees.
    degrees: Option<(StatusId, u16)>,
    shape: Shape,
    /// The largest radius a burst may have (the burst table's length).
    burst_most: u8,
}

/// `[spell.grows]` checked against the spell's `bases` (the spell scaling design §6.3): a
/// ladder only for a dimension the spell has; rolls that can do harm and never weaken (`min`
/// and `max` non-decreasing from the base, each rung different from the one before); cells and
/// degrees strictly ascending from the base, within the engine's limits (a range to
/// [`MAX_REACH`], a cone to [`MAX_CONE`], a line to [`MAX_REACH`], a radius to the burst
/// table's length, degrees to the status's `max` and [`MAX_SEVERITY`]). Nothing reads how much
/// a rung does (decision 16).
fn grows(spec: GrowsSpec, bases: &Bases, content: &Content) -> Result<Grows, String> {
    let lacks = |name: &str, why: &str| Err(format!("grows: `{name}` is for {why}"));
    let rolls = |name: &str, text: Option<String>, base: Option<Roll>, why: &str| {
        let Some(text) = text else {
            return Ok(None);
        };
        let Some(base) = base else {
            return lacks(name, why);
        };
        let ladder = Ladder::parse(&text, Unit::Roll).map_err(|e| format!("grows: {e}"))?;
        let mut before = base;
        for value in ladder.values() {
            let RungValue::Roll(roll) = *value else {
                continue;
            };
            if roll.min() < 0 || roll.max() < 1 {
                return Err(format!("grows: `{name}`'s rung `{roll}` can do nothing"));
            }
            if roll.max() < before.max() || roll.min() < before.min() || roll == before {
                return Err(format!(
                    "grows: `{name}`'s rung `{roll}` is below the one before it (`{before}`): a \
                     rung never weakens a spell, and each differs from the last"
                ));
            }
            before = roll;
        }
        Ok(Some(ladder))
    };
    let numbers =
        |name: &str, text: Option<String>, base: Option<u16>, most: u16, unit: Unit, why: &str| {
            let Some(text) = text else {
                return Ok(None);
            };
            let Some(base) = base else {
                return lacks(name, why);
            };
            let ladder = Ladder::parse(&text, unit).map_err(|e| format!("grows: {e}"))?;
            let mut before = i32::from(base);
            for value in ladder.values() {
                let n = value.number().unwrap_or(0);
                if n <= before || n > i32::from(most) {
                    return Err(format!(
                        "grows: `{name}`'s rungs rise strictly from its base, {base}, to at most \
                         {most}; not {n}"
                    ));
                }
                before = n;
            }
            Ok(Some(ladder))
        };
    let shape = bases.shape;
    let length_most = if shape == Shape::Cone {
        MAX_CONE
    } else {
        MAX_REACH
    };
    let degrees_most = (bases.degrees).map_or(MAX_SEVERITY, |(s, _)| {
        content.status(s).max.min(MAX_SEVERITY)
    });
    Ok(Grows {
        damage: rolls(
            "damage",
            spec.damage,
            bases.damage,
            "a `damage` spell's roll",
        )?,
        heal: rolls("heal", spec.heal, bases.heal, "a `heal` spell's roll")?,
        range: numbers(
            "range",
            spec.range,
            bases.range.map(u16::from),
            u16::from(MAX_REACH),
            Unit::Cells,
            "a ray's, a bolt's, or a burst's range",
        )?,
        length: numbers(
            "length",
            spec.length,
            bases.length.map(u16::from),
            u16::from(length_most),
            Unit::Cells,
            "a cone's or a line's length",
        )?,
        radius: numbers(
            "radius",
            spec.radius,
            bases.radius.map(u16::from),
            u16::from(bases.burst_most),
            Unit::Cells,
            "a burst's radius",
        )?,
        degrees: numbers(
            "degrees",
            spec.degrees,
            bases.degrees.map(|(_, d)| d),
            degrees_most,
            Unit::Degrees,
            "a buff's or a hex's status",
        )?,
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

/// A spell's `[spell.effect]`, checked: only what its kind uses.
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
        Some(h) => Some(base_roll("heal", h)?),
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
range = 6
damage = "1d4"

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
range = 5
damage = "1d4"

[[spell]]
id = "mend"
name = "Mend"
description = "Closes a scratch."
level = 1
school = "restoration"
kind = "heal"
shape = "touch"
resolution = "touch"
effect = { heal = "1d3" }

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
range = 8
damage = "1d4"

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
length = 3
damage = "1d4"

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
damage = "1d6"

[[spell]]
id = "cure_wounds"
name = "Cure wounds"
description = "Heals."
level = 3
school = "restoration"
kind = "heal"
shape = "touch"
resolution = "touch"
effect = { heal = "2d6" }

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
range = 8
damage = "1d6"

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
range = 8
radius = 2
damage = "2d6"

[spell.grows]
damage = "intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2"
range = "willpower 14 9, 18 10"

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
length = 12
damage = "2d6"
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
        assert_eq!(fireball.damage.expect("damage").max(), 12);
        let ladder = fireball.grows.damage.as_ref().expect("a damage ladder");
        assert_eq!(
            (ladder.stat, ladder.len()),
            (Stat::Intelligence, 4),
            "{ladder}"
        );
        assert_eq!(
            fireball.roll_at(0).map(|r| r.to_string()).as_deref(),
            Some("2d6")
        );
        assert_eq!(
            fireball.roll_at(2).map(|r| r.to_string()).as_deref(),
            Some("4d6")
        );
        assert_eq!(
            fireball.roll_at(9).map(|r| r.to_string()).as_deref(),
            Some("5d6+2")
        );
        assert_eq!(fireball.aim, Stat::Dexterity, "the default aim");
        let bless = list.iter().find(|s| s.id == "bless").expect("bless");
        assert_eq!(
            bless.effect.status,
            Some((content.status_id("blessed").expect("blessed"), 1))
        );
        assert_eq!(
            bless.grows,
            Grows::default(),
            "no ladder: it never grows by stats"
        );
    }

    /// Decision 16: nothing holds how much a spell does. A roll, a bonus, a heal, and a rung of
    /// any size load at any level; the author writes the strength they intend.
    #[test]
    fn no_check_holds_how_much_a_spell_does() {
        for (from, to) in [
            (
                "length = 12\ndamage = \"2d6\"",
                "length = 12\ndamage = \"20d20\"",
            ),
            ("25 5d6+2", "25 40d20+900"),
            ("heal = \"2d6\"", "heal = \"20d6+100\""),
            (
                "range = 5\ndamage = \"1d4\"",
                "range = 5\ndamage = \"10d12+50\"",
            ),
        ] {
            let strong = FIRST_LIST.replace(from, to);
            assert_ne!(strong, FIRST_LIST, "{from}");
            assert!(spells(&strong).is_ok(), "{to}");
        }
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
                    range = 8\nradius = 2\ndamage = \"2d6\"";
        assert!(spells(&fireball(good)).is_ok());
        for (from, to, why) in [
            (
                "resolution = \"area\"",
                "resolution = \"attack\"",
                "cannot resolve as",
            ),
            ("radius = 2", "radius = 4", "`radius` is 1 to 3"),
            ("range = 8\n", "", "needs `range`"),
            ("radius = 2", "radius = 2\nlength = 3", "has no `length`"),
            ("range = 8", "range = 21", "`range` is 1 to 20"),
            ("damage_type = \"fire\"\n", "", "needs a `damage_type`"),
            (
                "damage_type = \"fire\"",
                "damage_type = \"holy\"",
                "unknown damage type",
            ),
            ("\"2d6\"", "\"2x6\"", "bad roll"),
            ("\"2d6\"", "\"-3\"", "a roll that can do something"),
            ("damage = \"2d6\"", "", "needs `damage`"),
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
            // The retired fields, each naming what replaced it (the spell scaling design §6.1).
            (
                "damage = \"2d6\"",
                "damage = { dice = \"2d6\", per_levels = 4, max_dice = 3 }",
                "`damage` is a roll now, `damage = \"2d6\"`",
            ),
            (
                "range = 8",
                "range = 8\nmax_range = 10",
                "`max_range` is gone",
            ),
            (
                "radius = 2",
                "radius = 2\nmax_radius = 2",
                "`max_radius` is gone",
            ),
            (
                "range = 8",
                "range = 8\nmax_length = 3",
                "`max_length` is gone",
            ),
            (
                "damage = \"2d6\"",
                "damage = \"2d6\"\nscales = { damage = { intelligence = 10 } }",
                "`[spell.scales]` is gone",
            ),
            (
                "damage = \"2d6\"",
                "damage = \"2d6\"\nscroll = { level = 12 }",
                "`[spell.scroll] level` is gone",
            ),
            // Only a buff on its caster may hasten (creatures never step quicker).
            (
                "damage = \"2d6\"",
                "damage = \"2d6\"\neffect = { status = \"hasted\" }",
                "status `hasted` hastens: only a buff's may",
            ),
            (
                "damage = \"2d6\"",
                "damage = \"2d6\"\neffect = { diseases = true }",
                "a `damage` spell has no `diseases`",
            ),
            (
                "radius = 2",
                "radius = 2\nlength = 2",
                "a `burst` has no `length`",
            ),
            ("level = 11", "level = 11\ncasts = 0", "`casts` is 1 to 10"),
            (
                "damage = \"2d6\"",
                "damage = \"2d6\"\ncast = { effect = \"fireball\", frost = 10, heat = true }",
                "`heat` or `frost`, not both",
            ),
            (
                "damage = \"2d6\"",
                "damage = \"2d6\"\ncast = { effect = \"fireball\", noun = \" \" }",
                "must not be empty",
            ),
            // An area aims at nothing: no attack roll for `aim` to add to.
            (
                "damage = \"2d6\"",
                "damage = \"2d6\"\naim = \"intelligence\"",
                "`aim` is an `attack` or `touch` damage spell's",
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

    /// The spell scaling design §6.3: a ladder only for a dimension the spell has, its stat
    /// known, its rungs never below the base nor the one before, and within the engine's limits
    /// of reach and degrees. Nothing reads how much a rung does (decision 16).
    #[test]
    fn ladders_are_checked_against_the_base_and_the_engine() {
        let spell = |body: &str, grows: &str| {
            format!(
                "[[spell]]\nid = \"s\"\nname = \"S\"\ndescription = \"x\"\nlevel = 3\n\
                 school = \"evocation\"\n{body}\n\n[spell.grows]\n{grows}\n"
            )
        };
        let burst = "kind = \"damage\"\nshape = \"burst\"\nresolution = \"area\"\n\
                     damage_type = \"fire\"\nrange = 8\nradius = 1\ndamage = \"2d6\"";
        let ray = "kind = \"damage\"\nshape = \"ray\"\nresolution = \"attack\"\n\
                   damage_type = \"fire\"\nrange = 8\ndamage = \"1d6\"";
        let hex = "kind = \"hex\"\nshape = \"ray\"\nresolution = \"auto\"\nrange = 6\n\
                   effect = { status = \"sluggish\" }";
        let heal = "kind = \"heal\"\nshape = \"touch\"\nresolution = \"touch\"\n\
                    effect = { heal = \"2d6\" }";
        for (body, grows) in [
            (
                burst,
                "damage = \"intelligence 13 3d6, 16 3d6+1d10\"\nrange = \"willpower 14 9\"",
            ),
            (burst, "radius = \"willpower 14 2, 18 3\""),
            (
                ray,
                "damage = \"intelligence 16 1d8\"\nrange = \"willpower 14 20\"",
            ),
            (hex, "degrees = \"presence 14 2, 18 3\""),
            (heal, "heal = \"willpower 16 3d6, 22 3d6+3\""),
        ] {
            assert!(spells(&spell(body, grows)).is_ok(), "{grows}");
        }
        let ok = spells(&spell(
            &format!("{ray}\naim = \"intelligence\""),
            "range = \"willpower 14 9\"",
        ))
        .expect("an attack may aim with any stat");
        assert_eq!(ok[0].aim, Stat::Intelligence);
        let nine: Vec<String> = (1..=9).map(|i| format!("{} {}d6", 10 + i, 2 + i)).collect();
        let nine = format!("damage = \"intelligence {}\"", nine.join(", "));
        for (body, grows, why) in [
            (
                burst,
                "damage = \"intelligence 13 1d6\"",
                "below the one before it (`2d6`)",
            ),
            (
                burst,
                "damage = \"intelligence 13 4d6, 16 3d6\"",
                "below the one before it (`4d6`)",
            ),
            (
                burst,
                "damage = \"intelligence 13 2d6\"",
                "each differs from the last",
            ),
            (
                burst,
                "heal = \"willpower 13 3d6\"",
                "`heal` is for a `heal` spell's roll",
            ),
            (
                burst,
                "length = \"willpower 14 3\"",
                "`length` is for a cone's or a line's",
            ),
            (
                burst,
                "degrees = \"presence 14 2\"",
                "`degrees` is for a buff's or a hex's",
            ),
            (
                burst,
                "damage = \"cunning 13 3d6\"",
                "unknown stat `cunning`",
            ),
            (burst, "radius = \"willpower 14 4\"", "to at most 3; not 4"),
            (
                burst,
                "range = \"willpower 14 21\"",
                "to at most 20; not 21",
            ),
            (
                burst,
                "range = \"willpower 14 8\"",
                "rise strictly from its base, 8",
            ),
            (burst, "range = \"willpower 18 10, 14 9\"", "rise strictly"),
            (burst, &nine, "at most 8 rungs"),
            (
                ray,
                "radius = \"willpower 14 2\"",
                "`radius` is for a burst's radius",
            ),
            (
                hex,
                "degrees = \"presence 14 2, 18 4\"",
                "to at most 3; not 4",
            ),
            (
                hex,
                "damage = \"presence 14 1d6\"",
                "`damage` is for a `damage` spell's roll",
            ),
            (heal, "range = \"willpower 14 2\"", "`range` is for a ray's"),
            (burst, "speed = \"willpower 14 2\"", "unknown field `speed`"),
        ] {
            let e = spells(&spell(body, grows)).unwrap_err();
            assert!(e.message.contains(why), "{grows}: {}", e.message);
        }
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
    fn the_shipped_spells_are_at_the_designs_levels_and_costs() {
        let content = Content::builtin().expect("content");
        let r = content.spell_rules().expect("the cost table");
        // (spell, level, its base roll, mana, whether a class ability gives it; every other is
        // learned from scrolls only): the spell scaling design §5.8, §12.1.
        let table = [
            ("arcane_bolt", 6, Some("2d6"), 11, true),
            ("barkskin", 3, None, 3, true),
            ("blessing", 3, None, 3, true),
            ("blight", 7, Some("2d6"), 17, true),
            ("burning_hands", 4, Some("1d4"), 6, true),
            ("call_lightning", 11, Some("2d6"), 20, true),
            ("cause_wounds", 4, Some("1d6"), 3, true),
            ("cure_wounds", 3, Some("2d6"), 2, true),
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
            ("mend", 1, Some("1d3"), 1, true),
            ("ray_of_frost", 1, Some("1d6"), 1, true),
            ("identify", 6, None, 60, false),
            ("remove_disease", 11, None, 11, true),
            ("scorching_ray", 7, Some("1d6"), 7, false),
            ("sleep", 6, None, 11, false),
            ("spark", 1, Some("1d4"), 1, true),
            ("web", 7, None, 13, false),
        ];
        for (id, level, roll, mana, given) in table {
            let at = content.spell_id(id).expect("shipped");
            let spell = content.spell(at);
            assert_eq!(spell.level, level, "{id}");
            assert_eq!(r.mana(spell, 0), mana, "{id}");
            let rolled = spell.roll().map(|d| d.to_string());
            assert_eq!(rolled.as_deref(), roll, "{id}");
            assert_eq!(
                content.class_spell(at).is_some(),
                given,
                "{id} given by a class"
            );
        }
        assert_eq!(content.spells().len(), table.len());
    }

    /// The spell scaling design §12.1 and §12.2: every shipped ladder as written. The reach and
    /// degrees ladders are today's values at the scores where the old rounding changed them.
    #[test]
    fn the_shipped_ladders_are_the_designs() {
        let content = Content::builtin().expect("content");
        let w78 = "willpower 14 7, 18 8";
        let w910 = "willpower 14 9, 18 10";
        let w1112 = "willpower 14 11, 18 12";
        let p23 = "presence 14 2, 18 3";
        let mm = "intelligence 13 3d6, 16 4d6, 19 5d6, 25 5d6+2";
        let bolt = "intelligence 13 3d6, 19 3d6+2, 25 3d6+4";
        let table: [(&str, &[(&str, &str)]); 26] = [
            ("arcane_bolt", &[("damage", bolt), ("length", w78)]),
            ("barkskin", &[("degrees", p23)]),
            ("blessing", &[("degrees", p23)]),
            (
                "blight",
                &[("damage", "willpower 16 3d6, 22 3d6+3"), ("range", w910)],
            ),
            (
                "burning_hands",
                &[("damage", "intelligence 13 2d4, 16 3d4, 22 3d4+2")],
            ),
            (
                "call_lightning",
                &[
                    ("damage", "willpower 13 3d6, 16 4d6, 19 5d6, 25 5d6+2"),
                    ("length", w910),
                ],
            ),
            (
                "cause_wounds",
                &[("damage", "willpower 13 2d6, 19 2d8, 25 2d8+2")],
            ),
            ("cure_wounds", &[("heal", "willpower 16 3d6, 22 3d6+3")]),
            ("curse_of_sloth", &[("range", w78), ("degrees", p23)]),
            ("evil_eye", &[("range", w78), ("degrees", p23)]),
            ("fireball", &[("damage", mm), ("range", w1112)]),
            (
                "flame",
                &[
                    ("damage", "intelligence 16 4d6, 19 5d6, 25 5d6+2"),
                    ("range", w1112),
                ],
            ),
            ("frost", &[("damage", bolt), ("range", w1112)]),
            ("haste", &[("degrees", p23)]),
            ("hex_of_weakness", &[("range", w78), ("degrees", p23)]),
            ("identify", &[]),
            ("light", &[("degrees", p23)]),
            (
                "lightning_bolt",
                &[
                    ("damage", mm),
                    ("length", "willpower 14 9, 18 10, 22 11, 26 12"),
                ],
            ),
            (
                "magic_missile",
                &[
                    ("damage", "intelligence 13 2d4, 16 3d4, 22 3d4+1"),
                    ("range", w910),
                ],
            ),
            ("mend", &[("heal", "willpower 16 1d4, 22 1d6")]),
            (
                "ray_of_frost",
                &[("damage", "intelligence 16 1d8"), ("range", w78)],
            ),
            ("remove_disease", &[]),
            (
                "scorching_ray",
                &[
                    ("damage", "intelligence 13 2d6, 16 3d6, 19 4d6, 25 3d6+1d10"),
                    ("range", w910),
                ],
            ),
            ("sleep", &[("range", w910), ("degrees", "presence 14 3")]),
            (
                "spark",
                &[
                    ("damage", "intelligence 16 1d6"),
                    ("range", "willpower 14 6"),
                ],
            ),
            (
                "web",
                &[("range", w910), ("degrees", "presence 14 4, 18 5, 22 6")],
            ),
        ];
        for (id, ladders) in table {
            let spell = content.spell(content.spell_id(id).expect("shipped"));
            let got: Vec<(&str, String)> = (spell.grows.named().into_iter())
                .map(|(dim, l)| (dim, l.to_string()))
                .collect();
            let want: Vec<(&str, String)> =
                ladders.iter().map(|(d, l)| (*d, l.to_string())).collect();
            assert_eq!(got, want, "{id}");
            assert_eq!(
                spell.aim,
                Stat::Dexterity,
                "{id}: every spell aims with Dexterity"
            );
        }
    }
}
