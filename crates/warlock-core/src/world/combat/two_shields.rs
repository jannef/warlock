//! Two-shield fighting (spec §8 "Fighting styles", #611): a shield in each hand
//! ([`Grip::TwoShields`]), every blow a shield bash, fought in the Two Shields style, with
//! boons that grow with its rank and banes that shrink with it, as two-weapon fighting's
//! penalties do (`[two_shields]` in `data/weapon_skills.toml`, [`TwoShields`]):
//!
//! - **The blow**: a plain attack (a bump, a creature's blow) is a bash with the first shield in
//!   hand order ([`Actor::opening_bash`], read by `fight.rs::player_blow` and
//!   [`World::attack`]); after one that hits, the style's `bash` chance brings a bash with the
//!   other shield ([`World::other_shield`], from `fight.rs::off_hand_or_bash`). Every bash in
//!   the grip is at the to-hit bane ([`Actor::bash_with`]).
//! - **Slowness**: every attack in the grip costs the slowness bane's percent more
//!   (`schedule.rs::attack_price`).
//! - **Spells**: a spell the player casts with their own mana fails by the spell-failure bane's
//!   chance ([`World::shields_fumble`], before the cast goes off): the mana and the time are
//!   spent, said in words, never the chance.
//! - **Defence**: the style's `shield` adds to the shields' Shield bonus (`Actor::defence`).
//!
//! Decisions draw nothing; the second bash and the fumble each draw once from the gameplay
//! stream, only when their chance is above 0, so fights and casts without two shields keep
//! their draws.

use super::{Actor, World};
use crate::combat::Profile;
use crate::content::{Content, Grip, ObjectId, TwoShields};

/// What fighting with two shields costs now (each eased by the style's rank and talents).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShieldBanes {
    /// Added to every bash's to-hit (0 or less).
    pub to_hit: i32,
    /// Percent more every attack costs.
    pub slow: u32,
    /// The chance in percent that a spell cast with the bearer's own mana fails.
    pub spell_failure: u32,
}

/// What the player hears when their shields spoil a spell.
pub(super) const FUMBLED: &str =
    "Your shields get in the way, and the spell slips from your grasp.";

impl Actor {
    /// Where in its inventory its two shields are while it holds a shield in each hand, in hand
    /// order; `None` otherwise.
    pub fn two_shields(&self, content: &Content) -> Option<(usize, usize)> {
        match self.in_hands(content).as_slice() {
            [(a, first), (b, second)] if first.bash.is_some() && second.bash.is_some() => {
                Some((*a, *b))
            }
            _ => None,
        }
    }

    /// Where in its inventory what a plain blow is struck with is: the first shield with two,
    /// else the wielded weapon (none: bare hands or a natural attack).
    pub fn blow_item(&self, content: &Content) -> Option<usize> {
        (self.two_shields(content).map(|(first, _)| first)).or_else(|| self.weapon_item(content))
    }

    /// A plain blow with a shield in each hand: a bash with the first, and what it does.
    pub fn opening_bash(&self, content: &Content) -> Option<(ObjectId, Profile)> {
        let (first, _) = self.two_shields(content)?;
        self.bash_with(content, first)
    }

    /// The banes of the shields in its hands, eased by its rank in Two Shields and its talents;
    /// `None` unless it holds a shield in each hand.
    pub fn shield_banes(&self, content: &Content) -> Option<ShieldBanes> {
        self.two_shields(content)?;
        let table = content.weapon_skills();
        let base = table.two_shields();
        let eased = self.benefit(content, table.style_for(Grip::TwoShields, &[]));
        Some(ShieldBanes {
            to_hit: (base.to_hit + i32::from(eased.ease_bash)).min(0),
            slow: TwoShields::eased(base.slow, u32::from(eased.ease_slow)),
            spell_failure: TwoShields::eased(base.spell_failure, u32::from(eased.ease_spell)),
        })
    }
}

impl World {
    /// The bash with the other shield after `hit` (whether the opening bash of actor
    /// `attacker`'s attack hit), by the style's `bash` chance: one draw, only when the chance is
    /// above 0 and the first hit. `None` without two shields.
    pub(super) fn other_shield(
        &mut self,
        attacker: super::ActorId,
        hit: bool,
    ) -> Option<(ObjectId, Profile)> {
        let content = self.content.clone();
        let me = self.state.actors.get(&attacker)?;
        let (_, second) = me.two_shields(&content)?;
        let chance = me
            .attack_benefit(&content, crate::action::AttackKind::Melee, None)
            .bash;
        let bash = me.bash_with(&content, second)?;
        (hit && chance > 0 && self.state.rng.chance(u32::from(chance))).then_some(bash)
    }

    /// Whether the player's shields spoil the spell they are about to cast for `mana` (its
    /// base cost): by the spell-failure bane's chance, one draw only when it is above 0. A
    /// spoiled spell costs its mana (and, said by the caller, its time) and is said.
    pub(super) fn shields_fumble(&mut self, mana: i32) -> bool {
        let player = self.state.player;
        let Some(banes) = self.state.player().shield_banes(&self.content) else {
            return false;
        };
        if banes.spell_failure == 0 || !self.state.rng.chance(banes.spell_failure) {
            return false;
        }
        self.spend_mana(player, mana);
        self.say(FUMBLED);
        true
    }

    /// `price` of actor `id`'s attack made heavier by the slowness of two shields, if it holds
    /// them.
    pub(super) fn shields_slowed(&self, id: super::ActorId, price: i64) -> i64 {
        let banes = (self.state.actors.get(&id)).and_then(|a| a.shield_banes(&self.content));
        match banes {
            Some(ShieldBanes { slow, .. }) if slow > 0 => price * (100 + i64::from(slow)) / 100,
            _ => price,
        }
    }
}
