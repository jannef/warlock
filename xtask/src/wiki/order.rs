//! The order the tables list content in. The creature, object, material, and affix tables are
//! directories read in path order (docs/dev/content.md), so no table follows that order: each
//! list here sorts by an explicit key, the group a page reads by first, then the name.

use warlock_core::content::{
    AffixDef, AffixKind, CampaignDef, CampaignKind, Content, CreatureDef, CreatureId, MaterialDef,
    MaterialId, ObjectDef, ObjectId,
};

/// The campaigns the wiki describes, in content order: those of kind `campaign`, the test world
/// left out (spec §3); while content has none, every campaign.
pub(super) fn campaigns(content: &Content) -> Vec<&CampaignDef> {
    let all = content.campaigns();
    let playable: Vec<_> = (all.iter())
        .filter(|c| c.kind == CampaignKind::Campaign)
        .collect();
    if playable.is_empty() {
        all.iter().collect()
    } else {
        playable
    }
}

/// Whether the wiki describes what campaign `owner` owns (`None`: shared, always).
pub(super) fn described(content: &Content, owner: Option<&str>) -> bool {
    owner.is_none_or(|id| campaigns(content).iter().any(|c| c.id == id))
}

/// Every object with its id: by item class (the order of `item_classes.toml`, as item lists in
/// the game group), then by weapon skill (the order of `weapon_skills.toml`, so the weapons of one skill
/// stand together), then by value (the humble before the fine: leather cap, iron helm, great
/// helm), then by name.
pub(super) fn objects(content: &Content) -> Vec<(ObjectId, &ObjectDef)> {
    let mut all: Vec<_> = (content.object_defs().iter().enumerate())
        .map(|(i, o)| (ObjectId(i as u16), o))
        .collect();
    all.sort_by_cached_key(|(_, o)| {
        let class = o.class;
        let skill = o.skill.map_or(usize::MAX, |s| usize::from(s.0));
        (class, skill, o.value, o.name.to_lowercase())
    });
    all
}

/// Every creature type the wiki describes ([`described`]) with its id, by name.
pub(super) fn creatures(content: &Content) -> Vec<(CreatureId, &CreatureDef)> {
    let mut all: Vec<_> = (content.creature_defs().iter().enumerate())
        .filter(|(_, c)| described(content, c.campaign.as_deref()))
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
