//! Factions (`assets/data/factions.toml`, spec §7): who is hostile to whom.
//!
//! Every creature type belongs to a faction, and the player to the faction [`PLAYER_FACTION`].
//! Relations are pairwise and symmetric: if faction A lists B as `hostile`, A and B are hostile
//! to each other. Every pair that is not listed (a faction and itself included) is neutral.
//!
//! The file is optional: without it there are no factions and everyone is neutral to everyone.

use serde::Deserialize;
use std::collections::BTreeMap;

/// The faction the player belongs to; required when factions are defined.
pub const PLAYER_FACTION: &str = "player";

/// Index of a faction in [`Factions`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
}

/// Every faction and the hostility between them.
#[derive(Clone, Debug, Default)]
pub struct Factions {
    defs: Vec<FactionDef>,
    ids: BTreeMap<String, FactionId>,
    /// Row-major `defs.len()²` matrix, symmetric, false on the diagonal.
    hostile: Vec<bool>,
}

impl Factions {
    /// Resolves and validates the factions of `data/factions.toml`: unique ids, known hostile
    /// ids, no faction hostile to itself, and a [`PLAYER_FACTION`].
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
                if a == b {
                    return Err(format!("faction `{}` cannot be hostile to itself", spec.id));
                }
                hostile[a * n + b] = true;
                hostile[b * n + a] = true;
            }
        }
        let defs = specs
            .into_iter()
            .map(|s| FactionDef {
                id: s.id,
                name: s.name,
            })
            .collect();
        Ok(Self { defs, ids, hostile })
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

    /// Whether faction `a` is hostile to anyone at all.
    pub fn has_enemies(&self, a: FactionId) -> bool {
        let n = self.defs.len();
        let a = usize::from(a.0);
        a < n && self.hostile[a * n..(a + 1) * n].contains(&true)
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
    }

    #[test]
    fn bad_factions_are_rejected() {
        let cases = [
            (
                "[[faction]]\nid = \"player\"\nname = \"p\"\nhostile = [\"orcs\"]\n",
                "unknown faction `orcs`",
            ),
            (
                "[[faction]]\nid = \"player\"\nname = \"p\"\nhostile = [\"player\"]\n",
                "hostile to itself",
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
        ];
        for (text, expected) in cases {
            let err = resolve(text).unwrap_err();
            assert!(err.contains(expected), "{text}: {err}");
        }
    }
}
