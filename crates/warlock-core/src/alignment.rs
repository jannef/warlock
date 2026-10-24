//! Alignment (spec §8 "Alignment", #254): good against evil, ADOM's law and chaos turned. The
//! player has a hidden [`Score`] from −[`MAX`] (as evil as can be) to +[`MAX`] (as good); what
//! they do moves it ([`Score::shifted`]: the further along a direction they already are, the less
//! each deed that way moves them), and they know it only as one of seven [`Band`]s, by edges in
//! `data/alignment.toml` (`content/alignment.rs`). Creatures have a fixed [`Alignment`] instead:
//! their type's, or the one a level gave them.
//!
//! What reads it reads a band: dialogue and quests ([`Band::scorned_by`]), shop prices, and
//! abilities that must be used by someone of the right alignment ([`Requirement`]). The score
//! itself is never shown, and a creature's alignment neither.

use serde::{Deserialize, Serialize};

/// The far end of the score either way: −1000 is as evil as can be, 1000 as good.
pub const MAX: i32 = 1000;

/// A character's alignment score, −[`MAX`] to [`MAX`] (saved; loading refuses one out of range).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "i32", into = "i32")]
pub struct Score(i32);

impl Score {
    /// Neutral: neither good nor evil.
    pub const ZERO: Score = Score(0);

    /// `value` kept within −[`MAX`] to [`MAX`].
    pub fn new(value: i32) -> Score {
        Score(value.clamp(-MAX, MAX))
    }

    pub const fn value(self) -> i32 {
        self.0
    }

    /// The score after a deed worth `amount` (good if positive): a deed in the direction the
    /// score already leans (away from 0) moves it by `amount` times the share of the way still
    /// left to that end, rounded toward 0, so small deeds never make a saint; a deed the other
    /// way, or from 0, moves it by all of `amount`. Kept within the scale.
    pub fn shifted(self, amount: i32) -> Score {
        let s = self.0;
        let further = (amount > 0 && s > 0) || (amount < 0 && s < 0);
        let moved = if further {
            amount * (MAX - s.abs()) / MAX
        } else {
            amount
        };
        Score::new(s + moved)
    }
}

impl TryFrom<i32> for Score {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if (-MAX..=MAX).contains(&value) {
            Ok(Score(value))
        } else {
            Err(format!(
                "an alignment score is -{MAX} to {MAX}, not {value}"
            ))
        }
    }
}

impl From<Score> for i32 {
    fn from(score: Score) -> i32 {
        score.0
    }
}

/// Checks a race's or a class's part of a new character's score (`alignment` in
/// `data/races.toml` and `data/classes/`): −[`MAX`] to [`MAX`]; the sum is kept on the scale.
pub fn check_start(value: i32) -> Result<i32, String> {
    if (-MAX..=MAX).contains(&value) {
        Ok(value)
    } else {
        Err(format!(
            "alignment {value} is out of range (-{MAX} to {MAX})"
        ))
    }
}

/// Where a creature stands, fixed by its type (`alignment` in its file) or its level (a legend
/// entry's `alignment`). The player's comes from their score's band ([`Band::side`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    Good,
    #[default]
    Neutral,
    Evil,
}

impl Alignment {
    /// Every alignment, good first.
    pub const ALL: [Alignment; 3] = [Alignment::Good, Alignment::Neutral, Alignment::Evil];

    /// Its id in data files and dumps: `good`.
    pub const fn id(self) -> &'static str {
        match self {
            Alignment::Good => "good",
            Alignment::Neutral => "neutral",
            Alignment::Evil => "evil",
        }
    }

    /// The band a creature of this alignment counts as, where a band is asked for: Good,
    /// Neutral, or Evil.
    pub const fn band(self) -> Band {
        match self {
            Alignment::Good => Band::Good,
            Alignment::Neutral => Band::Neutral,
            Alignment::Evil => Band::Evil,
        }
    }
}

/// How good or evil a character is, as they know it: the band their score falls in. Ordered
/// from the most evil up, so `>=` is "as good or better".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Band {
    Wicked,
    Evil,
    Callous,
    Neutral,
    Kind,
    Good,
    Saintly,
}

impl Band {
    /// Every band, the best first (the order of `data/alignment.toml`).
    pub const ALL: [Band; 7] = [
        Band::Saintly,
        Band::Good,
        Band::Kind,
        Band::Neutral,
        Band::Callous,
        Band::Evil,
        Band::Wicked,
    ];

    /// Its name, as the character screen shows it: `Callous`.
    pub const fn name(self) -> &'static str {
        match self {
            Band::Wicked => "Wicked",
            Band::Evil => "Evil",
            Band::Callous => "Callous",
            Band::Neutral => "Neutral",
            Band::Kind => "Kind",
            Band::Good => "Good",
            Band::Saintly => "Saintly",
        }
    }

    /// Its id in data files and dumps: `callous`.
    pub const fn id(self) -> &'static str {
        match self {
            Band::Wicked => "wicked",
            Band::Evil => "evil",
            Band::Callous => "callous",
            Band::Neutral => "neutral",
            Band::Kind => "kind",
            Band::Good => "good",
            Band::Saintly => "saintly",
        }
    }

    /// The side of good and evil it is on: Saintly, Good, and Kind are good; Callous, Evil,
    /// and Wicked evil.
    pub const fn side(self) -> Alignment {
        match self {
            Band::Saintly | Band::Good | Band::Kind => Alignment::Good,
            Band::Neutral => Alignment::Neutral,
            Band::Callous | Band::Evil | Band::Wicked => Alignment::Evil,
        }
    }

    /// Whether someone of alignment `them` turns cold toward a character of this band (spec §8
    /// "Alignment"): a good creature toward an Evil or Wicked character, an evil one toward a
    /// Good or Saintly character. They greet them coldly and offer them no quest that would do
    /// them good.
    pub fn scorned_by(self, them: Alignment) -> bool {
        match them {
            Alignment::Good => self <= Band::Evil,
            Alignment::Evil => self >= Band::Good,
            Alignment::Neutral => false,
        }
    }
}

/// The alignment something asks of whoever uses it (`requires_alignment` in data): a quest
/// offered, an ability used. `min` is the worst band allowed, `max` the best, each optional:
/// `{ min = "good" }` is Good or better (a paladin's gifts), `{ max = "callous" }` Callous or
/// worse.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    #[serde(default)]
    pub min: Option<Band>,
    #[serde(default)]
    pub max: Option<Band>,
}

impl Requirement {
    /// Whether a character of `band` meets it.
    pub fn allows(self, band: Band) -> bool {
        self.min.is_none_or(|min| band >= min) && self.max.is_none_or(|max| band <= max)
    }

    /// Why it can never be met, if so: its `min` is better than its `max`, or it names neither.
    pub fn check(self) -> Result<(), String> {
        match (self.min, self.max) {
            (None, None) => Err("`requires_alignment` names a `min` band, a `max`, or both".into()),
            (Some(min), Some(max)) if min > max => Err(format!(
                "`requires_alignment` asks for {} or better and {} or worse at once",
                min.name(),
                max.name()
            )),
            _ => Ok(()),
        }
    }

    /// What it asks for, in words: "Good or better", "Callous or worse", "Kind to Saintly".
    pub fn words(self) -> String {
        match (self.min, self.max) {
            (Some(min), Some(max)) if min == max => min.name().to_string(),
            (Some(min), Some(max)) => format!("{} to {}", min.name(), max.name()),
            (Some(min), None) => format!("{} or better", min.name()),
            (None, Some(max)) => format!("{} or worse", max.name()),
            (None, None) => "any".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deeds_diminish_the_further_along_the_way_they_lead() {
        // From 0, all of it either way.
        assert_eq!(Score::ZERO.shifted(100), Score(100));
        assert_eq!(Score::ZERO.shifted(-100), Score(-100));
        // Halfway to good, a good deed counts half; at 900, a tenth.
        assert_eq!(Score(500).shifted(100), Score(550));
        assert_eq!(Score(900).shifted(100), Score(910));
        assert_eq!(Score(-500).shifted(-100), Score(-550));
        // Small deeds stop counting near the end: rounded toward 0.
        assert_eq!(Score(950).shifted(10), Score(950));
        assert_eq!(Score(600).shifted(2), Score(600));
        // Against the lean, all of it, across 0 too.
        assert_eq!(Score(500).shifted(-100), Score(400));
        assert_eq!(Score(-50).shifted(200), Score(150));
        // Never past the ends.
        assert_eq!(Score(-900).shifted(5000), Score(MAX));
        assert_eq!(Score(MAX).shifted(1), Score(MAX));
        assert_eq!(Score(MAX).shifted(-5000), Score(-MAX));
    }

    #[test]
    fn many_good_deeds_approach_but_never_pass_the_end() {
        let mut s = Score::ZERO;
        let mut last = s;
        for _ in 0..10_000 {
            s = s.shifted(10);
            assert!(s >= last, "only ever further");
            last = s;
        }
        assert!(s.value() > 900 && s.value() < MAX, "{s:?}");
    }

    #[test]
    fn scores_out_of_range_do_not_load() {
        assert_eq!(serde_json::from_str::<Score>("-1000").unwrap(), Score(-MAX));
        assert!(serde_json::from_str::<Score>("1001").is_err());
        assert_eq!(serde_json::to_string(&Score(-250)).unwrap(), "-250");
    }

    #[test]
    fn bands_are_ordered_from_wicked_up_and_have_sides() {
        let mut sorted = Band::ALL.to_vec();
        sorted.sort();
        sorted.reverse();
        assert_eq!(sorted, Band::ALL, "ALL is best first, the enum worst first");
        assert_eq!(Band::Kind.side(), Alignment::Good);
        assert_eq!(Band::Callous.side(), Alignment::Evil);
        assert_eq!(Band::Neutral.side(), Alignment::Neutral);
        for a in Alignment::ALL {
            assert_eq!(a.band().side(), a);
        }
    }

    #[test]
    fn good_folk_scorn_the_evil_and_evil_folk_the_good() {
        let scorned = |them| {
            (Band::ALL.into_iter())
                .filter(|b| b.scorned_by(them))
                .collect::<Vec<_>>()
        };
        assert_eq!(scorned(Alignment::Good), [Band::Evil, Band::Wicked]);
        assert_eq!(scorned(Alignment::Evil), [Band::Saintly, Band::Good]);
        assert!(scorned(Alignment::Neutral).is_empty());
    }

    #[test]
    fn requirements_allow_their_bands() {
        let good: Requirement = toml::from_str("min = \"good\"").unwrap();
        assert!(good.allows(Band::Saintly) && good.allows(Band::Good));
        assert!(!good.allows(Band::Kind));
        assert_eq!(good.words(), "Good or better");
        let dark: Requirement = toml::from_str("max = \"callous\"").unwrap();
        assert!(dark.allows(Band::Wicked) && dark.allows(Band::Callous));
        assert!(!dark.allows(Band::Neutral));
        assert_eq!(dark.words(), "Callous or worse");
        let between = Requirement {
            min: Some(Band::Kind),
            max: Some(Band::Good),
        };
        assert_eq!(between.words(), "Kind to Good");
        assert!(between.check().is_ok());
        assert!(Requirement::default().check().is_err());
        let never = Requirement {
            min: Some(Band::Good),
            max: Some(Band::Kind),
        };
        assert!(never.check().is_err());
        assert!(toml::from_str::<Requirement>("min = \"lawful\"").is_err());
    }
}
