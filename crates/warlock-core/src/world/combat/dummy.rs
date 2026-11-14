//! Target dummies (spec §8 "Weapon skills": "Target dummies", #1199): a creature whose type has
//! a `training` table ([`DummyTraining`]). It is a creature so that every attack (a blow, a
//! bash, a kick, a shot, a throw) finds it as it finds anyone, through `World::resolve`; the
//! rest is a hook each:
//! - it never acts ([`World::creature_turn`] waits for it) and never dies ([`World::hurt`] and
//!   [`World::dies`] pass it by), so it earns no one experience, kills, quest credit, or loot;
//! - the player bumps it as they bump a hostile (`travel.rs`), though it is of the peaceful
//!   folk, so nothing else treats it as an enemy;
//! - a blow at it wakes and angers no one, makes no noise of its own, and trains no stat
//!   (`World::resolve`): only weapon skills and styles, at its `percent` of the marks, and
//!   only below its `melee` rank (blows, bashes, kicks, throws) or `ranged` rank (shots)
//!   ([`World::practise_on`]);
//! - a missile that reaches it stops there, hit or miss, and drops beside it, or is destroyed
//!   by its `breaks` chance ([`World::dummy_catches`], called by `World::fly`).

use crate::action::AttackKind;
use crate::content::{DummyTraining, WeaponSkillId};
use crate::geom::{Dir, Pos};
use crate::object::Stack;
use crate::world::{ActorId, World};

/// Said once a skill or style reaches the rank past which a dummy teaches nothing.
pub(crate) const LEARNED_ALL: &str = "You have learned all the dummy can teach you.";

impl World {
    /// What actor `id` teaches as a target dummy, if it is one.
    pub(crate) fn dummy(&self, id: ActorId) -> Option<DummyTraining> {
        let actor = self.state.actors.get(&id)?;
        if id == self.state.player {
            return None;
        }
        self.content.creature(actor.creature).training
    }

    /// The player used weapon skill or style `skill` in an attack of `kind` at `target`, an
    /// opponent of `level`: its marks ([`World::practise`]), a dummy's share of them below its
    /// rank, nothing from the rank up.
    pub(super) fn practise_on(
        &mut self,
        target: ActorId,
        (kind, skill): (AttackKind, WeaponSkillId),
        success: bool,
        level: u32,
    ) {
        let Some(training) = self.dummy(target) else {
            self.practise(skill, success, level);
            return;
        };
        let most = match kind {
            AttackKind::Ranged => training.ranged,
            _ => training.melee,
        };
        if self.state.player().weapon_skills.rank(skill) >= most {
            return;
        }
        if self.practise_at(skill, success, level, training.percent)
            && self.state.player().weapon_skills.rank(skill) >= most
        {
            self.news.push(LEARNED_ALL.into());
        }
    }

    /// A missile (`lands`, if anything comes down) loosed at the dummy on cell `at`, hit or
    /// miss, with a `breaks` chance of its own: it is destroyed by the dummy's chance (never one
    /// that never breaks), else drops into a free cell beside the dummy (one gameplay draw for
    /// which), or on the dummy's own when there is none.
    pub(super) fn dummy_catches(
        &mut self,
        dummy: ActorId,
        at: Pos,
        (lands, breaks): (Option<Stack>, u32),
    ) {
        let Some(training) = self.dummy(dummy) else {
            return;
        };
        let Some(lands) = lands else {
            return;
        };
        if breaks > 0 && training.breaks > 0 && self.state.rng.chance(training.breaks) {
            return;
        }
        let level = self.state.current();
        let free: Vec<Pos> = (Dir::ALL.iter())
            .map(|d| at.step(*d))
            .filter(|p| level.is_walkable(&self.content, *p))
            .filter(|p| self.state.actor_at(&level.id, *p).is_none())
            .collect();
        let cell = match free.len() {
            0 => at,
            n => free[self.state.rng.index(n)],
        };
        self.come_down(cell, lands);
    }
}
