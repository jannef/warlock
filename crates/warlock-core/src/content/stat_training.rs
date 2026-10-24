//! Stat training (`assets/data/stats.toml`, spec §8 "Stats"; the design is
//! `docs/design/progression.md` §6): uses of the player's stats earn training marks, and at
//! every level gained each stat gets one check to rise by a point, its chance set by its marks.
//!
//! A use ([`StatUse`]) is reported where it happens (`World::train` for the uses skills share,
//! `World::train_stat` for the rest); each `[[training.use]]` rule for it gives its stat a mark
//! with the rule's chance. At a level, a stat rises with chance `marks / (marks + need)`, `need
//! = need_base + need_per_point × score`, divided by `over_potential_divisor` at or above the
//! potential ([`StatTraining::odds`]).
//!
//! Without the file nothing trains and no level checks a stat ([`StatTraining::default`]).

use super::ContentError;
use crate::stats::Stat;
use serde::Deserialize;
use std::ops::RangeInclusive;

/// What `need_base` and `need_per_point` may be.
pub const NEEDS: RangeInclusive<u16> = 0..=200;
/// What `over_potential_divisor` may be.
pub const DIVISORS: RangeInclusive<u16> = 1..=20;

/// A use of the player's stats that may earn a training mark (spec §8 "Stats"), reported at
/// the one place the thing happens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatUse {
    /// An action other than waiting while the load slows the player (Balance's use).
    Burdened,
    /// A melee blow of the player's that hits (a kick, a bash too).
    Blow,
    /// A turn of digging.
    Dig,
    /// A kick at a closed door (Athletics' use).
    Kick,
    /// An attack on the player that missed because of their Dexterity or Dodge (Tumble's use).
    Dodge,
    /// A shot or a throw of the player's that hits.
    Shot,
    /// A step within earshot of a creature that could hear it (Stealth's use).
    Sneak,
    /// Climbing out of the water onto ice, or out of a pit.
    Climb,
    /// A blow that costs the player hit points.
    Hurt,
    /// A big hit or a confirmed critical on the player.
    BigHit,
    /// A check against a disease won, catching it or as it progresses (not a Willpower one).
    Resisted,
    /// Falling through the ice.
    Plunge,
    /// A search that finds something (a secret door, a trap).
    Found,
    /// A scroll read.
    Read,
    /// An item identified by reading.
    Identified,
    /// The first kill of a creature type.
    Discovered,
    /// Noticing something hidden in passing (Spot's use).
    Spot,
    /// A long action of more than one turn done to its end: a rest, a wall dug through, a
    /// search for a while, a meal.
    Endured,
    /// A Willpower check against a disease won (brain fever).
    ResistedWill,
    /// A blow of the player's that lands while they are Hungry or worse.
    HungryFight,
    /// A chat (asking about a topic takes no time, so it trains nothing).
    Chat,
    /// A purchase paid for or a sale made.
    Trade,
    /// A quest completed.
    Quest,
    /// A creature making way for the player.
    Swapped,
}

impl StatUse {
    /// The use a skill use is for stats too, if any: the uses the two share (one report,
    /// `World::train`).
    pub const fn of_skill(used: crate::skills::Use) -> Option<StatUse> {
        use crate::skills::Use;
        match used {
            Use::Burdened => Some(StatUse::Burdened),
            Use::Kick => Some(StatUse::Kick),
            Use::Dodge => Some(StatUse::Dodge),
            Use::Sneak => Some(StatUse::Sneak),
            Use::Spot => Some(StatUse::Spot),
            _ => None,
        }
    }

    /// How the wiki says it.
    pub const fn describe(self) -> &'static str {
        match self {
            StatUse::Burdened => "acting (not waiting) while your load slows you",
            StatUse::Blow => "a blow of yours that hits up close (a kick or a shield bash too)",
            StatUse::Dig => "a turn of digging",
            StatUse::Kick => "kicking at a closed door",
            StatUse::Dodge => "an attack on you that your Dexterity or Dodge made miss",
            StatUse::Shot => "a shot or a throw of yours that hits",
            StatUse::Sneak => "a step within earshot of a creature that could hear it",
            StatUse::Climb => "climbing out of the water onto ice, or out of a pit",
            StatUse::Hurt => "a blow that costs you hit points",
            StatUse::BigHit => "a big hit or a critical on you",
            StatUse::Resisted => "fighting off a disease (a check won, other than a Willpower one)",
            StatUse::Plunge => "falling through the ice",
            StatUse::Found => "finding something hidden by searching",
            StatUse::Read => "reading a scroll",
            StatUse::Identified => "identifying an item",
            StatUse::Discovered => "your first kill of a kind of creature",
            StatUse::Spot => "noticing something hidden in passing",
            StatUse::Endured => {
                "a long action of more than one turn done to its end (a rest, a wall dug \
                 through, a search for a while, a meal)"
            }
            StatUse::ResistedWill => "fighting off a disease with Willpower (brain fever)",
            StatUse::HungryFight => "a blow of yours that lands while you are Hungry or worse",
            StatUse::Chat => "chatting with someone",
            StatUse::Trade => "paying for goods, or selling",
            StatUse::Quest => "completing a quest",
            StatUse::Swapped => "a creature making way for you",
        }
    }
}

/// When a stat earns a training mark: on each `on`, with `chance` percent (1 to 100).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatRule {
    pub stat: Stat,
    pub on: StatUse,
    pub chance: u8,
}

/// The rules of stat training.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatTraining {
    /// What a stat's need starts at: 20.
    pub need_base: u16,
    /// What each point of the score adds to its need: 3.
    pub need_per_point: u16,
    /// What the chance is divided by at or above the potential: 4.
    pub over_potential_divisor: u16,
    /// The mark rules, in file order (the order the training stream is drawn in).
    #[serde(default, rename = "use")]
    pub uses: Vec<StatRule>,
}

impl Default for StatTraining {
    /// No rules: nothing trains, and no level checks a stat.
    fn default() -> Self {
        Self {
            need_base: 20,
            need_per_point: 3,
            over_potential_divisor: 4,
            uses: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    training: StatTraining,
}

impl StatTraining {
    /// Parses `data/stats.toml`.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let File { training } = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for (name, value, range) in [
            ("need_base", training.need_base, NEEDS),
            ("need_per_point", training.need_per_point, NEEDS),
            (
                "over_potential_divisor",
                training.over_potential_divisor,
                DIVISORS,
            ),
        ] {
            if !range.contains(&value) {
                return Err(err(format!(
                    "{name} {value} is out of range ({} to {})",
                    range.start(),
                    range.end()
                )));
            }
        }
        if training.need_base == 0 && training.need_per_point == 0 {
            return Err(err("need_base and need_per_point may not both be 0".into()));
        }
        for rule in &training.uses {
            if !(1..=100).contains(&rule.chance) {
                return Err(err(format!(
                    "{} on {:?}: chance {} is out of range (1 to 100)",
                    rule.stat.id(),
                    rule.on,
                    rule.chance
                )));
            }
        }
        for (i, rule) in training.uses.iter().enumerate() {
            if (training.uses[..i].iter()).any(|r| r.stat == rule.stat && r.on == rule.on) {
                return Err(err(format!(
                    "{} on {:?} is listed twice",
                    rule.stat.id(),
                    rule.on
                )));
            }
        }
        Ok(training)
    }

    /// Whether anything trains at all: without rules no level checks a stat either.
    pub fn is_active(&self) -> bool {
        !self.uses.is_empty()
    }

    /// The rules for `used`, in file order.
    pub fn rules(&self, used: StatUse) -> impl Iterator<Item = &StatRule> {
        self.uses.iter().filter(move |r| r.on == used)
    }

    /// What a stat at `score` needs: `need_base + need_per_point × score`.
    pub fn need(&self, score: u8) -> u32 {
        u32::from(self.need_base) + u32::from(self.need_per_point) * u32::from(score)
    }

    /// The chance a stat at `score` with `potential` and `marks` rises at a level, as
    /// `(numerator, denominator)`: `marks / (marks + need)`, the denominator multiplied by
    /// `over_potential_divisor` at or above the potential.
    pub fn odds(&self, marks: u16, score: u8, potential: u8) -> (u32, u32) {
        let mut out = u32::from(marks) + self.need(score);
        if score >= potential {
            out *= u32::from(self.over_potential_divisor);
        }
        (u32::from(marks), out.max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<StatTraining, ContentError> {
        StatTraining::parse("data/stats.toml", text)
    }

    const HEAD: &str =
        "[training]\nneed_base = 20\nneed_per_point = 3\nover_potential_divisor = 4\n";

    #[test]
    fn the_odds_follow_the_marks_the_score_and_the_potential() {
        let t = parse(HEAD).unwrap();
        assert_eq!(t.need(10), 50);
        assert_eq!(t.need(14), 62);
        assert_eq!(t.odds(20, 14, 16), (20, 82));
        assert_eq!(t.odds(0, 14, 16), (0, 62));
        // At or above the potential, a quarter of the chance.
        assert_eq!(t.odds(20, 14, 14), (20, 328));
        assert!(!t.is_active());
    }

    #[test]
    fn rules_are_validated() {
        let rule = |chance: u32| {
            format!(
                "{HEAD}[[training.use]]\nstat = \"strength\"\non = \"blow\"\nchance = {chance}\n"
            )
        };
        let t = parse(&rule(5)).unwrap();
        assert!(t.is_active());
        assert_eq!(t.rules(StatUse::Blow).count(), 1);
        assert_eq!(t.rules(StatUse::Dig).count(), 0);
        assert!(
            parse(&rule(0))
                .unwrap_err()
                .to_string()
                .contains("chance 0")
        );
        assert!(parse(&rule(101)).is_err());
        let twice = format!("{}{}", rule(5), &rule(5)[HEAD.len()..]);
        assert!(parse(&twice).unwrap_err().to_string().contains("twice"));
        let bad = HEAD.replace("divisor = 4", "divisor = 0");
        assert!(
            parse(&bad)
                .unwrap_err()
                .to_string()
                .contains("over_potential_divisor")
        );
        let unknown =
            format!("{HEAD}[[training.use]]\nstat = \"luck\"\non = \"blow\"\nchance = 5\n");
        assert!(parse(&unknown).is_err());
    }
}
