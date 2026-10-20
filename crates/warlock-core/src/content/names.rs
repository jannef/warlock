//! Names for new characters (`assets/data/names.toml`, spec §8 "Character creation"): by race
//! id, a list for male and one for female characters; `other` picks from both. Character
//! creation's Name step offers one at random (`character::random_name`), and so does a run told
//! no name for a character that is not its scenario's own (`character::default_name`).
//!
//! Optional: without the file (or a race's lists) there is no name to offer, and a character
//! without one is `character::NAMELESS`. Names are flavour: not in the content fingerprint.

use super::ContentError;
use crate::character::{Gender, check_name};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The names offered to one race, by gender.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NameLists {
    #[serde(default)]
    pub male: Vec<String>,
    #[serde(default)]
    pub female: Vec<String>,
}

/// Every race's names, by race id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Names {
    by_race: BTreeMap<String, NameLists>,
}

impl Names {
    /// Reads `text` (tables by race id); `is_race` tells known race ids.
    pub(super) fn parse(
        path: &str,
        text: &str,
        is_race: impl Fn(&str) -> bool,
    ) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let by_race: BTreeMap<String, NameLists> =
            toml::from_str(text).map_err(|e| err(e.to_string()))?;
        for (race, lists) in &by_race {
            if !is_race(race) {
                return Err(err(format!("unknown race `{race}`")));
            }
            for name in lists.male.iter().chain(&lists.female) {
                match check_name(name) {
                    Ok(checked) if checked == name => {}
                    Ok(_) => return Err(err(format!("{race}: `{name}` has spaces around it"))),
                    Err(why) => return Err(err(format!("{race}: {why}"))),
                }
            }
        }
        Ok(Self { by_race })
    }

    /// The names offered to a character of `race` and `gender`, in file order: the gender's list,
    /// or for `other` the male list, then the female. Empty without any.
    pub fn offered(&self, race: &str, gender: Gender) -> Vec<&str> {
        let Some(lists) = self.by_race.get(race) else {
            return Vec::new();
        };
        let names: Vec<&String> = match gender {
            Gender::Male => lists.male.iter().collect(),
            Gender::Female => lists.female.iter().collect(),
            Gender::Other => lists.male.iter().chain(&lists.female).collect(),
        };
        names.into_iter().map(String::as_str).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Result<Names, ContentError> {
        Names::parse("data/names.toml", text, |id| ["dwarf", "elf"].contains(&id))
    }

    #[test]
    fn names_are_offered_by_race_and_gender() {
        let names = parse(
            "[dwarf]\nmale = [\"Borin\", \"Dain\"]\nfemale = [\"Brakka\"]\n[elf]\nfemale = [\"Ilse\"]\n",
        )
        .expect("valid");
        assert_eq!(names.offered("dwarf", Gender::Male), ["Borin", "Dain"]);
        assert_eq!(names.offered("dwarf", Gender::Female), ["Brakka"]);
        assert_eq!(
            names.offered("dwarf", Gender::Other),
            ["Borin", "Dain", "Brakka"]
        );
        assert!(names.offered("elf", Gender::Male).is_empty());
        assert!(names.offered("gnome", Gender::Female).is_empty());
    }

    #[test]
    fn bad_names_and_races_are_refused() {
        for (text, why) in [
            ("[orc]\nmale = [\"Grok\"]\n", "unknown race `orc`"),
            ("[dwarf]\nmale = [\"\"]\n", "cannot be empty"),
            ("[dwarf]\nmale = [\" Borin\"]\n", "spaces around it"),
            (
                "[dwarf]\nfemale = [\"Brakka Stonefist Ironhand\"]\n",
                "longer than 20",
            ),
            ("[dwarf]\nother = [\"Borin\"]\n", "unknown field `other`"),
        ] {
            let err = parse(text).expect_err(text);
            assert!(err.message.contains(why), "{text}: {}", err.message);
        }
    }
}
