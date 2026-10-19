//! Stats (spec §8 "Stats"): Strength, Dexterity, Toughness, Intelligence, Willpower, and
//! Presence, inspired by D&D (Toughness is its Constitution, Willpower its Wisdom, Presence its
//! Charisma).
//!
//! Every actor has a **score** in each stat, [`AVERAGE`] (10) unless its creature type (or, for
//! the player, the scenario) says otherwise, and a **potential**: the natural ceiling the score
//! could grow towards, ADOM-like. Nothing raises a score yet.
//!
//! The **modifier** of a score follows D&D 3.5: `floor((score − 10) / 2)` ([`modifier`]). It is
//! for the rules only. The player sees their own scores
//! ([`PerceptionView::own_scores`](crate::perception::PerceptionView::own_scores)) and nothing
//! else: never a modifier, never a potential, never a creature's stats. No rule uses stats yet.
//!
//! Adding a stat is one entry in the `stats!` list below; saves then need a new format version.

use crate::dice::Dice;
use crate::rng::Rng;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::ops::{Index, IndexMut, RangeInclusive};

/// The average score, and every score's default.
pub const AVERAGE: u8 = 10;

/// The scores and potentials content may give an actor.
pub const SCORES: RangeInclusive<u8> = 1..=40;

/// The adjustments a race, class, or birth sign may make to a stat (spec §8 "Character
/// creation"): every result a rolled adjustment can have lies within.
pub const ADJUSTMENTS: RangeInclusive<i8> = -10..=10;

/// Defines [`Stat`] from one entry per stat: the variant, its id in data files and saves, its
/// two-letter abbreviation, and its name, in display order.
macro_rules! stats {
    ($($(#[$doc:meta])* $variant:ident = $id:literal, $abbrev:literal, $name:literal;)+) => {
        /// A stat. [`Stat::ALL`] lists them in display order.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Stat {
            $($(#[$doc])* $variant,)+
        }

        impl Stat {
            /// How many stats there are.
            pub const COUNT: usize = [$($id),+].len();

            /// Every stat, in display order.
            pub const ALL: [Stat; Stat::COUNT] = [$(Stat::$variant),+];

            /// The id in data files and saves: `strength`.
            pub const fn id(self) -> &'static str {
                match self {
                    $(Stat::$variant => $id,)+
                }
            }

            /// The two-letter abbreviation: `St`.
            pub const fn abbrev(self) -> &'static str {
                match self {
                    $(Stat::$variant => $abbrev,)+
                }
            }

            /// The name: `Strength`.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Stat::$variant => $name,)+
                }
            }
        }
    };
}

stats! {
    /// Physical power.
    Strength = "strength", "St", "Strength";
    /// Agility, balance, and quick hands.
    Dexterity = "dexterity", "Dx", "Dexterity";
    /// Health and endurance (D&D's Constitution).
    Toughness = "toughness", "To", "Toughness";
    /// Reason and memory.
    Intelligence = "intelligence", "In", "Intelligence";
    /// Awareness and strength of will (D&D's Wisdom).
    Willpower = "willpower", "Wi", "Willpower";
    /// Force of personality (D&D's Charisma).
    Presence = "presence", "Pr", "Presence";
}

impl Stat {
    /// The stat with data id `id`.
    pub fn from_id(id: &str) -> Option<Stat> {
        Stat::ALL.into_iter().find(|s| s.id() == id)
    }

    fn index(self) -> usize {
        self as usize
    }
}

impl Serialize for Stat {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for Stat {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let id = String::deserialize(deserializer)?;
        Stat::from_id(&id).ok_or_else(|| {
            let known: Vec<&str> = Stat::ALL.iter().map(|s| s.id()).collect();
            D::Error::custom(format!(
                "unknown stat `{id}` (expected one of {})",
                known.join(", ")
            ))
        })
    }
}

/// The D&D 3.5 modifier of a score: `floor((score − 10) / 2)`, so 1 → −5, 8 and 9 → −1, 10 and
/// 11 → 0, 12 and 13 → +1, 18 → +4. For the rules only: **never shown** to the player.
pub const fn modifier(score: i32) -> i32 {
    (score - AVERAGE as i32).div_euclid(2)
}

/// `score` adjusted by `by`, kept within [`SCORES`].
pub fn adjusted(score: u8, by: i32) -> u8 {
    let (lo, hi) = (i32::from(*SCORES.start()), i32::from(*SCORES.end()));
    (i32::from(score) + by).clamp(lo, hi) as u8
}

/// Stat adjustments as data gives them (`stats = { strength = "1d3+1", presence = -1 }`): a
/// fixed number or [`Dice`] per stat, 0 if missing, every possible result within
/// [`ADJUSTMENTS`]; the error names the stat.
pub fn adjustments(data: &BTreeMap<Stat, Dice>) -> Result<PerStat<Dice>, String> {
    let mut out = PerStat::splat(Dice::fixed(0));
    let range = i32::from(*ADJUSTMENTS.start())..=i32::from(*ADJUSTMENTS.end());
    for (&stat, &by) in data {
        if !range.contains(&by.min()) || !range.contains(&by.max()) {
            return Err(format!(
                "{} {by} is out of range ({} to {})",
                stat.id(),
                ADJUSTMENTS.start(),
                ADJUSTMENTS.end()
            ));
        }
        out[stat] = by;
    }
    Ok(out)
}

/// Rolls an adjustment for every stat, in display order, from `rng` (fixed numbers draw
/// nothing).
pub fn roll(by: &PerStat<Dice>, rng: &mut Rng) -> PerStat<i32> {
    let mut out = PerStat::splat(0);
    for stat in Stat::ALL {
        out[stat] = by[stat].roll(rng);
    }
    out
}

/// One value for every stat, by stat. Serialized as a map from stat id to value, in display
/// order; reading one needs every stat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PerStat<T>([T; Stat::COUNT]);

impl<T: Copy> PerStat<T> {
    /// `value` for every stat.
    pub const fn splat(value: T) -> Self {
        Self([value; Stat::COUNT])
    }

    /// Every stat with its value, in display order.
    pub fn iter(&self) -> impl Iterator<Item = (Stat, T)> + '_ {
        Stat::ALL.into_iter().map(|s| (s, self[s]))
    }
}

impl<T> Index<Stat> for PerStat<T> {
    type Output = T;

    fn index(&self, stat: Stat) -> &T {
        &self.0[stat.index()]
    }
}

impl<T> IndexMut<Stat> for PerStat<T> {
    fn index_mut(&mut self, stat: Stat) -> &mut T {
        &mut self.0[stat.index()]
    }
}

impl<T: Serialize> Serialize for PerStat<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(Stat::COUNT))?;
        for stat in Stat::ALL {
            map.serialize_entry(stat.id(), &self[stat])?;
        }
        map.end()
    }
}

impl<'de, T: Deserialize<'de> + Copy + Default> Deserialize<'de> for PerStat<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map = BTreeMap::<Stat, T>::deserialize(deserializer)?;
        let mut out = PerStat::splat(T::default());
        for stat in Stat::ALL {
            out[stat] = *map
                .get(&stat)
                .ok_or_else(|| D::Error::custom(format!("missing stat `{}`", stat.id())))?;
        }
        Ok(out)
    }
}

/// An actor's stats: a score and a potential for every stat. Rules only, like everything about
/// an actor but what the player perceives: the player is shown their own scores, nothing more.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    score: PerStat<u8>,
    potential: PerStat<u8>,
}

impl Default for Stats {
    /// [`AVERAGE`] in every stat, with potentials to match.
    fn default() -> Self {
        Self {
            score: PerStat::splat(AVERAGE),
            potential: PerStat::splat(AVERAGE),
        }
    }
}

impl Stats {
    /// Stats as data gives them: each stat's score from `scores` ([`AVERAGE`] if missing) and
    /// its potential from `potentials` (the score if missing). Both must lie in [`SCORES`], and a
    /// potential may not be below its score; the error names the stat.
    pub fn from_data(
        scores: &BTreeMap<Stat, u8>,
        potentials: &BTreeMap<Stat, u8>,
    ) -> Result<Self, String> {
        let mut stats = Stats::default();
        let range = format!("{} to {}", SCORES.start(), SCORES.end());
        for stat in Stat::ALL {
            let score = scores.get(&stat).copied().unwrap_or(AVERAGE);
            let potential = potentials.get(&stat).copied().unwrap_or(score);
            let id = stat.id();
            if !SCORES.contains(&score) {
                return Err(format!("stats: {id} {score} is out of range ({range})"));
            }
            if !SCORES.contains(&potential) {
                return Err(format!(
                    "potential: {id} {potential} is out of range ({range})"
                ));
            }
            if potential < score {
                return Err(format!(
                    "potential: {id} {potential} is below its score {score}"
                ));
            }
            stats.score[stat] = score;
            stats.potential[stat] = potential;
        }
        Ok(stats)
    }

    /// Stats from a score and a potential for every stat; a potential below its score is raised
    /// to it.
    pub fn new(score: PerStat<u8>, mut potential: PerStat<u8>) -> Self {
        for stat in Stat::ALL {
            potential[stat] = potential[stat].max(score[stat]);
        }
        Self { score, potential }
    }

    /// The score in `stat`.
    pub fn score(&self, stat: Stat) -> u8 {
        self.score[stat]
    }

    /// Every score: what the player may see of their own stats.
    pub fn scores(&self) -> PerStat<u8> {
        self.score
    }

    /// The potential in `stat`: the natural ceiling its score could grow towards. Hidden from
    /// the player.
    pub fn potential(&self, stat: Stat) -> u8 {
        self.potential[stat]
    }

    /// The modifier of the score in `stat` ([`modifier`]). Rules only: never shown.
    pub fn modifier(&self, stat: Stat) -> i32 {
        modifier(i32::from(self.score(stat)))
    }

    /// These stats with every score adjusted by `by` (kept within [`SCORES`]; a potential below
    /// its score is raised to it): the scores the rules read while a status effect lowers some
    /// (spec §8 "Status effects").
    pub fn adjusted_by(&self, by: &PerStat<i32>) -> Self {
        let mut score = self.score;
        for stat in Stat::ALL {
            score[stat] = adjusted(score[stat], by[stat]);
        }
        Self::new(score, self.potential)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_six_stats_in_order() {
        let names: Vec<String> = Stat::ALL
            .iter()
            .map(|s| format!("{} {} {}", s.abbrev(), s.name(), s.id()))
            .collect();
        assert_eq!(
            names,
            [
                "St Strength strength",
                "Dx Dexterity dexterity",
                "To Toughness toughness",
                "In Intelligence intelligence",
                "Wi Willpower willpower",
                "Pr Presence presence",
            ]
        );
        assert_eq!(Stat::COUNT, 6);
        for stat in Stat::ALL {
            assert_eq!(Stat::from_id(stat.id()), Some(stat));
            assert_eq!(Stat::ALL[stat.index()], stat);
        }
        assert_eq!(Stat::from_id("Strength"), None);
        assert_eq!(Stat::from_id("constitution"), None);
    }

    #[test]
    fn modifiers_follow_dnd_3_5() {
        let table = [
            (1, -5),
            (2, -4),
            (3, -4),
            (7, -2),
            (8, -1),
            (9, -1),
            (10, 0),
            (11, 0),
            (12, 1),
            (13, 1),
            (17, 3),
            (18, 4),
            (19, 4),
            (20, 5),
            (40, 15),
        ];
        for (score, expected) in table {
            assert_eq!(modifier(score), expected, "score {score}");
        }
        // Rounded down, not toward zero, below 10 and below 0 alike.
        assert_eq!(modifier(0), -5);
        assert_eq!(modifier(-1), -6);
        assert_eq!(modifier(-2), -6);
        assert_eq!(modifier(-3), -7);
        for score in -50..=60 {
            let exact = f64::from(score - 10) / 2.0;
            assert_eq!(f64::from(modifier(score)), exact.floor(), "score {score}");
        }
    }

    #[test]
    fn stats_default_to_average_with_matching_potentials() {
        let stats = Stats::default();
        for stat in Stat::ALL {
            assert_eq!(stats.score(stat), 10);
            assert_eq!(stats.potential(stat), 10);
            assert_eq!(stats.modifier(stat), 0);
        }
        let none = BTreeMap::new();
        assert_eq!(Stats::from_data(&none, &none), Ok(stats));
    }

    #[test]
    fn data_gives_scores_and_potentials_that_default_to_the_score() {
        let scores = BTreeMap::from([(Stat::Strength, 14), (Stat::Presence, 7)]);
        let potentials = BTreeMap::from([(Stat::Strength, 18), (Stat::Willpower, 12)]);
        let stats = Stats::from_data(&scores, &potentials).expect("valid");
        assert_eq!(
            (stats.score(Stat::Strength), stats.potential(Stat::Strength)),
            (14, 18)
        );
        assert_eq!(
            (stats.score(Stat::Presence), stats.potential(Stat::Presence)),
            (7, 7)
        );
        assert_eq!(
            (
                stats.score(Stat::Willpower),
                stats.potential(Stat::Willpower)
            ),
            (10, 12)
        );
        assert_eq!(
            (
                stats.score(Stat::Dexterity),
                stats.potential(Stat::Dexterity)
            ),
            (10, 10)
        );
        assert_eq!(stats.modifier(Stat::Strength), 2);
        assert_eq!(stats.modifier(Stat::Presence), -2);
        assert_eq!(
            stats.scores().iter().map(|(_, v)| v).collect::<Vec<_>>(),
            [14, 10, 10, 10, 10, 7]
        );
    }

    #[test]
    fn data_out_of_range_or_below_the_score_is_refused() {
        let one = |stat, v| BTreeMap::from([(stat, v)]);
        let none = BTreeMap::new();
        for (scores, potentials, why) in [
            (
                one(Stat::Strength, 0),
                none.clone(),
                "stats: strength 0 is out of range",
            ),
            (
                one(Stat::Dexterity, 41),
                none.clone(),
                "stats: dexterity 41",
            ),
            (
                none.clone(),
                one(Stat::Toughness, 41),
                "potential: toughness 41",
            ),
            (
                none.clone(),
                one(Stat::Presence, 9),
                "presence 9 is below its score 10",
            ),
            (
                one(Stat::Willpower, 15),
                one(Stat::Willpower, 14),
                "below its score 15",
            ),
        ] {
            let err = Stats::from_data(&scores, &potentials).unwrap_err();
            assert!(err.contains(why), "{err}");
        }
        let edges = BTreeMap::from([(Stat::Strength, 1), (Stat::Dexterity, 40)]);
        assert!(Stats::from_data(&edges, &none).is_ok());
    }

    #[test]
    fn stats_serialize_by_stat_id_and_round_trip() {
        let scores = BTreeMap::from([(Stat::Intelligence, 15)]);
        let potentials = BTreeMap::from([(Stat::Intelligence, 17)]);
        let stats = Stats::from_data(&scores, &potentials).expect("valid");
        let json = serde_json::to_string(&stats).expect("serializes");
        assert_eq!(
            json,
            "{\"score\":{\"strength\":10,\"dexterity\":10,\"toughness\":10,\"intelligence\":15,\
             \"willpower\":10,\"presence\":10},\"potential\":{\"strength\":10,\"dexterity\":10,\
             \"toughness\":10,\"intelligence\":17,\"willpower\":10,\"presence\":10}}"
        );
        let back: Stats = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(back, stats);
        // Every stat must be there, and only stats.
        let missing = json.replace("\"presence\":10}}", "\"willpower\":10}}");
        assert!(serde_json::from_str::<Stats>(&missing).is_err());
        let unknown = json.replacen("strength", "luck", 1);
        let err = serde_json::from_str::<Stats>(&unknown).unwrap_err();
        assert!(err.to_string().contains("unknown stat `luck`"), "{err}");
    }

    #[test]
    fn adjustments_default_to_zero_and_stay_in_range() {
        let dice = |t: &str| t.parse::<Dice>().expect("dice");
        let data = BTreeMap::from([
            (Stat::Strength, Dice::fixed(3)),
            (Stat::Toughness, dice("1d3+1")),
            (Stat::Presence, Dice::fixed(-10)),
        ]);
        let adj = adjustments(&data).expect("valid");
        assert_eq!(
            adj.iter().map(|(_, v)| v.to_string()).collect::<Vec<_>>(),
            ["+3", "0", "1d3+1", "0", "0", "-10"]
        );
        let err = adjustments(&BTreeMap::from([(Stat::Willpower, Dice::fixed(11))])).unwrap_err();
        assert!(err.contains("willpower +11 is out of range"), "{err}");
        assert!(adjustments(&BTreeMap::from([(Stat::Dexterity, Dice::fixed(-11))])).is_err());
        // Every result of a roll must be in range: 3d4 can be 12, -1d12 can be -12.
        let err = adjustments(&BTreeMap::from([(Stat::Strength, dice("3d4"))])).unwrap_err();
        assert!(err.contains("strength 3d4 is out of range"), "{err}");
        assert!(adjustments(&BTreeMap::from([(Stat::Strength, dice("-1d12"))])).is_err());
        assert!(adjustments(&BTreeMap::from([(Stat::Strength, dice("2d4+2"))])).is_ok());
        // Rolls follow the stat order; fixed numbers draw nothing.
        let mut rng = Rng::stream(1, "stats-test");
        let rolled = roll(&adj, &mut rng);
        assert_eq!((rolled[Stat::Strength], rolled[Stat::Presence]), (3, -10));
        assert!((2..=4).contains(&rolled[Stat::Toughness]));
        let mut again = Rng::stream(1, "stats-test");
        assert_eq!(rolled[Stat::Toughness], dice("1d3+1").roll(&mut again));
        assert_eq!(rng, again);
        // Adjusted scores stay within 1 to 40.
        assert_eq!(adjusted(10, 3), 13);
        assert_eq!(adjusted(3, -5), 1);
        assert_eq!(adjusted(38, 9), 40);
        // A potential below its score is raised to it.
        let stats = Stats::new(PerStat::splat(12), PerStat::splat(11));
        assert_eq!(stats.potential(Stat::Strength), 12);
    }

    #[test]
    fn data_reads_stat_tables_from_toml() {
        #[derive(Deserialize)]
        struct Spec {
            stats: BTreeMap<Stat, u8>,
        }
        let spec: Spec = toml::from_str("stats = { strength = 12, presence = 8 }").expect("toml");
        assert_eq!(
            spec.stats,
            BTreeMap::from([(Stat::Strength, 12), (Stat::Presence, 8)])
        );
        let err = toml::from_str::<Spec>("stats = { charisma = 12 }")
            .err()
            .expect("refused");
        assert!(err.to_string().contains("unknown stat `charisma`"), "{err}");
    }
}
