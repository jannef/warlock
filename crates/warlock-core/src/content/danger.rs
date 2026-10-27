//! Danger levels and what comes to a level over time (spec §7 "Danger levels"): the data of a
//! dungeon branch (`data/dungeons.toml`) or a hand-made level that says how dangerous it is and
//! which creatures live there.
//!
//! ```toml
//! danger = 4                            # or { base = 4, per_depth = 1, player = { percent = 100, from = 1, max = 20 } }
//! population = ["rat", "kobold"]        # the types that live there (a branch's default: `creatures`)
//! population_count = { per_1000 = 35, least = 25, most = 80 }   # or a number: 40
//! stray = 5                             # percent of draws from the generic creature list
//! out_of_depth = 5                      # percent of draws made 1 deeper, then perhaps more
//! deeper = 5                            # percent, again and again, of 1 deeper still
//! deeper_cap = 10                       # the most dangers past the first one deeper
//! spawn = { every = "15m", chance = 60, cap = 40 }
//! ```
//!
//! The danger is never stored: [`DangerRule::at`] computes it from the depth and, for a
//! player-scaled area, the player's level. A level without these fields has danger 0 and nothing
//! comes there. The rules that use it are `world/spawn.rs`.
//!
//! The four odds of a draw ([`Odds`]) default to `data/dungeons.toml`'s `[defaults]` table
//! (required once any area has a danger); a branch or a level may give each of its own. The
//! generic creature list strays come from is [`Spawning::generic`], filled once everything is
//! loaded (`Content::fill_generic`).

use super::creature::CreatureId;
use crate::time::Duration;
use serde::Deserialize;

/// The highest danger a level may have.
pub const MAX_DANGER: u8 = 99;
/// The most a level down may add to a branch's danger.
pub const MAX_PER_DEPTH: u8 = 5;
/// The most a player-scaled area counts of each player level past `from`, in percent.
pub const MAX_PLAYER_PERCENT: u16 = 200;
/// The largest population a level may be counted for.
pub const MAX_POPULATION: u32 = 200;
/// The default population: creatures per 1000 walkable cells, at least, at most.
pub const POPULATION: Count = Count::PerCells {
    per_1000: 35,
    least: 25,
    most: 80,
};
/// The most dangers an out-of-depth draw may go past its first one deeper (`deeper_cap`).
pub const MAX_DEEPER_CAP: u8 = 50;
/// The default time between two attempts to bring a creature.
pub const EVERY: Duration = Duration::minutes(15);
/// The default chance in percent that an attempt brings one.
pub const CHANCE: u8 = 60;
/// The shortest and longest time allowed between attempts.
const EVERY_LIMITS: (Duration, Duration) = (Duration::minutes(1), Duration::hours(24));
/// Types this far below the danger and further are never drawn.
pub const TOO_WEAK: u8 = 8;

/// How dangerous the levels of an area are: `base + (depth - 1) × per_depth`, plus, for a
/// player-scaled area, its [`PlayerScale`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DangerRule {
    pub base: u8,
    pub per_depth: u8,
    pub player: Option<PlayerScale>,
}

/// An area that grows with the player (ADOM's puppy cave): `percent` of each player level past
/// `from` is added, up to `max` in all.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerScale {
    pub percent: u16,
    pub from: u8,
    pub max: u8,
}

impl DangerRule {
    /// The danger of level `depth` (1 for a hand-made level) for a player of `player_level`.
    pub fn at(&self, depth: u32, player_level: u8) -> u8 {
        let below = depth.saturating_sub(1);
        let fixed =
            u32::from(self.base).saturating_add(below.saturating_mul(self.per_depth.into()));
        let danger = match self.player {
            None => fixed,
            Some(scale) => {
                let past = u32::from(player_level.saturating_sub(scale.from));
                let more = past * u32::from(scale.percent) / 100;
                fixed.saturating_add(more).min(scale.max.into())
            }
        };
        u8::try_from(danger.min(MAX_DANGER.into())).unwrap_or(MAX_DANGER)
    }

    /// Whether the danger depends on the player's level.
    pub fn scales(&self) -> bool {
        self.player.is_some()
    }
}

/// How many creatures a level's population is: a number, or a count by its walkable cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Count {
    Fixed(u32),
    PerCells {
        per_1000: u32,
        least: u32,
        most: u32,
    },
}

impl Count {
    /// The population of a level with `walkable` walkable cells.
    pub fn for_cells(self, walkable: u32) -> u32 {
        match self {
            Count::Fixed(n) => n,
            Count::PerCells {
                per_1000,
                least,
                most,
            } => (walkable.saturating_mul(per_1000) / 1000).clamp(least, most),
        }
    }
}

/// The odds of a draw's two departures from an area's own folk at its danger (spec §7 "Danger
/// levels"): `data/dungeons.toml`'s `[defaults]`, each overridable by a branch or a level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Odds {
    /// Percent of draws made from the generic creature list instead of the population.
    pub stray: u8,
    /// Percent of draws made out of depth: one danger deeper, then perhaps more.
    pub out_of_depth: u8,
    /// Percent, again and again, that an out-of-depth draw goes one danger deeper still.
    pub deeper: u8,
    /// The most dangers an out-of-depth draw goes past its first one deeper.
    pub deeper_cap: u8,
}

impl Odds {
    /// Checks the bounds: percents 0 to 100, `deeper_cap` at most [`MAX_DEEPER_CAP`].
    pub(super) fn check(&self) -> Result<(), String> {
        for (name, p) in [
            ("stray", self.stray),
            ("out_of_depth", self.out_of_depth),
            ("deeper", self.deeper),
        ] {
            if p > 100 {
                return Err(format!("`{name}` is a percent, 0 to 100"));
            }
        }
        if self.deeper_cap > MAX_DEEPER_CAP {
            return Err(format!("`deeper_cap` is at most {MAX_DEEPER_CAP}"));
        }
        Ok(())
    }
}

/// What lives on the levels of an area and how it comes back (spec §7 "Danger levels").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spawning {
    /// The creature types that live there, sorted by id (each draw goes by
    /// [`weight`]).
    pub population: Vec<CreatureId>,
    /// The generic creature list strays come from, sorted by id: every type that lives in some
    /// area with a danger of the area's campaign or of none, uniques left out (filled once
    /// everything is loaded, `Content::fill_generic`).
    pub generic: Vec<CreatureId>,
    /// How many creatures a level holds: the cap the spawns refill toward, unless `cap` says.
    pub count: Count,
    /// The odds of strays and out-of-depth draws.
    pub odds: Odds,
    /// Game time between two attempts while the player is on the level.
    pub every: Duration,
    /// Percent that an attempt brings a creature.
    pub chance: u8,
    /// The most hostile creatures a level holds before attempts stop; `None`: its count.
    pub cap: Option<u32>,
}

impl Spawning {
    /// The most creatures hostile to the player a level of `walkable` walkable cells may hold
    /// before an attempt is skipped.
    pub fn cap_for(&self, walkable: u32) -> u32 {
        self.cap.unwrap_or_else(|| self.count.for_cells(walkable))
    }
}

/// The weight a type's level gives it in a draw at `danger` (spec §7 "Danger levels"): 4 at the
/// danger or one below, 2 two or three below, 1 down to [`TOO_WEAK`] below, else 0 (above the
/// danger too; out-of-depth draws are made at a higher danger instead).
pub fn level_weight(level: u8, danger: u8) -> u32 {
    if level > danger {
        return 0;
    }
    match danger - level {
        0 | 1 => 4,
        2 | 3 => 2,
        d if d <= TOO_WEAK => 1,
        _ => 0,
    }
}

/// The weight of a type of `level` and rarity `factor` (percent of a common type's,
/// [`super::CreatureRarity::factor`]) in a draw at `danger`: its [`level_weight`] times the
/// factor.
pub fn weight(level: u8, factor: u16, danger: u8) -> u32 {
    level_weight(level, danger) * u32::from(factor)
}

/// A type that may be drawn: its id, its levels, and its rarity's factor. An ordinary type's
/// levels are its own level twice; a family's (spec §8 "Animated objects": `animated_blade`)
/// are its range, its id its first member's, the others numbered after it in a row.
pub type Candidate = (CreatureId, (u8, u8), u16);

/// The level candidate levels `(from, to)` stand at in a draw at `danger`: the danger, kept
/// within them.
fn level_at((from, to): (u8, u8), danger: u8) -> u8 {
    danger.clamp(from, to)
}

/// The type a candidate drawn at `danger` gives: a family's member of the level it stands at.
pub fn member((c, levels, _): Candidate, danger: u8) -> CreatureId {
    CreatureId(c.0 + u16::from(level_at(levels, danger) - levels.0))
}

/// Picks a type among `candidates` at `danger` by [`weight`] (a family's at the level it stands
/// at there, [`member`] its pick), uniform among types of one weight (one `below(total)` of
/// `rng`); `None` if none weighs anything there.
fn pick(rng: &mut crate::rng::Rng, candidates: &[Candidate], danger: u8) -> Option<CreatureId> {
    let weigh = |&(_, l, f): &Candidate| weight(level_at(l, danger), f, danger);
    let total: u32 = candidates.iter().map(weigh).sum();
    if total == 0 {
        return None;
    }
    let mut r = rng.below(total);
    candidates.iter().find_map(|c| {
        let w = weigh(c);
        if r < w {
            Some(member(*c, danger))
        } else {
            r -= w;
            None
        }
    })
}

/// The danger an out-of-depth draw at `danger` is made at: one deeper, then with `deeper` %
/// one deeper still, again and again until the chance misses or `deeper_cap` steps are made.
fn deeper(rng: &mut crate::rng::Rng, danger: u8, odds: &Odds) -> u8 {
    let mut at = danger.saturating_add(1);
    for _ in 0..odds.deeper_cap {
        if !rng.chance(odds.deeper.into()) {
            break;
        }
        at = at.saturating_add(1);
    }
    at
}

/// Draws a type at `danger` (spec §7 "Danger levels"): from `own` (the population), or with
/// `odds.stray` % from `generic` (the generic creature list), both sorted by type, by
/// [`weight`]. With `odds.out_of_depth` % the draw is made deeper ([`deeper`]); where the own
/// folk weigh nothing at that depth (all too weak for it), the generic list is drawn there
/// instead. A draw that finds nothing that deep is made at the danger itself, and a stray that
/// finds nothing there comes from the own folk. The rolls, in order: stray, out of depth, each
/// step deeper, then one `below` per pick that finds something. `None` at danger 0 or if nothing
/// may be drawn.
pub fn draw(
    rng: &mut crate::rng::Rng,
    own: &[Candidate],
    generic: &[Candidate],
    danger: u8,
    odds: &Odds,
) -> Option<CreatureId> {
    if danger == 0 {
        return None;
    }
    let stray = rng.chance(odds.stray.into());
    let at = (rng.chance(odds.out_of_depth.into())).then(|| deeper(rng, danger, odds));
    let first = if stray { generic } else { own };
    if let Some(at) = at
        && let Some(found) = pick(rng, first, at).or_else(|| pick(rng, generic, at))
    {
        return Some(found);
    }
    pick(rng, first, danger).or_else(|| pick(rng, own, danger))
}

/// `danger = 4` or `danger = { base = 4, per_depth = 1, player = { ... } }`.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum DangerSpec {
    Base(u8),
    Table(DangerTable),
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DangerTable {
    #[serde(default)]
    base: u8,
    per_depth: Option<u8>,
    player: Option<PlayerScale>,
}

/// `population_count = 40` or `population_count = { per_1000 = 35, least = 25, most = 80 }`.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum CountSpec {
    Fixed(u32),
    PerCells(PerCellsSpec),
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PerCellsSpec {
    per_1000: u32,
    least: u32,
    most: u32,
}

/// `spawn = { every = "15m", chance = 60, cap = 40 }`, each optional.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SpawnSpec {
    every: Option<String>,
    chance: Option<u8>,
    cap: Option<u32>,
}

/// The danger fields of a branch or a level file, as written.
#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct SpawningSpec {
    pub(super) danger: Option<DangerSpec>,
    pub(super) population: Option<Vec<String>>,
    pub(super) population_count: Option<CountSpec>,
    #[serde(flatten)]
    pub(super) odds: OddsSpec,
    pub(super) spawn: Option<SpawnSpec>,
}

/// A branch's or a level's own odds ([`Odds`]), each as written; `None`: the default's.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub(super) struct OddsSpec {
    pub(super) stray: Option<u8>,
    pub(super) out_of_depth: Option<u8>,
    pub(super) deeper: Option<u8>,
    pub(super) deeper_cap: Option<u8>,
}

impl OddsSpec {
    fn given(&self) -> bool {
        self.stray.is_some()
            || self.out_of_depth.is_some()
            || self.deeper.is_some()
            || self.deeper_cap.is_some()
    }

    /// The odds: each given field, else `defaults`'.
    fn resolve(self, defaults: Odds) -> Result<Odds, String> {
        let odds = Odds {
            stray: self.stray.unwrap_or(defaults.stray),
            out_of_depth: self.out_of_depth.unwrap_or(defaults.out_of_depth),
            deeper: self.deeper.unwrap_or(defaults.deeper),
            deeper_cap: self.deeper_cap.unwrap_or(defaults.deeper_cap),
        };
        odds.check()?;
        Ok(odds)
    }
}

/// `data/dungeons.toml`'s `[defaults]`: the odds every area with a danger has unless it gives
/// its own. Any other key of a dungeons file is ignored here (the branches read it).
#[derive(Debug, Default, Deserialize)]
pub(super) struct DefaultsFile {
    defaults: Option<Odds>,
}

impl DefaultsFile {
    /// The default odds of `text` (`data/dungeons.toml`), checked; `None` without `[defaults]`.
    pub(super) fn parse(text: &str) -> Result<Option<Odds>, String> {
        let file: DefaultsFile = toml::from_str(text).map_err(|e| e.to_string())?;
        let Some(odds) = file.defaults else {
            return Ok(None);
        };
        odds.check().map_err(|m| format!("[defaults]: {m}"))?;
        Ok(Some(odds))
    }
}

impl DangerSpec {
    /// The rule; `per_depth` defaults to `default_per_depth` (1 on a branch, 0 on a level, which
    /// may not give one).
    pub(super) fn resolve(self, default_per_depth: u8, branch: bool) -> Result<DangerRule, String> {
        let rule = match self {
            DangerSpec::Base(base) => DangerRule {
                base,
                per_depth: default_per_depth,
                player: None,
            },
            DangerSpec::Table(t) => {
                if !branch && t.per_depth.is_some() {
                    return Err("`danger.per_depth` is for dungeon branches only".into());
                }
                DangerRule {
                    base: t.base,
                    per_depth: t.per_depth.unwrap_or(default_per_depth),
                    player: t.player,
                }
            }
        };
        if rule.base > MAX_DANGER {
            return Err(format!("`danger` must be 0 to {MAX_DANGER}"));
        }
        if rule.per_depth > MAX_PER_DEPTH {
            return Err(format!("`danger.per_depth` must be 0 to {MAX_PER_DEPTH}"));
        }
        if let Some(p) = rule.player
            && (!(1..=MAX_PLAYER_PERCENT).contains(&p.percent) || p.max > MAX_DANGER)
        {
            return Err(format!(
                "`danger.player`: percent is 1 to {MAX_PLAYER_PERCENT}, max at most {MAX_DANGER}"
            ));
        }
        Ok(rule)
    }
}

impl SpawningSpec {
    /// The danger rule and what lives there, or `None` without a `danger` (then no other field
    /// may be given). `default_population` is what lives there without a `population` (a
    /// branch's patrols); `creature` resolves a type to its id and level; `defaults` are the
    /// odds of `data/dungeons.toml`'s `[defaults]`, required with a `danger`.
    pub(super) fn resolve(
        self,
        branch: bool,
        default_population: &[CreatureId],
        creature: impl Fn(&str) -> Option<(CreatureId, u8)>,
        defaults: Option<Odds>,
    ) -> Result<Option<(DangerRule, Spawning)>, String> {
        let Some(danger) = self.danger else {
            let given = self.population.is_some()
                || self.population_count.is_some()
                || self.odds.given()
                || self.spawn.is_some();
            if given {
                return Err(
                    "`population`, `population_count`, `stray`, `out_of_depth`, \
                            `deeper`, `deeper_cap`, and `spawn` need a `danger`"
                        .into(),
                );
            }
            return Ok(None);
        };
        let Some(defaults) = defaults else {
            return Err("a `danger` needs the `[defaults]` odds of data/dungeons.toml".into());
        };
        let rule = danger.resolve(u8::from(branch), branch)?;
        let mut population = match &self.population {
            Some(names) => names
                .iter()
                .map(|n| {
                    creature(n)
                        .map(|(c, _)| c)
                        .ok_or(format!("unknown creature `{n}`"))
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => default_population.to_vec(),
        };
        population.sort();
        population.dedup();
        if population.is_empty() {
            return Err("a `danger` needs a `population` (the creatures that live there)".into());
        }
        let count = match self.population_count {
            None => POPULATION,
            Some(CountSpec::Fixed(n)) => Count::Fixed(n),
            Some(CountSpec::PerCells(PerCellsSpec {
                per_1000,
                least,
                most,
            })) => {
                if per_1000 > 1000 || least > most {
                    return Err(
                        "`population_count`: per_1000 is at most 1000, least at most \
                                most"
                            .into(),
                    );
                }
                Count::PerCells {
                    per_1000,
                    least,
                    most,
                }
            }
        };
        let most = match count {
            Count::Fixed(n) => n,
            Count::PerCells { most, .. } => most,
        };
        if most > MAX_POPULATION {
            return Err(format!("`population_count` is at most {MAX_POPULATION}"));
        }
        let odds = self.odds.resolve(defaults)?;
        let spawn = self.spawn.unwrap_or_default();
        let every = match &spawn.every {
            Some(text) => text.parse::<Duration>()?,
            None => EVERY,
        };
        if every < EVERY_LIMITS.0 || every > EVERY_LIMITS.1 {
            return Err(format!(
                "`spawn.every` must be {} to {}",
                EVERY_LIMITS.0, EVERY_LIMITS.1
            ));
        }
        let chance = spawn.chance.unwrap_or(CHANCE);
        if chance > 100 {
            return Err("`spawn.chance` is a percent, 0 to 100".into());
        }
        if spawn.cap.is_some_and(|c| c > MAX_POPULATION) {
            return Err(format!("`spawn.cap` is at most {MAX_POPULATION}"));
        }
        Ok(Some((
            rule,
            Spawning {
                population,
                generic: Vec::new(),
                count,
                odds,
                every,
                chance,
                cap: spawn.cap,
            },
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    /// The defaults of `data/dungeons.toml`.
    const ODDS: Odds = Odds {
        stray: 5,
        out_of_depth: 5,
        deeper: 5,
        deeper_cap: 10,
    };

    /// Every draw two deeper.
    const TWO_DEEPER: Odds = Odds {
        stray: 0,
        out_of_depth: 100,
        deeper: 100,
        deeper_cap: 1,
    };

    /// No strays and no out-of-depth draws.
    const PLAIN: Odds = Odds {
        stray: 0,
        out_of_depth: 0,
        deeper: 0,
        deeper_cap: 0,
    };

    fn rule(text: &str) -> Result<Option<(DangerRule, Spawning)>, String> {
        let spec: SpawningSpec = toml::from_str(text).map_err(|e| e.to_string())?;
        let known = |n: &str| match n {
            "rat" => Some((CreatureId(3), 1)),
            "kobold" => Some((CreatureId(1), 2)),
            _ => None,
        };
        spec.resolve(true, &[], known, Some(ODDS))
    }

    #[test]
    fn the_shipped_defaults_are_the_issues_odds() {
        let text = include_str!("../../../../assets/data/dungeons.toml");
        assert_eq!(DefaultsFile::parse(text).unwrap(), Some(ODDS));
        assert_eq!(DefaultsFile::parse("").unwrap(), None);
        let bad = "[defaults]\nstray = 5\nout_of_depth = 5\ndeeper = 5\ndeeper_cap = 51\n";
        assert!(DefaultsFile::parse(bad).is_err());
    }

    #[test]
    fn an_area_may_give_its_own_odds() {
        let (_, s) = rule("danger = 2\npopulation = [\"rat\"]\nstray = 0\ndeeper_cap = 3\n")
            .unwrap()
            .unwrap();
        assert_eq!(
            s.odds,
            Odds {
                stray: 0,
                deeper_cap: 3,
                ..ODDS
            }
        );
        // Without the defaults a danger is refused.
        let spec: SpawningSpec = toml::from_str("danger = 2\npopulation = [\"rat\"]\n").unwrap();
        let known = |_: &str| Some((CreatureId(3), 1));
        assert!(spec.resolve(true, &[], known, None).is_err());
    }

    #[test]
    fn a_branch_floor_is_base_plus_depth_times_per_depth() {
        let (r, s) = rule("danger = 4\npopulation = [\"rat\", \"kobold\", \"rat\"]\n")
            .unwrap()
            .unwrap();
        assert_eq!((r.at(1, 1), r.at(3, 30)), (4, 6));
        // Sorted by id, once each; the defaults.
        assert_eq!(s.population, vec![CreatureId(1), CreatureId(3)]);
        assert_eq!((s.every, s.chance, s.odds, s.cap), (EVERY, 60, ODDS, None));
        let (r, _) = rule("danger = { base = 2, per_depth = 3 }\npopulation = [\"rat\"]\n")
            .unwrap()
            .unwrap();
        assert_eq!(r.at(4, 1), 11);
    }

    #[test]
    fn a_player_scaled_area_grows_with_the_player_up_to_its_ceiling() {
        let text = "danger = { base = 1, per_depth = 0, player = { percent = 50, from = 3, max = \
                    6 } }\npopulation = [\"rat\"]\n";
        let (r, _) = rule(text).unwrap().unwrap();
        assert!(r.scales());
        // Level 3 and below count nothing; then half a danger a level, rounded down.
        assert_eq!(
            (r.at(1, 1), r.at(1, 3), r.at(1, 5), r.at(1, 8)),
            (1, 1, 2, 3)
        );
        assert_eq!(r.at(1, 60), 6);
    }

    #[test]
    fn counts_go_by_walkable_cells_within_bounds() {
        assert_eq!(POPULATION.for_cells(1100), 38);
        assert_eq!(POPULATION.for_cells(100), 25);
        assert_eq!(POPULATION.for_cells(3600), 80);
        assert_eq!(Count::Fixed(7).for_cells(5000), 7);
        let (_, s) = rule(
            "danger = 1\npopulation = [\"rat\"]\npopulation_count = 12\n\
                           spawn = { every = \"1h\", chance = 100, cap = 3 }\n",
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            (s.cap_for(9999), s.every, s.chance),
            (3, Duration::hours(1), 100)
        );
    }

    #[test]
    fn bad_danger_fields_are_refused() {
        assert_eq!(rule("").unwrap(), None);
        for bad in [
            "population = [\"rat\"]\n",
            "spawn = { chance = 5 }\n",
            "danger = 100\npopulation = [\"rat\"]\n",
            "danger = { base = 1, per_depth = 6 }\npopulation = [\"rat\"]\n",
            "danger = 1\n",
            "danger = 1\npopulation = [\"dragon\"]\n",
            "danger = 1\npopulation = [\"rat\"]\nout_of_depth = 101\n",
            "danger = 1\npopulation = [\"rat\"]\nstray = 101\n",
            "danger = 1\npopulation = [\"rat\"]\ndeeper_cap = 51\n",
            "stray = 5\n",
            "danger = 1\npopulation = [\"rat\"]\nspawn = { every = \"30s\" }\n",
            "danger = 1\npopulation = [\"rat\"]\nspawn = { chance = 101 }\n",
            "danger = 1\npopulation = [\"rat\"]\npopulation_count = 201\n",
            "danger = 1\npopulation = [\"rat\"]\npopulation_count = { per_1000 = 5, least = \
             9, most = 3 }\n",
            "danger = { base = 1, player = { percent = 0, from = 1, max = 9 } }\npopulation = \
             [\"rat\"]\n",
        ] {
            assert!(rule(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn weights_favour_the_danger_and_drop_the_weak_and_the_deep() {
        let at = |l| level_weight(l, 10);
        assert_eq!(
            [at(11), at(10), at(9), at(8), at(7), at(6), at(2), at(1)],
            [0, 4, 4, 2, 2, 1, 1, 0]
        );
    }

    #[test]
    fn draws_stay_within_reach_of_the_danger() {
        let kinds: Vec<Candidate> = (1..=40)
            .map(|l| (CreatureId(l.into()), (l, l), 100))
            .collect();
        let mut rng = Rng::stream(7, "test");
        let mut deeper = 0;
        for _ in 0..4000 {
            let c = draw(&mut rng, &kinds, &[], 10, &ODDS).expect("a draw");
            let level = kinds[usize::from(c.0) - 1].1.0;
            // At most one deeper and ten more; nothing more than eight below the danger.
            assert!((2..=21).contains(&level), "{level}");
            deeper += usize::from(level > 10);
        }
        // Out of depth 5 % of the time, then at D + 1 its own level or the one above it (4 of
        // 17 by weight): about 1 % of the draws, never a crowd.
        assert!((20..=80).contains(&deeper), "{deeper}");
        // Nothing within reach: nothing drawn; nothing at danger 0.
        assert_eq!(
            draw(&mut rng, &[(CreatureId(0), (30, 30), 100)], &[], 3, &PLAIN),
            None
        );
        assert_eq!(draw(&mut rng, &kinds, &kinds, 0, &ODDS), None);
    }

    #[test]
    fn about_one_draw_in_twenty_is_a_stray() {
        // The own folk and the generic list share nothing here, so every stray shows.
        let (own, generic) = (
            [(CreatureId(1), (5, 5), 100)],
            [(CreatureId(2), (5, 5), 100)],
        );
        let odds = Odds {
            out_of_depth: 0,
            ..ODDS
        };
        let mut rng = Rng::stream(3, "test");
        let strays = (0..20_000)
            .filter(|_| draw(&mut rng, &own, &generic, 5, &odds) == Some(CreatureId(2)))
            .count();
        // 5 % is 1000; the seeded count is fixed, the bounds allow for chance.
        assert!((880..=1120).contains(&strays), "{strays}");
        // A place with `stray = 0` never has one.
        let mut rng = Rng::stream(3, "test");
        assert!(
            (0..2000).all(|_| draw(&mut rng, &own, &generic, 5, &PLAIN) == Some(CreatureId(1)))
        );
        // A stray that finds nothing of the danger comes from the own folk.
        let far = [(CreatureId(2), (30, 30), 100)];
        let always = Odds { stray: 100, ..odds };
        assert_eq!(draw(&mut rng, &own, &far, 5, &always), Some(CreatureId(1)));
    }

    #[test]
    fn out_of_depth_draws_explode_geometrically() {
        let mut rng = Rng::stream(9, "test");
        let mut steps = [0u32; 12];
        for _ in 0..100_000 {
            steps[usize::from(deeper(&mut rng, 10, &ODDS) - 11)] += 1;
        }
        // One deeper 95 % of the time, two 4.75 %, three 0.24 %, each further 5 % of the last.
        assert!((94_600..=95_400).contains(&steps[0]), "{steps:?}");
        assert!((4_450..=5_050).contains(&steps[1]), "{steps:?}");
        assert!((170..=310).contains(&steps[2]), "{steps:?}");
        assert!(steps[3..].iter().sum::<u32>() < 40, "{steps:?}");
        // The cap holds: a sure explosion stops after `deeper_cap` steps.
        let sure = Odds {
            deeper: 100,
            deeper_cap: 10,
            ..ODDS
        };
        assert_eq!(deeper(&mut rng, 10, &sure), 21);
        assert_eq!(deeper(&mut rng, 250, &sure), 255);
    }

    #[test]
    fn a_population_too_weak_for_the_depth_falls_back_to_the_generic_list() {
        let own = [(CreatureId(1), (5, 5), 100)];
        let generic = [
            (CreatureId(2), (14, 14), 100),
            (CreatureId(3), (30, 30), 100),
        ];
        let mut rng = Rng::stream(4, "test");
        // One deeper: the own type of level 5 still weighs at 6, so it is drawn.
        let one = Odds {
            stray: 0,
            out_of_depth: 100,
            deeper: 0,
            deeper_cap: 0,
        };
        assert!((0..500).all(|_| draw(&mut rng, &own, &generic, 5, &one) == Some(CreatureId(1))));
        // Nine deeper, at 14: level 5 is too weak to weigh, so the generic list is drawn.
        let nine = Odds {
            deeper: 100,
            deeper_cap: 8,
            ..one
        };
        assert!((0..500).all(|_| draw(&mut rng, &own, &generic, 5, &nine) == Some(CreatureId(2))));
        // Nothing anywhere that deep: the draw is made at the danger.
        let bare = [(CreatureId(3), (30, 30), 100)];
        assert_eq!(draw(&mut rng, &own, &bare, 5, &nine), Some(CreatureId(1)));
    }

    #[test]
    fn spawns_from_the_shipped_odds_are_mostly_own_folk_at_the_danger() {
        // Seeded statistics with the shipped odds: the own folk at the danger, a generic list
        // of other kinds at every level.
        let own: Vec<Candidate> = (3..=6)
            .map(|l| (CreatureId(l.into()), (l, l), 100))
            .collect();
        let generic: Vec<Candidate> = (1..=30)
            .map(|l| (CreatureId(100 + u16::from(l)), (l, l), 100))
            .collect();
        let mut rng = Rng::stream(21, "test");
        let (mut strays, mut deep) = (0, 0);
        for _ in 0..20_000 {
            let c = draw(&mut rng, &own, &generic, 6, &ODDS).expect("a draw");
            strays += usize::from(c.0 > 100);
            deep += usize::from(c.0 > 106);
        }
        // About 5 % strays (and the generic list's draws out of depth: 0.25 % more).
        assert!((900..=1200).contains(&strays), "{strays}");
        // Deeper than the danger only from the generic list: a stray made out of depth, or an
        // out-of-depth draw the own folk cannot answer (none here, they weigh at 7).
        assert!(deep < 40, "{deep}");
    }

    /// A family (spec §8 "Animated objects": `animated_blade`, levels 5 to 20, numbered from 10)
    /// is one candidate: it weighs as its member at the danger, kept within its levels, and
    /// gives that member, out of depth too.
    #[test]
    fn a_family_weighs_and_gives_its_member_at_the_danger() {
        let blades: Candidate = (CreatureId(10), (5, 20), 100);
        let kobold: Candidate = (CreatureId(1), (15, 15), 100);
        let mut rng = Rng::stream(3, "test");
        // At 15 it weighs as a level-15 type: as much as the kobold, and it is the 15th blade.
        let mut blades_at_15 = 0;
        for _ in 0..2000 {
            match draw(&mut rng, &[kobold, blades], &[], 15, &PLAIN) {
                Some(CreatureId(20)) => blades_at_15 += 1,
                Some(CreatureId(1)) => {}
                other => panic!("{other:?}"),
            }
        }
        assert!((850..=1150).contains(&blades_at_15), "{blades_at_15}");
        // Below its least level it weighs nothing; above its most it weighs as that one, fading.
        assert_eq!(draw(&mut rng, &[blades], &[], 3, &PLAIN), None);
        assert_eq!(member(blades, 3), CreatureId(10));
        assert_eq!(
            draw(&mut rng, &[blades], &[], 26, &PLAIN),
            Some(CreatureId(25))
        );
        assert_eq!(draw(&mut rng, &[blades], &[], 30, &PLAIN), None);
        // Out of depth: two deeper, the member two levels up.
        for _ in 0..50 {
            assert_eq!(
                draw(&mut rng, &[blades], &[], 8, &TWO_DEEPER),
                Some(CreatureId(15))
            );
        }
        // Out of depth past its most: its last member.
        assert_eq!(
            draw(&mut rng, &[blades], &[], 20, &TWO_DEEPER),
            Some(CreatureId(25))
        );
        assert_eq!(weight(level_at((5, 20), 30), 100, 30), 0);
    }

    #[test]
    fn a_rarity_multiplies_the_level_weight() {
        assert_eq!(weight(10, 100, 10), 400);
        assert_eq!(weight(10, 10, 10), 40);
        assert_eq!(weight(5, 25, 10), 25);
        assert_eq!(weight(11, 100, 10), 0);
    }

    #[test]
    fn a_rare_type_is_drawn_far_less_often() {
        // Two types of one level: a common one and a rare one (a quarter of the weight).
        let kinds = [(CreatureId(1), (4, 4), 100), (CreatureId(2), (4, 4), 25)];
        let mut rng = Rng::stream(11, "test");
        let mut rare = 0;
        for _ in 0..4000 {
            rare += usize::from(draw(&mut rng, &kinds, &[], 4, &PLAIN) == Some(CreatureId(2)));
        }
        // A fifth of the draws (800), within chance; the seeded count is fixed.
        assert!((680..=920).contains(&rare), "{rare}");
        let mut again = Rng::stream(11, "test");
        let count = (0..4000)
            .filter(|_| draw(&mut again, &kinds, &[], 4, &PLAIN) == Some(CreatureId(2)))
            .count();
        assert_eq!(count, rare);
    }

    #[test]
    fn out_of_depth_draws_weigh_rarity_too() {
        // Nothing at the danger but a common type; deeper, a common and a very rare one.
        let kinds = [
            (CreatureId(1), (3, 3), 100),
            (CreatureId(2), (5, 5), 100),
            (CreatureId(3), (5, 5), 10),
        ];
        let mut rng = Rng::stream(5, "test");
        let (mut deep, mut very_rare) = (0, 0);
        for _ in 0..4000 {
            let c = draw(&mut rng, &kinds, &[], 3, &TWO_DEEPER).expect("a draw");
            deep += usize::from(c != CreatureId(1));
            very_rare += usize::from(c == CreatureId(3));
        }
        // Every draw is made two deeper, at 5: the very rare type a tenth as often as the common
        // one of its level.
        assert!(deep > 2400, "{deep}");
        assert!(very_rare * 6 < deep, "{very_rare} of {deep}");
        assert!(very_rare > 0);
    }
}
