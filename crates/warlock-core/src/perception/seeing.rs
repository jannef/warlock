//! Line of sight: the second viewpoint of Hunter's bond and Familiar's sight.

use crate::content::{Content, VisionProfile};
use crate::sight::{self, LineOfSight};
use crate::talent::TalentMods;
use crate::world::{Actor, WorldState};

/// The second viewpoint of Hunter's bond and Familiar's sight (the module docs): the kept beast
/// on the player's level, its line of sight within its type's sight (on the overworld no farther
/// than daylight allows), its eyes, and how far its nose reaches for them.
pub(super) struct BondView<'a> {
    pub(super) beast: &'a Actor,
    pub(super) los: LineOfSight,
    pub(super) eye: VisionProfile,
    pub(super) nose: Option<i32>,
}

/// The [`BondView`] of the beast the player's talents share, Hunter's bond the companion's and
/// Familiar's sight the familiar's, if it stands on their level (not in the retinue). No class
/// holds both talents; were one to, the lower id's would be the viewpoint.
pub(super) fn bond_view<'a>(
    state: &'a WorldState,
    content: &Content,
    talents: &TalentMods,
) -> Option<BondView<'a>> {
    if !talents.shared_sight && !talents.familiar_sight {
        return None;
    }
    let level = state.current();
    let beast = (state.actors.values()).find(|a| {
        a.master() == Some(state.player)
            && ((talents.shared_sight && a.companion().is_some())
                || (talents.familiar_sight && a.familiar().is_some()))
            && a.level == level.id
            && !a.in_retinue()
    })?;
    let def = content.creature(beast.creature);
    let radius = sight::sight_radius(state, content, def.sight);
    let stops = level.sight_stops(content, beast.pos, state.time);
    let los = LineOfSight::compute(beast.pos, radius, stops);
    let eye = (beast.eyes(content, &content.vision_profiles()[def.vision])).into_owned();
    let least = (beast.familiar())
        .and_then(|_| content.familiars())
        .map(|rules| rules.nose);
    Some(BondView {
        beast,
        los,
        eye,
        nose: def.nose.max(least),
    })
}
