//! The merchant's investment (`assets/data/rules/invest.toml`, spec §8 "Class abilities": Invest;
//! #1407): the fees on putting gold in and taking it out, the weekly return and its bonuses, and
//! the amounts the business partner's topics offer (`world/invest.rs`).
//!
//! Every game week the stake changes by `week.base` plus `week.dice` basis points (hundredths of
//! a percent), plus `bonus.quest` for each quest of the partner's town's people completed and the
//! partner's own `bonus` for each quest it names completed (a campaign's: the town file's
//! `[[town.servant]]`, `content/town.rs`). The fees are percents of the amount put in or taken
//! out, rounded up.
//!
//! The file is optional: without it no one invests.

use super::ContentError;
use crate::dice::Dice;
use serde::Deserialize;

/// The investment's rules that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvestRules {
    /// Percent of the gold put in that the partner keeps.
    pub fee_in: u32,
    /// Percent of the gold taken out that the partner keeps.
    pub fee_out: u32,
    /// The weekly return before the roll, in basis points.
    pub base: i32,
    /// The weekly roll, in basis points (its mean should be 0).
    pub dice: Dice,
    /// Basis points more each week per quest of the partner's town completed.
    pub per_quest: i32,

    /// The amounts the partner's topics offer to put in and take out, besides all of it.
    pub steps: Vec<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InvestFile {
    fees: FeesSpec,
    week: WeekSpec,
    bonus: BonusSpec,
    steps: Vec<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FeesSpec {
    invest: u32,
    withdraw: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WeekSpec {
    base: i32,
    dice: Dice,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BonusSpec {
    quest: i32,
}

impl InvestRules {
    /// Parses `data/rules/invest.toml`: the fees below 100 %, and the steps rising and above 0.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: InvestFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        if parsed.fees.invest >= 100 || parsed.fees.withdraw >= 100 {
            return Err(err("a fee must be below 100 percent".into()));
        }
        let rising = parsed.steps.windows(2).all(|w| w[0] < w[1]);
        if !rising || parsed.steps.first().is_some_and(|&s| s == 0) {
            return Err(err("`steps` must rise from above 0".into()));
        }
        Ok(Self {
            fee_in: parsed.fees.invest,
            fee_out: parsed.fees.withdraw,
            base: parsed.week.base,
            dice: parsed.week.dice,
            per_quest: parsed.bonus.quest,

            steps: parsed.steps,
        })
    }

    /// The fee of `percent` on `gold`, rounded up.
    pub fn fee(gold: u32, percent: u32) -> u32 {
        u32::try_from((u64::from(gold) * u64::from(percent)).div_ceil(100)).unwrap_or(u32::MAX)
    }
}

impl super::RulesFile for InvestRules {
    const PATH: &'static str = "data/rules/invest.toml";

    fn parse(path: &str, text: &str, _content: &super::Content) -> Result<Self, ContentError> {
        Self::parse(path, text)
    }
}

#[cfg(test)]
mod tests {
    use super::InvestRules;
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/rules/invest.toml";

    #[test]
    fn the_fees_round_up() {
        let content = crate::test_support::builtin();
        let rules = content.invest().expect("the investment is in the game");
        assert_eq!((rules.fee_in, rules.fee_out), (2, 2));
        assert_eq!(InvestRules::fee(100, 2), 2);
        assert_eq!(InvestRules::fee(1, 2), 1);
        assert_eq!(InvestRules::fee(0, 2), 0);
        assert_eq!(rules.per_quest, 1);
        assert_eq!(rules.steps, [100, 1000]);
    }

    #[test]
    fn a_bad_file_is_refused_and_none_is_no_investing() {
        let week = "week = { base = 10, dice = \"4d201-404\" }\n";
        for (rest, says) in [
            (
                "fees = { invest = 2, withdraw = 2 }\nsteps = [100]\nbonus = { quest = 1, quests = { fire_deep = 3 } }",
                "unknown field",
            ),
            (
                "fees = { invest = 2, withdraw = 2 }\nsteps = [100, 100]\nbonus = { quest = 1 }",
                "must rise",
            ),
            (
                "fees = { invest = 100, withdraw = 2 }\nsteps = [100]\nbonus = { quest = 1 }",
                "below 100",
            ),
        ] {
            let text = format!("{week}{rest}\n");
            let mut files = ContentFiles::builtin();
            files.insert(PATH, &text);
            let err = Content::load(&files).expect_err(&text);
            assert!(err.message.contains(says), "{}", err.message);
        }
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("the investment is optional");
        assert!(content.invest().is_none());
    }
}
