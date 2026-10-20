//! Parts (`assets/data/parts.toml`, spec §7 "Objects and materials", #199): what objects are
//! made of. Every object lists its parts in `materials`, each of one material, the first the
//! main one: `materials = [{ part = "chain", of = "gold" }, { part = "setting", of = "ruby",
//! share = 10 }]`; a plain material id is one unnamed part, so `materials = ["steel"]` is an
//! object of one material.
//!
//! The main part decides what a single material decides ([`ObjectDef::material`]): the material
//! a description names first, the material word a name begins with, and (with metals and organic
//! materials, #193 and #194) the glyph colour, the weight, and the weapon and armour modifiers.
//! Every other part names its part and has a share of the object's weight (percent); the main
//! part's share is what the others leave. What an object is worth follows its parts' shares
//! and their materials' values ([`ObjectDef::value_with`]); its name shows the notable parts,
//! gems and precious metals (`precious` materials), that it does not mention yet ([`names`]).
//!
//! Part names are a small vocabulary, so naming (and, with #194, construction rules) can refer
//! to them; a part's `adjective` is how a notable one is written before a name
//! ("ruby-pommelled long sword"). Without `data/parts.toml` there are none, and every object is
//! of one unnamed part, as the shorthand writes it.
//!
//! [`ObjectDef::material`]: super::ObjectDef::material
//! [`ObjectDef::value_with`]: super::ObjectDef::value_with

use super::ContentError;
use super::gem::Cutting;
use super::material::{MaterialDef, MaterialId};
use serde::Deserialize;
use std::collections::BTreeSet;

/// Index of a part in [`super::Content::parts`], in file order. Parts are not saved (an object's
/// parts come from its definition), so they are not in the save fingerprint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PartId(pub u8);

/// The most part names there may be.
pub const MAX_PARTS: usize = 64;
/// The most parts one object may have.
pub const MAX_OBJECT_PARTS: usize = 8;

/// A part as written in `data/parts.toml`.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PartSpec {
    id: String,
    name: String,
    adjective: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PartFile {
    #[serde(default)]
    part: Vec<PartSpec>,
}

/// A part name: what a part of an object is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PartDef {
    /// What objects name it by in their `materials`: `pommel`.
    pub id: String,
    /// How descriptions name it: "pommel" ("ruby pommel").
    pub name: String,
    /// How a notable part of this kind is written before an object's name, after its material
    /// and a hyphen: "pommelled" ("ruby-pommelled long sword"). `None`: its material joins the
    /// material the name begins with ("gold and ruby amulet").
    pub adjective: Option<String>,
}

/// Parses `data/parts.toml`.
pub(super) fn parse(file: &str, text: &str) -> Result<Vec<PartDef>, ContentError> {
    let err = |m: String| ContentError::new(file, m);
    let parsed: PartFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
    if parsed.part.len() > MAX_PARTS {
        return Err(err(format!("at most {MAX_PARTS} parts")));
    }
    let mut ids = BTreeSet::new();
    let mut defs = Vec::with_capacity(parsed.part.len());
    for spec in parsed.part {
        let bad = |m: &str| err(format!("part `{}`: {m}", spec.id));
        let not_a_word = |s: &str| s.trim().is_empty() || s.contains(char::is_whitespace);
        if spec.id.trim().is_empty() || spec.name.trim().is_empty() {
            return Err(bad("`id` and `name` must not be empty"));
        }
        if spec.adjective.as_deref().is_some_and(not_a_word) {
            return Err(bad("`adjective` is one word, not empty"));
        }
        if !ids.insert(spec.id.clone()) {
            return Err(bad("duplicate part id"));
        }
        defs.push(PartDef {
            id: spec.id,
            name: spec.name,
            adjective: spec.adjective,
        });
    }
    Ok(defs)
}

/// One part of an object as written in its `materials`: a material id (an unnamed part, the
/// shorthand) or a table.
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum PartEntry {
    /// `"steel"`: an unnamed part of that material.
    Material(String),
    /// `{ part = "setting", of = "ruby", share = 10 }`.
    Part(PartTable),
}

/// A part written as a table.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PartTable {
    /// What part it is: a part id in `data/parts.toml`.
    part: Option<String>,
    /// What it is made of: a material id.
    of: String,
    /// Its share of the object's weight, in percent (not for the main part).
    share: Option<u32>,
}

/// One part of an object: what part it is, what it is made of, and how much of the object it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Part {
    /// What part it is (a blade, a setting); `None` for an unnamed part (a main part written as
    /// a plain material id).
    pub part: Option<PartId>,
    pub material: MaterialId,
    /// Its share of the object's weight, in percent; an object's parts' shares sum to 100. A
    /// part an item has and its base item does not (a gem it was set with) has none: it weighs
    /// nothing beside the rest.
    pub share: u8,
    /// A gem's size and cut (spec §7 "Gems"): an item's own gem part (a loose gem, an
    /// ornament, [`crate::object::Stack::parts`]); never a base item's.
    pub gem: Option<Cutting>,
}

impl Part {
    /// An object's only part, unnamed: all of it is `material`.
    pub fn whole(material: MaterialId) -> Self {
        Self {
            part: None,
            material,
            share: 100,
            gem: None,
        }
    }
}

/// Resolves an object's `materials`: the first entry is the main part, which takes the share the
/// others leave; every other part names its `part` and gives its `share`, and the others' shares
/// leave the main part some. `material` and `part_id` look ids up.
pub(super) fn resolve(
    entries: &[PartEntry],
    material: impl Fn(&str) -> Option<MaterialId>,
    part_id: impl Fn(&str) -> Option<PartId>,
) -> Result<Vec<Part>, String> {
    if entries.is_empty() {
        return Err("`materials` lists its parts, at least one: `materials = [\"steel\"]`".into());
    }
    if entries.len() > MAX_OBJECT_PARTS {
        return Err(format!("at most {MAX_OBJECT_PARTS} parts in `materials`"));
    }
    let known = |id: &str| material(id).ok_or_else(|| format!("unknown material `{id}`"));
    let mut parts = Vec::with_capacity(entries.len());
    let mut named = BTreeSet::new();
    let mut others = 0;
    for (i, entry) in entries.iter().enumerate() {
        let (name, of, share) = match entry {
            PartEntry::Material(of) => (None, of, None),
            PartEntry::Part(t) => (t.part.as_deref(), &t.of, t.share),
        };
        let part = match name {
            Some(id) => {
                let found =
                    part_id(id).ok_or_else(|| format!("unknown part `{id}` (data/parts.toml)"))?;
                if !named.insert(found) {
                    return Err(format!("part `{id}` twice in `materials`"));
                }
                Some(found)
            }
            None if i > 0 => {
                return Err(format!(
                    "`{of}`: every part but the first (the main one) names its part and share: \
                     `{{ part = \"grip\", of = \"{of}\", share = 10 }}`"
                ));
            }
            None => None,
        };
        let share = match (i, share) {
            (0, Some(_)) => {
                return Err(
                    "the first part is the main one and takes the share the others leave: give \
                     only the others a `share`"
                        .into(),
                );
            }
            (0, None) => 0,
            (_, None) => {
                let name = name.unwrap_or(of);
                return Err(format!(
                    "part `{name}` needs a `share` (percent of the weight)"
                ));
            }
            (_, Some(share)) if !(1..=99).contains(&share) => {
                return Err(format!("`share = {share}`: a share is 1 to 99 percent"));
            }
            (_, Some(share)) => share,
        };
        others += share;
        parts.push(Part {
            part,
            material: known(of)?,
            share: share as u8,
            gem: None,
        });
    }
    if others >= 100 {
        return Err(format!(
            "the parts' shares sum to {others}%: they must leave the main part some"
        ));
    }
    parts[0].share = (100 - others) as u8;
    Ok(parts)
}

/// What `parts` are worth together, relative to one another: each part's share times its
/// material's value (percent).
pub fn worth(parts: &[Part], materials: &[MaterialDef]) -> u64 {
    (parts.iter())
        .map(|p| u64::from(p.share) * u64::from(materials[usize::from(p.material.0)].value))
        .sum()
}

/// `parts` in words, for an item's description: every part, the main one first, each as its
/// material and part ("steel blade, leather grip"), an unnamed part as its material ("steel").
pub fn phrase(parts: &[Part], materials: &[MaterialDef], names: &[PartDef]) -> String {
    let words: Vec<String> = (parts.iter())
        .map(|p| {
            let material = &materials[usize::from(p.material.0)].name;
            match p.part {
                Some(part) => format!("{material} {}", names[usize::from(part.0)].name),
                None => material.clone(),
            }
        })
        .collect();
    words.join(", ")
}

/// What an object written as `name` (and `plural`) is called, made of `parts`: its name with the
/// notable parts it does not mention yet (parts of a `precious` material: gems and precious
/// metals; spec §7 "Objects and materials"). The plain parts (a leather grip) show only in its
/// description ([`phrase`]).
///
/// A notable part whose part has an `adjective` goes before the name, "ruby-pommelled long
/// sword"; any other notable part's material joins the main part's material if the name begins
/// with it ("gold amulet", a ruby setting: "gold and ruby amulet"; "iron and ruby ring"), else
/// goes before the name ("gold and ruby amulet" from "amulet"). A material the name mentions as
/// a word ("gold ring", "ruby") is not said again.
pub fn names(
    name: &str,
    plural: &str,
    parts: &[Part],
    materials: &[MaterialDef],
    kinds: &[PartDef],
) -> (String, String) {
    let material = |p: &Part| &materials[usize::from(p.material.0)];
    // Whole words only: "gold" is in "gold ring" and "gold-shod boots", not in "golden".
    let words = format!(" {} ", name.replace('-', " "));
    let mentioned = |m: &str| words.contains(&format!(" {m} "));
    let (mut before, mut joined) = (Vec::new(), Vec::new());
    let mut said = BTreeSet::new();
    for part in parts {
        let m = material(part);
        if !m.precious || mentioned(&m.name) || !said.insert(m.name.as_str()) {
            continue;
        }
        match part
            .part
            .and_then(|p| kinds[usize::from(p.0)].adjective.as_ref())
        {
            Some(adjective) => before.push(format!("{}-{adjective}", m.name)),
            None => joined.push(m.name.as_str()),
        }
    }
    let main = parts.first().map(|p| material(p).name.as_str());
    let call = |written: &str| {
        let rest = if joined.is_empty() {
            written.to_string()
        } else {
            let begins = main.and_then(|m| Some((m, written.strip_prefix(m)?.strip_prefix(' ')?)));
            let (mut list, rest) = match begins {
                Some((main, rest)) => (vec![main], rest),
                None => (Vec::new(), written),
            };
            list.extend(&joined);
            format!("{} {rest}", and_list(&list))
        };
        before
            .iter()
            .map(String::as_str)
            .chain([rest.as_str()])
            .collect::<Vec<_>>()
            .join(" ")
    };
    (call(name), call(plural))
}

/// Words in a list: "gold", "gold and ruby", "gold, ruby, and emerald".
fn and_list(words: &[&str]) -> String {
    match words {
        [] => String::new(),
        [one] => (*one).to_string(),
        [a, b] => format!("{a} and {b}"),
        [init @ .., last] => format!("{}, and {last}", init.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn material(id: &str, precious: bool, value: u32) -> MaterialDef {
        MaterialDef {
            id: id.into(),
            name: id.into(),
            precious,
            value,
            gem: None,
        }
    }

    /// steel, leather, gold, ruby, iron, silver.
    fn materials() -> Vec<MaterialDef> {
        vec![
            material("steel", false, 100),
            material("leather", false, 50),
            material("gold", true, 2000),
            material("ruby", true, 5000),
            material("iron", false, 60),
            material("silver", true, 1500),
        ]
    }

    /// blade, grip, pommel (pommelled), chain, setting, band, inlay (inlaid).
    fn kinds() -> Vec<PartDef> {
        let part = |id: &str, adjective: Option<&str>| PartDef {
            id: id.into(),
            name: id.into(),
            adjective: adjective.map(str::to_string),
        };
        vec![
            part("blade", None),
            part("grip", None),
            part("pommel", Some("pommelled")),
            part("chain", None),
            part("setting", None),
            part("band", None),
            part("inlay", Some("inlaid")),
        ]
    }

    fn parse_parts(text: &str) -> Result<Vec<Part>, String> {
        #[derive(Deserialize)]
        struct Object {
            materials: Vec<PartEntry>,
        }
        let object: Object = toml::from_str(text).map_err(|e| e.to_string())?;
        let (materials, kinds) = (materials(), kinds());
        resolve(
            &object.materials,
            |id| (materials.iter().position(|m| m.id == id)).map(|i| MaterialId(i as u16)),
            |id| (kinds.iter().position(|k| k.id == id)).map(|i| PartId(i as u8)),
        )
    }

    fn named(name: &str, plural: &str, text: &str) -> (String, String) {
        let parts = parse_parts(text).expect("parts");
        names(name, plural, &parts, &materials(), &kinds())
    }

    #[test]
    fn the_shorthand_is_one_unnamed_part_each() {
        assert_eq!(
            parse_parts(r#"materials = ["steel"]"#),
            Ok(vec![Part::whole(MaterialId(0))])
        );
        let parts =
            parse_parts(r#"materials = ["gold", { part = "setting", of = "ruby", share = 10 }]"#)
                .expect("parts");
        assert_eq!(
            parts,
            vec![
                Part {
                    part: None,
                    material: MaterialId(2),
                    share: 90,
                    gem: None,
                },
                Part {
                    part: Some(PartId(4)),
                    material: MaterialId(3),
                    share: 10,
                    gem: None,
                },
            ]
        );
    }

    #[test]
    fn the_main_part_takes_what_the_others_leave() {
        let parts = parse_parts(
            r#"materials = [
                { part = "blade", of = "steel" },
                { part = "grip", of = "leather", share = 15 },
                { part = "pommel", of = "ruby", share = 5 },
            ]"#,
        )
        .expect("parts");
        let shares: Vec<u8> = parts.iter().map(|p| p.share).collect();
        assert_eq!(shares, [80, 15, 5]);
        assert_eq!(parts[0].part, Some(PartId(0)));
    }

    #[test]
    fn bad_parts_are_reported() {
        for (text, expected) in [
            ("materials = []", "at least one"),
            (r#"materials = ["bronze"]"#, "unknown material `bronze`"),
            (
                r#"materials = [{ part = "hilt", of = "steel" }]"#,
                "unknown part `hilt`",
            ),
            (
                r#"materials = [{ part = "blade", of = "steel", share = 90 }]"#,
                "the first part is the main one",
            ),
            (
                r#"materials = ["steel", "leather"]"#,
                "every part but the first",
            ),
            (
                r#"materials = ["steel", { of = "leather", share = 10 }]"#,
                "every part but the first",
            ),
            (
                r#"materials = ["steel", { part = "grip", of = "leather" }]"#,
                "needs a `share`",
            ),
            (
                r#"materials = ["steel", { part = "grip", of = "leather", share = 0 }]"#,
                "1 to 99 percent",
            ),
            (
                r#"materials = ["steel", { part = "grip", of = "leather", share = 60 },
                    { part = "pommel", of = "ruby", share = 40 }]"#,
                "sum to 100%",
            ),
            (
                r#"materials = ["steel", { part = "grip", of = "leather", share = 5 },
                    { part = "grip", of = "iron", share = 5 }]"#,
                "part `grip` twice",
            ),
            (
                r#"materials = ["steel", { part = "grip", of = "leather", share = 5, size = 2 }]"#,
                "did not match",
            ),
        ] {
            let err = parse_parts(text).expect_err(text);
            assert!(err.contains(expected), "{text}: {err}");
        }
    }

    #[test]
    fn names_show_gems_and_precious_metals_only() {
        let sword = r#"materials = [{ part = "blade", of = "steel" },
            { part = "grip", of = "leather", share = 10 }]"#;
        assert_eq!(
            named("long sword", "long swords", sword),
            ("long sword".into(), "long swords".into()),
            "a leather grip shows only in the description"
        );
        let pommelled = r#"materials = [{ part = "blade", of = "steel" },
            { part = "grip", of = "leather", share = 10 },
            { part = "pommel", of = "ruby", share = 5 }]"#;
        assert_eq!(
            named("long sword", "long swords", pommelled),
            (
                "ruby-pommelled long sword".into(),
                "ruby-pommelled long swords".into()
            )
        );
        let amulet = r#"materials = [{ part = "chain", of = "gold" },
            { part = "setting", of = "ruby", share = 10 }]"#;
        assert_eq!(
            named("gold amulet", "gold amulets", amulet),
            (
                "gold and ruby amulet".into(),
                "gold and ruby amulets".into()
            ),
            "the ruby joins the main material the name begins with"
        );
        assert_eq!(
            named("amulet", "amulets", amulet).0,
            "gold and ruby amulet",
            "a name without its materials gets them all"
        );
        let ring = r#"materials = [{ part = "band", of = "iron" },
            { part = "setting", of = "ruby", share = 5 }]"#;
        assert_eq!(
            named("iron ring", "iron rings", ring).0,
            "iron and ruby ring"
        );
        let three = r#"materials = [{ part = "chain", of = "gold" },
            { part = "setting", of = "ruby", share = 10 },
            { part = "inlay", of = "silver", share = 5 }]"#;
        assert_eq!(
            named("gold amulet", "gold amulets", three).0,
            "silver-inlaid gold and ruby amulet"
        );
        let four = r#"materials = ["silver", { part = "chain", of = "gold", share = 10 },
            { part = "setting", of = "ruby", share = 10 }]"#;
        assert_eq!(
            named("amulet", "amulets", four).0,
            "silver, gold, and ruby amulet"
        );
    }

    #[test]
    fn a_material_the_name_mentions_is_not_said_again() {
        assert_eq!(
            named("ruby", "rubies", r#"materials = ["ruby"]"#),
            ("ruby".into(), "rubies".into())
        );
        assert_eq!(
            named("gold coin", "gold coins", r#"materials = ["gold"]"#),
            ("gold coin".into(), "gold coins".into())
        );
        let hyphen = r#"materials = ["leather", { part = "inlay", of = "silver", share = 5 }]"#;
        assert_eq!(
            named(
                "pair of silver-shod boots",
                "pairs of silver-shod boots",
                hyphen
            )
            .0,
            "pair of silver-shod boots"
        );
    }

    #[test]
    fn phrases_list_every_part() {
        let parts = parse_parts(
            r#"materials = [{ part = "blade", of = "steel" },
                { part = "grip", of = "leather", share = 10 }]"#,
        )
        .expect("parts");
        assert_eq!(
            phrase(&parts, &materials(), &kinds()),
            "steel blade, leather grip"
        );
        let one = parse_parts(r#"materials = ["iron"]"#).expect("parts");
        assert_eq!(phrase(&one, &materials(), &kinds()), "iron");
    }

    #[test]
    fn worth_weighs_each_material_by_its_share() {
        let materials = materials();
        let parts = parse_parts(
            r#"materials = [{ part = "chain", of = "gold" },
                { part = "setting", of = "ruby", share = 10 }]"#,
        )
        .expect("parts");
        assert_eq!(worth(&parts, &materials), 90 * 2000 + 10 * 5000);
    }

    #[test]
    fn part_names_are_checked() {
        let defs = parse(
            "data/parts.toml",
            "[[part]]\nid = \"pommel\"\nname = \"pommel\"\nadjective = \"pommelled\"\n",
        )
        .expect("parts");
        assert_eq!(defs[0].adjective.as_deref(), Some("pommelled"));
        for (text, expected) in [
            ("[[part]]\nid = \"a\"\nname = \"\"\n", "must not be empty"),
            (
                "[[part]]\nid = \"a\"\nname = \"a\"\nadjective = \"well set\"\n",
                "one word",
            ),
            (
                "[[part]]\nid = \"a\"\nname = \"a\"\n[[part]]\nid = \"a\"\nname = \"b\"\n",
                "duplicate part id",
            ),
        ] {
            let err = parse("data/parts.toml", text).expect_err(text);
            assert!(err.message.contains(expected), "{text}: {}", err.message);
        }
    }
}
