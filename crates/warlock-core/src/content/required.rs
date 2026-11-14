//! The content ids the engine names in code (#1546): one list, resolved once at load into typed
//! ids ([`Content::ids`]), so a renamed `staggered` or `stairs_down` is a load error instead of a
//! rule that silently stops working.
//!
//! Each id is required only while the table it lives in, or the feature that uses it, is loaded:
//! content without `data/status/` (a test fixture, or a mod that leaves statuses out) still
//! loads, and the ids of that table are `None` there, as before. Once the table is there, a
//! missing id fails the load, whatever set of files was loaded (the built-in copy, `--content-dir`,
//! or files overlaid on it).

use super::{
    Content, ContentError, CreatureId, ObjectId, SlotId, StatusId, TalentEffect, TerrainId,
};
use crate::talent::CraftEffect;

/// The creature type the player is (`data/creatures/player.toml`).
pub const PLAYER: &str = "player";
/// The status a fall, a slick, or a stumble leaves one in (`data/status/`): required while
/// a room effect collapses.
pub const STAGGERED: &str = "staggered";
/// The statuses the priest's prayer for wounds and for poison ends (required, with Webbed,
/// while there is a prayer).
pub const BLEEDING: &str = "bleeding";
pub const POISONED: &str = "poisoned";
/// The status the dark face's webs leave (the prayer's face, `world/dark_face.rs`).
pub const WEBBED: &str = "webbed";
/// The spell effect drawn for the dev frost burst (`data/effects.toml`).
pub const FROST_BURST: &str = "frost_burst";
/// The terrain of a Pit's mouth, and of one sealed (`world/pit.rs`).
pub const STAIRS_DOWN: &str = "stairs_down";
pub const RUBBLE: &str = "rubble";
/// The terrain a field forge makes (`world/field_forge.rs`).
pub const MAKESHIFT_ANVIL: &str = "makeshift_anvil";
/// The object strewn about a den's hoard (`mapgen/den.rs`): required while a room effect is a
/// teeming den.
pub const BONES: &str = "bones";
/// The slot a kept beast wears things in (`world/collar.rs`).
pub const NECK: &str = "neck";

/// The ids of the list above, resolved; `None` only where the table or feature is not loaded.
#[derive(Clone, Debug, Default)]
pub struct RequiredIds {
    pub player: Option<CreatureId>,
    pub staggered: Option<StatusId>,
    pub bleeding: Option<StatusId>,
    pub poisoned: Option<StatusId>,
    pub webbed: Option<StatusId>,
    pub stairs_down: Option<TerrainId>,
    pub rubble: Option<TerrainId>,
    pub makeshift_anvil: Option<TerrainId>,
    pub bones: Option<ObjectId>,
    pub neck: Option<SlotId>,
}

/// Resolves the ids against loaded `content`: the error names the first missing one and the
/// file it belongs in.
pub(super) fn resolve(content: &Content) -> Result<RequiredIds, ContentError> {
    fn need<T>(
        found: Option<T>,
        wanted: bool,
        file: &str,
        what: &str,
        id: &str,
    ) -> Result<Option<T>, ContentError> {
        if wanted && found.is_none() {
            return Err(ContentError::new(
                file,
                format!("the {what} `{id}` is missing: the game's rules name it"),
            ));
        }
        Ok(found)
    }
    let creatures = !content.creature_defs().is_empty();
    let collapse = (content.room_effects.defs().iter()).any(|r| r.collapse.is_some());
    let prayer = content.prayer.is_some();
    let dens = (content.room_effects.defs().iter()).any(|r| r.swarm.is_some());
    let portal = content.dungeons().any(|d| d.portal.is_some());
    let forge = (content.talents.iter()).any(|(_, t)| {
        (t.effects.iter()).any(|e| matches!(e, TalentEffect::Craft(CraftEffect::FieldForge { .. })))
    });
    let beasts = content.companions.is_some() || content.familiars.is_some();
    let status = |id, wanted| need(content.status_id(id), wanted, "data/status/", "status", id);
    let terrain = |id, wanted| {
        need(
            content.terrain_id(id),
            wanted,
            "data/terrain/",
            "terrain",
            id,
        )
    };
    if !content.effects.is_empty() && content.effect(FROST_BURST).is_none() {
        need(None::<()>, true, "data/effects.toml", "effect", FROST_BURST)?;
    }
    Ok(RequiredIds {
        player: need(
            content.creature_id(PLAYER),
            creatures,
            "data/creatures",
            "creature",
            PLAYER,
        )?,
        staggered: status(STAGGERED, collapse)?,
        bleeding: status(BLEEDING, prayer)?,
        poisoned: status(POISONED, prayer)?,
        webbed: status(WEBBED, prayer)?,
        stairs_down: terrain(STAIRS_DOWN, portal)?,
        rubble: terrain(RUBBLE, portal)?,
        makeshift_anvil: terrain(MAKESHIFT_ANVIL, forge)?,
        bones: need(
            content.object_id(BONES),
            dens,
            "data/objects",
            "object",
            BONES,
        )?,
        neck: need(
            content.slot_id(NECK),
            beasts,
            "data/slots.toml",
            "slot",
            NECK,
        )?,
    })
}
