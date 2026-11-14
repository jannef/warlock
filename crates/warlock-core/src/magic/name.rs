//! What magic items are called (spec §8 "Magic items"):
//!
//! - normal: "a long sword";
//! - magic: its prefix and suffix around the base name, "a Sharp long sword of the Bear";
//! - rare: its name, then the base, "Grim Bite, a long sword";
//! - unidentified: the base and the rarity, "a long sword (magic)", "12 arrows (rare)".
//!
//! The base is the item as it is made (`Stack::names`): its gems and precious metals show,
//! identified or not, between the prefix and the base ("a Sharp emerald-pommelled long sword of
//! the Bear"); a rare item keeps its two-word name ("Grim Bite, an emerald-pommelled long
//! sword").
//!
//! What an anvil's work calls it (`worked.rs`) comes ahead of the rest of its name, whatever is
//! identified: "a Fine long sword", "a Marvellous Sharp long sword of the Bear", "a damaged long
//! sword (magic)", "Grim Bite, a Masterwork long sword".

use super::{AffixKind, Magic, Rarity};
use crate::content::{Content, counted};
use crate::text::article;

/// `count` of an item called `names` (singular and plural, as it is made) with `magic` in
/// words, as the player knows it: with `known` false (not identified) only its rarity shows;
/// `word` is what an anvil's work calls it.
pub(crate) fn describe(
    names: &(String, String),
    word: Option<&str>,
    count: u32,
    magic: Option<&Magic>,
    known: bool,
    content: &Content,
) -> String {
    let base = counted(&ahead(word, &names.0), &ahead(word, &names.1), count);
    let Some(magic) = magic else {
        return base;
    };
    if !known {
        return format!("{base} ({})", magic.rarity.word());
    }
    match (magic.rarity, &magic.name) {
        (Rarity::Rare, Some(name)) => format!("{name}, {base}"),
        _ => {
            let name = ahead(word, &magic_name(names, count, magic, content));
            if count == 1 {
                format!("{} {name}", article(&name))
            } else {
                format!("{count} {name}")
            }
        }
    }
}

/// What it is called without an article or count, as the player knows it: "Sharp long sword of
/// the Bear", "Grim Bite", "long sword", "Fine long sword".
pub(crate) fn title(
    names: &(String, String),
    word: Option<&str>,
    magic: Option<&Magic>,
    known: bool,
    content: &Content,
) -> String {
    match magic {
        Some(magic) if known => match &magic.name {
            Some(name) => name.clone(),
            None => ahead(word, &magic_name(names, 1, magic, content)),
        },
        _ => ahead(word, &names.0),
    }
}

/// `name` with what an anvil's work calls it ahead of it.
pub(crate) fn ahead(word: Option<&str>, name: &str) -> String {
    match word {
        Some(word) => format!("{word} {name}"),
        None => name.to_string(),
    }
}

/// A magic item's prefix and suffix around its base name (plural for several).
fn magic_name(names: &(String, String), count: u32, magic: &Magic, content: &Content) -> String {
    let base = if count == 1 { &names.0 } else { &names.1 };
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
