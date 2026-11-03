//! The witch's familiar's data (`assets/data/familiars.toml`; spec §8 "Allies": "The familiar";
//! docs/design/witch-familiar.md §9.1 and §16; `docs/dev/familiar.md`): the starting beasts by
//! race and the rares, the hex its touch lays, the names, and how each kind arrives. The rules
//! live in `world/familiar.rs`; the kept beasts' growth is `companions.toml`'s, the ally core's
//! constants `allies.toml`. The file is optional: without it no familiar can be made, and nothing
//! here is read.

use super::{Content, ContentError, CreatureId, SpellId, SpellPurpose, creature::Combat};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ops::RangeInclusive;

/// A race's list holds this many beasts.
pub const LIST_SIZES: RangeInclusive<usize> = 1..=3;
/// The rare list holds this many.
pub const RARE_SIZES: RangeInclusive<usize> = 1..=8;
/// The longest name (a character's own longest).
pub const NAME_CHARS: usize = 20;

/// The numbers of the familiar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FamiliarRules {
    /// The jackpot, in a thousand: a beast of `rare` instead of the race's list.
    pub rare_per_mille: u32,
    /// The rares, shared by every race, in file order.
    pub rare: Vec<CreatureId>,
    /// A race's list holds beasts of at most this level.
    pub most_level: u8,
    /// Every race's starting beasts, by race id, in file order.
    pub by_race: Vec<(String, Vec<CreatureId>)>,
    /// The hex spell whose status a familiar's blow lays (`kind = "hex"`, with a status).
    pub touch: SpellId,
    /// What a familiar may be called.
    pub names: Vec<String>,
    /// How each kind arrives ("winds about your ankles"), by creature.
    pub arrives: Vec<(CreatureId, String)>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    names: Vec<String>,
    touch: String,
    start: Start,
    arrives: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Start {
    rare_per_mille: u32,
    rare: Vec<String>,
    most_level: u8,
    by_race: BTreeMap<String, Vec<String>>,
}

impl FamiliarRules {
    /// The beasts of race `race`'s list, if it has one.
    pub fn list_of(&self, race: &str) -> Option<&[CreatureId]> {
        (self.by_race.iter())
            .find(|(r, _)| r == race)
            .map(|(_, list)| list.as_slice())
    }

    /// Every beast a start may draw, the races' lists and the rares, each once, in file order.
    pub fn every_start(&self) -> Vec<CreatureId> {
        let mut out: Vec<CreatureId> = Vec::new();
        for id in (self.by_race.iter().flat_map(|(_, l)| l.iter())).chain(&self.rare) {
            if !out.contains(id) {
                out.push(*id);
            }
        }
        out
    }

    /// How a familiar of type `creature` arrives ("winds about your ankles").
    pub fn arrives(&self, creature: CreatureId) -> Option<&str> {
        (self.arrives.iter())
            .find(|(c, _)| *c == creature)
            .map(|(_, line)| line.as_str())
    }

    /// Parses `data/familiars.toml` once the creatures, the races, and the spells are known, and
    /// checks it: every list is a known race's and holds 1 to 3 plain beasts of at most
    /// `most_level` (a race without a list draws no familiar: test fixtures add races;
    /// `familiar_tests` asks every shipped race for one), the rares 1 to 8 plain beasts,
    /// `touch` a hex spell with a status, at least one name and none longer than a character's,
    /// and an `arrives` line for every beast a start may draw and for nothing else.
    pub(super) fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let file: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        // A plain beast: what a tameable type must be (`content/mod.rs`), whatever its flag.
        let beast = |where_: &str, name: &str| -> Result<CreatureId, ContentError> {
            let id = (content.creature_id(name))
                .filter(|_| name != "player")
                .ok_or_else(|| err(format!("{where_}: unknown creature `{name}`")))?;
            let def = content.creature(id);
            let why = match () {
                _ if def.unique => Some("a unique"),
                _ if def.animated.is_some() => Some("an animated object"),
                _ if def.summon.is_some() => Some("a summoner"),
                _ if def.opens_doors => Some("one that opens doors"),
                _ if def.race.is_some() => Some("one of the peoples"),
                _ if def.combat != Combat::Melee => Some("one that fights from afar"),
                _ => None,
            };
            match why {
                Some(why) => Err(err(format!("{where_}: `{name}` is {why}"))),
                None => Ok(id),
            }
        };
        if !(0..=1000).contains(&file.start.rare_per_mille) {
            return Err(err(format!(
                "`start.rare_per_mille` is 0 to 1000, not {}",
                file.start.rare_per_mille
            )));
        }
        if !RARE_SIZES.contains(&file.start.rare.len()) {
            return Err(err(format!(
                "`start.rare` lists {} to {} beasts, not {}",
                RARE_SIZES.start(),
                RARE_SIZES.end(),
                file.start.rare.len()
            )));
        }
        let rare = (file.start.rare.iter())
            .map(|n| beast("`start.rare`", n))
            .collect::<Result<Vec<_>, _>>()?;
        let mut by_race = Vec::new();
        for race in &content.races().list {
            let Some(names) = file.start.by_race.get(&race.id) else {
                continue;
            };
            if !LIST_SIZES.contains(&names.len()) {
                return Err(err(format!(
                    "`start.by_race.{}` lists {} to {} beasts, not {}",
                    race.id,
                    LIST_SIZES.start(),
                    LIST_SIZES.end(),
                    names.len()
                )));
            }
            let where_ = format!("`start.by_race.{}`", race.id);
            let mut list = Vec::new();
            for name in names {
                let id = beast(&where_, name)?;
                let level = content.creature(id).level;
                if level > file.start.most_level {
                    return Err(err(format!(
                        "{where_}: `{name}` is level {level}, above `most_level` {}",
                        file.start.most_level
                    )));
                }
                list.push(id);
            }
            by_race.push((race.id.clone(), list));
        }
        if !content.races().list.is_empty()
            && let Some(race) = (file.start.by_race.keys()).find(|r| content.race(r).is_none())
        {
            return Err(err(format!("`start.by_race`: unknown race `{race}`")));
        }
        let touch = (content.spell_id(&file.touch))
            .filter(|&s| {
                let spell = content.spell(s);
                spell.kind == SpellPurpose::Hex && spell.effect.status.is_some()
            })
            .ok_or_else(|| {
                err(format!(
                    "`touch`: `{}` is no hex spell that lays a status",
                    file.touch
                ))
            })?;
        let long = |n: &String| n.trim().is_empty() || n.chars().count() > NAME_CHARS;
        if file.names.is_empty() || file.names.iter().any(long) {
            return Err(err(format!(
                "`names`: at least one name, none empty or longer than {NAME_CHARS} characters"
            )));
        }
        let mut arrives = Vec::new();
        for (name, line) in &file.arrives {
            let id = (content.creature_id(name))
                .ok_or_else(|| err(format!("`arrives`: unknown creature `{name}`")))?;
            if line.trim().is_empty() {
                return Err(err(format!("`arrives.{name}`: an empty line")));
            }
            arrives.push((id, line.clone()));
        }
        let rules = FamiliarRules {
            rare_per_mille: file.start.rare_per_mille,
            rare,
            most_level: file.start.most_level,
            by_race,
            touch,
            names: file.names,
            arrives,
        };
        let starts = rules.every_start();
        if let Some(&id) = starts.iter().find(|&&c| rules.arrives(c).is_none()) {
            return Err(err(format!(
                "`arrives`: no line for `{}`",
                content.creature(id).id
            )));
        }
        // A line no start draws (where the content has races to draw for: a fixture without
        // them keeps the shipped file).
        if !content.races().list.is_empty()
            && let Some((id, _)) = (rules.arrives.iter()).find(|(c, _)| !starts.contains(c))
        {
            return Err(err(format!(
                "`arrives.{}`: no start draws it",
                content.creature(*id).id
            )));
        }
        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Content, ContentFiles};

    const FAMILIARS: &str = include_str!("../../../../assets/data/familiars.toml");

    fn familiars(edit: impl Fn(&str) -> String) -> Result<Content, String> {
        let mut files = ContentFiles::builtin();
        files.insert("data/familiars.toml", &edit(FAMILIARS));
        Content::load(&files).map_err(|e| e.message)
    }

    #[test]
    fn the_shipped_file_loads_and_every_race_has_a_list() {
        let content = Content::builtin().expect("content");
        let rules = content.familiars().expect("familiars.toml");
        for race in &content.races().list {
            let list = rules
                .list_of(&race.id)
                .unwrap_or_else(|| panic!("{}", race.id));
            assert!(super::LIST_SIZES.contains(&list.len()), "{}", race.id);
        }
        assert_eq!(rules.rare.len(), 2);
        assert_eq!(rules.rare_per_mille, 50);
        assert_eq!(content.spell(rules.touch).id, "evil_eye");
        let cat = content.creature_id("cat").unwrap();
        assert_eq!(rules.arrives(cat), Some("winds about your ankles"));
        // The old craft's five are all drawn somewhere.
        for kind in ["cat", "toad", "raven", "hare", "bat"] {
            let id = content.creature_id(kind).unwrap();
            assert!(rules.every_start().contains(&id), "{kind}");
        }
    }

    #[test]
    fn the_file_is_checked() {
        let err = |r: Result<Content, String>| r.expect_err("refused");
        assert!(
            err(familiars(|t| t.replace(
                "rare = [\"black_hound\", \"panther\"]",
                "rare = [\"kobold\"]"
            )))
            .contains("opens doors"),
            "a rare is a plain beast"
        );
        assert!(
            err(familiars(|t| t.replace(
                "dwarf = [\"toad\", \"raven\"]",
                "dwarf = [\"bear\"]"
            )))
            .contains("above `most_level`"),
        );
        assert!(
            err(familiars(
                |t| t.replace("dwarf = [\"toad\", \"raven\"]", "dwarf = []")
            ))
            .contains("lists 1 to 3"),
        );
        assert!(
            err(familiars(|t| t.replace("ratfolk = ", "elvish = ")))
                .contains("unknown race `elvish`"),
        );
        assert!(
            err(familiars(
                |t| t.replace("rare_per_mille = 50", "rare_per_mille = 1001")
            ))
            .contains("0 to 1000")
        );
        assert!(
            err(familiars(|t| t.replace(
                "touch = \"evil_eye\"",
                "touch = \"magic_missile\""
            )))
            .contains("no hex spell"),
        );
        assert!(
            err(familiars(|t| t.replace("names = [", "names = [\"\", "))).contains("none empty")
        );
        assert!(
            err(familiars(
                |t| t.replace("cat = \"winds about your ankles\"\n", "")
            ))
            .contains("no line for `cat`")
        );
        assert!(err(familiars(|t| format!("{t}wolf = \"howls\"\n"))).contains("no start draws it"));
        // Without the file, no familiar at all, and nothing else minds.
        let mut files = ContentFiles::builtin();
        files.remove("data/familiars.toml");
        let content = Content::load(&files).expect("loads without");
        assert!(content.familiars().is_none());
    }
}
