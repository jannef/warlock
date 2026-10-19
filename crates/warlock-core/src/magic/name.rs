//! What magic items are called (spec §8 "Magic items"):
//!
//! - normal: "a long sword";
//! - magic: its prefix and suffix around the base name, "a Sharp long sword of the Bear";
//! - rare: its name, then the base, "Grim Bite, a long sword";
//! - unidentified: the base and the rarity, "a long sword (magic)", "12 arrows (rare)".

use super::{AffixKind, Magic, Rarity};
use crate::content::{Content, ObjectDef, article};

/// `count` of `def` with `magic` in words, as the player knows it: with `known` false (not
/// identified) only its rarity shows.
pub(crate) fn describe(
    def: &ObjectDef,
    count: u32,
    magic: Option<&Magic>,
    known: bool,
    content: &Content,
) -> String {
    let Some(magic) = magic else {
        return def.describe(count);
    };
    if !known {
        return format!("{} ({})", def.describe(count), magic.rarity.word());
    }
    match (magic.rarity, &magic.name) {
        (Rarity::Rare, Some(name)) => format!("{name}, {}", def.describe(count)),
        _ => {
            let name = magic_name(def, count, magic, content);
            if count == 1 {
                format!("{} {name}", article(&name))
            } else {
                format!("{count} {name}")
            }
        }
    }
}

/// What it is called without an article or count, as the player knows it: "Sharp long sword of
/// the Bear", "Grim Bite", "long sword".
pub(crate) fn title(
    def: &ObjectDef,
    magic: Option<&Magic>,
    known: bool,
    content: &Content,
) -> String {
    match magic {
        Some(magic) if known => match &magic.name {
            Some(name) => name.clone(),
            None => magic_name(def, 1, magic, content),
        },
        _ => def.name.clone(),
    }
}

/// A magic item's prefix and suffix around its base name (plural for several).
fn magic_name(def: &ObjectDef, count: u32, magic: &Magic, content: &Content) -> String {
    let base = if count == 1 { &def.name } else { &def.plural };
    let rolled = magic.rolled(content);
    let named = |kind: AffixKind| {
        (rolled.iter())
            .find(|r| r.def.kind == kind)
            .map(|r| r.def.name.as_str())
    };
    match (named(AffixKind::Prefix), named(AffixKind::Suffix)) {
        (Some(p), Some(s)) => format!("{p} {base} {s}"),
        (Some(p), None) => format!("{p} {base}"),
        (None, Some(s)) => format!("{base} {s}"),
        (None, None) => base.clone(),
    }
}
