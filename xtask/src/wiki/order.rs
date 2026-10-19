//! The order the tables list content in. The creature, object, material, and affix tables are
//! directories read in path order (docs/dev/content.md), so no table follows that order: each
//! list here sorts by an explicit key, the group a page reads by first, then the name.

use warlock_core::content::{
    AffixDef, AffixKind, Content, CreatureDef, CreatureId, MaterialDef, MaterialId, ObjectClass,
    ObjectDef, ObjectId,
};

/// Every object with its id: by class (the glyph order of the objects page, a glyph of no class
/// last), then by weapon skill (the order of `weapon_skills.toml`, so the weapons of one skill
/// stand together), then by value (the humble before the fine: leather cap, iron helm, great
/// helm), then by name.
pub(super) fn objects(content: &Content) -> Vec<(ObjectId, &ObjectDef)> {
    let mut all: Vec<_> = (content.object_defs().iter().enumerate())
        .map(|(i, o)| (ObjectId(i as u16), o))
        .collect();
    all.sort_by_cached_key(|(_, o)| {
        let class = (ObjectClass::of_glyph(o.glyph))
            .and_then(|c| ObjectClass::ALL.iter().position(|&k| k == c))
            .unwrap_or(ObjectClass::ALL.len());
        let skill = o.skill.map_or(usize::MAX, |s| usize::from(s.0));
        (class, skill, o.value, o.name.to_lowercase())
    });
    all
}

/// Every creature type with its id, by name.
pub(super) fn creatures(content: &Content) -> Vec<(CreatureId, &CreatureDef)> {
    let mut all: Vec<_> = (content.creature_defs().iter().enumerate())
        .map(|(i, c)| (CreatureId(i as u16), c))
        .collect();
    all.sort_by_cached_key(|(_, c)| c.name.to_lowercase());
    all
}

/// Every material with its id, by name.
pub(super) fn materials(content: &Content) -> Vec<(MaterialId, &MaterialDef)> {
    let mut all: Vec<_> = (content.materials().iter().enumerate())
        .map(|(i, m)| (MaterialId(i as u16), m))
        .collect();
    all.sort_by_cached_key(|(_, m)| m.name.to_lowercase());
    all
}

/// The affixes of `kind`, by name ("of Balance" before "of the Bear", case ignored).
pub(super) fn affixes(content: &Content, kind: AffixKind) -> Vec<&AffixDef> {
    let mut all: Vec<_> = (content.affixes().all().iter())
        .filter(|a| a.kind == kind)
        .collect();
    all.sort_by_cached_key(|a| a.name.to_lowercase());
    all
}
