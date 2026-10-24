//! The birth sign's gifts in the world (spec §8 "Birth sign"): the talents a character's sign
//! grants ([`crate::character::Character::grants`]) are put into the player's
//! [`Actor::talents`](super::Actor) when a run starts and again whenever one is restored, so they
//! are never stored as a choice: a gift added to the data reaches a saved character, one taken
//! out leaves it, and no save breaks. What each gift does is read through
//! [`crate::talent::TalentMods`] where its rule lives, as every talent is.

use crate::character::Character;
use crate::content::{Content, TerrainDef};
use crate::talent::Talents;

/// Puts the gifts of `character`'s sign into `talents` in grant order, in place of whatever
/// granted talents it held (none for a run without a character or a sign).
pub(super) fn grant(content: &Content, character: Option<&Character>, talents: &mut Talents) {
    let grants = character.map_or(Vec::new(), |c| c.grants(content));
    talents.regrant(content, &grants);
}

/// Whether stepping onto `terrain` is wading (spec §8 "Birth sign": Strong swimmer): water
/// (terrain that freezes) or ground a disease is caught wading in (a marsh).
pub(super) fn wading(content: &Content, terrain: &TerrainDef) -> bool {
    if terrain.freeze.is_some() {
        return true;
    }
    let Some(id) = content.terrain_id(&terrain.id) else {
        return false;
    };
    (content.disease_ids()).any(|(_, d)| d.wading.as_ref().is_some_and(|w| w.terrain.contains(&id)))
}
