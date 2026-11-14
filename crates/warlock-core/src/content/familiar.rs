//! The witch's familiar's data (`assets/data/familiars.toml`; spec §8 "Allies": "The familiar";
//! docs/design/witch-familiar.md §9.1 and §16; `docs/dev/familiar.md`): the starting beasts by
//! race and the rares, the hex its touch lays, the names, and how each kind arrives. The rules
//! live in `world/allies/familiar.rs`; the kept beasts' growth is `companions.toml`'s, the ally core's
//! constants `allies.toml`. The file is optional: without it no familiar can be made, and nothing
//! here is read.

use super::race_beasts::{self, RaceBeasts, StartSpec};
use super::{Content, ContentError, CreatureId, SpellId, SpellPurpose};
use serde::Deserialize;
use std::collections::BTreeMap;

/// The numbers of the familiar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FamiliarRules {
    /// The starting beasts by race and the rares (`[start]`).
    pub start: RaceBeasts,
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
    start: StartSpec,
    arrives: BTreeMap<String, String>,
}

impl FamiliarRules {
    /// Every beast a start or a calling may draw: [`RaceBeasts::every_start`], then the
    /// bargain's.
    pub fn every_kind(&self) -> Vec<CreatureId> {
        let mut out = self.start.every_start();
        for id in &self.bargain_kinds {
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

    /// Parses `data/familiars.toml` once the creatures, the races, and the spells are known: its
    /// `[start]` as [`RaceBeasts::parse`] checks it, `touch` a hex spell that lays a status (the
    /// touch lays it), the names not empty (the draw indexes them), and the `arrives` lines
    /// naming known creatures.
    pub(super) fn parse(path: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(path, m);
        let file: File = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let bargain_kinds = (file.bargain_kinds.iter())
            .map(|n| race_beasts::beast(path, content, "`bargain_kinds`", n))
            .collect::<Result<Vec<_>, _>>()?;
        let start = RaceBeasts::parse(path, file.start, content)?;
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
        let names = race_beasts::names(path, file.names)?;
        let mut arrives = Vec::new();
        for (name, line) in &file.arrives {
            let id = (content.creature_id(name))
                .ok_or_else(|| err(format!("`arrives`: unknown creature `{name}`")))?;
            arrives.push((id, line.clone()));
        }
        let rules = FamiliarRules {
            start,
            touch,
            nose: file.nose,
            names,
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
        let content = crate::test_support::builtin();
        let rules = content.familiars().expect("familiars.toml");
        for race in &content.races().list {
            let list = rules
                .start
                .list_of(&race.id)
                .unwrap_or_else(|| panic!("{}", race.id));
            assert!(!list.is_empty(), "{}", race.id);
        }
        assert_eq!(rules.start.rare.len(), 2);
        assert_eq!(rules.start.rare_per_mille, 50);
        assert_eq!(content.spell(rules.touch).id, "evil_eye");
        let cat = content.creature_id("cat").unwrap();
        assert_eq!(rules.arrives(cat), Some("winds about your ankles"));
        assert_eq!(rules.call_turns, 600);
        assert_eq!(rules.bargain_level, 30);
        let imp = content.creature_id("imp").unwrap();
        assert_eq!(rules.bargain_kinds, [imp]);
        assert!(rules.every_kind().contains(&imp) && !rules.start.every_start().contains(&imp));
        // The old craft's five are all drawn somewhere.
        for kind in ["cat", "toad", "raven", "hare", "bat"] {
            let id = content.creature_id(kind).unwrap();
            assert!(rules.start.every_start().contains(&id), "{kind}");
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
