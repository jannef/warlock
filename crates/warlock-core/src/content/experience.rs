//! Experience (`assets/data/experience.toml`, spec §8 "Experience"): the player's level
//! thresholds, the hit points and mana a level gives, and what a kill is worth at the player's level
//! (classic World of Warcraft scaling). Without the file the player stays at level 1 and kills
//! are scaled by a zero difference of 5.

use super::ContentError;
use serde::Deserialize;

/// The highest level a file may allow.
pub const MAX_LEVEL: u8 = 99;
/// The most XP a creature type may give.
pub const MAX_XP: u32 = 100_000;
/// The largest `linear` or `quadratic` a file may give.
const MAX_STEP: u32 = 100_000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExperienceFile {
    max_level: u8,
    linear: u32,
    quadratic: u32,
    hp_level: u8,
    #[serde(default)]
    mana_level: u8,
    bonus_percent: u32,
    bonus_levels: u8,
    zero_difference: Vec<Band>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Band {
    from: u8,
    zd: u8,
}

/// The experience rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Experience {
    max_level: u8,
    linear: u32,
    quadratic: u32,
    hp_level: u8,
    mana_level: u8,
    bonus_percent: u32,
    bonus_levels: u8,
    /// `(from player level, zero difference)`, ascending, the first from level 1.
    zero_difference: Vec<(u8, u8)>,
}

impl Default for Experience {
    /// Without a file: no levels beyond the first.
    fn default() -> Self {
        Self {
            max_level: 1,
            linear: 10,
            quadratic: 10,
            hp_level: 0,
            mana_level: 0,
            bonus_percent: 5,
            bonus_levels: 4,
            zero_difference: vec![(1, 5)],
        }
    }
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
        if p.linear > MAX_STEP || p.quadratic > MAX_STEP {
            return Err(err(format!(
                "`linear` and `quadratic` must be at most {MAX_STEP}"
            )));
        }
        if p.max_level > 1 && p.linear + p.quadratic == 0 {
            return Err(err("`linear` and `quadratic` must not both be 0".into()));
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
        Ok(Self {
            max_level: p.max_level,
            linear: p.linear,
            quadratic: p.quadratic,
            hp_level: p.hp_level,
            mana_level: p.mana_level,
            bonus_percent: p.bonus_percent,
            bonus_levels: p.bonus_levels,
            zero_difference: p.zero_difference.iter().map(|b| (b.from, b.zd)).collect(),
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

    /// The XP total needed to reach `level` (0 for level 1 and below).
    pub fn threshold(&self, level: u8) -> u32 {
        let n = u32::from(level.saturating_sub(1));
        self.linear * n + self.quadratic * n * n
    }

    /// The level a player with `xp` has reached.
    pub fn level_for(&self, xp: u32) -> u8 {
        (1..=self.max_level)
            .take_while(|&l| self.threshold(l) <= xp)
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
        if monster >= player {
            let over = u32::from((monster - player).min(self.bonus_levels));
            return (xp * (100 + self.bonus_percent * over) + 50) / 100;
        }
        let below = u32::from(player - monster);
        let zd = u32::from(self.zero_difference(player));
        if below >= zd {
            return 0;
        }
        ((xp * (zd - below) + zd / 2) / zd).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builtin() -> Experience {
        let text = include_str!("../../../../assets/data/experience.toml");
        Experience::parse("experience.toml", text).expect("parses")
    }

    #[test]
    fn thresholds_are_quadratic() {
        let x = builtin();
        let t: Vec<u32> = (1..=5).map(|l| x.threshold(l)).collect();
        assert_eq!(t, [0, 20, 60, 120, 200]);
        assert_eq!(x.threshold(10), 900);
        assert_eq!(x.threshold(50), 24_500);
        assert_eq!(x.max_level(), 50);
    }

    #[test]
    fn level_follows_xp() {
        let x = builtin();
        assert_eq!(x.level_for(0), 1);
        assert_eq!(x.level_for(19), 1);
        assert_eq!(x.level_for(20), 2);
        assert_eq!(x.level_for(119), 3);
        assert_eq!(x.level_for(120), 4);
        assert_eq!(x.level_for(24_499), 49);
        assert_eq!(x.level_for(24_500), 50);
        assert_eq!(x.level_for(u32::MAX), 50, "capped");
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
        assert_eq!([zd(30), zd(40), zd(45), zd(50)], [12, 13, 14, 14]);
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
    fn bad_files_are_refused() {
        let ok = include_str!("../../../../assets/data/experience.toml");
        for (from, to) in [
            ("max_level = 50", "max_level = 0"),
            ("hp_level = 20", "hp_level = 60"),
            ("mana_level = 30", "mana_level = 60"),
            ("from = 1\nzd = 5", "from = 2\nzd = 5"),
            ("zd = 5", "zd = 0"),
        ] {
            assert!(
                Experience::parse("x", &ok.replace(from, to)).is_err(),
                "{to}"
            );
        }
    }

    #[test]
    fn mana_comes_with_levels_up_to_mana_level() {
        let ok = include_str!("../../../../assets/data/experience.toml");
        let none = ok.replace("mana_level = 30\n", "");
        assert_eq!(
            Experience::parse("x", &none).expect("valid").mana_level(),
            0
        );
        let some = format!("mana_level = 3\n{none}");
        assert_eq!(
            Experience::parse("x", &some).expect("valid").mana_level(),
            3
        );
        let bad = format!("mana_level = 51\n{none}");
        let e = Experience::parse("x", &bad).unwrap_err();
        assert!(
            e.message.contains("mana_level is more than max_level"),
            "{}",
            e.message
        );
    }
}
