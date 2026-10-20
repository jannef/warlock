//! Item classes (`assets/data/item_classes.toml`, spec §7 "Objects and materials"), ADOM-like:
//! what kind of thing an object is (melee weapons, body armour, food, ...), in the order every
//! item list groups by (spec §6 "Item lists"). Shops sell and buy by class (spec §7 "Shops").
//!
//! Every object names its class (`class` in `data/objects/`). A class lists the magic item kinds
//! its objects may be (`kinds`, [`ItemKind`]), and each kind belongs to one class at most, so an
//! object's class and its kind cannot say different things. Corpses are of the class that
//! takes them (`corpses = true`).

use super::ContentError;
use super::affix::ItemKind;
use std::collections::BTreeSet;

/// Index of a class in [`super::Content::item_classes`], in list order. Classes are not saved
/// (what an object is comes from its definition), so they are not in the save fingerprint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemClassId(pub u8);

/// The most classes there may be.
pub const MAX_ITEM_CLASSES: usize = 64;

/// The built-in classes, for test fixtures that bring their own objects but not their own classes.
#[cfg(test)]
pub(crate) const BUILTIN: &str = include_str!("../../../../assets/data/item_classes.toml");

/// A class as written in `data/item_classes.toml`.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemClassSpec {
    id: String,
    name: String,
    #[serde(default)]
    kinds: Vec<ItemKind>,
    #[serde(default)]
    corpses: bool,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ItemClassFile {
    class: Vec<ItemClassSpec>,
}

/// A resolved class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemClassDef {
    /// What objects and shops name it by: `melee_weapons`.
    pub id: String,
    /// The header item lists show: "Melee weapons".
    pub name: String,
    /// The magic item kinds its objects may be; an object of one of these kinds is of this class.
    pub kinds: Vec<ItemKind>,
    /// Corpses are of this class.
    pub corpses: bool,
}

/// Parses `data/item_classes.toml`.
pub(super) fn parse(file: &str, text: &str) -> Result<Vec<ItemClassDef>, ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: ItemClassFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    if parsed.class.len() > MAX_ITEM_CLASSES {
        return Err(err(format!("at most {MAX_ITEM_CLASSES} classes")));
    }
    let (mut ids, mut kinds) = (BTreeSet::new(), BTreeSet::new());
    let mut defs = Vec::with_capacity(parsed.class.len());
    for spec in parsed.class {
        let bad = |m: &str| err(format!("class `{}`: {m}", spec.id));
        if spec.id.trim().is_empty() || spec.name.trim().is_empty() {
            return Err(bad("`id` and `name` must not be empty"));
        }
        if !ids.insert(spec.id.clone()) {
            return Err(bad("duplicate class id"));
        }
        if let Some(kind) = spec.kinds.iter().find(|k| !kinds.insert(**k)) {
            return Err(bad(&format!(
                "kind `{}` belongs to another class already: each kind is of one class",
                kind.id()
            )));
        }
        defs.push(ItemClassDef {
            id: spec.id,
            name: spec.name,
            kinds: spec.kinds,
            corpses: spec.corpses,
        });
    }
    if defs.iter().filter(|d| d.corpses).count() > 1 {
        return Err(err(
            "only one class may take corpses (`corpses = true`)".into()
        ));
    }
    Ok(defs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_classes_parse_in_order() {
        let text = BUILTIN;
        let classes = parse("item_classes.toml", text).expect("classes");
        assert_eq!(classes[0].id, "melee_weapons");
        assert_eq!(classes[0].name, "Melee weapons");
        assert_eq!(classes.iter().filter(|c| c.corpses).count(), 1);
        // Every magic item kind has its class.
        for kind in ItemKind::ALL {
            assert!(
                classes.iter().any(|c| c.kinds.contains(&kind)),
                "{}",
                kind.id()
            );
        }
    }

    #[test]
    fn every_object_has_its_class_and_kinds_agree() {
        let c = crate::content::Content::builtin().expect("content");
        let class_of = |id: &str| &c.item_class(c.object(c.object_id(id).expect(id)).class).id;
        assert_eq!(class_of("dagger"), "melee_weapons");
        assert_eq!(class_of("sling_stone"), "missiles");
        assert_eq!(class_of("small_shield"), "shields");
        assert_eq!(class_of("kobold_corpse"), "food");
        assert_eq!(class_of("gem"), "gems");
        assert_eq!(class_of("rock"), "miscellaneous");
        for o in c.object_defs() {
            if let Some(kind) = o.kind {
                assert!(c.item_class(o.class).kinds.contains(&kind), "{}", o.id);
            }
        }
    }

    #[test]
    fn corpses_need_a_class_that_takes_them() {
        let mut files = crate::content::ContentFiles::builtin();
        let text = BUILTIN.replace("corpses = true\n", "");
        files.insert("data/item_classes.toml", &text);
        let e = crate::content::Content::load(&files).unwrap_err();
        assert!(e.message.contains("`corpses = true`"), "{}", e.message);
    }

    #[test]
    fn bad_classes_are_refused() {
        for (text, why) in [
            ("[[class]]\nid = \"\"\nname = \"X\"\n", "must not be empty"),
            (
                "[[class]]\nid = \"a\"\nname = \"A\"\n[[class]]\nid = \"a\"\nname = \"B\"\n",
                "duplicate",
            ),
            (
                "[[class]]\nid = \"a\"\nname = \"A\"\nkinds = [\"ring\"]\n\
                 [[class]]\nid = \"b\"\nname = \"B\"\nkinds = [\"amulet\", \"ring\"]\n",
                "kind `ring` belongs to another class",
            ),
            (
                "[[class]]\nid = \"a\"\nname = \"A\"\ncorpses = true\n\
                 [[class]]\nid = \"b\"\nname = \"B\"\ncorpses = true\n",
                "only one class",
            ),
            (
                "[[class]]\nid = \"a\"\nname = \"A\"\nglyph = \")\"\n",
                "unknown field",
            ),
        ] {
            let e = parse("item_classes.toml", text).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
    }
}
