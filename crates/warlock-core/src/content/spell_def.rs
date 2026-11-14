//! Spells as data (spec §8 "Spells", the magic design `docs/design/magic.md` §3, §5, §6, the
//! spell scaling design `docs/design/spell-scaling.md` §4 to §6): the cost table
//! (`data/spells.toml`, [`SpellRules`]) and the spells (`data/spells/`, one `[[spell]]` file
//! each, [`SpellData`]), cast by the one cast path (`world/cast.rs`). Not the bursts and bolts
//! traps cast (`content/spell.rs`), whose code the cast path shares.
//!
//! - **Spell levels**: a spell has a level, 0 to [`MAX_SPELL_LEVEL`]: its base mana (at least
//!   one) and the learning at which a scroll of it teaches for certain. A spell of level 0
//!   or 1 is a cantrip. Nothing says how much a spell of a level may do (decision 16).
//! - **The base and the ladders**: a spell's own roll and reach are its **base**, what anyone
//!   who casts it does; `[spell.grows]` may give each dimension a ladder of the caster's
//!   scores (`content/spell_ladder.rs`, [`Grows`]). No check reads how much a rung does.
//! - **Tags**: a spell's school, kind, shape, and damage type ([`SpellData::tags`]); talents,
//!   abilities, and items key on them.

use super::effect::EffectShape;
use super::spell::{Burst, BurstIce, SpellFields, check_effect, default_verb};
use super::spell_ladder::{Ladder, RungValue, Unit};
use super::spell_type::SpellType;
use super::trap::TrapNoise;
use super::{Content, ContentError, CreatureId, DamageTypeId, StatusId, duplicate_id};
use crate::dice::Roll;
use crate::noise::MAX_LOUDNESS;
use crate::stats::{PerStat, Stat};
use crate::time::Duration;
use crate::weather::Weather;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The highest level a spell may have (the spell scaling design §5.1): 0 (a cantrip) to 70.
pub const MAX_SPELL_LEVEL: u8 = 70;
/// The farthest a ray, bolt, or burst reaches and a line runs, in cells, however far it grows
/// (`world/spell_rung.rs` clamps to it).
pub const MAX_REACH: u8 = 20;
/// The longest a cone is, in cells, however long it grows.
pub const MAX_CONE: u8 = 5;
/// How far (Chebyshev) a jump of a spell with `arcs` reaches; any other arc, one cell.
pub const ARC_REACH: i32 = 2;
/// The creature tag of those with no eyes: a hex with `only_sighted` (Flare) passes them by.
pub const EYELESS: &str = "eyeless";

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
    /// Calls creatures to the caster's side (spec §8 "Allies"; `world/summon_spell.rs`).
    Summon,
}

impl SpellPurpose {
    /// Every kind, in the order the design lists them (§6), the summon last.
    pub const ALL: [SpellPurpose; 7] = [
        SpellPurpose::Damage,
        SpellPurpose::Heal,
        SpellPurpose::Buff,
        SpellPurpose::Hex,
        SpellPurpose::Cleanse,
        SpellPurpose::Utility,
        SpellPurpose::Summon,
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
            SpellPurpose::Summon => "summon",
        }
    }
}

/// What lies on a zone's cells (`[spell.effect] zone`, `world/zone.rs`; the spell types design
/// §7 "Zones"): one reader a kind, each at an existing moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneKind {
    /// Burns who starts a turn on it or steps onto it, and sheds a fire's light.
    Fire,
    /// Poisons who starts a turn in it.
    Fouled,
    /// Trips who steps onto it or off it.
    Slick,
    /// No light reaches it, and a source on it sheds none.
    Dark,
    /// No eye sees into it or through it; light passes (`world/zone.rs`).
    Smoke,
}

impl ZoneKind {
    /// Every kind, in the order they are documented.
    pub const ALL: &'static [ZoneKind] = &[
        Self::Fire,
        Self::Fouled,
        Self::Slick,
        Self::Dark,
        Self::Smoke,
    ];

    /// As written in data and action text.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Fire => "fire",
            Self::Fouled => "fouled",
            Self::Slick => "slick",
            Self::Dark => "dark",
            Self::Smoke => "smoke",
        }
    }

    /// The damage type it does what it does in: the spell that lays it must be of it.
    pub const fn damage_type(self) -> Option<&'static str> {
        match self {
            Self::Fire => Some("fire"),
            Self::Fouled => Some("poison"),
            Self::Slick | Self::Dark | Self::Smoke => None,
        }
    }

    /// Whether it lies on the ground, so only on walkable ground out of the water: a cloud, a
    /// darkness, and smoke hang over anything sight passes through.
    pub const fn on_ground(self) -> bool {
        matches!(self, Self::Fire | Self::Slick)
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
    #[serde(default)]
    mana: ManaSpec,
    /// Read by [`super::spell_type::parse`], once the statuses it names are known.
    #[serde(default, rename = "types")]
    _types: serde::de::IgnoredAny,
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

/// `[mana]`: the floor under a spell's base mana (#1264).
#[derive(Deserialize)]
#[serde(deny_unknown_fields, default)]
struct ManaSpec {
    floor: u16,
}

impl Default for ManaSpec {
    fn default() -> Self {
        Self { floor: 1 }
    }
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
    /// The floor under a spell's base mana: its level, at least this (`[mana] floor`; #1264).
    pub floor: u16,
    /// What every spell of a damage type does beyond its damage (`[types]`).
    pub types: BTreeMap<DamageTypeId, SpellType>,
}

impl SpellRules {
    pub(super) fn parse(path: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let file: RulesFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let s = &file.shapes;
        let shapes = [s.ray, s.bolt, s.touch, s.myself, s.cone, s.line];
        // `shape_cost` indexes it, and its length is the largest radius (a clamp's top).
        if s.burst.is_empty() {
            return Err(err(
                "shapes: `burst` has a value per radius, at least one".into()
            ));
        }
        let r = &file.resolution;
        let resolutions = [r.attack, r.auto, r.area, r.touch];
        Ok(SpellRules {
            shapes,
            burst: s.burst.clone(),
            resolutions,
            floor: file.mana.floor.max(1),
            types: BTreeMap::new(),
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
    /// percent × (100 + `armour`) %, rounded to the nearest once at the end, at least 1: a known
    /// spell costs the same whoever casts it.
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
        let base = u64::from(spell.level).max(u64::from(self.floor));
        let num = factors.iter().product::<u64>() * base;
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
    level: u8,
    school: School,
    kind: SpellPurpose,
    shape: Shape,
    resolution: Resolution,
    damage_type: Option<String>,
    range: Option<u8>,
    radius: Option<u8>,
    length: Option<u8>,
    casts: Option<u8>,
    #[serde(default)]
    open_sky: bool,
    mana: Option<u16>,
    /// A roll (`"2d6"`).
    damage: Option<String>,
    aim: Option<String>,
    /// Added to the study check's difficulty (the owner's Q7 of #828).
    study_difficulty: Option<i32>,
    effect: Option<EffectSpec>,
    cast: Option<CastSpec>,
    grows: Option<GrowsSpec>,
    /// `[spell.ranks]`: what each rank from 2 adds (`spell_rank.rs`).
    ranks: Option<std::collections::BTreeMap<String, super::RankSpec>>,
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
    /// A roll (`"2d6"`).
    heal: Option<String>,
    status: Option<String>,
    severity: Option<u16>,
    #[serde(default)]
    cures: Vec<String>,
    #[serde(default)]
    diseases: bool,
    #[serde(default)]
    identify: bool,
    summon: Option<SummonSpellSpec>,
    /// A hex's charm: the creature it falls on is the caster's ally a while (Beguile).
    charm: Option<CharmSpec>,
    /// `"consumed"` or `"kept"`: it is aimed at a corpse the caster sees.
    corpse: Option<String>,
    rise: Option<RiseSpec>,
    /// A heal's mana back: a roll.
    mana: Option<String>,
    /// A percent of the harm a single-target damage spell does that comes back to its caster.
    leech: Option<u8>,
    /// A single-target damage spell whose kill leaves crystals, not a corpse (Soulglass).
    #[serde(default)]
    crystal: bool,
    /// A burst's targets: only creatures of these tags (Rebuke).
    #[serde(default)]
    only_tags: Vec<String>,
    /// An area hex's targets: only those standing on this ground (Bramble).
    only_terrain: Option<OnlyTerrain>,
    /// An area hex's targets: only those with eyes (Flare).
    #[serde(default)]
    only_sighted: bool,
    /// A line hex's: how many cells it drives everyone on it (Gale).
    push: Option<u8>,
    /// A line hex's: it puts out the fires on its cells (Gale).
    #[serde(default)]
    douses: bool,
    /// A line hex's: the zones of these kinds lose the cells on it (Gale).
    #[serde(default)]
    clears: Vec<ZoneKind>,
    blink: Option<BlinkSpec>,
    weather: Option<WeatherSpec>,
    zone: Option<SpellZone>,
    arcs: Option<u8>,
}

/// A rising spell's `rise = { most, minutes, hit_points, berserk, berserk_faction,
/// berserk_message }` as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RiseSpec {
    most: u8,
    minutes: u16,
    hit_points: u8,
    berserk: u8,
    berserk_faction: String,
    berserk_message: String,
}

/// A hex's `charm = { minutes, mindless, message, ends }` as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CharmSpec {
    minutes: u16,
    #[serde(default)]
    mindless: Vec<String>,
    message: String,
    ends: String,
}

/// A utility spell's `blink = { cells, ends }` as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BlinkSpec {
    cells: u8,
    #[serde(default)]
    ends: Vec<String>,
}

/// A utility spell's `weather = { kind, hours }` as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WeatherSpec {
    kind: Weather,
    hours: u8,
}

/// A summon spell's `summon = { creatures, count, most, minutes, spread }` as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SummonSpellSpec {
    creatures: Vec<String>,
    count: String,
    most: u8,
    minutes: u16,
    spread: Option<i32>,
}

/// `[spell.grows]` as written: one ladder string a dimension, each optional.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GrowsSpec {
    damage: Option<String>,
    heal: Option<String>,
    count: Option<String>,
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
    /// A `summon` spell's count, its roll.
    pub count: Option<Ladder>,
    /// A damage spell's certain jumps (`[spell.effect] arcs`; its `count` key), whole numbers.
    pub jumps: Option<Ladder>,
    /// A ray's, bolt's, or burst's range.
    pub range: Option<Ladder>,
    /// A cone's or a line's length.
    pub length: Option<Ladder>,
    /// A burst's radius.
    pub radius: Option<Ladder>,
    /// A buff's or a hex's degrees of its status; a summon's minutes.
    pub degrees: Option<Ladder>,
}

impl Grows {
    /// The ladder of its roll, harm, healing, or a summon's count.
    pub fn roll(&self) -> Option<&Ladder> {
        (self.damage.as_ref())
            .or(self.heal.as_ref())
            .or(self.count.as_ref())
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
            ("count", &self.count),
            ("jumps", &self.jumps),
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
    /// A `summon`'s call.
    pub summon: Option<SpellSummon>,
    /// That it is aimed at a corpse the caster sees, and what becomes of the corpse
    /// (`world/corpse_spell.rs`).
    pub corpse: Option<CorpseUse>,
    /// A `summon` that raises the corpse it is aimed at, in place of `summon`.
    pub rise: Option<SpellRise>,
    /// A `hex` that makes the creature it falls on the caster's ally a while
    /// (`world/charm.rs`).
    pub charm: Option<SpellCharm>,
    /// A `heal`'s mana back, never past the maximum.
    pub mana: Option<Roll>,
    /// The percent of the harm it does that comes back to its caster (Leech, 1 to 100).
    pub leech: Option<u8>,
    /// Whether a creature it kills leaves crystals of its colour in place of its corpse and its
    /// body's crafting stock (Soulglass, docs/design/enchanting.md §6; `world/crystal.rs`): a
    /// single-target `damage` spell's.
    pub crystal: bool,
    /// A burst's targets: only creatures with one of these tags, friend or foe (Rebuke); empty:
    /// everyone.
    pub only_tags: Vec<String>,
    /// An area hex's targets: only those standing on this ground (Bramble).
    pub only_terrain: Option<OnlyTerrain>,
    /// An area hex's targets: only those with eyes, never a creature tagged
    /// [`EYELESS`](crate::content::EYELESS) (Flare: a flash is light).
    pub only_sighted: bool,
    /// A line hex's push: everyone on it is driven this many cells on (Gale).
    pub push: Option<u8>,
    /// A line hex puts out the fires on its cells (Gale).
    pub douses: bool,
    /// A line hex clears its cells of zones of these kinds (Gale: fire, cloud, and dark).
    pub clears: Vec<ZoneKind>,
    /// A `utility` spell's move (Blink).
    pub blink: Option<SpellBlink>,
    /// A `utility` spell's weather (Cloudburst).
    pub weather: Option<SpellWeather>,
    /// The zone it leaves on its cells (`world/zone.rs`).
    pub zone: Option<SpellZone>,
    /// A `damage` spell's jumps taken without a draw, each up to [`ARC_REACH`] cells (its `count`
    /// ladder grows them; Chain lightning; `world/spell_arc.rs`).
    pub arcs: Option<u8>,
}

/// What a spell aimed at a corpse does to it (`[spell.effect] corpse`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorpseUse {
    /// The corpse is used up: no meal, no second use.
    Consumed,
    /// The corpse stays where it lies.
    Kept,
}

/// A summon spell's raising of a corpse (`[spell.effect] rise`; `world/corpse_spell.rs`): the
/// corpse's own creature type stands up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellRise {
    /// How many of this spell's risen the caster keeps at once: past it the oldest departs.
    pub most: u8,
    /// How long one stays at its base, in minutes (its `degrees` ladder grows it).
    pub minutes: u16,
    /// The hit points it stands at, in percent of its maximum (rounded up).
    pub hit_points: u8,
    /// The percent chance it rises berserk.
    pub berserk: u8,
    /// The faction one that rises berserk belongs to. Read at the cast: a world without it has
    /// no berserk, and the shipped data's is checked by a test (a fixture's own factions need
    /// not hold it).
    pub berserk_faction: String,
    /// Said in place of the spell's `message` when it rises berserk.
    pub berserk_message: String,
}

/// A hex's charm (`[spell.effect] charm`; `world/charm.rs`): the creature it falls on is the
/// caster's ally for its degrees in minutes, then itself again where it stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellCharm {
    /// How long the charm holds at its base, in minutes (its `degrees` ladder grows it).
    pub minutes: u16,
    /// The tags of the creatures with no mind to charm (undead, constructs, vermin).
    pub mindless: Vec<String>,
    /// Said after the creature's name when it takes: "takes you for a friend".
    pub message: String,
    /// Said after its name when the charm ends where the player sees it: "shakes off the charm".
    pub ends: String,
}

/// The ground a spell with `only_terrain` reaches: a flag of `data/terrain/`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnlyTerrain {
    Plants,
}

/// A spell's move (`[spell.effect] blink`; `world/spell_rule.rs`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellBlink {
    /// How far it may carry its caster.
    pub cells: u8,
    /// The statuses it ends on arriving (held, webbed, pinned).
    pub ends: Vec<StatusId>,
}

/// A spell's weather (`[spell.effect] weather`; `world/spell_rule.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellWeather {
    pub kind: Weather,
    /// How long it holds.
    pub lasts: Duration,
}

/// A spell's zone (`[spell.effect] zone = { kind, turns }`): what lies on the cells of its shape
/// after the cast, and for how many of the player's turns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellZone {
    pub kind: ZoneKind,
    pub turns: u16,
}

/// A summon spell's call (`[spell.effect] summon`; spec §8 "Allies": "Summons";
/// `world/summon_spell.rs`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellSummon {
    /// The pool as written: each called creature is one of these, each as likely.
    pub creatures: Vec<String>,
    /// The pool resolved once every creature type is known ([`resolve_summons`]).
    pub pool: Vec<CreatureId>,
    /// How many come at its base (its `count` ladder grows it): at least 1, at most
    /// [`summon::MAX_COUNT`].
    pub count: Roll,
    /// How many of this spell's calls the caster keeps at once: past it the oldest departs.
    pub most: u8,
    /// How long a call stays at its base, in minutes (its `degrees` ladder grows it).
    pub minutes: u16,
    /// How far from the caster they appear (Chebyshev), as an enemy summoner's.
    pub spread: i32,
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
    /// learning at which a scroll of it teaches for certain.
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
    /// Its ranks (`[spell.ranks]`): what a caster who has deepened it gains (`spell_rank.rs`).
    pub ranks: super::Ranks,
    /// The stat whose modifier aims an `attack` or `touch` spell of its caster's own (default
    /// Dexterity; a scroll's or a wand's always Dexterity).
    pub aim: Stat,
    /// Added to the study check's difficulty (0: none); a positive one makes the study a check
    /// at any learning (`world/learn.rs`).
    pub study_difficulty: i32,
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

    /// Whether it asks for a cell when cast: anything but a spell on oneself or a burst with
    /// `range = 0`, cast about the caster.
    pub fn aimed(&self) -> bool {
        (self.shape != Shape::Myself && !self.about_caster()) || self.aims_corpse()
    }

    /// Whether it is a burst with `range = 0`: it goes off on the caster's own cell.
    pub fn about_caster(&self) -> bool {
        self.shape == Shape::Burst && self.range == Some(0)
    }

    /// Whether it is aimed at a corpse the caster sees (Charnel burst, Rouse the dead, Grave
    /// feast).
    pub fn aims_corpse(&self) -> bool {
        self.effect.corpse.is_some()
    }

    /// Whether it is a healing spell (Cure wounds, Mend, Benison): a `heal` that does not feed
    /// on a corpse (Grave feast drains life, it does not heal). One on an undead harms it
    /// (`world/heal_harm.rs`).
    pub fn heals_wounds(&self) -> bool {
        self.kind == SpellPurpose::Heal && !self.aims_corpse()
    }

    /// Whether it asks for a stack the caster carries when cast (Identify).
    pub fn aims_item(&self) -> bool {
        self.effect.identify
    }

    /// Its base roll, harm, healing, or a summon's count.
    pub fn roll(&self) -> Option<Roll> {
        (self.damage)
            .or(self.effect.heal)
            .or(self.effect.summon.as_ref().map(|s| s.count))
    }

    /// The base its degrees ladder grows: a buff's, a hex's, or a damage spell's status's
    /// severity, or a summon's minutes.
    pub fn base_degrees(&self) -> Option<u16> {
        (self.effect.status.map(|(_, n)| n))
            .or(self.effect.summon.as_ref().map(|s| s.minutes))
            .or(self.effect.rise.as_ref().map(|r| r.minutes))
            .or(self.effect.charm.as_ref().map(|c| c.minutes))
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
    /// cone's or a line's length, a touch's 1; `None` on oneself or about oneself.
    pub fn base_range(&self) -> Option<i32> {
        match self.shape {
            Shape::Touch => Some(1),
            Shape::Myself => None,
            Shape::Burst if self.about_caster() => None,
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
        Some((path, text)) => Some(SpellRules {
            types: super::spell_type::parse(path, text, content)?,
            ..SpellRules::parse(path, text)?
        }),
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

/// A base roll (`damage`, `heal`, a heal's `mana`) as written: a [`Roll`] that never rolls
/// below 0 (a negative roll would turn harm into healing and back). Nothing checks how much it
/// rolls (decision 16).
fn base_roll(name: &str, text: &str) -> Result<Roll, String> {
    let roll: Roll = text.parse()?;
    if roll.min() < 0 {
        return Err(format!("`{name}` must never roll below 0, not `{roll}`"));
    }
    Ok(roll)
}

/// One spell, checked.
fn resolve(spec: SpellSpec, rules: &SpellRules, content: &Content) -> Result<SpellData, String> {
    let level = spec.level;
    if !spec.resolution.fits(spec.shape) {
        return Err(format!(
            "a `{}` cannot resolve as `{}`",
            spec.shape.id(),
            spec.resolution.id()
        ));
    }
    // Its base reach: each shape has exactly its own (casting clamps it to the engine's limits).
    let shape = spec.shape;
    let reach = |name: &str, value: Option<u8>, wanted: bool| match (value, wanted) {
        (Some(v), true) => Ok(Some(v)),
        (None, true) => Err(format!("a `{}` needs `{name}`", shape.id())),
        (Some(_), false) => Err(format!("a `{}` has no `{name}`", shape.id())),
        (None, false) => Ok(None),
    };
    let ranged = matches!(shape, Shape::Ray | Shape::Bolt | Shape::Burst);
    let burst = shape == Shape::Burst;
    // A burst may go off about its caster (Thunderclap, Benison): no aim, and no reach to grow.
    let about = burst && spec.range == Some(0);
    if about && !matches!(spec.kind, SpellPurpose::Damage | SpellPurpose::Heal) {
        return Err("only a `damage` or a `heal` burst may have `range = 0`".into());
    }
    if spec.kind == SpellPurpose::Heal && shape.is_area() && !about {
        return Err("a `heal` over an area is a burst with `range = 0`, about its caster".into());
    }
    if about && spec.grows.as_ref().is_some_and(|g| g.range.is_some()) {
        return Err("a burst with `range = 0` has no `range` ladder".into());
    }
    let range = if about {
        Some(0)
    } else {
        reach("range", spec.range, ranged)?
    };
    let radius = reach("radius", spec.radius, burst)?;
    let long = matches!(shape, Shape::Cone | Shape::Line);
    let length = reach("length", spec.length, long)?;
    let casts = spec.casts.unwrap_or(1);
    // A cast's time is its price times `casts`: none would take no time at all.
    if casts == 0 {
        return Err("`casts` is at least 1".into());
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
    let effect = match spec.effect {
        Some(e) => effect(e, (spec.kind, shape), content)?,
        None => SpellEffect::default(),
    };
    if spec.kind == SpellPurpose::Summon
        && (shape != Shape::Myself || spec.resolution != Resolution::Auto)
    {
        return Err(
            "a `summon` spell is cast on `self` with `auto`: they appear about the caster".into(),
        );
    }
    if effect.arcs.is_some() {
        if !matches!(shape, Shape::Ray | Shape::Bolt | Shape::Touch | Shape::Line) {
            return Err("`arcs` is a ray's, a bolt's, a touch's, or a line's".into());
        }
        let rule = damage_type.and_then(|kind| rules.types.get(&kind));
        if rule.is_none_or(|r| r.arc.is_none()) {
            return Err("`arcs` needs a damage type that arcs (`[types]`)".into());
        }
    }
    let study_difficulty = spec.study_difficulty.unwrap_or(0);
    if effect.zone.is_some() && !matches!(shape, Shape::Burst | Shape::Line) {
        return Err("a `zone` lies on a `burst`'s or a `line`'s cells".into());
    }
    if effect.identify && shape != Shape::Myself {
        return Err(
            "an `identify` spell is cast on `self`: it asks for a stack, not a cell".into(),
        );
    }
    if (effect.blink.is_some() || effect.weather.is_some()) && shape != Shape::Myself {
        return Err("a `blink` or a `weather` spell is cast on `self`".into());
    }
    corpse_fits(&effect, spec.kind, shape)?;
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
    // A damage spell's status too (Flay's Bleeding): it falls on everyone harmed.
    let status_degrees = matches!(
        spec.kind,
        SpellPurpose::Buff | SpellPurpose::Hex | SpellPurpose::Damage
    )
    .then_some(effect.status)
    .flatten()
    .map(|(_, n)| n);
    let summon_minutes = (effect.summon.as_ref().map(|s| s.minutes))
        .or(effect.rise.as_ref().map(|r| r.minutes))
        .or(effect.charm.as_ref().map(|c| c.minutes));
    let bases = Bases {
        damage: damage.is_some(),
        heal: effect.heal.is_some(),
        count: effect.summon.is_some() || effect.arcs.is_some(),
        jumps: effect.arcs.is_some(),
        range: range.is_some(),
        length: length.is_some(),
        radius: radius.is_some(),
        degrees: status_degrees.or(summon_minutes).is_some(),
    };
    let grows = match spec.grows {
        Some(g) => grows(g, &bases)?,
        None => Grows::default(),
    };
    let rolls_to_hit = spec.kind == SpellPurpose::Damage
        && matches!(spec.resolution, Resolution::Attack | Resolution::Touch)
        && matches!(shape, Shape::Ray | Shape::Bolt | Shape::Touch);
    let ranks = match spec.ranks {
        Some(table) => super::Ranks::parse(
            table,
            super::spell_rank::RankBases {
                roll: bases.damage || bases.heal || (bases.count && !bases.jumps),
                reach: bases.range || bases.length,
                radius: bases.radius,
                degrees: bases.degrees,
                to_hit: rolls_to_hit,
            },
        )?,
        None => super::Ranks::default(),
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
        ranks,
        aim,
        study_difficulty,
    })
}

/// Which dimensions a spell has, what its ladders are checked against.
struct Bases {
    damage: bool,
    heal: bool,
    /// A summon's count, or a damage spell's jumps.
    count: bool,
    /// Whether `count` is a damage spell's jumps.
    jumps: bool,
    range: bool,
    length: bool,
    radius: bool,
    /// A buff's, a hex's, or a damage spell's status degrees; a summon's minutes.
    degrees: bool,
}

/// `[spell.grows]` checked against the spell's `bases` (the spell scaling design §6.3): a
/// ladder only for a dimension the spell has (another would never be read), and a jump count
/// in whole numbers. Nothing reads how much a rung does (decision 16); casting clamps reach to
/// the engine's limits (`world/spell_rung.rs`).
fn grows(spec: GrowsSpec, bases: &Bases) -> Result<Grows, String> {
    let ladder = |name: &str, text: Option<String>, has: bool, unit: Unit, why: &str| {
        let Some(text) = text else {
            return Ok(None);
        };
        if !has {
            return Err(format!("grows: `{name}` is for {why}"));
        }
        Ladder::parse(&text, unit)
            .map(Some)
            .map_err(|e| format!("grows: {e}"))
    };
    let why = "a `summon` spell's count, or a `damage` spell's `arcs`";
    let count = ladder("count", spec.count, bases.count, Unit::Roll, why)?;
    if bases.jumps
        && let Some(ladder) = &count
        && let Some(roll) = (ladder.values())
            .filter_map(RungValue::roll)
            .find(|r| !r.is_fixed())
    {
        return Err(format!(
            "grows: `count`'s rung `{roll}` is a number of jumps: no dice"
        ));
    }
    let (count, jumps) = if bases.jumps {
        (None, count)
    } else {
        (count, None)
    };
    Ok(Grows {
        damage: ladder(
            "damage",
            spec.damage,
            bases.damage,
            Unit::Roll,
            "a `damage` spell's roll",
        )?,
        heal: ladder(
            "heal",
            spec.heal,
            bases.heal,
            Unit::Roll,
            "a `heal` spell's roll",
        )?,
        count,
        jumps,
        range: ladder(
            "range",
            spec.range,
            bases.range,
            Unit::Cells,
            "a ray's, a bolt's, or a burst's range",
        )?,
        length: ladder(
            "length",
            spec.length,
            bases.length,
            Unit::Cells,
            "a cone's or a line's length",
        )?,
        radius: ladder(
            "radius",
            spec.radius,
            bases.radius,
            Unit::Cells,
            "a burst's radius",
        )?,
        degrees: ladder(
            "degrees",
            spec.degrees,
            bases.degrees,
            Unit::Degrees,
            "a buff's, a hex's, or a damage spell's status, or a summon's minutes",
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
/// drawn), blast and heat or frost only for a burst, a noise at most [`MAX_LOUDNESS`] (which
/// bounds how far a sound is carried). Its effect is checked against `data/effects.toml` once
/// effects are loaded ([`check_effects`]).
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
    if let Some(noise) = c.noise
        && noise.loudness > MAX_LOUDNESS
    {
        return Err(format!("cast: `noise` loudness is at most {MAX_LOUDNESS}"));
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

/// A spell's `[spell.effect]`, checked: only what its kind and shape use.
fn effect(
    e: EffectSpec,
    (kind, shape): (SpellPurpose, Shape),
    content: &Content,
) -> Result<SpellEffect, String> {
    let only = |field: &str, present: bool, kinds: &[SpellPurpose]| {
        if present && !kinds.contains(&kind) {
            Err(format!("a `{}` spell has no `{field}`", kind.id()))
        } else {
            Ok(())
        }
    };
    let shaped = |field: &str, present: bool, shapes: &[Shape]| {
        if present && !shapes.contains(&shape) {
            Err(format!("a `{}` has no `{field}`", shape.id()))
        } else {
            Ok(())
        }
    };
    let single = [Shape::Ray, Shape::Bolt, Shape::Touch];
    let hexes = [SpellPurpose::Damage, SpellPurpose::Hex];
    only("leech", e.leech.is_some(), &[SpellPurpose::Damage])?;
    shaped("leech", e.leech.is_some(), &single)?;
    only("crystal", e.crystal, &[SpellPurpose::Damage])?;
    shaped("crystal", e.crystal, &single)?;
    only("only_tags", !e.only_tags.is_empty(), &hexes)?;
    shaped("only_tags", !e.only_tags.is_empty(), &[Shape::Burst])?;
    only(
        "only_terrain",
        e.only_terrain.is_some(),
        &[SpellPurpose::Hex],
    )?;
    shaped("only_terrain", e.only_terrain.is_some(), &[Shape::Burst])?;
    only("push", e.push.is_some(), &[SpellPurpose::Hex])?;
    shaped("push", e.push.is_some(), &[Shape::Line])?;
    only("douses", e.douses, &[SpellPurpose::Hex])?;
    shaped("douses", e.douses, &[Shape::Line])?;
    only("clears", !e.clears.is_empty(), &[SpellPurpose::Hex])?;
    shaped("clears", !e.clears.is_empty(), &[Shape::Line])?;
    only("blink", e.blink.is_some(), &[SpellPurpose::Utility])?;
    only("weather", e.weather.is_some(), &[SpellPurpose::Utility])?;
    only("heal", e.heal.is_some(), &[SpellPurpose::Heal])?;
    // A damage spell's status falls on everyone it harms (blight's poison).
    let marks = [SpellPurpose::Buff, SpellPurpose::Hex, SpellPurpose::Damage];
    only("status", e.status.is_some(), &marks)?;
    only("severity", e.severity.is_some(), &marks)?;
    only("cures", !e.cures.is_empty(), &[SpellPurpose::Cleanse])?;
    only("diseases", e.diseases, &[SpellPurpose::Cleanse])?;
    only("identify", e.identify, &[SpellPurpose::Utility])?;
    only("summon", e.summon.is_some(), &[SpellPurpose::Summon])?;
    only("arcs", e.arcs.is_some(), &[SpellPurpose::Damage])?;
    only("rise", e.rise.is_some(), &[SpellPurpose::Summon])?;
    only("charm", e.charm.is_some(), &[SpellPurpose::Hex])?;
    shaped("charm", e.charm.is_some(), &[Shape::Ray, Shape::Bolt])?;
    if e.charm.is_some() && (e.status.is_some() || e.push.is_some() || e.zone.is_some()) {
        return Err("a `charm` hex has no `status`, `push`, or `zone`".into());
    }
    only("mana", e.mana.is_some(), &[SpellPurpose::Heal])?;
    let lays = [SpellPurpose::Damage, SpellPurpose::Hex];
    only("zone", e.zone.is_some(), &lays)?;
    let summon = match e.summon {
        Some(s) => Some(summon_call(s)?),
        None => None,
    };
    let rise = e.rise.map(spell_rise);
    let charm = e.charm.map(|c| SpellCharm {
        minutes: c.minutes,
        mindless: c.mindless,
        message: c.message,
        ends: c.ends,
    });
    let corpse = match e.corpse.as_deref() {
        None => None,
        Some("consumed") => Some(CorpseUse::Consumed),
        Some("kept") => Some(CorpseUse::Kept),
        Some(other) => {
            return Err(format!("`corpse` is `consumed` or `kept`, not `{other}`"));
        }
    };
    let mana = match &e.mana {
        Some(m) => Some(base_roll("mana", m)?),
        None => None,
    };
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
    let blink = match e.blink {
        Some(b) => Some(SpellBlink {
            cells: b.cells,
            ends: (b.ends.iter())
                .map(|name| status(name))
                .collect::<Result<Vec<_>, _>>()?,
        }),
        None => None,
    };
    let weather = e.weather.map(|w| SpellWeather {
        kind: w.kind,
        lasts: Duration::hours(u64::from(w.hours)),
    });
    Ok(SpellEffect {
        heal,
        status: marked,
        cures,
        diseases: e.diseases,
        identify: e.identify,
        summon,
        corpse,
        rise,
        charm,
        mana,
        leech: e.leech,
        crystal: e.crystal,
        only_tags: e.only_tags,
        only_terrain: e.only_terrain,
        only_sighted: e.only_sighted,
        push: e.push,
        douses: e.douses,
        clears: e.clears,
        blink,
        weather,
        zone: e.zone,
        arcs: e.arcs,
    })
}

/// A spell's `corpse` and `rise` fit together and with its kind and shape: `corpse` on a burst of
/// harm, a touch that heals, or a summon on `self` that raises (`rise`, which asks for `corpse`
/// and stands in for `summon`).
fn corpse_fits(effect: &SpellEffect, kind: SpellPurpose, shape: Shape) -> Result<(), String> {
    if effect.rise.is_some() && effect.summon.is_some() {
        return Err("a spell has `summon` or `rise`, not both".into());
    }
    if effect.rise.is_some() && effect.corpse.is_none() {
        return Err("`rise` raises a corpse: it needs `corpse`".into());
    }
    let fits = match kind {
        SpellPurpose::Damage => shape == Shape::Burst,
        SpellPurpose::Heal => shape == Shape::Touch,
        SpellPurpose::Summon => shape == Shape::Myself && effect.rise.is_some(),
        _ => false,
    };
    if effect.corpse.is_some() && !fits {
        return Err(
            "`corpse` is a burst's of harm, a touch's that heals, or a `rise`'s on `self`".into(),
        );
    }
    Ok(())
}

/// A rising spell's `rise` table.
fn spell_rise(r: RiseSpec) -> SpellRise {
    SpellRise {
        most: r.most,
        minutes: r.minutes,
        hit_points: r.hit_points,
        berserk: r.berserk,
        berserk_faction: r.berserk_faction,
        berserk_message: r.berserk_message,
    }
}

/// A summon spell's `summon` table, checked as far as it can be before the creature types are
/// known ([`resolve_summons`] resolves its pool): a pool of at least one (the call draws from
/// it), a count that parses, a spread (default 2).
fn summon_call(s: SummonSpellSpec) -> Result<SpellSummon, String> {
    if s.creatures.is_empty() {
        return Err("summon: `creatures` names at least one".into());
    }
    let count: Roll = s.count.parse().map_err(|e| format!("summon: {e}"))?;
    let spread = s.spread.unwrap_or(2);
    Ok(SpellSummon {
        creatures: s.creatures,
        pool: Vec::new(),
        count,
        most: s.most,
        minutes: s.minutes,
        spread,
    })
}

/// With the creature types loaded, every spell's `only_tags` names a tag some creature carries.
pub(super) fn check_only_tags(content: &Content) -> Result<(), String> {
    for spell in content.spells() {
        for tag in &spell.effect.only_tags {
            if content.tagged(tag).next().is_none() {
                return Err(format!(
                    "spell `{}`: `only_tags`: no creature is tagged `{tag}`",
                    spell.id
                ));
            }
        }
    }
    Ok(())
}

/// Resolves every summon spell's pool once the creature types are known: each a known type, not
/// the player's, not a unique, not an animated thing.
pub(super) fn resolve_summons(content: &mut Content) -> Result<(), ContentError> {
    for i in 0..content.spells.len() {
        let Some(call) = &content.spells[i].effect.summon else {
            continue;
        };
        let mut pool = Vec::new();
        for name in &call.creatures {
            let err = |m: String| {
                ContentError::new(
                    "data/spells/",
                    format!("spell `{}`: summon: {m}", content.spells[i].id),
                )
            };
            let c = (content.creature_id(name))
                .ok_or_else(|| err(format!("unknown creature `{name}`")))?;
            let def = content.creature(c);
            if def.id == "player" {
                return Err(err(format!("`{name}` cannot be called")));
            }
            pool.push(c);
        }
        if let Some(call) = content.spells[i].effect.summon.as_mut() {
            call.pool = pool;
        }
    }
    Ok(())
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
            ("burst = [180, 240, 300]", "burst = []", "at least one"),
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
    fn the_small_rules_fields_fit_only_their_kinds_and_shapes() {
        let spell = |kind: &str, shape: &str, rest: &str| {
            let resolution = match shape {
                "burst" | "line" | "cone" => "area",
                _ => "auto",
            };
            format!(
                "[[spell]]\nid = \"x\"\nname = \"X\"\ndescription = \"x\"\nlevel = 5\n\
                 school = \"evocation\"\nkind = \"{kind}\"\nshape = \"{shape}\"\n\
                 resolution = \"{resolution}\"\n{rest}\n"
            )
        };
        let harm = "damage_type = \"fire\"\ndamage = \"1d6\"\n";
        let heal = "effect = { heal = \"1d6\" }\n";
        let charm = "effect = { charm = { minutes = 5, message = \"m\", ends = \"e\" } }\n";
        let refused = [
            // `range = 0`: a damage's or a heal's burst, and nothing to grow.
            (
                spell(
                    "hex",
                    "burst",
                    "range = 0\nradius = 1\neffect = { status = \"webbed\" }",
                ),
                "`range = 0`",
            ),
            (
                spell(
                    "damage",
                    "burst",
                    &format!(
                        "range = 0\nradius = 1\n{harm}[spell.grows]\nrange = \"willpower 14 2\""
                    ),
                ),
                "no `range` ladder",
            ),
            // A heal over an area is about its caster.
            (
                spell("heal", "burst", &format!("range = 3\nradius = 1\n{heal}")),
                "about its caster",
            ),
            // The rest, each its kind's and shape's.
            (
                spell(
                    "damage",
                    "burst",
                    &format!("range = 3\nradius = 1\n{harm}effect = {{ leech = 50 }}"),
                ),
                "a `burst` has no `leech`",
            ),
            (
                spell(
                    "damage",
                    "burst",
                    &format!("range = 3\nradius = 1\n{harm}effect = {{ crystal = true }}"),
                ),
                "a `burst` has no `crystal`",
            ),
            (
                spell("hex", "ray", "range = 3\neffect = { crystal = true }"),
                "a `hex` spell has no `crystal`",
            ),
            (
                spell(
                    "buff",
                    "self",
                    "effect = { status = \"hasted\", only_terrain = \"plants\" }",
                ),
                "a `buff` spell has no `only_terrain`",
            ),
            (
                spell(
                    "hex",
                    "burst",
                    "range = 3\nradius = 1\neffect = { push = 1 }",
                ),
                "a `burst` has no `push`",
            ),
            (
                spell("hex", "burst", &format!("range = 3\nradius = 1\n{charm}")),
                "a `burst` has no `charm`",
            ),
            (
                spell("buff", "self", charm),
                "a `buff` spell has no `charm`",
            ),
            (
                spell(
                    "hex",
                    "ray",
                    "range = 3\neffect = { status = \"sluggish\", charm = { minutes = 5, \
                     message = \"m\", ends = \"e\" } }",
                ),
                "a `charm` hex has no `status`",
            ),
            (
                spell(
                    "utility",
                    "self",
                    "effect = { blink = { cells = 2, ends = [\"nope\"] } }",
                ),
                "unknown status",
            ),
            (
                spell(
                    "utility",
                    "burst",
                    "range = 3\nradius = 1\neffect = { weather = { kind = \"rain\", hours = 2 } }",
                ),
                "cast on `self`",
            ),
        ];
        for (text, why) in refused {
            let e = spells(&text).expect_err(&text).message;
            assert!(e.contains(why), "{e}: wanted {why}");
        }
        let about = spells(&spell(
            "heal",
            "burst",
            &format!("range = 0\nradius = 1\n{heal}"),
        ));
        let about = &about.expect("a heal about its caster")[0];
        assert!(about.about_caster() && !about.aimed());
        assert_eq!(about.base_range(), None);
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
            ("range = 8\n", "", "needs `range`"),
            ("radius = 2", "radius = 2\nlength = 3", "has no `length`"),
            (
                "damage_type = \"fire\"",
                "damage_type = \"holy\"",
                "unknown damage type",
            ),
            ("\"2d6\"", "\"2x6\"", "bad roll"),
            ("\"2d6\"", "\"-3\"", "never roll below 0"),
            ("damage = \"2d6\"", "", "needs `damage`"),
            ("level = 11\n", "", "missing field `level`"),
            ("level = 11", "level = 11\ntier = 3", "unknown field `tier`"),
            (
                "damage = \"2d6\"",
                "damage = { dice = \"2d6\", per_levels = 4, max_dice = 3 }",
                "invalid type",
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
            (
                "level = 11",
                "level = 11\ncasts = 0",
                "`casts` is at least 1",
            ),
            (
                "damage = \"2d6\"",
                "damage = \"2d6\"\ncast = { effect = \"fireball\", frost = 10, heat = true }",
                "`heat` or `frost`, not both",
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
    /// known, its scores rising. Nothing reads how much a rung does (decision 16).
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
        for (body, grows, why) in [
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
                "`degrees` is for a buff's, a hex's, or a damage spell's",
            ),
            (
                burst,
                "damage = \"cunning 13 3d6\"",
                "unknown stat `cunning`",
            ),
            (burst, "range = \"willpower 18 10, 14 9\"", "rise strictly"),
            (
                ray,
                "radius = \"willpower 14 2\"",
                "`radius` is for a burst's radius",
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
        // (spell, armour %, mana): the spell scaling design §5.3 and §5.8, at the mana floor of
        // 4 (#1264: a level below it counts as 4).
        let table = [
            ("spark", 0, 4),           // level 1, single target: the floor
            ("magic_missile", 0, 4),   // level 3: the floor
            ("scorching_ray", 0, 7),   // level 7
            ("burning_hands", 0, 6),   // a cone: 4 × 1.6 = 6.4
            ("cause_wounds", 0, 3),    // touch: 4 × 0.7 = 2.8
            ("fireball", 0, 26),       // a burst of radius 2: 11 × 2.4 = 26.4
            ("lightning_bolt", 0, 20), // a line: 11 × 1.8 = 19.8
            ("magic_missile", 40, 6),  // plate: 4 × 1.4 = 5.6
            ("fireball", 30, 34),      // banded mail: 26.4 × 1.3 = 34.32
        ];
        for (id, armour, mana) in table {
            assert_eq!(at(id, armour), mana, "{id} in {armour} %");
        }
        // Area is dearer than a single target of its level.
        let ball = spell("fireball");
        assert!(r.mana(&ball, 0) > i32::from(ball.level));
        // A level-0 spell costs the floor; without a `[mana]` table the floor is 1.
        let light = SpellData {
            level: 0,
            ..spell("spark")
        };
        assert_eq!(r.mana(&light, 0), 4);
        let plain = SpellRules::parse(
            "data/spells.toml",
            "[shapes]\nray = 100\nbolt = 100\ntouch = 100\nself = 100\ncone = 160\nline = 180\n\
             burst = [180]\n[resolution]\nattack = { cost = 100 }\nauto = { cost = 100 }\n\
             area = { cost = 100 }\ntouch = { cost = 70 }\n",
        )
        .expect("valid");
        assert_eq!(plain.floor, 1);
        assert_eq!(plain.mana(&light, 0), 1);
    }

    #[test]
    fn the_shipped_spells_are_at_the_designs_levels_and_costs() {
        let content = Content::builtin().expect("content");
        let r = content.spell_rules().expect("the cost table");
        // (spell, level, its base roll, mana, whether a class ability gives it; every other is
        // learned from scrolls only): the spell scaling design §5.8, §12.1.
        let table = [
            ("acid_arrow", 5, Some("2d6"), 5, false),
            ("acid_splash", 3, Some("2d4"), 7, false),
            ("arcane_bolt", 6, Some("4d6"), 11, true),
            ("balefire", 16, Some("4d6"), 29, false),
            ("barkskin", 3, None, 4, true),
            ("beguile", 18, None, 18, true),
            ("benison", 12, Some("2d6"), 22, true),
            ("bind", 9, None, 9, true),
            ("blessing", 3, None, 4, true),
            ("blight", 7, Some("4d6"), 17, true),
            ("blink", 6, None, 6, true),
            ("bramble", 8, None, 19, true),
            ("brand", 7, None, 7, true),
            ("burning_hands", 1, Some("2d4"), 6, true),
            ("call_lightning", 11, Some("4d6"), 20, true),
            ("call_of_the_forest", 20, Some("+1"), 20, true),
            ("call_of_the_wild", 5, Some("+1"), 5, true),
            ("chain_lightning", 20, Some("6d6"), 20, true),
            ("charnel_burst", 9, Some("4d6"), 16, true),
            ("call_the_elements", 35, Some("+1"), 35, true),
            ("cause_wounds", 4, Some("2d6"), 3, true),
            ("cloudburst", 20, None, 20, true),
            ("cure_wounds", 3, Some("2d6"), 3, true),
            ("curse_of_sloth", 3, None, 4, true),
            ("evil_eye", 0, None, 4, true),
            ("fear", 10, None, 18, true),
            ("fire_cloak", 12, None, 12, false),
            ("fireball", 11, Some("4d6"), 26, false),
            ("flame", 12, Some("6d6"), 29, true),
            ("flare", 2, None, 7, false),
            ("flay", 8, Some("2d4"), 8, true),
            ("frost", 6, Some("4d6"), 14, true),
            ("fume", 4, None, 10, false),
            ("gale", 11, None, 20, true),
            ("glamour", 8, None, 8, false),
            ("grave_breath", 1, Some("2d4"), 6, true),
            ("grave_feast", 6, Some("2d6"), 4, true),
            ("grave_spear", 14, Some("4d6"), 25, false),
            ("leech", 10, Some("4d6"), 10, true),
            ("haste", 11, None, 11, false),
            ("hex_of_weakness", 3, None, 4, true),
            ("light", 0, None, 4, false),
            ("lightning_bolt", 11, Some("4d6"), 20, false),
            ("magic_missile", 3, Some("2d4"), 4, true),
            ("mend", 1, Some("1d3"), 3, true),
            ("mesmerise", 5, None, 5, true),
            ("mirk", 12, None, 29, false),
            ("nightsight", 4, None, 4, true),
            ("pitch_fire", 8, Some("2d6"), 14, false),
            ("ray_of_frost", 1, Some("2d6"), 4, true),
            ("rebuke", 14, Some("4d6"), 34, true),
            ("identify", 6, None, 60, false),
            ("reek", 13, Some("2d4"), 31, false),
            ("remove_disease", 11, None, 11, true),
            ("renewal", 10, None, 10, true),
            ("rime", 9, Some("4d6"), 14, true),
            ("rouse_the_dead", 24, None, 24, true),
            ("scorching_ray", 7, Some("2d6"), 7, false),
            ("see_invisible", 7, None, 7, false),
            ("sense_the_quick", 5, None, 5, true),
            ("shatter", 8, Some("4d6"), 14, true),
            ("silence", 11, None, 20, true),
            ("sleep", 6, None, 11, false),
            ("slick", 6, None, 11, false),
            ("slumber", 15, None, 15, false),
            ("soulglass", 11, Some("3d6"), 11, true),
            ("spark", 1, Some("2d4"), 4, true),
            ("sunfire", 30, Some("10d6"), 90, true),
            ("thunderclap", 7, Some("2d6"), 13, true),
            ("unbinding", 8, None, 8, true),
            ("water_walking", 5, None, 5, true),
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
        let mm = "intelligence 13 6d6, 16 8d6, 19 10d6, 25 10d6+2";
        let bolt = "intelligence 13 6d6, 19 6d6+2, 25 6d6+4";
        let table: [(&str, &[(&str, &str)]); 44] = [
            (
                "acid_arrow",
                &[
                    ("damage", "intelligence 13 4d6, 16 6d6, 25 6d6+2"),
                    ("range", w910),
                ],
            ),
            ("bind", &[("range", w78), ("degrees", "presence 14 3")]),
            ("fear", &[("range", w910), ("degrees", "presence 14 3")]),
            (
                "flay",
                &[
                    ("damage", "willpower 16 2d6, 22 2d8"),
                    ("range", w78),
                    ("degrees", "presence 14 3, 18 4"),
                ],
            ),
            ("glamour", &[("degrees", p23)]),
            (
                "grave_spear",
                &[
                    ("damage", "willpower 13 6d6, 16 8d6, 19 10d6, 25 10d6+2"),
                    ("length", w78),
                ],
            ),
            (
                "mesmerise",
                &[("range", w78), ("degrees", "presence 14 3, 18 4")],
            ),
            ("nightsight", &[]),
            ("renewal", &[("degrees", p23)]),
            (
                "rime",
                &[
                    ("damage", "willpower 13 6d6, 16 8d6, 22 8d6+2"),
                    ("length", "willpower 18 5"),
                ],
            ),
            ("silence", &[("range", w910), ("radius", "willpower 18 2")]),
            ("slumber", &[("range", w78)]),
            (
                "sunfire",
                &[
                    (
                        "damage",
                        "intelligence 16 12d6, 19 14d6, 22 16d6, 25 16d6+4",
                    ),
                    ("range", w1112),
                ],
            ),
            ("unbinding", &[]),
            ("arcane_bolt", &[("damage", bolt), ("length", w78)]),
            ("barkskin", &[("degrees", p23)]),
            ("blessing", &[("degrees", p23)]),
            ("brand", &[("degrees", p23)]),
            (
                "blight",
                &[("damage", "willpower 16 6d6, 22 6d6+3"), ("range", w910)],
            ),
            (
                "burning_hands",
                &[("damage", "intelligence 13 2d6, 16 4d4, 22 4d4+2")],
            ),
            (
                "call_lightning",
                &[
                    ("damage", "willpower 13 6d6, 16 8d6, 19 10d6, 25 10d6+2"),
                    ("length", w910),
                ],
            ),
            (
                "cause_wounds",
                &[("damage", "willpower 13 4d6, 19 4d8, 25 4d8+2")],
            ),
            ("cure_wounds", &[("heal", "willpower 16 3d6, 22 3d6+3")]),
            ("curse_of_sloth", &[("range", w78), ("degrees", p23)]),
            ("evil_eye", &[("range", w78), ("degrees", p23)]),
            ("fire_cloak", &[("degrees", p23)]),
            ("fireball", &[("damage", mm), ("range", w1112)]),
            (
                "flame",
                &[
                    ("damage", "intelligence 16 8d6, 19 10d6, 25 10d6+2"),
                    ("range", w1112),
                ],
            ),
            ("frost", &[("damage", bolt), ("range", w1112)]),
            (
                "grave_breath",
                &[("damage", "willpower 13 2d6, 16 4d4, 22 4d4+2")],
            ),
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
                    ("damage", "intelligence 13 4d4, 16 6d4, 22 6d4+1"),
                    ("range", w910),
                ],
            ),
            ("mend", &[("heal", "willpower 16 1d4, 22 1d6")]),
            (
                "ray_of_frost",
                &[("damage", "intelligence 16 2d8"), ("range", w78)],
            ),
            ("remove_disease", &[]),
            (
                "scorching_ray",
                &[
                    ("damage", "intelligence 13 4d6, 16 6d6, 19 8d6, 25 6d6+2d10"),
                    ("range", w910),
                ],
            ),
            ("sleep", &[("range", w910), ("degrees", "presence 14 3")]),
            (
                "spark",
                &[
                    ("damage", "intelligence 16 2d6"),
                    ("range", "willpower 14 6"),
                ],
            ),
            ("water_walking", &[("degrees", p23)]),
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
