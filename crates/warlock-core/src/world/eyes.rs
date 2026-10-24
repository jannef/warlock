//! Effective eyes (spec §5 "Vision"; the design's B4): the player's vision profile as their
//! talents change it. `WorldState::vision` stays the base profile (the race's, or the dev
//! `vision:` override); [`WorldState::eyes`] applies the talents (Night eyes' threshold, Dark
//! adaptation's and Owl eyes' darkvision, Rune sight's UV) and is what perception reads.
//! Creatures keep their types' profiles.

use super::{World, WorldState};
use crate::content::{Content, VisionProfile};
use crate::talent::TalentMods;

impl WorldState {
    /// The player's eyes: the base profile with what their talents change.
    pub(crate) fn eyes(&self, content: &Content) -> VisionProfile {
        let base = content.vision_index(&self.vision).unwrap_or(0);
        let base = &content.vision_profiles()[base];
        eyes_with(base, &self.player().talent_mods(content))
    }
}

impl World {
    /// The player's eyes now (spec §5 "Vision"): their vision profile as their talents change
    /// it ([`WorldState::eyes`]).
    pub fn eyes(&self) -> VisionProfile {
        self.state.eyes(&self.content)
    }
}

/// `base` as talents `mods` change it: the threshold by its percent (never below 0), the
/// darkvision radius farther, ultraviolet seen with the talent. Its id and name stay the base's.
pub(crate) fn eyes_with(base: &VisionProfile, mods: &TalentMods) -> VisionProfile {
    let mut eye = base.clone();
    if mods.threshold_percent != 0 {
        let percent = u32::try_from((100 + mods.threshold_percent).max(0)).unwrap_or(0);
        let threshold = u32::from(eye.threshold) * percent / 100;
        eye.threshold = u16::try_from(threshold).unwrap_or(u16::MAX);
    }
    eye.darkvision = (eye.darkvision + mods.darkvision).max(0);
    eye.sees_uv |= mods.sees_uv;
    eye
}
