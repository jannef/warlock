//! The condition creature gear spawns in (`assets/data/gear_condition.toml`, spec §8 "Creature
//! gear"; #1193): the chances in per mille that a piece is damaged, or Fine, Marvellous, or
//! Masterwork, by default and by tribe (a creature tag), and what damaged gear of each material
//! group is called; and by item level (`[[depth]]`, #1436), the better make every source's arms
//! and armour take deeper, and the more often loot is set with a gem. The rule itself is
//! `crate::world::items::gear_condition`. Without the file every piece spawns as made.

use super::{Content, ContentError, ItemHarm};
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
        u32::from(self.damaged) + self.improved()
    }

    /// The chances of an improvement, all three together.
    pub fn improved(&self) -> u32 {
        [self.fine, self.marvellous, self.masterwork]
            .into_iter()
            .map(u32::from)
            .sum()
    }

    /// These odds with `depth`'s added to each improvement, damaged giving way to what they then
    /// leave of [`PER_MILLE`] (the loader keeps the improvements within it).
    pub fn deeper(&self, depth: &Depth) -> Odds {
        let fine = self.fine + depth.fine;
        let marvellous = self.marvellous + depth.marvellous;
        let masterwork = self.masterwork + depth.masterwork;
        let improved = u32::from(fine) + u32::from(marvellous) + u32::from(masterwork);
        let room = u16::try_from(PER_MILLE.saturating_sub(improved)).unwrap_or(0);
        Odds {
            damaged: self.damaged.min(room),
            fine,
            marvellous,
            masterwork,
        }
    }
}

/// From an item level on, the better make and the gems of deeper finds (#1436): `fine`,
/// `marvellous`, and `masterwork` per mille added to every source's arms and armour (loot's own,
/// which is otherwise as made, and creature gear's tribe odds), `ornament` percent added to
/// loot's chance of a set gem (`gems.toml`'s `[[ornament]]`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Depth {
    /// The least item level the row is for (it holds up to the next row's).
    pub level: u8,
    #[serde(default)]
    pub fine: u16,
    #[serde(default)]
    pub marvellous: u16,
    #[serde(default)]
    pub masterwork: u16,
    /// Percent more of loot set with a gem, for the classes that are.
    #[serde(default)]
    pub ornament: u8,
}

impl Depth {
    /// As odds: no damaged, the three improvements.
    pub fn odds(&self) -> Odds {
        Odds {
            damaged: 0,
            fine: self.fine,
            marvellous: self.marvellous,
            masterwork: self.masterwork,
        }
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
    /// By item level, ascending: what deeper finds take besides ([`Depth`]).
    pub depth: Vec<Depth>,
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

    /// What finds at item level `level` take besides: the last `[[depth]]` row whose level it
    /// has reached, else nothing.
    pub fn depth_at(&self, level: u8) -> Depth {
        (self.depth.iter())
            .rev()
            .find(|d| d.level <= level.max(1))
            .copied()
            .unwrap_or_default()
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
    #[serde(default)]
    depth: Vec<Depth>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TribeSpec {
    tag: String,
    #[serde(flatten)]
    odds: Odds,
}

impl GearConditions {
    /// Parses `data/gear_condition.toml` against `content` (its materials and creatures loaded):
    /// [`GearConditions::parse_text`]'s checks, then every `[kinds]` key a material's `group`
    /// and every tribe's tag one a creature carries (#1556), so a misspelt `mettal` is an error,
    /// not a table that never matches.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let rules = Self::parse_text(file, text)?;
        for (group, _) in &rules.kinds {
            let known = (content.materials.iter()).any(|m| m.group.as_deref() == Some(group));
            if !known {
                return Err(ContentError::new(
                    file,
                    format!("`[kinds]`: no material is of the group `{group}`"),
                ));
            }
        }
        let tags = content.tags();
        for tribe in &rules.tribes {
            if !tags.contains(&tribe.tag.as_str()) {
                return Err(ContentError::new(
                    file,
                    format!("tribe `{}`: no creature carries the tag", tribe.tag),
                ));
            }
        }
        Ok(rules)
    }

    /// Parses the file alone: odds that add up to at most [`PER_MILLE`] (a roll out of it falls
    /// in one band), each tribe's tag once; `[[depth]]` rows by rising level, each one's
    /// improvements with any table's within [`PER_MILLE`], its `ornament` at most 100.
    fn parse_text(file: &str, text: &str) -> Result<Self, ContentError> {
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
        let mut last = 0;
        for depth in &parsed.depth {
            let level = depth.level;
            if level <= last {
                return Err(err(format!(
                    "`[[depth]]` level {level} does not rise above the row before"
                )));
            }
            last = level;
            let most = (tribes.iter().map(|t| t.odds.improved()))
                .chain([parsed.default.improved()])
                .max()
                .unwrap_or(0);
            if most + depth.odds().improved() > PER_MILLE {
                return Err(err(format!(
                    "`[[depth]]` level {level}'s improvements with a table's add up to more \
                     than {PER_MILLE}"
                )));
            }
            if depth.ornament > 100 {
                return Err(err(format!(
                    "`[[depth]]` level {level}'s `ornament` is over 100"
                )));
            }
        }
        Ok(Self {
            default: parsed.default,
            tribes,
            kinds,
            depth: parsed.depth,
        })
    }
}

impl super::RulesFile for GearConditions {
    const PATH: &'static str = "data/gear_condition.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
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

    #[test]
    fn depth_rows_rise_and_add_to_the_odds() {
        let text = "[default]\ndamaged = 900\nfine = 40\nmarvellous = 0\nmasterwork = 0\n\
                    [[depth]]\nlevel = 4\nfine = 60\nornament = 2\n\
                    [[depth]]\nlevel = 8\nfine = 100\nmarvellous = 50\n";
        let rules = GearConditions::parse_text("t", text).expect("parses");
        assert_eq!(rules.depth_at(3), Depth::default());
        assert_eq!(
            (rules.depth_at(4).fine, rules.depth_at(7).ornament),
            (60, 2)
        );
        assert_eq!(rules.depth_at(99).marvellous, 50);
        let deeper = rules.default.deeper(&rules.depth_at(8));
        // 190 improved: damaged gives way to the 810 left.
        assert_eq!(
            (deeper.damaged, deeper.fine, deeper.marvellous),
            (810, 140, 50)
        );
        let falling = text.replace("level = 8", "level = 4");
        assert!(GearConditions::parse_text("t", &falling).is_err());
        let over = text.replace("marvellous = 50", "marvellous = 950");
        assert!(GearConditions::parse_text("t", &over).is_err());
    }
}
