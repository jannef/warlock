//! Intimidating (`assets/data/intimidate.toml`, spec §8 "Skills": Intimidate; #868): the numbers
//! of the Intimidate skill's use, a threat at one foe.
//!
//! The rule itself lives in `world/intimidate.rs`, on War cry's check (`world/war_cry.rs`): who
//! can (anyone trained in a skill with the `intimidate` benefit, `data/skills/`), at whom (a
//! hostile creature they see within `range`, with vitals), the check (its Willpower against
//! `difficulty` + their Strength modifier + their Intimidate), and the status a failed check
//! leaves, at the degrees the ladder gives for what their Intimidate adds.
//!
//! The file is optional: without it nobody intimidates, and the Intimidate skill stays a skill
//! whose use is not in the game yet (the abilities menu's "not usable yet").

use super::{Content, ContentError, StatusId};
use serde::Deserialize;

/// The rules of intimidating that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntimidateRules {
    /// How far the foe may stand, in cells (a disc, as ranges are).
    pub range: i32,
    /// The check's base difficulty, before the Strength modifier and Intimidate are added.
    pub difficulty: i32,
    /// What a foe that fails suffers (one that `flees`: Frightened).
    pub status: StatusId,
    /// The degrees of it, by what the intimidator's Intimidate adds (the `intimidate` benefit's
    /// points): `(points, degrees)`, from 0 points, rising both ways.
    pub degrees: Vec<(i32, u16)>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IntimidateFile {
    range: i32,
    difficulty: i32,
    status: String,
    degrees: Vec<(i32, u16)>,
}

impl IntimidateRules {
    /// Parses `data/intimidate.toml`: the status must be known, and the ladder's points rise
    /// ([`Self::degrees_at`] reads the rungs in order).
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: IntimidateFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let status = (content.status_id(&parsed.status))
            .ok_or_else(|| err(format!("unknown status `{}`", parsed.status)))?;
        if !parsed.degrees.windows(2).all(|w| w[0].0 < w[1].0) {
            return Err(err(
                "`degrees`: [points, degrees] rungs, the points rising".into()
            ));
        }
        Ok(Self {
            range: parsed.range,
            difficulty: parsed.difficulty,
            status,
            degrees: parsed.degrees,
        })
    }

    /// The degrees a foe that fails suffers from an intimidator whose Intimidate adds `points`:
    /// the highest rung reached.
    pub fn degrees_at(&self, points: i32) -> u16 {
        (self.degrees.iter())
            .take_while(|&&(from, _)| from <= points)
            .last()
            .map_or(1, |&(_, d)| d)
    }
}

impl super::RulesFile for IntimidateRules {
    const PATH: &'static str = "data/intimidate.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}

#[cfg(test)]
mod tests {
    use crate::content::{Content, ContentFiles};

    const PATH: &str = "data/intimidate.toml";

    #[test]
    fn the_ladder_rises_from_one_degree_to_war_crys_three() {
        let content = Content::builtin().expect("content");
        let rules = content.intimidate().expect("intimidating is in the game");
        assert_eq!(content.status(rules.status).id, "frightened");
        let at: Vec<u16> = [0, 2, 3, 5, 6, 12].map(|p| rules.degrees_at(p)).into();
        assert_eq!(at, [1, 1, 2, 2, 3, 3]);
        // War cry's degrees are the top rung's.
        let cry = (content
            .class_active(crate::content::ClassActive::WarCry)
            .into_iter())
        .flat_map(|def| def.effects.iter())
        .find_map(|e| match e {
            crate::content::TalentEffect::Martial(crate::talent::MartialEffect::WarCry {
                severity,
                ..
            }) => Some(*severity),
            _ => None,
        });
        assert_eq!(cry, Some(3));
        assert_eq!(rules.degrees.last().map(|r| r.1), cry);
    }

    #[test]
    fn a_bad_file_is_refused_and_none_is_no_intimidating() {
        let good = "range = 6\ndifficulty = 10\nstatus = \"frightened\"\n";
        for (text, why) in [
            (
                "range = 6\ndifficulty = 10\nstatus = \"nobody\"\ndegrees = [[0, 1]]\n",
                "unknown status",
            ),
            (
                &format!("{good}degrees = [[0, 1], [3, 2], [3, 3]]\n"),
                "the points rising",
            ),
        ] {
            let mut files = ContentFiles::builtin();
            files.insert(PATH, text);
            let err = Content::load(&files).expect_err(text);
            assert!(err.message.contains(why), "{text}: {}", err.message);
        }
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("intimidating is optional");
        assert!(content.intimidate().is_none());
    }
}
