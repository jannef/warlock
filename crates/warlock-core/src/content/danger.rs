//! Danger levels and what comes to a level over time (spec §7 "Danger levels"): the data of a
//! dungeon branch (`data/dungeons.toml`) or a hand-made level that says how dangerous it is and
//! which creatures live there.
//!
//! ```toml
//! danger = 4                            # or { base = 4, per_depth = 1, player = { percent = 100, from = 1, max = 20 } }
//! population = ["rat", "kobold"]        # the types that live there (a branch's default: `creatures`)
//! population_count = { per_1000 = 35, least = 25, most = 80 }   # or a number: 40
//! out_of_depth = 5                      # percent of draws made 1 to 3 deeper
//! spawn = { every = "15m", chance = 60, cap = 40 }
//! ```
//!
//! The danger is never stored: [`DangerRule::at`] computes it from the depth and, for a
//! player-scaled area, the player's level. A level without these fields has danger 0 and nothing
//! comes there. The rules that use it are `world/spawn.rs`.

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
/// The default percent of draws made out of depth.
pub const OUT_OF_DEPTH: u8 = 5;
/// How much deeper an out-of-depth draw is made, at most.
pub const OUT_OF_DEPTH_STEPS: u8 = 3;
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

/// What lives on the levels of an area and how it comes back (spec §7 "Danger levels").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spawning {
    /// The creature types that live there, sorted by id (each draw goes by
    /// [`weight`]).
    pub population: Vec<CreatureId>,
    /// How many creatures a level holds: the cap the spawns refill toward, unless `cap` says.
    pub count: Count,
    /// Percent of the draws made 1 to [`OUT_OF_DEPTH_STEPS`] deeper.
    pub out_of_depth: u8,
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

/// A type that may be drawn: its id, its level, and its rarity's factor.
pub type Candidate = (CreatureId, u8, u16);

/// Draws a type among `candidates` (sorted by type) at `danger`: by [`weight`], uniform among
/// types of one weight. With `out_of_depth` percent the draw is made 1
/// to [`OUT_OF_DEPTH_STEPS`] deeper (and at the danger itself if nothing lives that deep).
/// `None` if nothing may be drawn.
pub fn draw(
    rng: &mut crate::rng::Rng,
    candidates: &[Candidate],
    danger: u8,
    out_of_depth: u8,
) -> Option<CreatureId> {
    let deeper = rng.chance(out_of_depth.into()).then(|| {
        let steps = rng.range(1, OUT_OF_DEPTH_STEPS.into());
        danger.saturating_add(u8::try_from(steps).unwrap_or(1))
    });
    let pick = |rng: &mut crate::rng::Rng, at: u8| {
        let total: u32 = candidates.iter().map(|&(_, l, f)| weight(l, f, at)).sum();
        if total == 0 {
            return None;
        }
        let mut r = rng.below(total);
        candidates.iter().find_map(|&(c, l, f)| {
            let w = weight(l, f, at);
            if r < w {
                Some(c)
            } else {
                r -= w;
                None
            }
        })
    };
    if let Some(at) = deeper
        && let Some(found) = pick(rng, at)
    {
        return Some(found);
    }
    if danger == 0 {
        return None;
    }
    pick(rng, danger)
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
    pub(super) out_of_depth: Option<u8>,
    pub(super) spawn: Option<SpawnSpec>,
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
    /// branch's patrols); `creature` resolves a type to its id and level.
    pub(super) fn resolve(
        self,
        branch: bool,
        default_population: &[CreatureId],
        creature: impl Fn(&str) -> Option<(CreatureId, u8)>,
    ) -> Result<Option<(DangerRule, Spawning)>, String> {
        let Some(danger) = self.danger else {
            let given = self.population.is_some()
                || self.population_count.is_some()
                || self.out_of_depth.is_some()
                || self.spawn.is_some();
            if given {
                return Err(
                    "`population`, `population_count`, `out_of_depth`, and `spawn` \
                            need a `danger`"
                        .into(),
                );
            }
            return Ok(None);
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
        let out_of_depth = self.out_of_depth.unwrap_or(OUT_OF_DEPTH);
        if out_of_depth > 100 {
            return Err("`out_of_depth` is a percent, 0 to 100".into());
        }
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
                count,
                out_of_depth,
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

    fn rule(text: &str) -> Result<Option<(DangerRule, Spawning)>, String> {
        let spec: SpawningSpec = toml::from_str(text).map_err(|e| e.to_string())?;
        let known = |n: &str| match n {
            "rat" => Some((CreatureId(3), 1)),
            "kobold" => Some((CreatureId(1), 2)),
            _ => None,
        };
        spec.resolve(true, &[], known)
    }

    #[test]
    fn a_branch_floor_is_base_plus_depth_times_per_depth() {
        let (r, s) = rule("danger = 4\npopulation = [\"rat\", \"kobold\", \"rat\"]\n")
            .unwrap()
            .unwrap();
        assert_eq!((r.at(1, 1), r.at(3, 30)), (4, 6));
        // Sorted by id, once each; the defaults.
        assert_eq!(s.population, vec![CreatureId(1), CreatureId(3)]);
        assert_eq!(
            (s.every, s.chance, s.out_of_depth, s.cap),
            (EVERY, 60, 5, None)
        );
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
        let kinds: Vec<Candidate> = (1..=20).map(|l| (CreatureId(l.into()), l, 100)).collect();
        let mut rng = Rng::stream(7, "test");
        let mut deeper = 0;
        for _ in 0..2000 {
            let c = draw(&mut rng, &kinds, 10, 5).expect("a draw");
            let level = kinds[usize::from(c.0) - 1].1;
            assert!((2..=13).contains(&level), "{level}");
            deeper += usize::from(level > 10);
        }
        // About 5 %, never a crowd.
        assert!((40..=160).contains(&deeper), "{deeper}");
        // Nothing within reach: nothing drawn.
        assert_eq!(draw(&mut rng, &[(CreatureId(0), 30, 100)], 3, 0), None);
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
        let kinds = [(CreatureId(1), 4, 100), (CreatureId(2), 4, 25)];
        let mut rng = Rng::stream(11, "test");
        let mut rare = 0;
        for _ in 0..4000 {
            rare += usize::from(draw(&mut rng, &kinds, 4, 0) == Some(CreatureId(2)));
        }
        // A fifth of the draws (800), within chance; the seeded count is fixed.
        assert!((680..=920).contains(&rare), "{rare}");
        let mut again = Rng::stream(11, "test");
        let count = (0..4000)
            .filter(|_| draw(&mut again, &kinds, 4, 0) == Some(CreatureId(2)))
            .count();
        assert_eq!(count, rare);
    }

    #[test]
    fn out_of_depth_draws_weigh_rarity_too() {
        // Nothing at the danger but a common type; deeper, a common and a very rare one.
        let kinds = [
            (CreatureId(1), 3, 100),
            (CreatureId(2), 5, 100),
            (CreatureId(3), 5, 10),
        ];
        let mut rng = Rng::stream(5, "test");
        let (mut deep, mut very_rare) = (0, 0);
        for _ in 0..4000 {
            let c = draw(&mut rng, &kinds, 3, 100).expect("a draw");
            deep += usize::from(c != CreatureId(1));
            very_rare += usize::from(c == CreatureId(3));
        }
        // Every draw is made 1 to 3 deeper: at 4 the level-5 types are out of reach, at 5 and
        // 6 they are drawn, the very rare one a tenth as often as the common one.
        assert!(deep > 1200, "{deep}");
        assert!(very_rare * 6 < deep, "{very_rare} of {deep}");
        assert!(very_rare > 0);
    }
}
