//! Overworld encounters (spec §7 "Overworld encounters"; `docs/dev/encounters.md`; the design
//! `docs/design/overworld-encounters.md`, its §15 the owner's decisions): the roll after an
//! overworld step, the two lookouts, the four outcomes, the pending choice that must be answered
//! ([`crate::action::Action::Encounter`]), evading, and the dev `meet`. Where everyone stands
//! when a fight opens is `encounter_area.rs`.
//!
//! - **The roll** ([`World::encounter_roll`]): after a step onto an overworld cell with a local
//!   area, in a campaign with tables, with nothing pending: the destination landscape's rate
//!   (basis points an hour) × the step's seconds × the phase's percent × each matching table's
//!   `rate_percent`, one `below(10_000)` of the `encounters` stream (made on the first roll);
//!   then the matching tables' entries by weight (one `below`), and each group row's dice.
//! - **Camping and foraging** ([`World::watch_roll`], #807; one line in each turn of a rest,
//!   `rest.rs`, and of a forage, `forage.rs`): the same roll for each turn's time, drawn exactly
//!   (`below(3_600_000_000)`, a chance in basis points × seconds × percent), a rest's at
//!   `[rest] rate_percent` of the rate. A forager keeps a traveller's lookouts; a resting
//!   player's is by ear alone ([`World::sleepers_lookout`]) and the party always finds them:
//!   made, met; failed, ambushed. The message it says stops the rest or the forage.
//! - **The lookouts** ([`World::lookouts`]): two `check::check`s on the gameplay stream, the
//!   player's then the party's, only for a hostile entry. Made / failed: the drop; both: met; the
//!   party alone: ambushed; neither: they pass each other by (nothing happens, nothing is said).
//! - **The pending choice** ([`Pending`], `WorldState::encounter`): the drop, met, a peaceful or
//!   a wildlife party wait for `encounter:CHOICE`; every other action but the free ones is
//!   refused ([`World::refuse_pending`]). Ambushed and a failed evade never pend: the area opens.
//! - **Opening** happens after the action's turn (`World::encounter_tick`, one line in
//!   `World::apply`), so the step's or the wait's time passes on the overworld, never in the area.

use super::{ActorId, Spent, World};
use crate::check;
use crate::content::{
    ActionKind, BASE_TURN, Content, CreatureId, EncounterEntry, EncounterTable, EncounterWhen,
    EntryStance, Landscape, LevelKind, local_area_id,
};
use crate::daylight::{self, DayPhase};
use crate::geom::{Dir, Pos};
use crate::rng::Rng;
use crate::skills::{Effect, Use};
use crate::stats::Stat;
use crate::text::capitalized;
use crate::weather::Weather;
use serde::{Deserialize, Serialize};

/// What the player answers a pending encounter with (`encounter:CHOICE`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Choice {
    /// Lie in wait: the party walks into the player's ambush (the drop only).
    Ambush,
    /// Talk your way past: a Bluff check (`talk_past.rs`; a flagged entry, met, trained only).
    Bluff,
    /// Slip past or away: a check.
    Evade,
    /// Stand and fight.
    Fight,
    /// Hail travellers.
    Hail,
    /// Hunt animals.
    Hunt,
    /// Pass by travellers or animals.
    Pass,
}

impl Choice {
    pub const ALL: &[Choice] = &[
        Choice::Ambush,
        Choice::Bluff,
        Choice::Evade,
        Choice::Fight,
        Choice::Hail,
        Choice::Hunt,
        Choice::Pass,
    ];

    /// Its id in action text: `ambush`.
    pub const fn id(self) -> &'static str {
        match self {
            Choice::Ambush => "ambush",
            Choice::Bluff => "bluff",
            Choice::Evade => "evade",
            Choice::Fight => "fight",
            Choice::Hail => "hail",
            Choice::Hunt => "hunt",
            Choice::Pass => "pass",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.id() == id)
    }
}

/// How a pending encounter came about: what the player may answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// The player saw them first.
    Drop,
    /// Each saw the other.
    Met,
    /// Travellers.
    Peaceful,
    /// Animals.
    Wildlife,
}

impl Outcome {
    /// Its id (dumps): `drop`.
    pub const fn id(self) -> &'static str {
        match self {
            Outcome::Drop => "drop",
            Outcome::Met => "met",
            Outcome::Peaceful => "peaceful",
            Outcome::Wildlife => "wildlife",
        }
    }

    /// The choices it offers anyone, in the dialog's order (`talk_past::offered` adds Bluff's).
    pub const fn choices(self) -> &'static [Choice] {
        match self {
            Outcome::Drop => &[Choice::Ambush, Choice::Evade, Choice::Fight],
            Outcome::Met => &[Choice::Evade, Choice::Fight],
            Outcome::Peaceful => &[Choice::Hail, Choice::Pass],
            Outcome::Wildlife => &[Choice::Hunt, Choice::Pass],
        }
    }

    /// How the dialog labels `choice`.
    pub const fn label(self, choice: Choice) -> &'static str {
        match (self, choice) {
            (_, Choice::Ambush) => "Lie in wait",
            (_, Choice::Bluff) => "Talk your way past",
            (Outcome::Met, Choice::Evade) => "Give them the slip",
            (_, Choice::Evade) => "Slip past",
            (_, Choice::Fight) => "Stand and fight",
            (_, Choice::Hail) => "Hail them",
            (_, Choice::Hunt) => "Hunt them",
            (Outcome::Wildlife, Choice::Pass) => "Let them be",
            (_, Choice::Pass) => "Pass by",
        }
    }
}

/// The dev `meet:ENTRY:OUTCOME`'s forced outcome of the lookouts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Forced {
    Drop,
    Met,
    Ambushed,
    Pass,
}

impl Forced {
    pub const ALL: &[Forced] = &[Forced::Drop, Forced::Met, Forced::Ambushed, Forced::Pass];

    pub const fn id(self) -> &'static str {
        match self {
            Forced::Drop => "drop",
            Forced::Met => "met",
            Forced::Ambushed => "ambushed",
            Forced::Pass => "pass",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.id() == id)
    }
}

/// An encounter the player must answer (saved: a quit mid-question resumes on it).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pending {
    /// The entry's id in the run's campaign's tables.
    pub entry: String,
    /// The overworld cell: its local area is where a fight opens.
    pub cell: Pos,
    /// The side of the area the player arrives on (the step's opposite).
    pub side: Dir,
    pub outcome: Outcome,
    /// The party's types, as rolled; empty for a known party.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<CreatureId>,
    /// The step's energy price: a detour takes its share of it.
    pub step: i64,
    /// The area holds this party already: one the player got away from (the owner, #796 Q10).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub known: bool,
    /// Met while the player rested: they woke to the party (#807).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub woken: bool,
}

/// What the player was doing on the overworld when an encounter is rolled (#807).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Watch {
    /// A step: the rate for its time, both lookouts.
    Travel,
    /// A rest: `[rest] rate_percent` of the rate, the sleeper's lookout by ear alone.
    Rest,
    /// A forage: the rate for its time, both lookouts, as a traveller's.
    Forage,
}

/// A party made by an encounter, on its area's level (saved): who to place again when the same
/// entry is met there, and the dev dumps' `of encounter ID`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Party {
    pub entry: String,
    pub level: String,
    pub members: Vec<ActorId>,
}

/// How a fight opens (`encounter_area.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum How {
    Fight,
    Ambush,
    Ambushed,
    Close,
    Hail,
    Hunt,
}

/// An area to open once the action's turn has run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Opening {
    pub pending: Pending,
    pub how: How,
}

/// What an encounter has under way within one action (not saved: resolved before it returns).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Underway {
    /// The overworld step just taken: its price, for the roll after its turn.
    pub step: Option<i64>,
    pub opening: Option<Opening>,
}

/// What the player is shown of a pending encounter ([`crate::perception::PerceptionView::encounter`]):
/// the one dialog's title, text, and rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncounterView {
    /// The entry's name, as written: "a band of footpads".
    pub name: String,
    pub title: String,
    pub text: String,
    pub choices: Vec<(String, Choice)>,
}

/// "Ahead" as the way there is: on the road, on the trail, or across the land.
fn ahead(landscape: Option<Landscape>) -> &'static str {
    match landscape {
        Some(Landscape::Road) => "on the road ahead",
        Some(Landscape::Forest | Landscape::DenseForest) => "on the trail ahead",
        _ => "ahead",
    }
}

/// Where an ambush comes from, by the land.
fn from_cover(landscape: Option<Landscape>) -> &'static str {
    match landscape {
        Some(Landscape::Road) => "from the roadside",
        Some(Landscape::Forest | Landscape::DenseForest) => "from the undergrowth",
        Some(Landscape::Hills) => "from behind the rocks",
        Some(Landscape::Swamp) => "from the reeds",
        _ => "from the long grass",
    }
}

/// Where a party is lost, by the land.
fn lost_in(landscape: Option<Landscape>) -> &'static str {
    match landscape {
        Some(Landscape::Road) => "the byways",
        Some(Landscape::Forest | Landscape::DenseForest) => "the woods",
        Some(Landscape::Hills) => "the hills",
        Some(Landscape::Swamp) => "the marsh",
        _ => "the long grass",
    }
}

/// The line that acknowledges a party met again (#796 Q10).
pub const KNOWN_LINE: &str = "You remember them: you got away from them here before.";

/// What the player is told of a party `name` met with `outcome` on `landscape`.
fn told(outcome: Outcome, name: &str, landscape: Option<Landscape>, woken: bool) -> String {
    if woken {
        return match outcome {
            Outcome::Drop | Outcome::Met => {
                format!("You wake to the sound of {name} close by your camp.")
            }
            Outcome::Peaceful => format!("You wake to find {name} at your camp."),
            Outcome::Wildlife => format!("You wake to find {name} near your camp."),
        };
    }
    match outcome {
        Outcome::Drop => format!(
            "You spot {name} {}. They have not seen you.",
            ahead(landscape)
        ),
        Outcome::Met => format!("You and {name} see each other at the same moment."),
        Outcome::Peaceful => format!("You meet {name} on the way."),
        Outcome::Wildlife => format!("You come upon {name}."),
    }
}

/// The view of `pending`, its entry named by `name`, offering `choices`.
pub(crate) fn view(
    pending: &Pending,
    name: &str,
    landscape: Option<Landscape>,
    choices: &[Choice],
) -> EncounterView {
    let text = told(pending.outcome, name, landscape, pending.woken);
    let text = if pending.known {
        format!("{text} {KNOWN_LINE}")
    } else {
        text
    };
    EncounterView {
        name: name.to_string(),
        title: capitalized(name),
        text,
        choices: (choices.iter())
            .map(|&c| (pending.outcome.label(c).to_string(), c))
            .collect(),
    }
}

/// What a cell and the moment are, for a table's `when`.
struct Here {
    region: Option<String>,
    landscape: Landscape,
    terrain: crate::content::TerrainId,
    phase: DayPhase,
    weather: Weather,
    level: u8,
    days: u32,
}

impl World {
    /// The overworld step just taken, for the roll after its turn (one line in `player_moves`).
    pub(super) fn encounter_step(&mut self, price: i64) {
        self.encounter.step = Some(price);
    }

    /// After the action's turn: the roll for an overworld step just taken, then any area an
    /// encounter opens (perceived at once). One line in `World::apply`.
    pub(super) fn encounter_tick(&mut self, events: &mut Vec<crate::action::Event>) {
        let step = self.encounter.step.take();
        if self.state.death.is_some() {
            self.encounter.opening = None;
            return;
        }
        if let Some(step) = step {
            self.encounter_roll(step);
        }
        if let Some(opening) = self.encounter.opening.take() {
            self.open_encounter(opening, events);
        }
    }

    /// The roll after an overworld step whose energy price was `step` (module docs). Draws
    /// nothing without the rules or the campaign's tables, off a cell with a local area, or
    /// while an encounter is pending.
    fn encounter_roll(&mut self, step: i64) {
        let content = self.content.clone();
        let Some((rules, cell, here, tables, percent)) = self.odds(&content) else {
            return;
        };
        let seconds = self.seconds_of(step);
        let chance = u64::from(rules.rate_of(here.landscape)) * seconds * percent / (3600 * 100);
        if chance == 0 {
            return;
        }
        let rng = self.encounters_rng();
        if u64::from(rng.below(10_000)) >= chance {
            return;
        }
        self.draw_entry(&tables, &here, cell, step, Watch::Travel);
    }

    /// The roll for a turn of a rest or a forage on the overworld whose energy price is
    /// `price` (#807; one line in each one's turn): the rate for the turn's time (a rest's
    /// `[rest] rate_percent` of it), drawn exactly (`below` of 10 000 × 3600 × 100: the chance
    /// in basis points × seconds × percent), so turns of a few seconds add up to the rate an
    /// hour. Draws nothing where a step's roll draws nothing, or while an area is to open.
    pub(super) fn watch_roll(&mut self, watch: Watch, price: i64) {
        if self.encounter.opening.is_some() {
            return;
        }
        let content = self.content.clone();
        let Some((rules, cell, here, tables, percent)) = self.odds(&content) else {
            return;
        };
        let share = match watch {
            Watch::Rest => u64::from(rules.rest_percent),
            Watch::Travel | Watch::Forage => 100,
        };
        let seconds = self.seconds_of(price);
        let chance = u64::from(rules.rate_of(here.landscape)) * seconds * percent * share / 100;
        if chance == 0 {
            return;
        }
        const OUT_OF: u32 = 10_000 * 3600 * 100;
        if u64::from(self.encounters_rng().below(OUT_OF)) >= chance {
            return;
        }
        let step = price_of_travel(&content, Some(here.terrain));
        self.draw_entry(&tables, &here, cell, step, watch);
    }

    /// The game seconds an energy price is to the player, at their speed.
    fn seconds_of(&self, price: i64) -> u64 {
        let speed = u64::from(self.state.player().speed(&self.content).max(1));
        u64::try_from(price).unwrap_or(0) / speed
    }

    /// What a roll on the player's overworld cell now goes by: the rules, the cell, the moment,
    /// the matching tables, and the percent of the rate (the phase's × each table's); `None`
    /// where nothing is rolled (the module docs' roll).
    #[allow(clippy::type_complexity)]
    fn odds<'c>(
        &self,
        content: &'c Content,
    ) -> Option<(
        &'c crate::content::Meetings,
        Pos,
        Here,
        Vec<&'c EncounterTable>,
        u64,
    )> {
        let rules = content.meetings()?;
        if self.state.encounter.is_some() || self.state.current().kind != LevelKind::Overworld {
            return None;
        }
        let campaign = &self.state.campaign;
        content.encounter_tables(campaign).next()?;
        let cell = self.player_pos();
        let here = self.here(cell)?;
        let tables = self.matching(content, campaign, &here);
        let percent = (tables.iter()).fold(u64::from(rules.phase.of(here.phase)), |p, t| {
            p * u64::from(t.rate_percent) / 100
        });
        Some((rules, cell, here, tables, percent))
    }

    /// A roll made: the matching tables' entries whose own `when` holds, one by weight (one
    /// draw), met on `cell` (`step` the price a detour takes its share of).
    fn draw_entry(
        &mut self,
        tables: &[&EncounterTable],
        here: &Here,
        cell: Pos,
        step: i64,
        watch: Watch,
    ) {
        let entries: Vec<&EncounterEntry> = (tables.iter())
            .flat_map(|t| &t.entries)
            .filter(|e| self.when_holds(&e.when, here))
            .collect();
        // The stream is made on its first draw: none while nothing weighs anything.
        if entries.iter().all(|e| e.weight == 0) {
            return;
        }
        let Some(&entry) = self.encounters_rng().weighted(&entries, |e| e.weight) else {
            return;
        };
        let side = self.arrival_side();
        self.meet(entry, cell, side, step, None, watch);
    }

    /// The `encounters` stream, made on the first draw.
    fn encounters_rng(&mut self) -> &mut Rng {
        let seed = self.state.seed;
        (self.state.encounters).get_or_insert_with(|| Rng::stream(seed, "encounters"))
    }

    /// The side of the area the player arrives on: the opposite of their last overworld step,
    /// else the south.
    fn arrival_side(&self) -> Dir {
        self.state.overworld_step.map_or(Dir::S, Dir::opposite)
    }

    /// The cell and the moment for the tables; `None` off a cell with a local area.
    fn here(&self, cell: Pos) -> Option<Here> {
        let area = self
            .content
            .local_area(&self.state.campaign, &local_area_id(cell))?;
        let over = self.state.weather_override.as_ref();
        let now = crate::weather::conditions(
            self.state.seed,
            self.state.time,
            self.content.climate(),
            over,
        );
        let start =
            (self.content.scenario(&self.state.scenario)).map_or(self.state.time, |s| s.start);
        let days = self.state.time.since(start).as_seconds() / crate::time::DAY;
        Some(Here {
            region: area.overworld.region_at(cell).map(str::to_string),
            landscape: area.landscape,
            terrain: area.terrain,
            phase: daylight::phase(self.state.time),
            weather: now.weather,
            level: self.player_level(),
            days: u32::try_from(days).unwrap_or(u32::MAX),
        })
    }

    /// The campaign's tables whose `when` holds here, less those a kept one `replaces`.
    fn matching<'c>(
        &self,
        content: &'c Content,
        campaign: &str,
        here: &Here,
    ) -> Vec<&'c EncounterTable> {
        let kept: Vec<&EncounterTable> = (content.encounter_tables(campaign))
            .filter(|t| self.when_holds(&t.when, here))
            .collect();
        let replaced: Vec<&String> = kept.iter().flat_map(|t| &t.replaces).collect();
        kept.into_iter()
            .filter(|t| !replaced.contains(&&t.id))
            .collect()
    }

    /// Whether `when` holds here and now.
    fn when_holds(&self, when: &EncounterWhen, here: &Here) -> bool {
        let quests = &self.state.quests;
        let active =
            |q: &String| (quests.record(q)).is_some_and(|r| r.stage == super::Stage::Active);
        let done = |q: &String| {
            (quests.record(q)).is_some_and(|r| r.completed > 0 || r.stage == super::Stage::Done)
        };
        (when.regions.is_empty()
            || here
                .region
                .as_ref()
                .is_some_and(|r| when.regions.contains(r)))
            && (when.landscapes.is_empty() || when.landscapes.contains(&here.landscape))
            && (when.terrains.is_empty() || when.terrains.contains(&here.terrain))
            && (when.phase.is_empty() || when.phase.contains(&here.phase))
            && (when.weather.is_empty() || when.weather.contains(&here.weather))
            && when.level_max.is_none_or(|m| here.level <= m)
            && when.quest_active.as_ref().is_none_or(active)
            && when.quest_done.as_ref().is_none_or(done)
            && (when.quest_not.as_ref()).is_none_or(|q| quests.record(q).is_none())
            && (when.quest_variant.as_ref()).is_none_or(|(q, v)| {
                (quests.record(q)).is_some_and(|r| {
                    (r.roll.as_ref()).is_some_and(|roll| roll.variant.as_ref() == Some(v))
                })
            })
            && (when.revealed.as_ref()).is_none_or(|h| self.state.revealed.contains(h))
            && (when.hidden.as_ref()).is_none_or(|h| !self.state.revealed.contains(h))
            && when.after_days.is_none_or(|d| here.days >= d)
            && when.before_days.is_none_or(|d| here.days < d)
            && (when.calling.as_ref()).is_none_or(|c| self.calling_open(c))
    }

    /// The ids of the tables that match the player's overworld cell now (dev dumps).
    pub fn encounter_tables_here(&self) -> Vec<String> {
        let content = self.content.clone();
        match self.here(self.player_pos()) {
            Some(here) if self.state.current().kind == LevelKind::Overworld => (self
                .matching(&content, &self.state.campaign, &here)
                .into_iter())
            .map(|t| t.id.clone())
            .collect(),
            _ => Vec::new(),
        }
    }

    /// The ids of the entries a roll on the player's overworld cell could draw now: the
    /// matching tables' entries whose own `when` holds (dev; tests).
    #[cfg(test)]
    pub fn encounter_entries_here(&self) -> Vec<String> {
        let content = self.content.clone();
        match self.here(self.player_pos()) {
            Some(here) if self.state.current().kind == LevelKind::Overworld => (self
                .matching(&content, &self.state.campaign, &here)
                .into_iter())
            .flat_map(|t| &t.entries)
            .filter(|e| self.when_holds(&e.when, &here))
            .map(|e| e.id.clone())
            .collect(),
            _ => Vec::new(),
        }
    }

    /// The encounter rate on the player's overworld cell now, in basis points an hour, with
    /// the phase and the matching tables (dev dumps); `None` where nothing is rolled.
    pub fn encounter_rate_here(&self) -> Option<u64> {
        let content = self.content.clone();
        let rules = content.meetings()?;
        let tabled = content
            .encounter_tables(&self.state.campaign)
            .next()
            .is_some();
        if !tabled || self.state.current().kind != LevelKind::Overworld {
            return None;
        }
        let here = self.here(self.player_pos())?;
        let tables = self.matching(&content, &self.state.campaign, &here);
        let percent = (tables.iter()).fold(u64::from(rules.phase.of(here.phase)), |p, t| {
            p * u64::from(t.rate_percent) / 100
        });
        Some(u64::from(rules.rate_of(here.landscape)) * percent / 100)
    }

    /// The entry of the encounter that made actor `id`, if one did (dev dumps).
    pub fn party_of(&self, id: ActorId) -> Option<&str> {
        (self.state.parties.iter())
            .find(|p| p.members.contains(&id))
            .map(|p| p.entry.as_str())
    }

    /// The living members of earlier parties of `entry` in the area of `cell`.
    pub(super) fn known_party(&self, entry: &str, cell: Pos) -> Vec<ActorId> {
        let level = local_area_id(cell);
        (self.state.parties.iter())
            .filter(|p| p.entry == entry && p.level == level)
            .flat_map(|p| &p.members)
            .copied()
            .filter(|id| self.state.actors.get(id).is_some_and(|a| a.level == level))
            .collect()
    }

    /// The type that leads a party of `entry` (rolled as `members`, or the known one in the
    /// area of `cell`): the entry's `lead`, else the first member (a `draw`'s first drawn).
    pub(super) fn lead_of(
        &self,
        entry: &EncounterEntry,
        members: &[CreatureId],
        cell: Pos,
    ) -> Option<CreatureId> {
        let known = || {
            (self.known_party(&entry.id, cell).first())
                .and_then(|id| self.state.actors.get(id))
                .map(|a| a.creature)
        };
        entry
            .lead
            .or_else(|| members.first().copied())
            .or_else(known)
    }

    /// A `draw` entry's members (the `encounters` stream: the count's dice, then `danger::draw`
    /// for each): the branch's population at the draw's danger with its odds, uniques and types
    /// not hostile to the player left out (as a hoard's guardians). Empty when nothing may be
    /// drawn.
    fn draw_members(&mut self, draw: &crate::content::EncounterDraw) -> Vec<CreatureId> {
        let content = self.content.clone();
        let Some(spawning) = (content.dungeon(&draw.pool)).and_then(|b| b.spawning.as_ref()) else {
            return Vec::new();
        };
        let player = content.ids().player;
        let fit = |list: Vec<crate::content::danger::Candidate>| -> Vec<_> {
            (list.into_iter())
                .filter(|&(c, ..)| !content.creature(c).unique)
                .filter(|&(c, ..)| {
                    player.is_some_and(|p| content.stance(c, p) == crate::content::Stance::Hostile)
                })
                .collect()
        };
        let once = self.state.progress.uniques.clone();
        let own = fit(content.candidates(&spawning.population, &once));
        let generic = fit(content.candidates(&spawning.generic, &once));
        let rng = self.encounters_rng();
        let n = draw.count.roll(rng).max(0);
        (0..n)
            .filter_map(|_| {
                let rng = self.encounters_rng();
                crate::content::danger::draw(rng, &own, &generic, draw.danger, &spawning.odds)
            })
            .collect()
    }

    /// `entry` is met on `cell`, arriving on `side`, after a step of price `step`: the party
    /// rolled (or the known one), then the lookouts (or `forced`), and what comes of them.
    fn meet(
        &mut self,
        entry: &EncounterEntry,
        cell: Pos,
        side: Dir,
        step: i64,
        forced: Option<Forced>,
        watch: Watch,
    ) {
        // Parties whose members are all gone are forgotten.
        let state = &mut *self.state;
        let actors = &state.actors;
        state
            .parties
            .retain(|p| p.members.iter().any(|m| actors.contains_key(m)));
        let known = !self.known_party(&entry.id, cell).is_empty();
        let mut members = Vec::new();
        if !known {
            for &(creature, dice) in &entry.group {
                let n = dice.roll(self.encounters_rng()).max(0);
                members.extend(std::iter::repeat_n(creature, n as usize));
            }
            if let Some(draw) = &entry.draw {
                members = self.draw_members(draw);
                if members.is_empty() {
                    return;
                }
            }
            // The lead first: it stands where the outcome puts the lead.
            if let Some(lead) = entry.lead {
                members.sort_by_key(|&c| c != lead);
            }
        }
        let Some(lead) = self.lead_of(entry, &members, cell) else {
            return;
        };
        let outcome = match entry.stance {
            EntryStance::Peaceful => Some(Outcome::Peaceful),
            EntryStance::Wildlife => Some(Outcome::Wildlife),
            EntryStance::Hostile => {
                let (mine, theirs) = match forced {
                    Some(Forced::Drop) => (true, false),
                    Some(Forced::Met) => (true, true),
                    Some(Forced::Ambushed) => (false, true),
                    Some(Forced::Pass) => (false, false),
                    // Asleep: the party always finds the camp; woken in time, they meet.
                    None if watch == Watch::Rest => (self.sleepers_lookout(lead), true),
                    None => self.lookouts(lead),
                };
                match (mine, theirs) {
                    (true, false) => Some(Outcome::Drop),
                    (true, true) => Some(Outcome::Met),
                    (false, true) => None,
                    (false, false) => return,
                }
            }
        };
        let landscape = self.here(cell).map(|h| h.landscape);
        let pending = Pending {
            entry: entry.id.clone(),
            cell,
            side,
            outcome: outcome.unwrap_or(Outcome::Met),
            members,
            step,
            known,
            woken: watch == Watch::Rest,
        };
        let Some(outcome) = outcome else {
            // Ambushed: no question; the fight opens on the party's terms.
            let line = match pending.woken {
                true => format!("You wake with {} upon you!", entry.name),
                false => format!(
                    "You are set upon by {} {}!",
                    entry.name,
                    from_cover(landscape)
                ),
            };
            self.say(line);
            self.encounter.opening = Some(Opening {
                pending,
                how: How::Ambushed,
            });
            return;
        };
        self.say(told(outcome, &entry.name, landscape, pending.woken));
        if known {
            self.say(KNOWN_LINE);
        }
        self.state.encounter = Some(pending);
    }

    /// The two lookouts (gameplay stream, two draws each): whether the player sees the party
    /// first, and whether the party sees them. Marks Spot's and Survival's lookout, and
    /// Stealth's sneak when the party missed the player.
    fn lookouts(&mut self, lead: CreatureId) -> (bool, bool) {
        let content = self.content.clone();
        let Some(rules) = content.meetings() else {
            return (false, false);
        };
        let cell = self.player_pos();
        let Some(here) = self.here(cell) else {
            return (false, false);
        };
        let c = &rules.checks;
        let lead = content.creature(lead);
        let player = self.state.player();
        let talents = player.talent_mods(&content);
        let dark = match here.phase {
            DayPhase::Day => 0,
            DayPhase::Twilight => c.twilight,
            DayPhase::Night => c.night,
        };
        let rain = match here.weather {
            Weather::Rain => c.rain,
            Weather::HeavyRain => c.heavy_rain,
            _ => 0,
        };
        let cover = rules.cover_of(here.landscape);
        let seeing = |profile: Option<&crate::content::VisionProfile>| {
            profile.is_some_and(|v| v.darkvision > 0)
        };
        let mine_dark = match seeing(content.vision(&self.state.vision)) {
            true => dark / 2,
            false => dark,
        };
        let theirs_dark = match seeing(content.vision_profiles().get(lead.vision)) {
            true => 0,
            false => dark,
        };
        let lit = here.phase != DayPhase::Day && self.state.carried_light(&content).is_some();
        let mine = player.effective_stats(&content).modifier(Stat::Willpower)
            + talents.check(Stat::Willpower)
            + player.skill_effect(&content, Effect::Lookout)
            + talents.notice.unwrap_or(0);
        let mine_against =
            c.difficulty + lead.skills.effect(&content, Effect::Stalk) + cover + mine_dark + rain;
        let theirs =
            lead.stats.modifier(Stat::Willpower) + lead.skills.effect(&content, Effect::Lookout);
        let theirs_against = c.difficulty
            + player.skill_effect(&content, Effect::Stalk)
            + cover
            + theirs_dark
            + rain
            - if lit { c.light } else { 0 };
        let saw = check::check(&mut self.state.rng, mine, mine_against);
        let seen = check::check(&mut self.state.rng, theirs, theirs_against);
        if self.trains(Use::Lookout) {
            self.train(Use::Lookout);
        }
        if !seen && self.trains(Use::Sneak) {
            self.train(Use::Sneak);
        }
        (saw, seen)
    }

    /// The resting player's lookout (gameplay stream, two draws; #807): whether they wake in
    /// time. By ear alone: Willpower modifier + talents' Willpower checks + their hearing
    /// (Listen's `hearing` and Keen ears, as [`super::Actor::hearing`] reads them, at most
    /// [`crate::skills::MAX_ENCOUNTER_BONUS`]; none for the deaf) + Alert's `notice`, against
    /// `difficulty` + the lead type's `stalk` + the rain (neither cover nor the dark hides a
    /// sound). Marks Listen's `hear`.
    fn sleepers_lookout(&mut self, lead: CreatureId) -> bool {
        let content = self.content.clone();
        let Some(rules) = content.meetings() else {
            return false;
        };
        let Some(here) = self.here(self.player_pos()) else {
            return false;
        };
        let c = &rules.checks;
        let lead = content.creature(lead);
        let player = self.state.player();
        let talents = player.talent_mods(&content);
        let ears = match player.hearing(&content) {
            Some(_) => (player.skill_effect(&content, Effect::Hearing) + talents.hearing)
                .clamp(0, crate::skills::MAX_ENCOUNTER_BONUS),
            None => 0,
        };
        let rain = match here.weather {
            Weather::Rain => c.rain,
            Weather::HeavyRain => c.heavy_rain,
            _ => 0,
        };
        let mine = player.effective_stats(&content).modifier(Stat::Willpower)
            + talents.check(Stat::Willpower)
            + ears
            + talents.notice.unwrap_or(0);
        let against = c.difficulty + lead.skills.effect(&content, Effect::Stalk) + rain;
        let woke = check::check(&mut self.state.rng, mine, against);
        if self.trains(Use::Hear) {
            self.train(Use::Hear);
        }
        woke
    }

    /// Refuses `action` while an encounter is pending, unless it answers it or takes no game
    /// time and makes nothing happen (a stance, the level's picks, the dev switches).
    pub(super) fn refuse_pending(&mut self, action: &crate::action::Action) -> Option<Spent> {
        use crate::action::Action as A;
        self.state.encounter.as_ref()?;
        let free = matches!(
            action,
            A::Encounter(_)
                | A::Talent { .. }
                | A::Advance(_)
                | A::LevelUpAuto
                | A::SetStance(_)
                | A::SetVision(_)
                | A::SetWeather(_)
                | A::Identify(_)
                | A::Afflict(_)
                | A::Drain(_)
                | A::Resource(_)
                | A::Infect(_)
                | A::Reveal(_)
                | A::QuestRoll { .. }
                | A::GrantXp(_)
                | A::SetRank { .. }
                | A::Effect { .. }
        );
        if free {
            return None;
        }
        self.say(PENDING_REFUSED);
        Some(Spent::Nothing)
    }

    /// `encounter:CHOICE` ([`crate::action::Action::Encounter`]): the answer to the pending
    /// encounter. Refused (no time) with nothing pending or a choice it does not offer.
    pub(super) fn answer_encounter(&mut self, choice: Choice) -> Spent {
        let Some(pending) = self.state.encounter.clone() else {
            self.say("There is nothing to decide.");
            return Spent::Nothing;
        };
        let content = self.content.clone();
        let Some((_, entry)) = content.encounter_entry(&self.state.campaign, &pending.entry) else {
            // Content changed under a save: the encounter is gone.
            self.state.encounter = None;
            return Spent::NoTime;
        };
        let offered = super::talk_past::offered(&pending, entry, self.state.player(), &content);
        if !offered.contains(&choice) {
            self.say("That is not a choice now.");
            return Spent::Nothing;
        }
        self.state.encounter = None;
        let player = self.state.player;
        let open = |how| {
            Some(Opening {
                pending: pending.clone(),
                how,
            })
        };
        match choice {
            Choice::Pass => Spent::NoTime,
            Choice::Evade => self.evade(entry, pending),
            Choice::Bluff => self.talk_past(entry, pending),
            Choice::Ambush => {
                self.say("You wait in cover.");
                self.encounter.opening = open(How::Ambush);
                let wait = content.meetings().map_or(0, |r| u64::from(r.ambush_wait));
                let turns = i64::try_from(wait / BASE_TURN.as_seconds().max(1))
                    .unwrap_or(1)
                    .max(1);
                Spent::Turn(self.price_of(player, ActionKind::Wait) * turns)
            }
            Choice::Fight | Choice::Hail | Choice::Hunt => {
                let (how, line) = match choice {
                    Choice::Fight => (How::Fight, "You stand your ground.".to_string()),
                    Choice::Hail => (How::Hail, format!("You go to meet {}.", entry.name)),
                    _ => (How::Hunt, format!("You go after {}.", entry.name)),
                };
                self.say(line);
                self.encounter.opening = open(how);
                Spent::Turn(self.price_of(player, ActionKind::Stairs))
            }
        }
    }

    /// Slipping away (gameplay stream, two draws): the detour's time whatever comes of it;
    /// made, nothing more; failed, the fight opens with the party close.
    fn evade(&mut self, entry: &EncounterEntry, pending: Pending) -> Spent {
        let content = self.content.clone();
        let Some(rules) = content.meetings() else {
            return Spent::NoTime;
        };
        let unaware = pending.outcome == Outcome::Drop;
        let player = self.state.player();
        let talents = player.talent_mods(&content);
        let stalk = match unaware {
            true => player.skill_effect(&content, Effect::Stalk),
            false => 0,
        };
        let modifier = player.effective_stats(&content).modifier(Stat::Dexterity)
            + talents.check(Stat::Dexterity)
            + player.skill_effect(&content, Effect::Evade)
            + stalk;
        let e = rules.evade;
        let lead = self.lead_of(entry, &pending.members, pending.cell);
        let speed = lead.map_or(crate::content::NORMAL_SPEED, |c| content.creature(c).speed);
        let pursuit =
            i32::try_from(speed.saturating_sub(crate::content::NORMAL_SPEED) / e.pursuit_per)
                .unwrap_or(i32::MAX);
        let aware = if unaware { 0 } else { e.aware };
        let made = check::check(
            &mut self.state.rng,
            modifier,
            e.difficulty + pursuit + aware,
        );
        let price = (pending.step * i64::from(e.detour_percent) / 100).max(1);
        let landscape = self.here(pending.cell).map(|h| h.landscape);
        if made {
            match unaware {
                true => self.say("You slip away unseen."),
                false => self.say(format!("You lose them in {}.", lost_in(landscape))),
            }
            if self.trains(Use::Travel) {
                self.train(Use::Travel);
            }
            if unaware && self.trains(Use::Sneak) {
                self.train(Use::Sneak);
            }
        } else {
            match unaware {
                true => self.say("They are onto you."),
                false => self.say("They run you down."),
            }
            self.encounter.opening = Some(Opening {
                pending,
                how: How::Close,
            });
        }
        Spent::Turn(price)
    }

    /// The dev `meet:ENTRY[:OUTCOME]`: entry `id` of the run's campaign is met now on the
    /// player's overworld cell, by the lookouts or with `forced`'s outcome. Takes no time;
    /// refused (no time) off the overworld, on a cell without a local area, for an unknown entry,
    /// or while one is pending.
    pub(super) fn meet_dev(&mut self, id: &str, forced: Option<Forced>) -> Spent {
        let content = self.content.clone();
        let Some((_, entry)) = content.encounter_entry(&self.state.campaign, id) else {
            self.say(format!("There is no encounter `{id}` (dev)."));
            return Spent::Nothing;
        };
        let cell = self.player_pos();
        let local = content.local_area(&self.state.campaign, &local_area_id(cell));
        if self.state.current().kind != LevelKind::Overworld || local.is_none() {
            self.say("Nobody is met here: no local area (dev).");
            return Spent::Nothing;
        }
        if content.meetings().is_none() {
            self.say("Nobody meets anybody: no encounter rules (dev).");
            return Spent::Nothing;
        }
        let side = self.arrival_side();
        let step = price_of_travel(&content, local.map(|l| l.terrain));
        self.meet(entry, cell, side, step, forced, Watch::Travel);
        Spent::NoTime
    }
}

/// Text said when an action is refused while an encounter is pending.
pub const PENDING_REFUSED: &str = "You must decide what to do about them.";

/// The energy price of a plain step onto overworld terrain `terrain` (the dev `meet`'s stand-in
/// for the step a roll follows).
fn price_of_travel(content: &Content, terrain: Option<crate::content::TerrainId>) -> i64 {
    let minutes = terrain
        .and_then(|t| content.terrain(t).travel_minutes)
        .unwrap_or(1);
    crate::content::price(
        1000,
        crate::time::Duration::seconds(u64::from(minutes) * 60),
    )
}
