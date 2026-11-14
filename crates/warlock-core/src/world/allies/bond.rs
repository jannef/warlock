//! Hunter's bond's shared buffs (spec §8 "Allies": "Hunter's bond"; docs/design/ranger-companion.md
//! §11.2, §20; `docs/dev/companion.md`): with `TalentMods::shared_buffs`, a **buff** status that
//! lands on the player lands on their companion too (on the level or in the retinue), and one
//! that lands on the companion lands on the player: the same status and severity, each its own
//! instance, easing by its own clock. One hook, in [`World::afflict`], the one door every status
//! passes, so every source counts: a spell, a scroll of one, a potion, a boon.
//!
//! A buff is a status a buff spell lays (`circles.toml`'s rows that are not a hex's) or a potion
//! lends (`Draught::lends`). A hex never crosses, nor anything else (a harm, a cooldown, Winded,
//! Raging, a mimic's disguise). The shared one does not cross back (no echo). Nothing is drawn.
//!
//! The shared sight is the other half of the bond, in `Perception::compute` (the second
//! viewpoint); the witch's familiar will read that one alone.

use crate::action::Event;
use crate::content::{Content, StatusId};
use crate::world::{ActorId, World};

impl World {
    /// Status `status` at `severity` landed on `id`: with Hunter's bond, a buff crosses to the
    /// other of the player and their companion ([`World::afflict_one`]), said when it begins on
    /// the companion ("Snel is bark-skinned too.").
    pub(in crate::world) fn share_buff(
        &mut self,
        id: ActorId,
        status: StatusId,
        severity: u16,
        by: Option<ActorId>,
        events: &mut Vec<Event>,
    ) {
        let player = self.state.player;
        if !self
            .state
            .player()
            .talent_mods(&self.content)
            .beast
            .shared_buffs
            || !is_buff(&self.content, status)
        {
            return;
        }
        let Some(companion) = self.companion() else {
            return;
        };
        let other = match id {
            _ if id == player => companion,
            _ if id == companion => player,
            _ => return,
        };
        let had = self.state.actors[&other].effects.has(status);
        self.afflict_one(other, status, severity, by, events);
        let has = self.state.actors[&other].effects.has(status);
        if other == companion && has && !had {
            let who = self.ally_subject(companion);
            let def = self.content.status(status);
            let seen = def.seen.clone().unwrap_or_else(|| def.name.to_lowercase());
            self.news.push(format!("{who} is {seen} too."));
        }
    }
}

/// Whether `status` is a buff (the module docs): a buff spell's (circled, not a hex) or one a
/// potion lends.
pub(crate) fn is_buff(content: &Content, status: StatusId) -> bool {
    if let Some(row) = content.circles().of(status) {
        return !row.hex;
    }
    (content.object_defs().iter())
        .filter_map(|o| o.drink.as_ref())
        .any(|d| d.does.lends().is_some_and(|(s, _)| s == status))
}
