//! The witch's familiar's data (`assets/data/familiars.toml`; spec §8 "Allies": "The familiar";
//! docs/design/witch-familiar.md §9.1 and §16; `docs/dev/familiar.md`): the starting beasts by
//! race and the rares, the hex its touch lays, the names, and how each kind arrives. The rules
//! live in `world/familiar.rs`; the kept beasts' growth is `companions.toml`'s, the ally core's
//! constants `allies.toml`. The file is optional: without it no familiar can be made, and nothing
//! here is read.

use super::{Content, ContentError, CreatureId, SpellId, SpellPurpose};
use serde::Deserialize;
use std::collections::BTreeMap;

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
    /// How far its nose reaches for a master with Familiar's sight, at the least.
    pub nose: i32,
    /// What a familiar may be called.
    pub names: Vec<String>,
    /// How each kind arrives ("winds about your ankles"), by creature.
    pub arrives: Vec<(CreatureId, String)>,
    /// Call familiar's ritual, in turns of the `call_familiar` action.
    pub call_turns: u32,
    /// What a calling may also draw from `bargain_level` of her class on.
    pub bargain_kinds: Vec<CreatureId>,
    /// The level of her class that opens `bargain_kinds`.
    pub bargain_level: u8,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    names: Vec<String>,
    touch: String,
    call_turns: u32,
    bargain_kinds: Vec<String>,
    bargain_level: u8,
    nose: i32,
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

    /// Every beast a start or a calling may draw: [`Self::every_start`], then the bargain's.
    pub fn every_kind(&self) -> Vec<CreatureId> {
        let mut out = self.every_start();
        for id in &self.bargain_kinds {
            if !out.contains(id) {
                out.push(*id);
            }
        }
        out
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

    /// Parses `data/familiars.toml` once the creatures, the races, and the spells are known: every
    /// list is a known race's and holds known creatures (a race without a list draws no
    /// familiar: test fixtures add races), the lists and the names are not empty (the draws
    /// index them), `touch` is a hex spell that lays a status (the touch lays it), and the
    /// `arrives` lines name known creatures.
    pub(super) fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let file: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let beast = |where_: &str, name: &str| -> Result<CreatureId, ContentError> {
            (content.creature_id(name))
                .filter(|_| name != "player")
                .ok_or_else(|| err(format!("{where_}: unknown creature `{name}`")))
        };
        let bargain_kinds = (file.bargain_kinds.iter())
            .map(|n| beast("`bargain_kinds`", n))
            .collect::<Result<Vec<_>, _>>()?;
        // The draws index the lists and the names.
        if file.start.rare.is_empty() {
            return Err(err("`start.rare` lists at least one beast".into()));
        }
        let rare = (file.start.rare.iter())
            .map(|n| beast("`start.rare`", n))
            .collect::<Result<Vec<_>, _>>()?;
        let mut by_race = Vec::new();
        for race in &content.races().list {
            let Some(names) = file.start.by_race.get(&race.id) else {
                continue;
            };
            if names.is_empty() {
                return Err(err(format!(
                    "`start.by_race.{}` lists at least one beast",
                    race.id
                )));
            }
            let where_ = format!("`start.by_race.{}`", race.id);
            let list = (names.iter())
                .map(|name| beast(&where_, name))
                .collect::<Result<Vec<_>, _>>()?;
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
        if file.names.is_empty() {
            return Err(err("`names`: at least one name".into()));
        }
        let mut arrives = Vec::new();
        for (name, line) in &file.arrives {
            let id = (content.creature_id(name))
                .ok_or_else(|| err(format!("`arrives`: unknown creature `{name}`")))?;
            arrives.push((id, line.clone()));
        }
        let rules = FamiliarRules {
            rare_per_mille: file.start.rare_per_mille,
            rare,
            most_level: file.start.most_level,
            by_race,
            touch,
            nose: file.nose,
            names: file.names,
            arrives,
            call_turns: file.call_turns,
            bargain_kinds,
            bargain_level: file.bargain_level,
        };
        Ok(rules)
    }
}

impl super::RulesFile for FamiliarRules {
    const PATH: &'static str = "data/familiars.toml";

    fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        Self::parse(path, text, content)
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
            assert!(!list.is_empty(), "{}", race.id);
        }
        assert_eq!(rules.rare.len(), 2);
        assert_eq!(rules.rare_per_mille, 50);
        assert_eq!(content.spell(rules.touch).id, "evil_eye");
        let cat = content.creature_id("cat").unwrap();
        assert_eq!(rules.arrives(cat), Some("winds about your ankles"));
        assert_eq!(rules.call_turns, 600);
        assert_eq!(rules.bargain_level, 30);
        let imp = content.creature_id("imp").unwrap();
        assert_eq!(rules.bargain_kinds, [imp]);
        assert!(rules.every_kind().contains(&imp) && !rules.every_start().contains(&imp));
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
                "rare = [\"nothing\"]"
            )))
            .contains("`start.rare`: unknown creature"),
        );
        assert!(
            err(familiars(
                |t| t.replace("dwarf = [\"toad\", \"raven\"]", "dwarf = []")
            ))
            .contains("lists at least one"),
        );
        assert!(
            err(familiars(|t| t.replace("ratfolk = ", "elvish = ")))
                .contains("unknown race `elvish`"),
        );
        assert!(
            err(familiars(|t| t.replace(
                "touch = \"evil_eye\"",
                "touch = \"magic_missile\""
            )))
            .contains("no hex spell"),
        );
        assert!(
            err(familiars(|t| t.replace(
                "bargain_kinds = [\"imp\"]",
                "bargain_kinds = [\"nothing\"]"
            )))
            .contains("`bargain_kinds`: unknown creature")
        );
        // Without the file, no familiar at all, and nothing else minds.
        let mut files = ContentFiles::builtin();
        files.remove("data/familiars.toml");
        let content = Content::load(&files).expect("loads without");
        assert!(content.familiars().is_none());
    }
}
