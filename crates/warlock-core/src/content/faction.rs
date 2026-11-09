//! Factions (`assets/data/factions.toml`, spec §7): who is hostile to whom.
//!
//! Every creature type belongs to a faction, and the player to the faction [`PLAYER_FACTION`].
//! Relations are pairwise and symmetric: if faction A lists B as `hostile`, A and B are hostile
//! to each other. Every pair that is not listed (a faction and itself included) is neutral. A
//! faction with `hostile_to_all` is hostile to every faction and to itself: friendly to nothing
//! (the berserk risen, `world/corpse_spell.rs`, and a backfired summon's creatures,
//! `world/backfire.rs`). A faction with `hostile_to_others` is hostile to
//! every other faction but neutral to itself: loyal to its own and to nothing else (the loosed,
//! what a miscast lets through; spec §8 "Spells"). A faction with `friendly_to_all` is neutral to
//! every faction but those its own `hostile` names, whatever theirs says: the other two flags
//! pass it by (the gates a miscast opens, at peace with every monster and hostile to the player
//! and the player's side alone; #1316).
//!
//! The file is optional: without it there are no factions and everyone is neutral to everyone.

use super::MaterialId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The faction the player belongs to; required when factions are defined.
pub const PLAYER_FACTION: &str = "player";

/// Index of a faction in [`Factions`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FactionId(pub u16);

/// How two factions (or two actors) stand toward each other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stance {
    /// They fight each other on sight.
    Hostile,
    /// They leave each other alone.
    Neutral,
}

/// A faction as written in `data/factions.toml`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FactionSpec {
    pub id: String,
    pub name: String,
    /// Factions this one is hostile to (and that are hostile to it).
    #[serde(default)]
    pub hostile: Vec<String>,
    /// Animals (Beast-friend's, spec §8 "Talents").
    #[serde(default)]
    pub animal: bool,
    /// The faction one of its creatures joins when the player attacks it (the Seelie court's
    /// wrath, spec §7 "Factions"); none: it stays.
    #[serde(default)]
    pub provoked: Option<String>,
    /// Hostile to every faction, itself too.
    #[serde(default)]
    pub hostile_to_all: bool,
    /// Hostile to every other faction, neutral to itself.
    #[serde(default)]
    pub hostile_to_others: bool,
    /// Neutral to every faction but its own `hostile` list, a `hostile_to_all` or
    /// `hostile_to_others` one too.
    #[serde(default)]
    pub friendly_to_all: bool,
    /// The materials its creatures favour for their gear (#1191): a material's percent of the
    /// draws for an item that may be made of it, `gear = { bone = 40, iron = 25 }`
    /// ([`FactionDef::gear`]).
    #[serde(default)]
    pub gear: BTreeMap<String, u8>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FactionFile {
    pub faction: Vec<FactionSpec>,
}

/// A loaded faction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactionDef {
    pub id: String,
    pub name: String,
    /// Its creatures are animals: a Beast-friend's neutral friends (spec §8 "Talents").
    pub animal: bool,
    /// The faction one of its creatures joins when the player attacks it
    /// (`World::provoke_court`); none: it stays.
    pub provoked: Option<FactionId>,
    /// Hostile to every faction, itself too.
    pub hostile_to_all: bool,
    /// Hostile to every other faction, neutral to itself.
    pub hostile_to_others: bool,
    /// Neutral to every faction but those it lists hostile, whatever theirs says.
    pub friendly_to_all: bool,
    /// The materials its creatures favour for their gear and drops (#1191), by material id,
    /// each with its percent of the draws for an item whose main part may be made of it
    /// ([`crate::variant::weights`]); together at most 100. Resolved once the materials are
    /// loaded ([`Factions::resolve_gear`]).
    pub gear: Vec<(MaterialId, u8)>,
    gear_names: BTreeMap<String, u8>,
}

impl FactionDef {
    /// Hostile to every other faction (`hostile_to_all` or `hostile_to_others`).
    pub fn hates_everyone(&self) -> bool {
        self.hostile_to_all || self.hostile_to_others
    }
}

/// Every faction and the hostility between them.
#[derive(Clone, Debug, Default)]
pub struct Factions {
    defs: Vec<FactionDef>,
    ids: BTreeMap<String, FactionId>,
    /// Row-major `defs.len()²` matrix, symmetric, false on the diagonal but a `hostile_to_all`
    /// faction's; a `hostile_to_all` or `hostile_to_others` faction's row and column all true
    /// (but the latter's own diagonal cell, and a `friendly_to_all` faction's cells).
    hostile: Vec<bool>,
}

impl Factions {
    /// Resolves the factions of `data/factions.toml`: unique ids, known hostile and provoked ids,
    /// and a [`PLAYER_FACTION`].
    pub(super) fn resolve(specs: Vec<FactionSpec>) -> Result<Self, String> {
        let mut ids = BTreeMap::new();
        for (i, spec) in specs.iter().enumerate() {
            let id = FactionId(u16::try_from(i).map_err(|_| "too many factions".to_string())?);
            if ids.insert(spec.id.clone(), id).is_some() {
                return Err(format!("duplicate faction id `{}`", spec.id));
            }
        }
        if !ids.contains_key(PLAYER_FACTION) {
            return Err(format!(
                "a faction with id `{PLAYER_FACTION}` (the player's) is required"
            ));
        }
        if let Some(spec) =
            (specs.iter()).find(|s| s.friendly_to_all && (s.hostile_to_all || s.hostile_to_others))
        {
            return Err(format!(
                "faction `{}`: `friendly_to_all` goes with neither `hostile_to_all` nor `hostile_to_others`",
                spec.id
            ));
        }
        let n = specs.len();
        let mut hostile = vec![false; n * n];
        for (a, spec) in specs.iter().enumerate() {
            for other in &spec.hostile {
                let Some(b) = ids.get(other) else {
                    return Err(format!(
                        "faction `{}` is hostile to unknown faction `{other}`",
                        spec.id
                    ));
                };
                let b = usize::from(b.0);
                hostile[a * n + b] = true;
                hostile[b * n + a] = true;
            }
            if spec.hostile_to_all || spec.hostile_to_others {
                let at_war =
                    |b: usize| (b != a || spec.hostile_to_all) && !specs[b].friendly_to_all;
                for b in (0..n).filter(|&b| at_war(b)) {
                    hostile[a * n + b] = true;
                    hostile[b * n + a] = true;
                }
            }
        }
        let mut defs = Vec::with_capacity(n);
        for s in specs {
            let provoked = match &s.provoked {
                None => None,
                Some(other) => Some(*ids.get(other).ok_or_else(|| {
                    format!(
                        "faction `{}` is provoked into unknown faction `{other}`",
                        s.id
                    )
                })?),
            };
            defs.push(FactionDef {
                id: s.id,
                name: s.name,
                animal: s.animal,
                provoked,
                hostile_to_all: s.hostile_to_all,
                hostile_to_others: s.hostile_to_others,
                friendly_to_all: s.friendly_to_all,
                gear: Vec::new(),
                gear_names: s.gear,
            });
        }
        Ok(Self { defs, ids, hostile })
    }

    /// Resolves each faction's `gear` by `material` (a material's id and whether it has a
    /// group, which is what an item of another material may be made of): known materials, together
    /// at most 100 percent (each one's share of the draw is out of the whole, `variant.rs`).
    pub(super) fn resolve_gear(
        &mut self,
        material: impl Fn(&str) -> Option<MaterialId>,
    ) -> Result<(), String> {
        for def in &mut self.defs {
            let mut gear = Vec::with_capacity(def.gear_names.len());
            for (name, &percent) in &def.gear_names {
                let Some(id) = material(name) else {
                    return Err(format!(
                        "faction `{}`: `gear` names unknown material `{name}`",
                        def.id
                    ));
                };
                gear.push((id, percent));
            }
            if gear.iter().map(|(_, p)| u32::from(*p)).sum::<u32>() > 100 {
                return Err(format!(
                    "faction `{}`: `gear` is at most 100 percent in all",
                    def.id
                ));
            }
            gear.sort_unstable();
            def.gear = gear;
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    pub fn id(&self, name: &str) -> Option<FactionId> {
        self.ids.get(name).copied()
    }

    pub fn def(&self, id: FactionId) -> &FactionDef {
        &self.defs[usize::from(id.0)]
    }

    /// Every faction in file order.
    pub fn defs(&self) -> &[FactionDef] {
        &self.defs
    }

    /// Whether faction `a` is hostile to anyone at all, by its own quarrels: being on a
    /// `hostile_to_all` or `hostile_to_others` faction's list is no quarrel of its own, or no one
    /// would be peaceable.
    pub fn has_enemies(&self, a: FactionId) -> bool {
        let n = self.defs.len();
        let a = usize::from(a.0);
        a < n
            && (self.defs[a].hates_everyone()
                || (0..n).any(|b| self.hostile[a * n + b] && !self.defs[b].hates_everyone()))
    }

    /// How factions `a` and `b` stand toward each other; symmetric.
    pub fn stance(&self, a: FactionId, b: FactionId) -> Stance {
        let n = self.defs.len();
        let (a, b) = (usize::from(a.0), usize::from(b.0));
        if a < n && b < n && self.hostile[a * n + b] {
            Stance::Hostile
        } else {
            Stance::Neutral
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostile_to_all_is_friendly_to_nothing_and_peace_stays_peace() {
        let f = resolve(
            r#"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "villagers"
name = "villagers"

[[faction]]
id = "berserk"
name = "the berserk"
hostile_to_all = true
"#,
        )
        .expect("valid");
        let id = |n: &str| f.id(n).expect("known");
        for other in ["player", "villagers", "berserk"] {
            assert_eq!(
                f.stance(id("berserk"), id(other)),
                Stance::Hostile,
                "{other}"
            );
            assert_eq!(
                f.stance(id(other), id("berserk")),
                Stance::Hostile,
                "{other}"
            );
        }
        assert_eq!(f.stance(id("player"), id("villagers")), Stance::Neutral);
        assert!(f.has_enemies(id("berserk")));
        assert!(!f.has_enemies(id("villagers")), "still peaceable");
        assert!(f.def(id("berserk")).hostile_to_all);
    }

    #[test]
    fn hostile_to_others_is_loyal_to_its_own_alone() {
        let f = resolve(
            r#"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "villagers"
name = "villagers"

[[faction]]
id = "demons"
name = "demons"
hostile = ["player"]

[[faction]]
id = "loosed"
name = "the loosed"
hostile_to_others = true

[[faction]]
id = "berserk"
name = "the berserk"
hostile_to_all = true
"#,
        )
        .expect("valid");
        let id = |n: &str| f.id(n).expect("known");
        let loosed = id("loosed");
        assert_eq!(
            f.stance(loosed, loosed),
            Stance::Neutral,
            "loyal to its own"
        );
        for other in ["player", "villagers", "demons", "berserk"] {
            assert_eq!(f.stance(loosed, id(other)), Stance::Hostile, "{other}");
            assert_eq!(f.stance(id(other), loosed), Stance::Hostile, "{other}");
        }
        // The rest of the matrix is as their own lists make it.
        assert_eq!(f.stance(id("demons"), id("player")), Stance::Hostile);
        assert_eq!(f.stance(id("villagers"), id("demons")), Stance::Neutral);
        assert_eq!(f.stance(id("berserk"), id("berserk")), Stance::Hostile);
        // Being on the loosed's list makes no one a fighter.
        assert!(f.has_enemies(loosed));
        assert!(!f.has_enemies(id("villagers")), "still peaceable");
        let def = f.def(loosed);
        assert!(def.hostile_to_others && !def.hostile_to_all && def.hates_everyone());
    }

    #[test]
    fn the_shipped_loosed_are_hostile_to_every_other_faction_but_the_gates() {
        let f = resolve(include_str!("../../../../assets/data/factions.toml")).expect("valid");
        let loosed = f.id("loosed").expect("the loosed are shipped");
        assert!(f.def(loosed).hostile_to_others);
        for (i, other) in f.defs().iter().enumerate() {
            let other_id = FactionId(i as u16);
            let expected = if other_id == loosed || other.friendly_to_all {
                Stance::Neutral
            } else {
                Stance::Hostile
            };
            assert_eq!(f.stance(loosed, other_id), expected, "{}", other.id);
        }
    }

    fn resolve(text: &str) -> Result<Factions, String> {
        let file: FactionFile = toml::from_str(text).map_err(|e| e.to_string())?;
        Factions::resolve(file.faction)
    }

    const FILE: &str = r#"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "vermin"
name = "vermin"
hostile = ["player"]

[[faction]]
id = "kobolds"
name = "kobolds"
hostile = ["player", "vermin"]

[[faction]]
id = "wildlife"
name = "wildlife"
"#;

    #[test]
    fn hostility_is_symmetric_and_everything_else_neutral() {
        let f = resolve(FILE).unwrap();
        let id = |n: &str| f.id(n).unwrap();
        let (player, vermin, kobolds, wildlife) =
            (id("player"), id("vermin"), id("kobolds"), id("wildlife"));
        for (a, b) in [(vermin, player), (kobolds, player), (kobolds, vermin)] {
            assert_eq!(f.stance(a, b), Stance::Hostile);
            assert_eq!(f.stance(b, a), Stance::Hostile, "symmetric");
        }
        for other in [player, vermin, kobolds, wildlife] {
            assert_eq!(f.stance(wildlife, other), Stance::Neutral);
            assert_eq!(f.stance(other, other), Stance::Neutral, "own faction");
        }
        assert_eq!(f.def(kobolds).name, "kobolds");
        assert_eq!(f.def(kobolds).provoked, None);
    }

    #[test]
    fn a_provoked_faction_names_another() {
        let text = format!(
            "{FILE}\n[[faction]]\nid = \"seelie\"\nname = \"s\"\nprovoked = \"wrathful\"\n\
             [[faction]]\nid = \"wrathful\"\nname = \"w\"\nhostile = [\"player\"]\n"
        );
        let f = resolve(&text).unwrap();
        let (seelie, wrathful) = (f.id("seelie").unwrap(), f.id("wrathful").unwrap());
        assert_eq!(f.def(seelie).provoked, Some(wrathful));
    }

    #[test]
    fn friendly_to_all_keeps_its_own_quarrels_alone() {
        let f = resolve(
            r#"
[[faction]]
id = "player"
name = "you"

[[faction]]
id = "demons"
name = "demons"
hostile = ["player"]

[[faction]]
id = "loosed"
name = "the loosed"
hostile_to_others = true

[[faction]]
id = "berserk"
name = "the berserk"
hostile_to_all = true

[[faction]]
id = "gates"
name = "the gates"
hostile = ["player"]
friendly_to_all = true
"#,
        )
        .expect("valid");
        let id = |n: &str| f.id(n).expect("known");
        let gates = id("gates");
        assert_eq!(f.stance(gates, id("player")), Stance::Hostile);
        assert_eq!(f.stance(id("player"), gates), Stance::Hostile);
        for other in ["demons", "loosed", "berserk", "gates"] {
            assert_eq!(f.stance(gates, id(other)), Stance::Neutral, "{other}");
            assert_eq!(f.stance(id(other), gates), Stance::Neutral, "{other}");
        }
        // The loosed and the berserk keep their war with everyone else.
        assert_eq!(f.stance(id("loosed"), id("demons")), Stance::Hostile);
        assert_eq!(f.stance(id("berserk"), id("berserk")), Stance::Hostile);
        assert!(f.has_enemies(gates));
        assert!(f.def(gates).friendly_to_all);
        let err = resolve(
            "[[faction]]\nid = \"player\"\nname = \"p\"\n[[faction]]\nid = \"g\"\nname = \"g\"\n\
             friendly_to_all = true\nhostile_to_others = true\n",
        )
        .unwrap_err();
        assert!(err.contains("`friendly_to_all` goes with neither"), "{err}");
    }

    #[test]
    fn bad_factions_are_rejected() {
        let cases = [
            (
                "[[faction]]\nid = \"player\"\nname = \"p\"\nhostile = [\"orcs\"]\n",
                "unknown faction `orcs`",
            ),
            (
                "[[faction]]\nid = \"player\"\nname = \"p\"\n[[faction]]\nid = \"player\"\nname = \"q\"\n",
                "duplicate faction id",
            ),
            (
                "[[faction]]\nid = \"orcs\"\nname = \"orcs\"\n",
                "`player` (the player's) is required",
            ),
            (
                "[[faction]]\nid = \"player\"\nname = \"p\"\nfriendly = []\n",
                "unknown field",
            ),
            (
                "[[faction]]\nid = \"player\"\nname = \"p\"\nprovoked = \"wrath\"\n",
                "provoked into unknown faction `wrath`",
            ),
        ];
        for (text, expected) in cases {
            let err = resolve(text).unwrap_err();
            assert!(err.contains(expected), "{text}: {err}");
        }
    }

    #[test]
    fn gear_names_known_materials_within_a_hundred_percent() {
        let material = |name: &str| match name {
            "bone" => Some(MaterialId(0)),
            "iron" => Some(MaterialId(1)),
            _ => None,
        };
        let with = |gear: &str| {
            let mut f = resolve(&format!(
                "[[faction]]\nid = \"player\"\nname = \"p\"\n\n[[faction]]\nid = \"orcs\"\n\
                 name = \"orcs\"\ngear = {gear}\n"
            ))
            .expect("valid");
            f.resolve_gear(material).map(|()| f)
        };
        let f = with("{ iron = 15, bone = 55 }").expect("valid");
        let orcs = f.def(f.id("orcs").expect("orcs"));
        assert_eq!(orcs.gear, [(MaterialId(0), 55), (MaterialId(1), 15)]);
        assert!(f.def(f.id("player").expect("player")).gear.is_empty());
        for (gear, expected) in [
            ("{ mithril = 10 }", "unknown material `mithril`"),
            ("{ bone = 60, iron = 41 }", "at most 100"),
        ] {
            let err = with(gear).expect_err(gear);
            assert!(err.contains(expected), "{gear}: {err}");
        }
    }
}
