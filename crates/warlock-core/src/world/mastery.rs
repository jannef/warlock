//! A master's hands (spec §8 "Class abilities"; the craftsperson design §7.8, §7.9; #1425): the
//! craftsperson's passives at levels 40 and 50, each read in one place.
//!
//! - **Quick hands** (`craft_time_percent`, [`craft_seconds`]): every craft takes that percent of
//!   its time. Every craft's loop asks [`craft_seconds`] for its seconds as the work is started,
//!   and every list that names a craft's time asks it too, so the time shown is the time spent:
//!   a recipe (forged, counted, brewed, enchanted, Smelt; after Fletcher's share, so the two
//!   multiply), Temper and Whittle, Cut and Set, Salvage, Make do and mend, and Field forge. A
//!   recipe's time is read in one place, [`super::fletch::fletcher_seconds`], so a recipe of any
//!   profession gets it. Gather, Forage, Scribe, and Decipher are no crafts and keep their time.
//!   At least a second, as ever.
//! - **Master's hand** (`grade_ease`, [`graded_margin`]): added to a check's margin as its grade
//!   or its cut is read (`craft_make::make_out`, `cut.rs`'s check), only when the check is made:
//!   whether it is made never changes, and a natural 2 or 24 is what it was. So Fine comes 4
//!   sooner, and so do Marvellous, Masterwork, and every cut. Set has no grade; a counted batch
//!   and a brew have none either.

use super::Actor;
use crate::content::Content;

/// How long a craft of `base` seconds takes `actor`: Quick hands' share of it, at least a second.
pub(super) fn craft_seconds(content: &Content, actor: &Actor, base: u32) -> u32 {
    match actor.talent_mods(content).craft_time_percent {
        Some(percent) => (u64::from(base) * u64::from(percent) / 100).max(1) as u32,
        None => base,
    }
}

/// The margin a made check of `margin` is graded at for `actor`: Master's hand's ease added when
/// the check is made (`margin` 0 or more), never to a failure.
pub(super) fn graded_margin(content: &Content, actor: &Actor, margin: i32) -> i32 {
    match margin >= 0 {
        true => margin + actor.talent_mods(content).grade_ease,
        false => margin,
    }
}

impl super::World {
    /// How long a craft of `base` seconds takes the player ([`craft_seconds`]): what every craft's
    /// loop asks as it is started.
    pub(super) fn craft_seconds(&self, base: u32) -> u32 {
        craft_seconds(&self.content, self.state.player(), base)
    }
}
