//! The dark face of the priest's prayer (the priest design `docs/design/priest.md` §6.2, §6.3;
//! `docs/dev/prayer.md`): what the dark powers add to the cures they answer, and Communion with
//! the dark. No draw here but a passed-on disease's incubation (`World::infect`'s).
//!
//! - **The plague passed on**: each disease the prayer cures is caught by the nearest creature
//!   the player sees ([`World::nearest_seen`]), incubating as if it had lost the check; one
//!   immune to disease by its type is spared. None seen: the disease is simply gone.
//! - **The poison passed on**: the degrees of poison cured are laid on that creature, met by its
//!   immunity and resistance as any dose (`World::afflict`).
//! - **The hex turned back**: each released status whose cause the player sees is laid on that
//!   cause at the degrees it had; a hold's holder is Webbed instead, for the hold's degrees.
//! - **The brim**: the hunger use feeds to one short of Bloated (in `orison.rs`).
//!
//! Every twist is caused by the player (`by`), so a creature that dies of it is a kill of
//! theirs. Nothing here is a blow.
//!
//! **Communion** ([`World::communes`]): a holder of the Communion talent within its bands, whom
//! the dark answers, standing where an unholy effect lies (a cursed, draining, or haunted room,
//! or a level one covers whole): its prayers add no strain (`World::orison`), and the unholy
//! place's status never touches it (`World::dwell`, the room status's one place).

use super::{ActorId, World};
use crate::action::Event;
use crate::content::{DiseaseId, Face, TalentEffect};
use crate::status::Effect;

/// The status a hold's holder is laid under when the hold is turned back.
const WEBBED: &str = "webbed";

impl World {
    /// The creature nearest the player that they see on the current level: the least squared
    /// distance, the lowest id on a tie.
    pub(super) fn nearest_seen(&self) -> Option<ActorId> {
        let (player, here) = (self.state.player, self.player_pos());
        let level = &self.state.current_level;
        (self.state.actors.values())
            .filter(|a| a.id != player && a.level == *level)
            .filter(|a| self.perception.sees_actors_at(a.pos))
            .min_by_key(|a| (here.dist2(a.pos), a.id))
            .map(|a| a.id)
    }

    /// The diseases the prayer cured go to the nearest creature the player sees.
    pub(super) fn pass_on_diseases(&mut self, diseases: &[DiseaseId]) {
        let Some(target) = self.nearest_seen() else {
            return;
        };
        let actor = &self.state.actors[&target];
        let immune = &self.content.creature(actor.creature).immune;
        if (self.content.disease_status()).is_some_and(|s| immune.contains(&s)) {
            return;
        }
        for &disease in diseases {
            self.infect(target, disease, Some(self.state.player));
        }
    }

    /// The poison the prayer cured (`ended`) goes to the nearest creature the player sees, at
    /// the degrees it had.
    pub(super) fn pass_on_poison(&mut self, ended: &[Effect], events: &mut Vec<Event>) {
        let Some(target) = self.nearest_seen() else {
            return;
        };
        for e in ended {
            self.afflict(
                target,
                e.status,
                e.severity,
                Some(self.state.player),
                events,
            );
        }
    }

    /// The hexes and holds the prayer released (`ended`) are turned on their causes the player
    /// sees: a hex at the degrees it had, a hold's holder Webbed for the hold's degrees.
    pub(super) fn turn_hexes_back(&mut self, ended: &[Effect], events: &mut Vec<Event>) {
        let player = self.state.player;
        let webbed = self.content.status_id(WEBBED);
        for e in ended {
            let Some(by) = e.by.filter(|&by| by != player) else {
                continue;
            };
            let seen = (self.state.actors.get(&by)).is_some_and(|a| {
                a.level == self.state.current_level && self.perception.sees_actors_at(a.pos)
            });
            if !seen {
                continue;
            }
            let status = if self.content.status(e.status).hold.is_some() {
                webbed
            } else {
                Some(e.status)
            };
            if let Some(status) = status {
                self.afflict(by, status, e.severity, Some(player), events);
            }
        }
    }

    /// Whether actor `id` is in communion with the dark: it holds Communion, its band is one
    /// the talent's `alignment` allows (or no rules say otherwise), and the dark answers its
    /// prayers. Where it stands is the caller's question.
    pub(super) fn communes(&self, id: ActorId) -> bool {
        let content = &self.content;
        let Some(actor) = self.state.actors.get(&id) else {
            return false;
        };
        if actor.talents.is_empty() || !actor.talent_mods(content).communion {
            return false;
        }
        let bands = (content.talents().iter())
            .map(|(_, def)| def)
            .find(|def| {
                def.effects
                    .iter()
                    .any(|e| matches!(e, TalentEffect::Communion))
            })
            .and_then(|def| def.alignment);
        let allowed = match (bands, content.alignment()) {
            (Some(bands), Some(_)) => bands.allows(actor.band(content)),
            _ => true,
        };
        allowed && actor.prayer_face(content) == Face::Dark
    }

    /// Whether the player stands in communion: they commune, and an unholy effect lies where
    /// they stand.
    pub(super) fn communing_here(&self) -> bool {
        let here = self.effects_at(&self.state.current_level, self.player_pos());
        here.each().any(|e| e.unholy) && self.communes(self.state.player)
    }
}
