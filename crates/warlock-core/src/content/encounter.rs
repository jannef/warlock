//! Encounters on the overworld (`assets/data/encounters.toml`, spec §7 "Leaving an outdoor
//! area", "Overworld encounters"; `docs/dev/encounters.md`): the edge rule, how near a hostile
//! creature holds the player in an outdoor area when they walk off its edge
//! (`world/edge_leave.rs`); the numbers of meeting someone on the way ([`Meetings`]); and each
//! campaign's encounter tables (`campaigns/<id>/data/encounters/*.toml`, [`EncounterTable`]).
//!
//! The meetings live in `world/encounter.rs` (the roll, the lookouts, the choice) and
//! `world/encounter_area.rs` (the local area and who stands where). Without the file the rules
//! are [`EncounterRules::default`]: the edge rule at 3 and nobody meets anybody; so does a
//! campaign without tables.

use super::{Content, ContentError, CreatureId, Landscape, Stance};
use crate::daylight::DayPhase;
use crate::dice::Dice;
use crate::weather::Weather;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// How near a hostile creature may hold the player at an area's edge, in cells (0: never).
pub const EDGE_REACHES: RangeInclusive<i32> = 0..=10;

/// The numbers of encounters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncounterRules {
    /// Walking off the edge of an outdoor area is refused while a creature hostile to the player
    /// is within this many cells (Chebyshev) of them: 3. 0 never refuses it.
    pub edge_reach: i32,
    /// Meeting someone on the way; `None` when nobody meets anybody.
    pub meetings: Option<Meetings>,
}

impl Default for EncounterRules {
    fn default() -> Self {
        Self {
            edge_reach: 3,
            meetings: None,
        }
    }
}

/// The landscapes a step may roll an encounter on: every land and the road.
const LANDSCAPES: [Landscape; 7] = [
    Landscape::Road,
    Landscape::Plains,
    Landscape::Grassland,
    Landscape::Forest,
    Landscape::Hills,
    Landscape::DenseForest,
    Landscape::Swamp,
];

/// The most members a group row may roll.
const MAX_MEMBERS: i32 = 12;
/// The longest an entry's name may be.
const MAX_NAME: usize = 60;
/// The farthest a party stands from the player, in cells.
const MAX_CELLS: i32 = 20;

/// The numbers of meeting someone on the overworld.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Meetings {
    /// Chance per hour of travel, in basis points (1/100 of a percent), by the destination's
    /// landscape, by day.
    pub rate: BTreeMap<Landscape, u32>,
    /// Percent of the rate in each phase of the day.
    pub phase: PhaseRates,
    /// The two lookout checks' numbers.
    pub checks: Checks,
    /// Where each side stands by outcome.
    pub positions: Positions,
    /// Slipping away from a party.
    pub evade: Evade,
    /// How long lying in wait takes, in seconds (`encounter:ambush`).
    pub ambush_wait: u32,
    /// Percent of the landscape's rate an hour of rest on the overworld rolls (#807).
    pub rest_percent: u32,
}

/// Percent of the rate by day, in twilight, and at night.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseRates {
    pub day: u32,
    pub twilight: u32,
    pub night: u32,
}

impl PhaseRates {
    /// The percent in `phase`.
    pub fn of(&self, phase: DayPhase) -> u32 {
        match phase {
            DayPhase::Day => self.day,
            DayPhase::Twilight => self.twilight,
            DayPhase::Night => self.night,
        }
    }
}

/// The lookout checks' numbers (2d12 + modifier against `difficulty` plus the terms).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checks {
    pub difficulty: i32,
    /// What the land hides, by landscape: added to both checks.
    pub cover: BTreeMap<Landscape, i32>,
    /// The dark at night, and in twilight, for eyes without darkvision (halved for a player with
    /// it).
    pub night: i32,
    pub twilight: i32,
    /// Rain and heavy rain, for both.
    pub rain: i32,
    pub heavy_rain: i32,
    /// Taken off the party's check while the player carries a lit light at night or twilight.
    pub light: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChecksSpec {
    difficulty: i32,
    cover: BTreeMap<String, i32>,
    night: i32,
    twilight: i32,
    rain: i32,
    heavy_rain: i32,
    light: i32,
}

/// Where each side stands by outcome (spec §7 "Overworld encounters", the owner's: the more
/// advantaged the encounter, the nearer the player stands to the edge they came from).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Positions {
    /// The player's place, in percent of the way from the arrival edge to the area's middle.
    pub player: PlayerPlaces,
    /// The party's distance from the player, in cells: a fight (chosen, or after meeting).
    pub fight: i32,
    /// Lying in wait: the party further in, walking toward the player's cover.
    pub ambush: i32,
    /// Ambushed: around the player.
    pub ambushed: i32,
    /// A failed evade: between the player and the edge.
    pub close: i32,
}

/// The player's place by outcome, in percent of the way from the arrival edge to the middle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerPlaces {
    pub ambush: u32,
    pub fight: u32,
    pub close: u32,
    pub ambushed: u32,
}

/// Slipping away (`encounter:evade`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evade {
    pub difficulty: i32,
    /// Percent of the step's time the detour takes, whatever comes of it.
    pub detour_percent: u32,
    /// Added to the difficulty when the party has seen the player.
    pub aware: i32,
    /// Speed above normal per point added to the difficulty (the lead's pursuit).
    pub pursuit_per: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RulesFile {
    edge_reach: i32,
    rate: Option<BTreeMap<String, u32>>,
    phase: Option<PhaseRates>,
    checks: Option<ChecksSpec>,
    positions: Option<Positions>,
    evade: Option<Evade>,
    ambush: Option<AmbushSpec>,
    rest: Option<RestSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AmbushSpec {
    wait_seconds: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RestSpec {
    rate_percent: u32,
}

/// The landscape an id names, among those a step may roll on.
fn landscape(id: &str) -> Result<Landscape, String> {
    (LANDSCAPES.into_iter())
        .find(|l| l.id() == id)
        .ok_or_else(|| format!("`{id}` is no landscape of land or road"))
}

/// A table keyed by landscape id, with every landscape of [`LANDSCAPES`] named.
fn by_landscape<T: Copy>(
    what: &str,
    table: &BTreeMap<String, T>,
) -> Result<BTreeMap<Landscape, T>, String> {
    let mut out = BTreeMap::new();
    for (id, &v) in table {
        out.insert(landscape(id).map_err(|m| format!("`{what}`: {m}"))?, v);
    }
    if let Some(missing) = LANDSCAPES.iter().find(|l| !out.contains_key(l)) {
        return Err(format!("`{what}` names no `{}`", missing.id()));
    }
    Ok(out)
}

impl EncounterRules {
    /// Parses `data/encounters.toml`: an `edge_reach` within [`EDGE_REACHES`]; and the
    /// meetings' tables all given or none ([`Meetings::parse`]).
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: RulesFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if !EDGE_REACHES.contains(&parsed.edge_reach) {
            return Err(err(format!(
                "edge_reach {} is out of range ({} to {})",
                parsed.edge_reach,
                EDGE_REACHES.start(),
                EDGE_REACHES.end()
            )));
        }
        let edge_reach = parsed.edge_reach;
        let meetings = match parsed {
            RulesFile {
                rate: None,
                phase: None,
                checks: None,
                positions: None,
                evade: None,
                ambush: None,
                rest: None,
                ..
            } => None,
            RulesFile {
                rate: Some(rate),
                phase: Some(phase),
                checks: Some(checks),
                positions: Some(positions),
                evade: Some(evade),
                ambush: Some(ambush),
                rest: Some(rest),
                ..
            } => Some(Meetings::parse(MeetingsFile {
                rate,
                phase,
                checks,
                positions,
                evade,
                ambush,
                rest,
            })),
            _ => Some(Err(
                "`rate`, `phase`, `checks`, `positions`, `evade`, `ambush`, and `rest` are all \
                 given or none"
                    .to_string(),
            )),
        };
        Ok(Self {
            edge_reach,
            meetings: meetings.transpose().map_err(err)?,
        })
    }
}

/// The meetings' tables of the rules file.
struct MeetingsFile {
    rate: BTreeMap<String, u32>,
    phase: PhaseRates,
    checks: ChecksSpec,
    positions: Positions,
    evade: Evade,
    ambush: AmbushSpec,
    rest: RestSpec,
}

impl Meetings {
    /// Every landscape of land and the road rated and covered, the numbers in range.
    fn parse(parsed: MeetingsFile) -> Result<Self, String> {
        let err = |m: String| m;
        let rate = by_landscape("rate", &parsed.rate).map_err(err)?;
        if rate.values().any(|&r| r > 10_000) {
            return Err(err("`rate` is 0 to 10000 basis points an hour".into()));
        }
        let p = parsed.phase;
        if [p.day, p.twilight, p.night].iter().any(|&v| v > 1000) {
            return Err(err("`phase` is 0 to 1000 percent".into()));
        }
        let c = &parsed.checks;
        let cover = by_landscape("checks.cover", &c.cover).map_err(err)?;
        let small = |v: i32| (0..=10).contains(&v);
        if !(1..=30).contains(&c.difficulty) {
            return Err(err("`checks.difficulty` is 1 to 30".into()));
        }
        let terms = [c.night, c.twilight, c.rain, c.heavy_rain, c.light];
        if !cover.values().copied().chain(terms).all(small) {
            return Err(err("the checks' terms are 0 to 10".into()));
        }
        let pos = parsed.positions;
        let cells = [pos.fight, pos.ambush, pos.ambushed, pos.close];
        if cells.iter().any(|&c| !(1..=MAX_CELLS).contains(&c)) {
            return Err(err(format!("`positions` are 1 to {MAX_CELLS} cells")));
        }
        let pl = pos.player;
        if [pl.ambush, pl.fight, pl.close, pl.ambushed]
            .iter()
            .any(|&v| v > 100)
        {
            return Err(err("`positions.player` is 0 to 100 percent".into()));
        }
        let e = parsed.evade;
        if !(1..=30).contains(&e.difficulty)
            || e.detour_percent > 500
            || !small(e.aware)
            || e.pursuit_per == 0
        {
            return Err(err(
                "`evade`: difficulty 1 to 30, detour_percent 0 to 500, aware 0 to 10, \
                 pursuit_per at least 1"
                    .into(),
            ));
        }
        if !(1..=crate::time::MINUTES_PER_DAY * 60).contains(&parsed.ambush.wait_seconds) {
            return Err(err("`ambush.wait_seconds` is 1 to a day".into()));
        }
        if parsed.rest.rate_percent > 1000 {
            return Err(err("`rest.rate_percent` is 0 to 1000".into()));
        }
        Ok(Self {
            rate,
            phase: parsed.phase,
            checks: Checks {
                difficulty: c.difficulty,
                cover,
                night: c.night,
                twilight: c.twilight,
                rain: c.rain,
                heavy_rain: c.heavy_rain,
                light: c.light,
            },
            positions: pos,
            evade: e,
            ambush_wait: parsed.ambush.wait_seconds,
            rest_percent: parsed.rest.rate_percent,
        })
    }

    /// The rate per hour on `landscape` by day, in basis points; 0 off land and road.
    pub fn rate_of(&self, landscape: Landscape) -> u32 {
        self.rate.get(&landscape).copied().unwrap_or(0)
    }

    /// What `landscape` hides, for both lookouts.
    pub fn cover_of(&self, landscape: Landscape) -> i32 {
        self.checks.cover.get(&landscape).copied().unwrap_or(0)
    }
}

/// Where and when a table (or an entry) applies: every condition given must hold.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct When {
    /// Region names of the campaign's overworld (`[[region]]`).
    pub regions: Vec<String>,
    pub landscapes: Vec<Landscape>,
    pub terrains: Vec<super::TerrainId>,
    pub phase: Vec<DayPhase>,
    pub weather: Vec<Weather>,
    /// The highest player level it stands at (the owner: entries may retire as the player
    /// rises; no lower gate).
    pub level_max: Option<u8>,
    /// A quest active (given, not done), done (completed at least once), or never given.
    pub quest_active: Option<String>,
    pub quest_done: Option<String>,
    pub quest_not: Option<String>,
    /// A hidden location of the overworld revealed, or still hidden.
    pub revealed: Option<String>,
    pub hidden: Option<String>,
    /// Whole days since the run began: at least, and fewer than.
    pub after_days: Option<u32>,
    pub before_days: Option<u32>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct WhenSpec {
    #[serde(default)]
    regions: Vec<String>,
    #[serde(default)]
    landscapes: Vec<String>,
    #[serde(default)]
    terrains: Vec<String>,
    #[serde(default)]
    phase: Vec<String>,
    #[serde(default)]
    weather: Vec<String>,
    level: Option<LevelSpec>,
    quest_active: Option<String>,
    quest_done: Option<String>,
    quest_not: Option<String>,
    revealed: Option<String>,
    hidden: Option<String>,
    after_days: Option<u32>,
    before_days: Option<u32>,
}

/// `level = { max = 8 }`: only a ceiling (the owner, #796 Q5: no level gates).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelSpec {
    max: u8,
}

/// How a party stands toward the player.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryStance {
    /// Checks are made: the drop, met, ambushed, or passing by.
    #[default]
    Hostile,
    /// Travellers: no checks, a question (hail them or pass by).
    Peaceful,
    /// Animals: no checks, a question (hunt them or let them be).
    Wildlife,
}

/// One row of a table: who may be met.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncounterEntry {
    pub id: String,
    /// What the player is told: "a band of footpads".
    pub name: String,
    pub weight: u32,
    /// Narrows its table's conditions.
    pub when: When,
    /// Counted types; the first row gives at least one.
    pub group: Vec<(CreatureId, Dice)>,
    /// The type whose eyes and Stealth the party's checks use.
    pub lead: CreatureId,
    pub stance: EntryStance,
}

/// A campaign's encounter table (spec §7 "Overworld encounters").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncounterTable {
    pub id: String,
    /// The campaign whose `data/encounters/` it is in.
    pub campaign: String,
    pub when: When,
    /// Scales the landscape's rate while it matches (several multiply).
    pub rate_percent: u32,
    /// Tables whose entries are left out while this one matches (one-way, never transitive).
    pub replaces: Vec<String>,
    pub entries: Vec<EncounterEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TableFile {
    #[serde(default)]
    table: Vec<TableSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TableSpec {
    id: String,
    #[serde(default)]
    when: WhenSpec,
    #[serde(default = "hundred")]
    rate_percent: u32,
    #[serde(default)]
    replaces: Vec<String>,
    #[serde(default)]
    entry: Vec<EntrySpec>,
}

fn hundred() -> u32 {
    100
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EntrySpec {
    id: String,
    name: String,
    weight: u32,
    #[serde(default)]
    when: WhenSpec,
    group: Vec<MemberSpec>,
    lead: Option<String>,
    #[serde(default)]
    stance: EntryStance,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MemberSpec {
    creature: String,
    count: String,
}

/// Parses the campaigns' encounter tables (`campaigns/<id>/data/encounters/*.toml`; the
/// table's single file `data/encounters.toml` is the rules, not a table) against the levels,
/// creatures, and quests loaded already. Sorted by campaign, then id.
pub(super) fn parse<'a>(
    table: impl Iterator<Item = (&'a str, &'a str)>,
    content: &Content,
) -> Result<Vec<EncounterTable>, ContentError> {
    let mut tables: Vec<EncounterTable> = Vec::new();
    let mut seen: BTreeMap<(String, String), &str> = BTreeMap::new();
    for (file, text) in table.filter(|(f, _)| *f != RULES) {
        let err = |m: String| ContentError::new(file, m);
        let Some(campaign) = super::campaign::of_path(file) else {
            return Err(err(
                "encounter tables belong to a campaign's `data/encounters/`".into(),
            ));
        };
        let parsed: TableFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for spec in parsed.table {
            let id = spec.id.clone();
            let bad = |m: String| err(format!("encounter table `{id}`: {m}"));
            if !super::dialogue::is_id(&id) {
                return Err(bad(
                    "ids are lower-case letters, digits, and underscores".into()
                ));
            }
            if let Some(first) = seen.insert((campaign.to_string(), id.clone()), file) {
                return Err(super::duplicate_id("encounter table", &id, file, first));
            }
            let table = resolve(spec, campaign, content).map_err(bad)?;
            tables.push(table);
        }
    }
    // Entries are unique within a campaign; `replaces` names its tables.
    for t in &tables {
        let file = super::campaign::table_file(Some(&t.campaign), "encounters");
        let err = |m: String| ContentError::new(&file, format!("encounter table `{}`: {m}", t.id));
        for r in &t.replaces {
            if !tables
                .iter()
                .any(|o| o.campaign == t.campaign && &o.id == r && o.id != t.id)
            {
                return Err(err(format!("`replaces`: no other table `{r}`")));
            }
        }
        for e in &t.entries {
            let twice = (tables.iter())
                .filter(|o| o.campaign == t.campaign)
                .flat_map(|o| &o.entries)
                .filter(|o| o.id == e.id)
                .count();
            if twice > 1 {
                return Err(err(format!("entry id `{}` is used twice", e.id)));
            }
        }
    }
    tables.sort_by(|a, b| (&a.campaign, &a.id).cmp(&(&b.campaign, &b.id)));
    Ok(tables)
}

/// The rules file, which [`super::ContentFiles::table`] lists with the tables.
pub(super) const RULES: &str = "data/encounters.toml";

fn resolve(spec: TableSpec, campaign: &str, content: &Content) -> Result<EncounterTable, String> {
    let when = when(spec.when, campaign, content)?;
    if spec.rate_percent > 1000 {
        return Err("`rate_percent` is 0 to 1000".into());
    }
    let mut entries = Vec::new();
    for e in spec.entry {
        let id = e.id.clone();
        entries.push(entry(e, campaign, content).map_err(|m| format!("entry `{id}`: {m}"))?);
    }
    Ok(EncounterTable {
        id: spec.id,
        campaign: campaign.to_string(),
        when,
        rate_percent: spec.rate_percent,
        replaces: spec.replaces,
        entries,
    })
}

fn entry(spec: EntrySpec, campaign: &str, content: &Content) -> Result<EncounterEntry, String> {
    if !super::dialogue::is_id(&spec.id) {
        return Err("ids are lower-case letters, digits, and underscores".into());
    }
    let n = spec.name.chars().count();
    if n == 0 || n > MAX_NAME || spec.name.contains('\n') {
        return Err(format!("`name` is one line of 1 to {MAX_NAME} characters"));
    }
    if !(1..=1000).contains(&spec.weight) {
        return Err("`weight` is 1 to 1000".into());
    }
    if spec.group.is_empty() {
        return Err("`group` names at least one creature".into());
    }
    let player = content.creature_id("player");
    let mut group = Vec::new();
    for m in &spec.group {
        let c = (content.creature_id(&m.creature))
            .ok_or_else(|| format!("unknown creature `{}`", m.creature))?;
        let def = content.creature(c);
        if def.campaign.as_deref().is_some_and(|own| own != campaign) {
            return Err(format!(
                "creature `{}` is campaign `{}`'s",
                def.id,
                def.campaign.as_deref().unwrap_or_default()
            ));
        }
        if def.unique {
            return Err(format!("`{}` is unique: no party is made of one", def.id));
        }
        let dice: Dice = m.count.parse()?;
        if dice.max() > MAX_MEMBERS {
            return Err(format!("`count` gives at most {MAX_MEMBERS}"));
        }
        // Peaceful and wildlife parties are not the player's enemies; hostile ones are.
        if let Some(player) = player {
            let hostile = content.stance(c, player) == Stance::Hostile;
            match (spec.stance, hostile) {
                (EntryStance::Hostile, false) => {
                    return Err(format!("`{}` is not hostile to the player", def.id));
                }
                (EntryStance::Peaceful | EntryStance::Wildlife, true) => {
                    return Err(format!("`{}` is hostile to the player", def.id));
                }
                _ => {}
            }
        }
        group.push((c, dice));
    }
    if group[0].1.min() < 1 {
        return Err("the first `group` row's `count` must give at least 1".into());
    }
    let lead = match &spec.lead {
        None => group[0].0,
        Some(name) => {
            let c = content.creature_id(name);
            c.filter(|c| group.iter().any(|(g, _)| g == c))
                .ok_or_else(|| format!("`lead`: `{name}` is none of the group"))?
        }
    };
    Ok(EncounterEntry {
        id: spec.id,
        name: spec.name,
        weight: spec.weight,
        when: when(spec.when, campaign, content)?,
        group,
        lead,
        stance: spec.stance,
    })
}

fn when(spec: WhenSpec, campaign: &str, content: &Content) -> Result<When, String> {
    let overworld = content.overworld_of(campaign);
    for r in &spec.regions {
        if !overworld.is_some_and(|o| o.regions.iter().any(|g| &g.name == r)) {
            return Err(format!(
                "`when.regions`: no region `{r}` on the campaign's overworld"
            ));
        }
    }
    let landscapes = (spec.landscapes.iter())
        .map(|l| landscape(l).map_err(|m| format!("`when.landscapes`: {m}")))
        .collect::<Result<_, _>>()?;
    let terrains = (spec.terrains.iter())
        .map(|t| {
            let id = content.terrain_id(t);
            id.filter(|&id| content.terrain(id).landscape.is_some())
                .ok_or_else(|| format!("`when.terrains`: `{t}` is no overworld terrain"))
        })
        .collect::<Result<_, _>>()?;
    let phase = (spec.phase.iter())
        .map(|p| match p.as_str() {
            "day" => Ok(DayPhase::Day),
            "twilight" => Ok(DayPhase::Twilight),
            "night" => Ok(DayPhase::Night),
            _ => Err(format!("`when.phase`: `{p}` is day, twilight, or night")),
        })
        .collect::<Result<_, _>>()?;
    let weather = (spec.weather.iter())
        .map(|w| match w.as_str() {
            "heavy_rain" => Ok(Weather::HeavyRain),
            other => (Weather::ALL.into_iter())
                .find(|k| k.id() == other && *k != Weather::HeavyRain)
                .ok_or_else(|| {
                    format!("`when.weather`: `{w}` is clear, overcast, rain, or heavy_rain")
                }),
        })
        .collect::<Result<_, _>>()?;
    let level_max = match spec.level {
        Some(LevelSpec { max }) if !(1..=60).contains(&max) => {
            return Err("`when.level.max` is 1 to 60".into());
        }
        level => level.map(|l| l.max),
    };
    for q in [&spec.quest_active, &spec.quest_done, &spec.quest_not]
        .into_iter()
        .flatten()
    {
        let quest = content.quest_id(q).map(|id| content.quest(id));
        if !quest.is_some_and(|d| d.campaign.as_deref().is_none_or(|c| c == campaign)) {
            return Err(format!("`when`: no quest `{q}` of the campaign"));
        }
    }
    for h in [&spec.revealed, &spec.hidden].into_iter().flatten() {
        if content.hidden_location(campaign, h).is_none() {
            return Err(format!("`when`: no hidden location `{h}` on the overworld"));
        }
    }
    for d in [spec.after_days, spec.before_days].into_iter().flatten() {
        if d > 3600 {
            return Err("`when.after_days` and `before_days` are 0 to 3600".into());
        }
    }
    Ok(When {
        regions: spec.regions,
        landscapes,
        terrains,
        phase,
        weather,
        level_max,
        quest_active: spec.quest_active,
        quest_done: spec.quest_done,
        quest_not: spec.quest_not,
        revealed: spec.revealed,
        hidden: spec.hidden,
        after_days: spec.after_days,
        before_days: spec.before_days,
    })
}

impl Content {
    /// The numbers of meeting someone on the overworld (`data/encounters.toml`); `None`
    /// without them, when nobody meets anybody.
    pub fn meetings(&self) -> Option<&Meetings> {
        self.encounters().meetings.as_ref()
    }

    /// Campaign `campaign`'s encounter tables, by id (spec §7 "Overworld encounters").
    pub fn encounter_tables(&self, campaign: &str) -> impl Iterator<Item = &EncounterTable> {
        (self.encounter_tables.iter()).filter(move |t| t.campaign == campaign)
    }

    /// Entry `id` of campaign `campaign`'s tables, with its table.
    pub fn encounter_entry(
        &self,
        campaign: &str,
        id: &str,
    ) -> Option<(&EncounterTable, &EncounterEntry)> {
        (self.encounter_tables(campaign))
            .find_map(|t| t.entries.iter().find(|e| e.id == id).map(|e| (t, e)))
    }
}
