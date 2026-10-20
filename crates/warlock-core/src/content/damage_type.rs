//! Damage types (`assets/data/damage_types.toml`, spec §8 "Damage types"): what kind of harm a
//! blow, a missile, a trap, or a spell does, as data names them: the physical subtypes
//! (slashing, piercing, bludgeoning) and the elements (fire, cold, electricity, acid, poison).
//!
//! The list is ordered: a [`DamageTypeId`] is an index into it, in the content fingerprint, so
//! a new type is a new `[[type]]` entry (never a reordering). Every type has an `id`, a `name`
//! (as the player reads it: "fire"), whether it is `physical` (Protection soaks only physical
//! damage), and the verbs its blows are said with (`verb` "burns", `you` "burn"). Exactly one
//! type may be marked `unarmed = true`: bare hands, kicks, and shield bashes do that kind.
//!
//! Without the file nothing is typed: weapons and attacks name no type, nobody resists
//! anything, and every blow is soaked and wounds as before.

use super::ContentError;
use serde::{Deserialize, Serialize};

/// Index of a damage type in [`super::Content::damage_types`], in file order (in the content
/// fingerprint).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DamageTypeId(pub u8);

/// The most damage types a content set may define (a resistance table is one value per type).
pub const MAX_DAMAGE_TYPES: usize = 32;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DamageTypeFile {
    #[serde(rename = "type")]
    types: Vec<DamageTypeSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DamageTypeSpec {
    id: String,
    name: String,
    #[serde(default)]
    physical: bool,
    verb: String,
    you: String,
    #[serde(default)]
    unarmed: bool,
}

/// A damage type, in file order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DamageTypeDef {
    pub id: String,
    /// As the player reads it: "slashing", "fire".
    pub name: String,
    /// Whether Protection soaks it (the physical subtypes); elements ignore armour.
    pub physical: bool,
    /// What a blow of it does to its target: "slashes", "burns" ("The kobold slashes you.").
    pub verb: String,
    /// The same in the player's person: "slash", "burn" ("You slash the kobold.").
    pub you: String,
    /// Whether bare hands, kicks, and shield bashes do this kind (one type at most).
    pub unarmed: bool,
}

/// Parses `data/damage_types.toml`: at most [`MAX_DAMAGE_TYPES`] types with unique ids, each
/// with its words, at most one of them `unarmed`.
pub(super) fn parse(file: &str, text: &str) -> Result<Vec<DamageTypeDef>, ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: DamageTypeFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    if parsed.types.len() > MAX_DAMAGE_TYPES {
        return Err(err(format!("at most {MAX_DAMAGE_TYPES} damage types")));
    }
    let mut list: Vec<DamageTypeDef> = Vec::new();
    for spec in parsed.types {
        let id = spec.id.clone();
        let err = |m: &str| err(format!("damage type `{id}`: {m}"));
        if id.trim().is_empty() {
            return Err(err("`id` must not be empty"));
        }
        if list.iter().any(|t| t.id == id) {
            return Err(err("duplicate id"));
        }
        if [&spec.name, &spec.verb, &spec.you]
            .iter()
            .any(|s| s.trim().is_empty())
        {
            return Err(err("`name`, `verb`, and `you` must not be empty"));
        }
        if spec.unarmed && list.iter().any(|t| t.unarmed) {
            return Err(err("only one damage type is `unarmed`"));
        }
        list.push(DamageTypeDef {
            id: spec.id,
            name: spec.name,
            physical: spec.physical,
            verb: spec.verb,
            you: spec.you,
            unarmed: spec.unarmed,
        });
    }
    Ok(list)
}

/// A damage type named in data (`damage_type = "fire"`), or a few of them (`["slashing",
/// "piercing"]`: a weapon that does whichever is better against its target).
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum TypeNames {
    One(String),
    Many(Vec<String>),
}

impl TypeNames {
    pub(crate) fn names(&self) -> &[String] {
        match self {
            TypeNames::One(one) => std::slice::from_ref(one),
            TypeNames::Many(many) => many,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_parse_in_order_with_their_words() {
        let text = "[[type]]\nid = \"slashing\"\nname = \"slashing\"\nphysical = true\n\
                    verb = \"slashes\"\nyou = \"slash\"\n\
                    [[type]]\nid = \"fire\"\nname = \"fire\"\nverb = \"burns\"\nyou = \"burn\"\n\
                    [[type]]\nid = \"bludgeoning\"\nname = \"bludgeoning\"\nphysical = true\n\
                    verb = \"strikes\"\nyou = \"strike\"\nunarmed = true\n";
        let types = parse("t.toml", text).expect("parses");
        assert_eq!(types.len(), 3);
        assert!(types[0].physical && !types[1].physical);
        assert_eq!(
            (types[1].verb.as_str(), types[1].you.as_str()),
            ("burns", "burn")
        );
        assert!(types[2].unarmed && !types[0].unarmed);
    }

    #[test]
    fn bad_type_files_are_refused() {
        let one = "[[type]]\nid = \"fire\"\nname = \"fire\"\nverb = \"burns\"\nyou = \"burn\"\n";
        for (bad, why) in [
            (format!("{one}{one}"), "duplicate id"),
            (
                "[[type]]\nid = \"fire\"\nname = \"\"\nverb = \"burns\"\nyou = \"burn\"\n".into(),
                "must not be empty",
            ),
            (
                format!(
                    "{one}[[type]]\nid = \"cold\"\nname = \"cold\"\nverb = \"chills\"\n\
                     you = \"chill\"\nunarmed = true\n[[type]]\nid = \"acid\"\nname = \"acid\"\n\
                     verb = \"burns\"\nyou = \"burn\"\nunarmed = true\n"
                ),
                "only one damage type is `unarmed`",
            ),
        ] {
            let error = parse("t.toml", &bad).expect_err(why);
            assert!(error.message.contains(why), "{error}");
        }
    }
}
