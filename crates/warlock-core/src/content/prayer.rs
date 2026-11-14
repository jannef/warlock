//! The priest's prayer (`assets/data/prayer.toml`, spec §8 "Class abilities"; the priest design,
//! `docs/design/priest.md` §6.1; `docs/dev/prayer.md`): the strain a prayer adds and how fast it
//! passes, the chance a strained prayer fizzles, the bands each face of the prayer answers, the
//! statuses the release use ends, and the uses in their priority order with their words.
//!
//! The rule itself lives in `world/classes/orison.rs`; which uses a priest has, how many answers a
//! prayer gives, and the titles are the prayer talents' (`FaithEffect::Prayer`). The uses are a
//! closed list in code ([`PrayerUse`]): the file gives each one's place in the order and its
//! words, once each.
//!
//! The file is optional: without it every prayer is refused, so fixtures load.

use super::{Content, ContentError, StatusId};
use crate::alignment::{Band, Requirement};
use crate::time::Duration;
use serde::{Deserialize, Serialize};

/// What a prayer can answer: each a problem the priest may have and its one fix
/// (`world/classes/orison.rs`). The order here is the code's; the priority is the file's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrayerUse {
    /// A disease showing: every disease goes, the brewing ones too.
    Disease,
    /// Poisoned: it ends.
    Poison,
    /// Bleeding or hurt: the bleeding stops and the hit points fill.
    Wounds,
    /// A status of the release list (hexes, holds, webs): every one ends.
    Release,
    /// Mana short of its maximum: it fills.
    Mana,
    /// Hungry or worse: fed to Satiated.
    Hunger,
    /// Stat damage or drained points: all mended and given back.
    Restoration,
}

impl PrayerUse {
    /// Every use, in the code's order.
    pub const ALL: [PrayerUse; 7] = [
        PrayerUse::Disease,
        PrayerUse::Poison,
        PrayerUse::Wounds,
        PrayerUse::Release,
        PrayerUse::Mana,
        PrayerUse::Hunger,
        PrayerUse::Restoration,
    ];

    /// Its id in data and dumps.
    pub const fn id(self) -> &'static str {
        match self {
            PrayerUse::Disease => "disease",
            PrayerUse::Poison => "poison",
            PrayerUse::Wounds => "wounds",
            PrayerUse::Release => "release",
            PrayerUse::Mana => "mana",
            PrayerUse::Hunger => "hunger",
            PrayerUse::Restoration => "restoration",
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|u| u.id() == id)
    }
}

/// Who answers a prayer: the holy powers or the dark ones, by the priest's band at the moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face {
    Holy,
    Dark,
}

impl Face {
    /// Its id in dumps.
    pub const fn id(self) -> &'static str {
        match self {
            Face::Holy => "holy",
            Face::Dark => "dark",
        }
    }
}

/// A line for each face.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Faced {
    pub holy: String,
    pub dark: String,
}

impl Faced {
    /// The line of `face`.
    pub fn of(&self, face: Face) -> &str {
        match face {
            Face::Holy => &self.holy,
            Face::Dark => &self.dark,
        }
    }
}

/// A face's own lines: a prayer begun and answered, one that fizzles (locked or unlucky, the
/// same words), and one refused with nothing to answer.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FaceWords {
    pub pray: String,
    pub fizzle: String,
    pub nothing: String,
}

/// A use in the priority order, with its words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseDef {
    pub id: PrayerUse,
    pub words: Faced,
}

/// The rules of the prayer that live in data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrayerRules {
    /// The strain every attempt adds, answered or not.
    pub strain: u16,
    /// A point of strain passes every this long.
    pub decay: Duration,
    /// The chance in percent a prayer fizzles for every 100 points of strain.
    pub fizzle_per_100: u16,
    /// The bands the holy powers answer.
    pub holy: Requirement,
    /// The bands the dark powers answer.
    pub dark: Requirement,
    /// What the release use ends.
    pub release: Vec<StatusId>,
    /// Each face's own lines.
    pub holy_words: FaceWords,
    pub dark_words: FaceWords,
    /// The uses, the first the most pressing.
    pub uses: Vec<UseDef>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrayerFile {
    strain: u16,
    decay: String,
    fizzle_per_100: u16,
    holy: Requirement,
    dark: Requirement,
    release: Vec<String>,
    words: WordsFile,
    #[serde(rename = "use")]
    uses: Vec<UseFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WordsFile {
    holy: FaceWords,
    dark: FaceWords,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UseFile {
    id: String,
    holy: String,
    dark: String,
}

impl PrayerRules {
    /// Parses `data/prayer.toml`: the release list of known statuses, none the diseases' status
    /// or a status a resource brings (each lasts exactly while its disease or pool does), and
    /// each use named at most once (a second's words would never be read).
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: PrayerFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let decay: Duration = parsed.decay.parse().map_err(err)?;
        let mut release = Vec::new();
        for id in &parsed.release {
            let status = (content.status_id(id))
                .ok_or_else(|| err(format!("release: unknown status `{id}`")))?;
            let held = content.disease_status() == Some(status)
                || (content.resources().iter()).any(|r| r.status == Some(status));
            if held {
                return Err(err(format!(
                    "release: `{id}` lasts while a disease or a resource does"
                )));
            }
            release.push(status);
        }
        let mut uses: Vec<UseDef> = Vec::new();
        for u in parsed.uses {
            let id = (PrayerUse::from_id(&u.id))
                .ok_or_else(|| err(format!("use: unknown use `{}`", u.id)))?;
            if uses.iter().any(|d| d.id == id) {
                return Err(err(format!("use: `{}` twice", u.id)));
            }
            uses.push(UseDef {
                id,
                words: Faced {
                    holy: u.holy,
                    dark: u.dark,
                },
            });
        }
        Ok(Self {
            strain: parsed.strain,
            decay,
            fizzle_per_100: parsed.fizzle_per_100,
            holy: parsed.holy,
            dark: parsed.dark,
            release,
            holy_words: parsed.words.holy,
            dark_words: parsed.words.dark,
            uses,
        })
    }

    /// Who answers a priest of `band` (`None` without alignment rules: the holy powers). A band
    /// both faces allow is the holy one's.
    pub fn face(&self, band: Option<Band>) -> Face {
        match band {
            Some(band) if !self.holy.allows(band) && self.dark.allows(band) => Face::Dark,
            _ => Face::Holy,
        }
    }

    /// The lines of `face`.
    pub fn words(&self, face: Face) -> &FaceWords {
        match face {
            Face::Holy => &self.holy_words,
            Face::Dark => &self.dark_words,
        }
    }

    /// The use `id`'s words.
    pub fn use_words(&self, id: PrayerUse) -> Option<&Faced> {
        self.uses.iter().find(|d| d.id == id).map(|d| &d.words)
    }

    /// The chance in percent a prayer fizzles at `strain`, at most 100.
    pub fn fizzle_chance(&self, strain: u16) -> u32 {
        (u32::from(strain) * u32::from(self.fizzle_per_100) / 100).min(100)
    }
}

impl super::RulesFile for PrayerRules {
    const PATH: &'static str = "data/prayer.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentFiles;

    const PATH: &str = "data/prayer.toml";

    #[test]
    fn the_builtin_file_orders_the_uses_as_the_owner_listed() {
        let content = crate::test_support::builtin();
        let rules = content.prayer().expect("the prayer is in the game");
        let order: Vec<&str> = rules.uses.iter().map(|u| u.id.id()).collect();
        assert_eq!(
            order,
            [
                "disease",
                "poison",
                "wounds",
                "release",
                "mana",
                "hunger",
                "restoration"
            ]
        );
        assert_eq!((rules.strain, rules.decay.as_seconds()), (100, 60));
        assert_eq!(
            [100, 270, 500, 1000, 2000].map(|s| rules.fizzle_chance(s)),
            [10, 27, 50, 100, 100]
        );
        assert_eq!(rules.face(Some(Band::Neutral)), Face::Holy);
        assert_eq!(rules.face(Some(Band::Callous)), Face::Dark);
        assert_eq!(rules.face(Some(Band::Wicked)), Face::Dark);
        assert_eq!(rules.face(None), Face::Holy);
        assert_eq!(rules.release.len(), 12);
    }

    fn load(edit: impl Fn(&str) -> String) -> Result<Content, ContentError> {
        let mut files = ContentFiles::builtin();
        let text = files
            .get_optional(PATH)
            .expect("the prayer file")
            .to_string();
        files.insert(PATH, &edit(&text));
        Content::load(&files)
    }

    #[test]
    fn a_bad_file_is_refused_and_none_refuses_every_prayer() {
        let bad = |from: &str, to: &str| {
            load(|t| {
                assert!(t.contains(from), "{from}");
                t.replacen(from, to, 1)
            })
            .expect_err(to)
        };
        bad("decay = \"1m\"", "decay = \"soon\"");
        bad("\"ill_wished\",", "\"raging\",");
        bad("\"ill_wished\",", "\"no_such_status\",");
        bad("id = \"mana\"", "id = \"wounds\"");
        bad("id = \"mana\"", "id = \"glory\"");
        let mut files = ContentFiles::builtin();
        files.remove(PATH);
        let content = Content::load(&files).expect("content without the file");
        assert!(content.prayer().is_none());
    }
}
