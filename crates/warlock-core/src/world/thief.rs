//! The thief's identity abilities (spec §8 "Class abilities"; the surprise design §6.2, #684):
//! what the class table gives that no skill does.
//!
//! - **Lurk** (`use:lurk`, a class active): a wait in ambush, a degree of its status (Lurking)
//!   on the player. While it lasts a hostile that steps up beside them takes their blow at once
//!   (the gatekeeper's free blow, switched on in [`World::free_blows_of`]), and is caught
//!   unawares by it unless it was already hunting them (`Mind::last_seen` set): the blow is
//!   resolved as on an unaware target (`startle`, the sneak dice, flat-footed). Refused, no
//!   time, with a hostile they see already beside them.
//! - **Low blow** (`use:low_blow:E`, a class active for openings): a melee blow that is a sneak
//!   attack whatever the foe knows ([`super::weapon_active::Technique::sneak`], taken by the
//!   blow's `resolve`); the foe is not surprised and keeps its Defence. Refused, no time,
//!   without a hostile there.
//! - **Vanishing act** (a class passive, `sneak_kill_status`): a sneak attack of theirs that
//!   kills leaves them a degree of its status (Light-footed), read where `Build::SneakKill` is
//!   raised in `fight.rs::resolve`.
//!
//! Nothing here draws: the blows draw as any blow does.

use super::{Actor, ActorId, Spent, World};
use crate::action::{AttackKind, Event};
use crate::content::{ActionKind, ClassActive, Stance};
use crate::geom::Dir;

impl World {
    /// The thief's refusals of class active `active` aimed `dir`, before anything is spent (no
    /// time; said): a lurk with a hostile the player sees beside them, a low blow with no
    /// hostile there. Every other active passes.
    pub(super) fn thief_may(&mut self, active: ClassActive, dir: Option<Dir>) -> bool {
        match (active, dir) {
            (ClassActive::Lurk, _) if self.hostiles_beside().iter().any(|&(_, seen)| seen) => {
                self.say("Too late to lurk.");
                false
            }
            (ClassActive::LowBlow, Some(dir)) => {
                let player = self.state.player;
                let to = self.player_pos().step(dir);
                let level = &self.state.current().id;
                let foe = (self.state.actor_at(level, to)).map(|a| a.id);
                let hostile = foe.is_some_and(|id| self.stance(player, id) == Stance::Hostile);
                if !hostile {
                    self.say("There is no one there to blind.");
                }
                hostile
            }
            _ => true,
        }
    }

    /// Lurk: the player waits in ambush (a degree of the Lurking status), for the time of a
    /// wait.
    pub(super) fn lurk(&mut self, events: &mut Vec<Event>) -> Spent {
        let player = self.state.player;
        let Some(status) = self.state.player().talent_mods(&self.content).lurk else {
            return Spent::Nothing;
        };
        self.afflict(player, status, 1, Some(player), events);
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }

    /// Whether `actor` lies in wait: it suffers the status of its Lurk.
    pub(super) fn lurking(&self, actor: &Actor) -> bool {
        !actor.talents.is_empty()
            && (actor.talent_mods(&self.content).lurk).is_some_and(|s| actor.effects.has(s))
    }

    /// Actor `keeper`'s free blow at `mover`, which stepped up beside it, is about to be struck:
    /// if the keeper is the player lying in wait, the blow finds `mover` as an unaware target
    /// would be found (taken by its `resolve`, [`World::lurked`]).
    pub(super) fn lie_in_wait(&mut self, keeper: ActorId, mover: ActorId) {
        let player = self.state.player;
        if keeper == player && self.lurking(self.state.player()) {
            self.technique.lurk = Some(mover);
        }
    }

    /// Whether `attacker`'s attack on `target` is the blow of a lurk ([`World::lie_in_wait`])
    /// on a creature not hunting the player: it is caught unawares. The one place
    /// [`super::weapon_active::Technique::lurk`] is read, which this attack takes.
    pub(super) fn lurked(&mut self, attacker: ActorId, target: ActorId) -> bool {
        if attacker != self.state.player {
            return false;
        }
        let ambushed = self.technique.lurk.take() == Some(target);
        ambushed && (self.state.actors.get(&target)).is_some_and(|t| t.mind.last_seen.is_none())
    }

    /// Whether `attacker`'s attack is a low blow, a sneak attack whatever its target knows. The
    /// one place [`super::weapon_active::Technique::sneak`] is read, which this attack takes.
    pub(super) fn low_blow_taken(&mut self, attacker: ActorId) -> bool {
        attacker == self.state.player && std::mem::take(&mut self.technique.sneak)
    }

    /// Low blow: a blow with what the player wields at the hostile in `dir` that is a sneak
    /// attack if it hits (no dice on a foe without vitals), for the price of a melee attack and
    /// its exertion. Its openings were spent by `use_class`, which refused (no time) with no
    /// hostile there.
    pub(super) fn low_blow(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some((target, seen)) = self.hostile_toward(dir) else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let melee = me.attack_skills(&content, AttackKind::Melee, None);
        let with = me.weapon_item(&content);
        let price = self.attack_price(player, ActionKind::Melee, with, melee);
        let profile = me.melee(&content);
        let vitalless = !self.has_vitals(target);
        self.technique.sneak = true;
        let blow = self.player_blow_with(target, seen, profile, events);
        self.technique.sneak = false;
        self.no_vital_spot(vitalless, blow);
        self.after_attack(player, target, super::free_blow::parried(blow), events);
        self.exert(ActionKind::Melee);
        Spent::Turn(price)
    }

    /// Vanishing act: `attacker`'s sneak attack killed; it takes a degree of the status its
    /// talents name for that (Light-footed), if any.
    pub(super) fn vanishing_act(&mut self, attacker: ActorId, events: &mut Vec<Event>) {
        let status = (self.state.actors.get(&attacker))
            .filter(|a| !a.talents.is_empty())
            .and_then(|a| a.talent_mods(&self.content).sneak_kill_status);
        if let Some(status) = status {
            self.afflict(attacker, status, 1, Some(attacker), events);
        }
    }
}
