//! A haunted room's `haunt` (spec §7 "Special rooms", design §9.8; `world/haunt.rs`): while the
//! player rests inside, at every full `every` of the clock a creature of `faction` of the floor's
//! danger comes inside.

use crate::time::Duration;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HauntSpec {
    faction: String,
    every: String,
}

/// A haunted room's hauntings (`haunt = { faction = "undead", every = "1h" }`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Haunt {
    /// The faction (an id in `data/factions.toml`) what comes is of (checked once the factions
    /// load, `RoomEffects::check_factions`).
    pub faction: String,
    /// How often: at every full multiple of it on the clock while the player rests inside.
    pub every: Duration,
}

impl HauntSpec {
    /// The haunt it describes, or why not: `every` no time (the clock is taken modulo it), or
    /// no `needs_danger` (what comes is of the floor's danger).
    pub(super) fn resolve(&self, needs_danger: bool) -> Result<Haunt, String> {
        let every: Duration = self.every.parse()?;
        if every.as_seconds() == 0 {
            return Err("haunt.every is more than no time".into());
        }
        if !needs_danger {
            return Err("a `haunt` needs `needs_danger`".into());
        }
        Ok(Haunt {
            faction: self.faction.clone(),
            every,
        })
    }
}
