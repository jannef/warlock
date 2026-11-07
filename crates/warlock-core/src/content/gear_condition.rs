//! The condition creature gear spawns in (`assets/data/gear_condition.toml`, spec §8 "Creature
//! gear"; #1193): the chances in per mille that a piece is damaged, or Fine, Marvellous, or
//! Masterwork, by default and by tribe (a creature tag), and what damaged gear of each material
//! group is called. The rule itself is `crate::gear_condition`. Without the file every piece
//! spawns as made.

use super::{ContentError, ItemHarm};
use serde::Deserialize;

/// What the chances are out of.
pub const PER_MILLE: u32 = 1000;

/// The chances of one condition table, in per mille.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Odds {
    /// Damaged: its improvements gone and a point worse (`magic/worked.rs`).
    pub damaged: u16,
    /// One improvement.
    pub fine: u16,
    /// Two.
    pub marvellous: u16,
    /// Three.
    pub masterwork: u16,
}

impl Odds {
    /// The improvements a draw of `roll` (below [`PER_MILLE`]) gives, or `None` for damaged:
    /// `Some(0)` is as made.
    pub fn of(&self, roll: u32) -> Option<u8> {
        let bands = [self.damaged, self.fine, self.marvellous, self.masterwork];
        let mut below = 0;
        for (i, band) in bands.into_iter().enumerate() {
            below += u32::from(band);
            if roll < below {
                return (i > 0).then_some(i as u8);
            }
        }
        Some(0)
    }

    fn total(&self) -> u32 {
        [self.damaged, self.fine, self.marvellous, self.masterwork]
            .into_iter()
            .map(u32::from)
            .sum()
    }
}

/// One tribe's own odds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TribeOdds {
    /// The creature tag that makes a creature of the tribe.
    pub tag: String,
    /// Its odds.
    pub odds: Odds,
}

/// The condition tables of creature gear.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GearConditions {
    /// The odds of a creature of no listed tribe.
    pub default: Odds,
    /// The tribes, in file order: a creature takes the first whose tag it has.
    pub tribes: Vec<TribeOdds>,
    /// By material group, the kinds damaged gear of it may show (module `crate::gear_condition`
    /// says which one is taken); a group not listed is plainly "damaged".
    pub kinds: Vec<(String, Vec<ItemHarm>)>,
}

impl GearConditions {
    /// The odds of a creature with `tags`.
    pub fn odds(&self, tags: &[String]) -> Odds {
        (self.tribes.iter())
            .find(|t| tags.contains(&t.tag))
            .map_or(self.default, |t| t.odds)
    }

    /// The kinds damaged gear of material group `group` may show.
    pub fn kinds(&self, group: Option<&str>) -> &[ItemHarm] {
        (self.kinds.iter())
            .find(|(g, _)| Some(g.as_str()) == group)
            .map_or(&[], |(_, kinds)| kinds.as_slice())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    default: Odds,
    #[serde(default)]
    tribe: Vec<TribeSpec>,
    #[serde(default)]
    kinds: toml::Table,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TribeSpec {
    tag: String,
    #[serde(flatten)]
    odds: Odds,
}

impl GearConditions {
    /// Parses `data/gear_condition.toml`: odds that add up to at most [`PER_MILLE`] (a roll out
    /// of it falls in one band), each tribe's tag once.
    pub(super) fn parse(file: &str, text: &str) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let check = |name: &str, odds: &Odds| {
            (odds.total() <= PER_MILLE).then_some(()).ok_or_else(|| {
                err(format!(
                    "`{name}`'s chances add up to more than {PER_MILLE}"
                ))
            })
        };
        check("default", &parsed.default)?;
        let mut tribes: Vec<TribeOdds> = Vec::new();
        for spec in parsed.tribe {
            check(&spec.tag, &spec.odds)?;
            if tribes.iter().any(|t| t.tag == spec.tag) {
                return Err(err(format!("tribe `{}` is listed twice", spec.tag)));
            }
            tribes.push(TribeOdds {
                tag: spec.tag,
                odds: spec.odds,
            });
        }
        let mut kinds = Vec::new();
        for (group, list) in parsed.kinds {
            let list: Vec<ItemHarm> = list
                .try_into()
                .map_err(|e: toml::de::Error| err(format!("kinds of `{group}`: {e}")))?;
            kinds.push((group, list));
        }
        Ok(Self {
            default: parsed.default,
            tribes,
            kinds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_roll_falls_in_its_band() {
        let odds = Odds {
            damaged: 300,
            fine: 40,
            marvellous: 8,
            masterwork: 2,
        };
        let of = |r| odds.of(r);
        assert_eq!(
            [of(0), of(299), of(300), of(339)],
            [None, None, Some(1), Some(1)]
        );
        assert_eq!(
            [of(340), of(347), of(348), of(349)],
            [Some(2), Some(2), Some(3), Some(3)]
        );
        assert_eq!((of(350), of(999)), (Some(0), Some(0)));
    }
}
