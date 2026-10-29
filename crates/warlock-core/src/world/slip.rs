//! Slipping past (spec §8 "Surprise and sneak attacks"; `docs/dev/surprise.md`): the one
//! contest of hiding, because most creatures see a few cells in any darkness.
//!
//! - **The moment** ([`looks_at`]): a creature hunting no one ([`super::Mind::last_seen`] unset)
//!   that would first see the **player** by its darkvision alone (their cell too dark for its
//!   eyes) makes a contest, `check::contest` (four gameplay draws): the player's Dexterity
//!   modifier and their skills' `slip` against its Willpower modifier and its skills' `spot`.
//!   Made, [`super::Mind::missed`] is the turn and it does not see them; lost, it sees them as
//!   it would have, and rolls no more while it hunts them. Nothing is said either way.
//! - **While slipped past**: it does not see the player by darkvision (nor take their noise
//!   for one it sees, `ai.rs::noise_to_look_into`), and is unaware of them
//!   (`World::unaware`).
//! - **The endings** ([`World::end_slip`]): the player's cell lit enough for its eyes at a look;
//!   the player attacks it or anything while it sees them (`World::attacked_near`); it is hurt
//!   (`World::hurt`); `[slip] lasts_turns` pass (`World::slip_lapses`, at its turn). Ended, it
//!   sees the player as anyone would: at once, if its eyes reach them.
//!
//! Creatures slip past nobody. Without the surprise rules or with `[slip] enabled = false`,
//! nobody slips past anyone and nothing here draws.

use super::{ActorId, ActorKind, World, WorldState};
use crate::check;
use crate::content::{Content, VisionProfile};
use crate::geom::Pos;
use crate::light::LightCache;
use crate::sight::{self, By, LineOfSight};
use crate::skills::Effect;
use crate::stats::Stat;

/// Whether creature `id`, with line of sight `los` and `eye`, sees hostile `other` on cell `p`
/// at its look (`ai.rs::nearest_hostile_in_sight`): by the rules of sight, but for the player
/// slipping past it (module docs). The contest is rolled here, and only here.
#[allow(clippy::too_many_arguments)]
pub(super) fn looks_at(
    state: &mut WorldState,
    content: &Content,
    lights: &mut LightCache,
    los: &LineOfSight,
    eye: &VisionProfile,
    id: ActorId,
    other: ActorId,
    p: Pos,
) -> bool {
    let Some(by) = sight::sees_by(state, content, lights, los, eye, p) else {
        return false;
    };
    if other != state.player || slip_turns(content).is_none() {
        return true;
    }
    let Some(mind) = state.actors.get(&id).map(|a| a.mind) else {
        return true;
    };
    // Hunting them and not slipped past, it sees them however it does: the light need not be
    // computed.
    if mind.missed.is_none() && mind.last_seen.is_some() {
        return true;
    }
    if by == By::Light || sight::lit_for(state, content, lights, eye, p) {
        // In light its eyes see by, the player is seen: any slip past is over.
        if let Some(me) = state.actors.get_mut(&id) {
            me.mind.missed = None;
        }
        return true;
    }
    if mind.missed.is_some() {
        return false;
    }
    let (Some(me), Some(player)) = (state.actors.get(&id), state.actors.get(&other)) else {
        return true;
    };
    let modifier = player.effective_stats(content).modifier(Stat::Dexterity)
        + player.skill_effect(content, Effect::Slip);
    let against = me.effective_stats(content).modifier(Stat::Willpower)
        + me.skill_effect(content, Effect::Spot);
    let made = check::contest(&mut state.rng, modifier, against);
    if made {
        let turn = state.turn;
        if let Some(me) = state.actors.get_mut(&id) {
            me.mind.missed = Some(turn);
        }
    }
    !made
}

/// How long a slip past lasts, in turns, if anyone slips past anyone.
fn slip_turns(content: &Content) -> Option<u64> {
    content.surprise().and_then(|r| r.slip_turns)
}

/// [`World::slipped_past`] of a state: `Mind::missed` on creature `id` within the rules'
/// turns, when `other` is the player.
pub(super) fn slipped(state: &WorldState, content: &Content, id: ActorId, other: ActorId) -> bool {
    let Some(lasts) = slip_turns(content) else {
        return false;
    };
    other == state.player
        && (state.actors.get(&id))
            .and_then(|a| a.mind.missed)
            .is_some_and(|turn| state.turn.saturating_sub(turn) <= lasts)
}

impl World {
    /// Whether the player has slipped past creature `id` (and it has not lapsed): it does not
    /// see them by darkvision and is unaware of them.
    pub(crate) fn slipped_past(&self, id: ActorId, other: ActorId) -> bool {
        slipped(&self.state, &self.content, id, other)
    }

    /// The player's slip past creature `id` is over: it sees them as anyone would, at once if
    /// its eyes reach them now (it hunts them: `Mind::last_seen`). Nothing when it was not
    /// slipped past.
    pub(super) fn end_slip(&mut self, id: ActorId) {
        let Some(me) = self.state.actors.get_mut(&id) else {
            return;
        };
        if me.mind.missed.take().is_none() {
            return;
        }
        let player = self.state.player;
        if self.sees_now(id, player)
            && let Some(at) = self.state.actors.get(&player).map(|a| a.pos)
            && let Some(me) = self.state.actors.get_mut(&id)
        {
            me.mind.last_seen = Some(at);
        }
    }

    /// At creature `id`'s turn: a slip past that has lasted its `lasts_turns` is over.
    pub(super) fn slip_lapses(&mut self, id: ActorId) {
        let missed = self.state.actors.get(&id).and_then(|a| a.mind.missed);
        if missed.is_some() && !self.slipped_past(id, self.state.player) {
            self.end_slip(id);
        }
    }

    /// `attacker` attacked `target`: when it is the player, every creature on the level they
    /// had slipped past that is the target or whose eyes reach them or it now knows they are
    /// there ([`World::end_slip`]).
    pub(super) fn attacked_near(&mut self, attacker: ActorId, target: ActorId) {
        if attacker != self.state.player {
            return;
        }
        let level = self.state.current_level.clone();
        let slipped: Vec<ActorId> = (self.state.actors.values())
            .filter(|a| a.kind == ActorKind::Creature && a.level == level)
            .filter(|a| a.mind.missed.is_some())
            .map(|a| a.id)
            .collect();
        for id in slipped {
            if id == target || self.sees_now(id, attacker) || self.sees_now(id, target) {
                self.end_slip(id);
            }
        }
    }
}
