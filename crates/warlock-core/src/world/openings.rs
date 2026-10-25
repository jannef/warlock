//! The thief's Openings in the world (spec §8 "Resources" and "Class abilities"; the progression
//! design §4.18): Improved sneak attack grants the pool, every sneak attack that hits builds it
//! (`builds` in `data/resources.toml`, through `ambush.rs`), it decays with time, and Master
//! thief's finisher spends it for a blow that is a critical if it hits, with no confirmation
//! roll ([`crate::combat::Profile::sure_critical`]).
//!
//! The grant is the general one for talents that grant a resource (`grants_resource`,
//! `world/focus.rs::hold`).

use super::{Spent, World};
use crate::action::{AttackKind, Event};
use crate::content::ActionKind;
use crate::geom::Dir;

impl World {
    /// Master thief's finisher: a blow with what the player wields at the hostile in `dir` that
    /// is a critical if it hits (no confirmation roll), for the price of a melee attack and its
    /// exertion. Its openings were spent by `use_class`, which refused (no time) with no
    /// hostile there.
    pub(super) fn finisher(&mut self, dir: Dir, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let Some((target, seen)) = self.hostile_toward(dir) else {
            return Spent::Nothing;
        };
        let me = self.state.player();
        let melee = me.attack_skills(&content, AttackKind::Melee, None);
        let with = me.weapon_item(&content);
        let price = self.attack_price(player, ActionKind::Melee, with, melee);
        let mut profile = me.melee(&content);
        profile.sure_critical = true;
        self.say("You strike at the opening!");
        let blow = self.player_blow_with(target, seen, profile, events);
        self.after_attack(player, target, super::free_blow::parried(blow), events);
        self.exert(ActionKind::Melee);
        Spent::Turn(price)
    }
}
