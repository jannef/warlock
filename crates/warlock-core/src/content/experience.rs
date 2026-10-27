//! Experience (`assets/data/experience.toml`, spec §8 "Experience"): the player's level
//! thresholds (a curve written in kills of the player's own level, scaled by their race's and
//! their class's experience factors), the hit points and mana a level gives, what a kill is worth
//! at the player's level (classic World of Warcraft scaling), and the small rewards for discovery
//! (the first kill of a creature type, the first arrival on a level). Without the file the player
//! stays at level 1, kills are scaled by a zero difference of 5, and nothing is discovered.

use super::{ClassDef, Content, ContentError, RaceDef};
use crate::character::CharacterSpec;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// The highest level a file may allow.
pub const MAX_LEVEL: u8 = 99;
/// The most XP a creature type may give.
pub const MAX_XP: u32 = 100_000;
/// The largest number a field of `[kills]` or `[discovery]` may hold.
const MAX_PARAM: u32 = 100_000;
/// What a race's or a class's `experience` may be, in percent of the XP a level needs.
pub const FACTORS: RangeInclusive<u16> = 50..=200;
/// The most a character's factor can scale a threshold: both factors at their highest, 400 %.
/// The curve's last threshold must fit a `u32` this many times over.
const MAX_SCALE: u64 = 4;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExperienceFile {
    max_level: u8,
    hp_level: u8,
    #[serde(default)]
    mana_level: u8,
    bonus_percent: u32,
    bonus_levels: u8,
    kills: Kills,
    #[serde(default)]
    early: Early,
    #[serde(default)]
    discovery: Discovery,
    #[serde(default)]
    levels: Levels,
    zero_difference: Vec<Band>,
}

/// The early levels' discount (`[early]`, owner 2026-10-27): level 2 costs `percent[0]` percent of
/// its curve XP, level 3 `percent[1]`, and so on; the levels after the list cost 100 %.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Early {
    #[serde(default)]
    percent: Vec<u8>,
}

/// What levels bring besides hit points (`[levels]`; spec §8 "Talents").
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Levels {
    /// Every this many levels (3: levels 3, 6, 9, ...) brings a talent pick; 0: never.
    #[serde(default)]
    pub talent_every: u8,
    /// Talent picks at creation for a character the player chose (a class may add more).
    #[serde(default)]
    pub creation_talents: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Band {
    from: u8,
    zd: u8,
}

/// The level curve in kills of the player's own level (`[kills]`): a level n (m = n − 1) costs
/// `plateau × m ÷ (m + ramp) + slope_per_100 ÷ 100 × m` such kills, each worth the
/// [`Guideline`] at that level.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Kills {
    /// What the first term ramps toward.
    pub plateau: u32,
    /// How many levels the first term takes to reach half its plateau.
    pub ramp: u32,
    /// The second term: hundredths of a kill more for each level.
    pub slope_per_100: u32,
    /// What a creature of a level is worth, which turns kills into XP.
    pub guideline: Guideline,
}

/// The creature XP guideline: a creature of level n is worth about `per_level × n + base` XP.
/// The curve is written in kills of creatures on it; new creature data should follow it.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Guideline {
    pub per_level: u32,
    pub base: u32,
}

impl Guideline {
    /// What a creature of `level` is worth by the guideline.
    pub fn xp(self, level: u8) -> u32 {
        self.per_level * u32::from(level) + self.base
    }
}

/// The rewards for discovery (`[discovery]`): neither draws anything at random.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Discovery {
    /// The player's first kill of a creature type gives this percent of the type's XP once more,
    /// whatever the levels.
    #[serde(default)]
    pub first_kill: u32,
    /// The player's first arrival on a level gives this times the level's item level.
    #[serde(default)]
    pub arrival: u32,
}

/// A character's experience factor (spec §8 "Experience"): their race's and their class's
/// `experience`, in percent, which scale every threshold (Angband's `expfact`). 100 % each
/// without a character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Factor {
    pub race: u16,
    pub class: u16,
}

impl Default for Factor {
    fn default() -> Self {
        Self {
            race: 100,
            class: 100,
        }
    }
}

impl Factor {
    /// The factor of a race and a class (100 % for either missing).
    pub fn new(race: Option<&RaceDef>, class: Option<&ClassDef>) -> Self {
        Self {
            race: race.map_or(100, |r| r.experience),
            class: class.map_or(100, |c| c.experience),
        }
    }

    /// The factor of the character `spec` (100 % each without one).
    pub fn of(content: &Content, spec: Option<&CharacterSpec>) -> Self {
        let race = spec.and_then(|s| s.race(content).ok());
        let class = spec.and_then(|s| s.class(content).ok());
        Self::new(race, class)
    }

    /// `xp` scaled by the factor, rounded: `(xp × race × class + 5 000) ÷ 10 000`.
    pub fn scale(self, xp: u32) -> u32 {
        let product = u64::from(self.race) * u64::from(self.class);
        let scaled = (u64::from(xp) * product + 5_000) / 10_000;
        u32::try_from(scaled).unwrap_or(u32::MAX)
    }

    /// The whole factor in percent, rounded: 175 for an elf wizard.
    pub fn percent(self) -> u32 {
        (u32::from(self.race) * u32::from(self.class) + 50) / 100
    }
}

/// How a race's or a class's `experience` reads when it is chosen (spec §6 "Character
/// creation"): "Levels come slowly." from 115 %, "Levels come quickly." to 95 %, else nothing.
pub fn pace_words(experience: u16) -> Option<&'static str> {
    match experience {
        115.. => Some("Levels come slowly."),
        ..=95 => Some("Levels come quickly."),
        _ => None,
    }
}

/// The experience rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Experience {
    max_level: u8,
    hp_level: u8,
    mana_level: u8,
    bonus_percent: u32,
    bonus_levels: u8,
    kills: Kills,
    /// The early levels' discount in percent, from level 2 on; later levels pay 100 %.
    early: Vec<u8>,
    discovery: Discovery,
    levels: Levels,
    /// `(from player level, zero difference)`, ascending, the first from level 1.
    zero_difference: Vec<(u8, u8)>,
    /// The XP total needed to reach each level at a factor of 100 %: index 0 is level 1 (0),
    /// the last `max_level`.
    thresholds: Vec<u32>,
}

impl Default for Experience {
    /// Without a file: no levels beyond the first, and no discovery.
    fn default() -> Self {
        Self {
            max_level: 1,
            hp_level: 0,
            mana_level: 0,
            bonus_percent: 5,
            bonus_levels: 4,
            kills: Kills::default(),
            early: Vec::new(),
            discovery: Discovery::default(),
            levels: Levels::default(),
            zero_difference: vec![(1, 5)],
            thresholds: vec![0],
        }
    }
}

/// The percent of its curve level `n` costs: the `[early]` discount's, else 100.
fn early_percent(early: &[u8], n: u8) -> u128 {
    usize::from(n)
        .checked_sub(2)
        .and_then(|i| early.get(i))
        .map_or(100, |&p| u128::from(p))
}

/// The kills of the player's own level that level `n` (2 or more) costs, as an exact fraction
/// `(numerator, denominator)` in hundredths of a kill:
/// `(plateau × 100 × m + slope × m × (m + ramp)) × percent ÷ ((m + ramp) × 100)` with m = n − 1
/// and percent the level's early discount (100 past it).
fn kills_fraction(kills: &Kills, early: &[u8], n: u8) -> (u128, u128) {
    let m = u128::from(n.saturating_sub(1));
    let over = m + u128::from(kills.ramp);
    let numerator =
        u128::from(kills.plateau) * 100 * m + u128::from(kills.slope_per_100) * m * over;
    (numerator * early_percent(early, n), over.max(1) * 100)
}

/// Rounds `numerator ÷ denominator` to the nearest whole number, halves up.
fn rounded(numerator: u128, denominator: u128) -> u128 {
    (2 * numerator + denominator) / (2 * denominator)
}

/// The XP level `n` (2 or more) costs at a factor of 100 %: its kills times the guideline at
/// `n`, rounded once, from the exact fraction (the early discount included).
fn level_xp(kills: &Kills, early: &[u8], n: u8) -> u128 {
    let (numerator, denominator) = kills_fraction(kills, early, n);
    let guideline = u128::from(kills.guideline.xp(n));
    rounded(numerator * guideline, denominator * 100)
}

impl Experience {
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let p: ExperienceFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if !(1..=MAX_LEVEL).contains(&p.max_level) {
            return Err(err(format!("max_level must be 1 to {MAX_LEVEL}")));
        }
        if p.hp_level > p.max_level {
            return Err(err("hp_level is more than max_level".into()));
        }
        if p.mana_level > p.max_level {
            return Err(err("mana_level is more than max_level".into()));
        }
        let k = &p.kills;
        let params = [
            k.plateau,
            k.ramp,
            k.slope_per_100,
            k.guideline.per_level,
            k.guideline.base,
            p.discovery.first_kill,
            p.discovery.arrival,
        ];
        if params.iter().any(|&v| v > MAX_PARAM) {
            return Err(err(format!(
                "every number in [kills] and [discovery] must be at most {MAX_PARAM}"
            )));
        }
        if p.bonus_percent > 100 {
            return Err(err("bonus_percent must be at most 100".into()));
        }
        if p.zero_difference.first().map(|b| b.from) != Some(1) {
            return Err(err("the first zero_difference must be from level 1".into()));
        }
        if (p.zero_difference.windows(2)).any(|pair| pair[1].from <= pair[0].from) {
            return Err(err("zero_difference `from` levels must ascend".into()));
        }
        if p.zero_difference.iter().any(|b| b.zd == 0) {
            return Err(err("zero_difference `zd` must be at least 1".into()));
        }
        let early = p.early.percent;
        if early.len() >= usize::from(p.max_level) {
            return Err(err(format!(
                "[early] percent lists more levels than 2 to max_level ({})",
                p.max_level
            )));
        }
        if early.iter().any(|&v| !(1..=100).contains(&v)) {
            return Err(err("[early] percent values must be 1 to 100".into()));
        }
        if early.windows(2).any(|pair| pair[1] < pair[0]) {
            return Err(err("[early] percent values must not decrease".into()));
        }
        let mut thresholds = vec![0u32];
        let mut total: u128 = 0;
        for n in 2..=p.max_level {
            let xp = level_xp(&p.kills, &early, n);
            if xp == 0 {
                return Err(err(format!("[kills] makes level {n} cost no XP")));
            }
            total += xp;
            if total * u128::from(MAX_SCALE) > u128::from(u32::MAX) {
                return Err(err(format!(
                    "[kills] makes the XP for level {n} too large to count"
                )));
            }
            thresholds.push(u32::try_from(total).unwrap_or(u32::MAX));
        }
        Ok(Self {
            max_level: p.max_level,
            hp_level: p.hp_level,
            mana_level: p.mana_level,
            bonus_percent: p.bonus_percent,
            bonus_levels: p.bonus_levels,
            kills: p.kills,
            early,
            discovery: p.discovery,
            levels: p.levels,
            zero_difference: p.zero_difference.iter().map(|b| (b.from, b.zd)).collect(),
            thresholds,
        })
    }

    /// The highest level there is.
    pub fn max_level(&self) -> u8 {
        self.max_level
    }

    /// The last level whose gain adds hit points.
    pub fn hp_level(&self) -> u8 {
        self.hp_level
    }

    /// The last level whose gain adds mana (spec §8 "Mana"); 0 when the file says none.
    pub fn mana_level(&self) -> u8 {
        self.mana_level
    }

    /// How many levels above the player a monster's worth still grows, and by how many percent
    /// per level.
    pub fn bonus(&self) -> (u8, u32) {
        (self.bonus_levels, self.bonus_percent)
    }

    /// The bands of the zero difference: `(from player level, zero difference)`, ascending.
    pub fn bands(&self) -> &[(u8, u8)] {
        &self.zero_difference
    }

    /// The creature XP guideline the curve is written in.
    pub fn guideline(&self) -> Guideline {
        self.kills.guideline
    }

    /// The rewards for discovery.
    pub fn discovery(&self) -> Discovery {
        self.discovery
    }

    /// Every this many levels brings a talent pick (spec §8 "Talents"); 0: never.
    pub fn talent_every(&self) -> u8 {
        self.levels.talent_every
    }

    /// The talent picks a character the player chose takes at creation, before the class's
    /// (spec §8 "Talents").
    pub fn creation_talents(&self) -> u8 {
        self.levels.creation_talents
    }

    /// The XP total needed to reach `level` at a factor of 100 % (0 for level 1 and below; the
    /// highest level's beyond it).
    pub fn threshold(&self, level: u8) -> u32 {
        let i = usize::from(level.saturating_sub(1));
        self.thresholds
            .get(i)
            .or(self.thresholds.last())
            .copied()
            .unwrap_or(0)
    }

    /// The XP total a character of `factor` needs to reach `level`.
    pub fn threshold_for(&self, level: u8, factor: Factor) -> u32 {
        factor.scale(self.threshold(level))
    }

    /// The XP level `level` itself costs at a factor of 100 % (0 for level 1).
    pub fn xp_for(&self, level: u8) -> u32 {
        self.threshold(level) - self.threshold(level.saturating_sub(1))
    }

    /// How many kills of creatures of the player's own level, each worth the guideline, level
    /// `level` costs (rounded; 0 for level 1).
    pub fn kills_for(&self, level: u8) -> u32 {
        if level < 2 {
            return 0;
        }
        let (numerator, denominator) = kills_fraction(&self.kills, &self.early, level);
        u32::try_from(rounded(numerator, denominator * 100)).unwrap_or(u32::MAX)
    }

    /// The level a character of `factor` with `xp` has reached.
    pub fn level_for(&self, xp: u32, factor: Factor) -> u8 {
        (1..=self.max_level)
            .take_while(|&l| self.threshold_for(l, factor) <= xp)
            .last()
            .unwrap_or(1)
    }

    /// The "zero difference" for a player of `level`: a monster this many levels below (or more)
    /// gives nothing.
    pub fn zero_difference(&self, level: u8) -> u8 {
        (self.zero_difference.iter())
            .rev()
            .find(|(from, _)| *from <= level)
            .map_or(5, |&(_, zd)| zd)
    }

    /// What killing a monster of level `monster` worth `xp` gives a player of level `player`:
    /// more above, less below, nothing when grey, otherwise at least 1. A monster worth nothing
    /// gives nothing.
    pub fn kill_xp(&self, xp: u32, monster: u8, player: u8) -> u32 {
        if xp == 0 {
            return 0;
        }
        // In u64: a creature's hand-set `xp_percent` has no ceiling.
        let wide = u64::from(xp);
        let scaled = if monster >= player {
            let over = u64::from((monster - player).min(self.bonus_levels));
            (wide * (100 + u64::from(self.bonus_percent) * over) + 50) / 100
        } else {
            let below = u64::from(player - monster);
            let zd = u64::from(self.zero_difference(player));
            if below >= zd {
                return 0;
            }
            ((wide * (zd - below) + zd / 2) / zd).max(1)
        };
        u32::try_from(scaled).unwrap_or(u32::MAX)
    }

    /// What the player's first kill of a creature type worth `xp` gives besides the kill's own
    /// XP: `first_kill` percent of it, rounded, unscaled by the levels.
    pub fn first_kill_xp(&self, xp: u32) -> u32 {
        let scaled = (u64::from(xp) * u64::from(self.discovery.first_kill) + 50) / 100;
        u32::try_from(scaled).unwrap_or(u32::MAX)
    }

    /// What the player's first arrival on a level of item level `item_level` gives.
    pub fn arrival_xp(&self, item_level: u8) -> u32 {
        self.discovery.arrival * u32::from(item_level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHIPPED: &str = include_str!("../../../../assets/data/experience.toml");

    fn builtin() -> Experience {
        Experience::parse("experience.toml", SHIPPED).expect("parses")
    }

    fn factor(race: u16, class: u16) -> Factor {
        Factor { race, class }
    }

    #[test]
    fn the_curve_is_written_in_kills() {
        let x = builtin();
        assert_eq!(x.max_level(), 60);
        // Level 2: 213 × 1/6 + 2.85 = 38.35 kills of level-2 creatures worth 20 (767 XP), at
        // the early discount's 25 %: 9.5875 kills, 191.75 XP.
        assert_eq!(
            (x.kills_for(2), x.xp_for(2), x.threshold(2)),
            (10, 192, 192)
        );
        assert_eq!(
            (x.kills_for(3), x.xp_for(3), x.threshold(3)),
            (20, 499, 691)
        );
        assert_eq!((x.kills_for(10), x.threshold(10)), (146, 29_715));
        assert_eq!((x.kills_for(20), x.threshold(20)), (223, 205_205));
        assert_eq!((x.kills_for(40), x.kills_for(60)), (300, 365));
        assert_eq!(x.threshold(60), 2_845_856);
        assert_eq!((x.threshold(1), x.threshold(0), x.xp_for(1)), (0, 0, 0));
        assert_eq!(x.threshold(61), 2_845_856, "past the cap: the cap's");
        // About 2 600 even-level kills to level 20, 14 600 to 60 (3 000 and 15 000 before the
        // early discount).
        let kills = |to: u8| (2..=to).map(|l| x.kills_for(l)).sum::<u32>();
        assert!((2_550..=2_650).contains(&kills(20)), "{}", kills(20));
        assert!((14_500..=14_700).contains(&kills(60)), "{}", kills(60));
        // Every level costs more than the last, in XP and in kills.
        for l in 3..=60 {
            assert!(x.xp_for(l) > x.xp_for(l - 1), "level {l}");
            assert!(x.kills_for(l) >= x.kills_for(l - 1), "level {l}");
        }
    }

    #[test]
    fn early_levels_are_discounted() {
        // The owner's numbers (2026-10-27): the XP to reach levels 2 to 12.
        let x = builtin();
        let expected = [
            192, 691, 1_619, 3_104, 5_519, 9_118, 14_165, 20_936, 29_715, 40_243, 52_688,
        ];
        let got: Vec<u32> = (2..=12).map(|l| x.threshold(l)).collect();
        assert_eq!(got, expected);
        // From level 12 on a level costs what it would without the discount.
        let plain = Experience::parse(
            "x",
            &SHIPPED.replace(
                "percent = [25, 30, 35, 40, 50, 60, 70, 80, 90, 95]",
                "percent = []",
            ),
        )
        .expect("parses");
        assert_eq!((plain.threshold(2), plain.threshold(10)), (767, 45_052));
        for l in 12..=60 {
            assert_eq!(x.xp_for(l), plain.xp_for(l), "level {l}");
            assert_eq!(x.kills_for(l), plain.kills_for(l), "level {l}");
        }
        for l in 2..=11 {
            assert!(x.xp_for(l) < plain.xp_for(l), "level {l}");
        }
        assert_eq!(plain.threshold(60) - x.threshold(60), 15_892);
    }

    #[test]
    fn the_table_is_pinned() {
        // FNV-1a over every threshold: integer math, the same on every platform. A deliberate
        // change to the curve updates this value.
        let x = builtin();
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for level in 1..=x.max_level() {
            for byte in x.threshold(level).to_le_bytes() {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        assert_eq!(
            hash, 0xf8c7_abaa_9ae1_32e9,
            "experience table hash {hash:#018x}"
        );
    }

    #[test]
    fn factors_scale_the_thresholds() {
        let x = builtin();
        let at = |f: Factor| [2, 10, 60].map(|l| x.threshold_for(l, f));
        assert_eq!(at(Factor::default()), [192, 29_715, 2_845_856]);
        assert_eq!(at(factor(140, 125)), [336, 52_001, 4_980_248], "elf wizard");
        assert_eq!(
            at(factor(90, 80)),
            [138, 21_395, 2_049_016],
            "half-orc commoner"
        );
        assert_eq!(factor(140, 125).percent(), 175);
        assert_eq!(factor(90, 80).percent(), 72);
        assert_eq!(factor(115, 115).percent(), 132);
        // The level follows the character's own thresholds.
        let wizard = factor(140, 125);
        assert_eq!(x.level_for(335, wizard), 1);
        assert_eq!(x.level_for(336, wizard), 2);
        assert_eq!(x.level_for(497, factor(90, 80)), 2, "691 × 72 %: 498");
        assert_eq!(x.level_for(498, factor(90, 80)), 3);
        // The steepest factor still counts to the top.
        let steepest = factor(*FACTORS.end(), *FACTORS.end());
        assert_eq!(x.level_for(u32::MAX, steepest), 60);
        assert_eq!(x.threshold_for(60, steepest), 4 * 2_845_856);
    }

    #[test]
    fn level_follows_xp() {
        let x = builtin();
        let normal = Factor::default();
        assert_eq!(x.level_for(0, normal), 1);
        assert_eq!(x.level_for(191, normal), 1);
        assert_eq!(x.level_for(192, normal), 2);
        assert_eq!(x.level_for(690, normal), 2);
        assert_eq!(x.level_for(691, normal), 3);
        assert_eq!(x.level_for(2_845_855, normal), 59);
        assert_eq!(x.level_for(2_845_856, normal), 60);
        assert_eq!(x.level_for(u32::MAX, normal), 60, "capped");
    }

    #[test]
    fn pace_words_say_slow_and_quick_factors() {
        assert_eq!(pace_words(140), Some("Levels come slowly."));
        assert_eq!(pace_words(115), Some("Levels come slowly."));
        assert_eq!(pace_words(110), None);
        assert_eq!(pace_words(100), None);
        assert_eq!(pace_words(96), None);
        assert_eq!(pace_words(95), Some("Levels come quickly."));
        assert_eq!(pace_words(80), Some("Levels come quickly."));
    }

    #[test]
    fn discovery_rewards() {
        let x = builtin();
        assert_eq!(x.first_kill_xp(15), 15, "a kobold's XP once more");
        assert_eq!(x.first_kill_xp(0), 0);
        assert_eq!(x.arrival_xp(1), 5, "a hand-made level");
        assert_eq!(x.arrival_xp(4), 20, "the fourth level of a dungeon");
        let none = Experience::parse("x", &SHIPPED.replace("[discovery]", "[nothing]"));
        assert!(none.is_err(), "unknown tables are refused");
        let start = SHIPPED.find("\n[discovery]").expect("discovery");
        let end = SHIPPED.find("# ZD from").expect("bands");
        let without = format!("{}{}", &SHIPPED[..start], &SHIPPED[end..]);
        let x = Experience::parse("x", &without).expect("discovery is optional");
        assert_eq!((x.first_kill_xp(15), x.arrival_xp(4)), (0, 0));
        assert_eq!(Experience::default().discovery(), Discovery::default());
    }

    #[test]
    fn zero_difference_bands() {
        let x = builtin();
        let zd = |l| x.zero_difference(l);
        assert_eq!(
            [zd(1), zd(7), zd(8), zd(9), zd(10), zd(11)],
            [5, 5, 6, 6, 7, 7]
        );
        assert_eq!(
            [zd(12), zd(15), zd(16), zd(19), zd(20), zd(29)],
            [8, 8, 9, 9, 11, 11]
        );
        assert_eq!([zd(30), zd(40), zd(45), zd(49)], [12, 13, 14, 14]);
        assert_eq!(
            [zd(50), zd(54), zd(55), zd(59), zd(60)],
            [15, 15, 16, 16, 17]
        );
    }

    #[test]
    fn scaling_below_the_player() {
        let x = builtin();
        // A 40 XP kobold shaman of level 4 (ZD 5 up to level 7, 6 at level 8).
        assert_eq!(x.kill_xp(40, 4, 4), 40, "equal level: all of it");
        assert_eq!(x.kill_xp(40, 4, 5), 32, "one below: 4/5");
        assert_eq!(x.kill_xp(40, 4, 6), 24);
        assert_eq!(x.kill_xp(40, 4, 7), 16);
        assert_eq!(x.kill_xp(40, 4, 8), 13, "four below with ZD 6: 2/6");
    }

    #[test]
    fn grey_monsters_give_nothing() {
        let x = builtin();
        assert_eq!(x.kill_xp(5, 1, 6), 0, "five below at ZD 5");
        assert_eq!(x.kill_xp(500, 1, 9), 0, "eight below at ZD 6");
        assert_eq!(x.kill_xp(5, 1, 5), 1, "four below, not grey: at least 1");
        assert_eq!(x.kill_xp(100, 10, 16), 33, "six below at ZD 9 is not grey");
        assert_eq!(x.kill_xp(100, 10, 19), 0, "nine below at ZD 9");
        assert_eq!(
            x.kill_xp(100, 44, 60),
            6,
            "sixteen below at ZD 17 is not grey"
        );
        assert_eq!(x.kill_xp(100, 43, 60), 0, "seventeen below at ZD 17");
    }

    #[test]
    fn higher_monsters_give_a_bonus_up_to_four_levels() {
        let x = builtin();
        assert_eq!(x.kill_xp(100, 2, 1), 105);
        assert_eq!(x.kill_xp(100, 3, 1), 110);
        assert_eq!(x.kill_xp(100, 5, 1), 120);
        assert_eq!(x.kill_xp(100, 20, 1), 120, "capped at +20%");
        assert_eq!(x.kill_xp(3, 5, 1), 4, "rounded");
    }

    #[test]
    fn a_creature_worth_nothing_gives_nothing() {
        let x = builtin();
        assert_eq!(x.kill_xp(0, 5, 1), 0);
        assert_eq!(x.kill_xp(0, 1, 1), 0);
    }

    #[test]
    fn creatures_follow_the_guideline() {
        // New creature data should be worth about the guideline's XP at its level: a creature
        // more than three times off it either way is a mistake, unless it is one of these
        // deliberate outliers (the vermin, worth less; the keepers, not meant to be fought).
        const OUTLIERS: &[&str] = &["bat", "deer", "duergar_trader", "shopkeeper"];
        let content = Content::builtin().expect("content");
        let guideline = content.experience().guideline();
        assert_eq!(
            guideline,
            Guideline {
                per_level: 5,
                base: 10
            }
        );
        for c in content.creature_defs().iter().filter(|c| c.xp > 0) {
            let by = guideline.xp(c.level);
            let off = c.xp * 3 < by || c.xp > by * 3;
            let listed = OUTLIERS.contains(&c.id.as_str());
            assert_eq!(
                off,
                listed,
                "{} (level {}, {} XP; the guideline says {by}): {}",
                c.id,
                c.level,
                c.xp,
                if off {
                    "more than three times off the guideline"
                } else {
                    "on the guideline now: take it off the outliers"
                }
            );
        }
    }

    #[test]
    fn bad_files_are_refused() {
        for (from, to) in [
            ("max_level = 60", "max_level = 0"),
            ("hp_level = 20", "hp_level = 61"),
            ("mana_level = 60", "mana_level = 61"),
            ("from = 1\nzd = 5", "from = 2\nzd = 5"),
            ("zd = 5", "zd = 0"),
            ("plateau = 213", "plateau = 100001"),
            ("plateau = 213\nramp = 5\nslope_per_100 = 285", "ramp = 5"),
            (
                "plateau = 213\nramp = 5\nslope_per_100 = 285",
                "plateau = 0\nramp = 5\nslope_per_100 = 0",
            ),
            ("bonus_levels = 4", "bonus_levels = 4\nlinear = 10"),
            ("arrival = 5", "arrival = 5\nidentify = 10"),
            ("[25, 30,", "[0, 30,"),
            ("[25, 30,", "[101, 30,"),
            ("[25, 30,", "[35, 30,"),
            ("percent = [25,", "speed = 2\npercent = [25,"),
        ] {
            let text = SHIPPED.replace(from, to);
            assert_ne!(text, SHIPPED, "{from} is in the file");
            assert!(Experience::parse("x", &text).is_err(), "{to}");
        }
        // The early discount may cover levels 2 to max_level, no more.
        let capped = |max: u8| {
            SHIPPED
                .replace("max_level = 60", &format!("max_level = {max}"))
                .replace("hp_level = 20", "hp_level = 1")
                .replace("mana_level = 60", "mana_level = 1")
        };
        assert!(Experience::parse("x", &capped(11)).is_ok());
        assert!(Experience::parse("x", &capped(10)).is_err());
        // A curve too steep to count at the top is refused, not wrapped.
        let steep = SHIPPED
            .replace("max_level = 60", "max_level = 99")
            .replace("plateau = 213", "plateau = 100000");
        assert!(Experience::parse("x", &steep).is_err());
    }

    #[test]
    fn mana_comes_with_levels_up_to_mana_level() {
        // Mana grows to the last level, 60, for tier 9's comfortable caster level (spec §8
        // "Spells").
        assert_eq!(
            Experience::parse("x", SHIPPED).expect("valid").mana_level(),
            60
        );
        let none = SHIPPED.replace("mana_level = 60\n", "");
        assert_ne!(none, SHIPPED, "mana_level is in the file");
        assert_eq!(
            Experience::parse("x", &none).expect("valid").mana_level(),
            0
        );
        let some = format!("mana_level = 3\n{none}");
        assert_eq!(
            Experience::parse("x", &some).expect("valid").mana_level(),
            3
        );
        let bad = format!("mana_level = 61\n{none}");
        let e = Experience::parse("x", &bad).unwrap_err();
        assert!(
            e.message.contains("mana_level is more than max_level"),
            "{}",
            e.message
        );
    }
}
