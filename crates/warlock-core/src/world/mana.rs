//! Mana in the world (spec §8 "Mana"): the second pool beside hit points, built the same way;
//! the pure rules are [`crate::mana`].
//!
//! - **The maximum** ([`super::Actor::max_mana`]): what race and class (a creature type, a
//!   scenario without a character) give, what levels up to `mana_level` added
//!   ([`World::level_up_mana`]), what magic items add, and the mental stats' modifiers. When it
//!   falls, mana follows it as hit points follow theirs: down to the lasting maximum when the
//!   player takes off what raised it ([`World::keep_mana_within_max`]), down to the new maximum
//!   when a disease damages a mental stat (`world/disease.rs`); a status penalty passes, so the
//!   mana kept under one stays (none comes back above the maximum meanwhile).
//! - **Regeneration** ([`World::mana_rate`], the one pace hook, the twin of
//!   `World::healing_rate`): a point back every [`crate::mana::regen_seconds`] of Willpower at
//!   the normal pace, as a moment of `World::live`'s loop (`world/status.rs`: [`regain`] and
//!   [`until_regained`]), so it makes no difference how time is cut up. Burden, temperature, and
//!   combat stances do nothing to it.
//! - **Spending** ([`World::spend_mana`]): the one way, refused when the actor has too little.
//!   What an ability costs passes the actor's talents first ([`Actor::mana_cost`]: Frugal
//!   caster), and the player's spending is a `cast` use of their stats (it trains Willpower).
//!   Abilities will spend it; for now only the dev action `drain:N` does ([`World::drain_dev`]).
//! - **Blood magic** ([`World::blood_magic`]): the talent's active trades hit points for mana.
//! - **Draining**: a status whose ticks have `mana_damage` drains that much a tick (`live`),
//!   never below 0 and never a death.

use super::{Actor, ActorId, Spent, World};
use crate::content::Effect as ItemEffect;
use crate::content::{ActionKind, Content, StatUse};
use crate::hunger::Hunger;
use crate::mana;
use crate::stats::Stat;

impl World {
    /// How fast actor `id` gets mana back now, in percent of the normal pace (spec §8 "Mana"):
    /// the slowest `mana_regen` of its statuses and of the diseases that show (100 without
    /// any), times `[healing] mana_resting` while the player rests (Meditation adds its percent
    /// to that), times the `mana_regen` of a special room it stands in, times what the mana
    /// regeneration affixes of what it wears and wields and its talents add (+50 % is half
    /// again as fast; Tidal's and Clear mind's +25 %). None for a Starving player (spec §8
    /// "Hunger"). The one place that decides it.
    pub(crate) fn mana_rate(&self, id: ActorId) -> u32 {
        let Some(actor) = self.state.actors.get(&id) else {
            return 0;
        };
        if actor.hunger(&self.content) == Some(Hunger::Starving) {
            return 0;
        }
        let statuses = actor.effects.mana_regen(&self.content);
        let mut rate = u32::from(statuses.min(actor.diseases.mana_regen(&self.content)));
        if self.resting && id == self.state.player {
            // Resting's pace, and half again with Meditation (spec §8 "Talents").
            let faster = actor.talent_mods(&self.content).mana_resting_percent;
            let resting = u32::from(self.content.healing().mana_resting);
            let resting = resting * u32::try_from((100 + faster).max(0)).unwrap_or(0) / 100;
            rate = rate * resting / 100;
        }
        let room = (self.state.levels.get(&actor.level)).map_or(100, |level| {
            level.effects_at(&self.content, actor.pos).mana_regen()
        });
        let rate = rate * self.room_pace(actor, room) / 100;
        let faster = actor
            .mods(&self.content)
            .get(ItemEffect::ManaRegeneration)
            .max(0);
        // Talents (spec §8 "Birth sign": Tidal) add their percent to the affixes'.
        let faster = faster + actor.talent_mods(&self.content).mana_regen_percent;
        rate * u32::try_from((100 + faster).max(0)).unwrap_or(0) / 100
    }

    /// Actor `id` spends what costs `base` mana (spec §8 "Mana"): the one way mana is spent.
    /// The cost is what its talents make of `base` ([`Actor::mana_cost`]: Frugal caster's fifth
    /// less). Refused, spending nothing, when it has less than that (or `base` is below 0);
    /// returns whether it was spent, and while a status of theirs blocks casting (a raging
    /// barbarian spends none). The player's spending of any is a `cast` use of their
    /// stats (spec §8 "Stats": it trains Willpower). Says nothing: the spell or ability that
    /// spends it does.
    pub(crate) fn spend_mana(&mut self, id: ActorId, base: i32) -> bool {
        let content = self.content.clone();
        let Some(actor) = self.state.actors.get_mut(&id) else {
            return false;
        };
        let cost = actor.mana_cost(&content, base);
        if cost < 0 || actor.mana < cost {
            return false;
        }
        // A status that blocks casting (Raging) blocks spending mana on anything.
        if cost > 0 && actor.effects.blocking(&content, ActionKind::Cast).is_some() {
            return false;
        }
        actor.mana -= cost;
        if id == self.state.player && cost > 0 {
            self.train_stat(StatUse::Cast);
        }
        true
    }

    /// Keeps actor `id`'s mana within its maximum, which falls when it takes off something that
    /// raised it: its lasting maximum ([`Actor::lasting_max_mana`]), as with hit points.
    pub(super) fn keep_mana_within_max(&mut self, id: ActorId) {
        let content = self.content.clone();
        if let Some(actor) = self.state.actors.get_mut(&id) {
            let max = actor.lasting_max_mana(&content);
            if actor.mana > max {
                actor.mana = max;
            }
        }
    }

    /// The player went up from level `before` to `after` (spec §8 "Experience"): each level
    /// gained up to the experience file's `mana_level` adds [`World::level_mana`] to their
    /// maximum mana and to their mana.
    pub(super) fn level_up_mana(&mut self, before: u8, after: u8) {
        let content = self.content.clone();
        let rules = content.experience();
        let grown = (before + 1..=after)
            .filter(|&l| l <= rules.mana_level())
            .count();
        let mana = i32::try_from(grown).unwrap_or(0) * self.level_mana(&content);
        if let Some(player) = self.state.actors.get_mut(&self.state.player) {
            player.mana_base += mana;
            player.mana += mana;
        }
    }

    /// What one level adds to the player's mana: half the class's mana (the scenario's `mana`
    /// without a character) plus the better of the Intelligence and Willpower modifiers, at
    /// least 0 (the progression design, `docs/design/progression.md` §3.7). The stats as what
    /// the player wears raises them, not as a status lowers them: the penalty passes, this mana
    /// stays.
    pub(super) fn level_mana(&self, content: &Content) -> i32 {
        let class_mana = match &self.state.character {
            Some(c) => c.spec.class(content).map_or(0, |class| class.mana),
            None => content
                .scenario(&self.state.scenario)
                .map_or(0, |s| i32::from(s.mana)),
        };
        let mind = mana::level_modifier(&self.state.player().raised_stats(content));
        (class_mana / 2 + mind).max(0)
    }

    /// The dev action `drain:N` ([`crate::action::Action::Drain`]): the player spends what
    /// costs `amount` mana ([`World::spend_mana`], as an ability would: Frugal caster's cut,
    /// and the `cast` use), for captures. Refused when they have less. Takes no time.
    pub(super) fn drain_dev(&mut self, amount: u16) -> Spent {
        let player = self.state.player;
        let cost = (self.state.player()).mana_cost(&self.content, i32::from(amount));
        if !self.spend_mana(player, i32::from(amount)) {
            let left = self.state.player().mana;
            self.say(format!("You have only {left} mana (dev)."));
            return Spent::Nothing;
        }
        self.say(format!("You spend {cost} mana (dev)."));
        Spent::NoTime
    }

    /// Blood magic (spec §8 "Talents", the Scholar tree's active): the player gives the
    /// talent's hit points for its mana, for the time of a wait. Refused, taking no time,
    /// without the talent, when it would cost their last hit point, and while their mana is
    /// full. Nothing is drawn; it is not a blow (no Protection, no pain, no training of
    /// Toughness), and never a death.
    pub(super) fn blood_magic(&mut self) -> Spent {
        let content = self.content.clone();
        let player = self.state.player;
        let me = self.state.player();
        let Some(blood) = me.talent_mods(&content).blood_magic else {
            self.say("You know no blood magic.");
            return Spent::Nothing;
        };
        if me.hp <= blood.hp {
            self.say("You have too little blood left to give.");
            return Spent::Nothing;
        }
        let max = me.max_mana(&content);
        if me.mana >= max {
            self.say("Your mana is already full.");
            return Spent::Nothing;
        }
        let me = self.player_mut();
        me.hp -= blood.hp;
        me.mana = (me.mana + blood.mana).min(max);
        self.say("You give your blood, and power wells up in you.");
        Spent::Turn(self.price_of(player, ActionKind::Wait))
    }
}

impl Actor {
    /// What an ability whose cost is `base` mana costs this actor, its talents' changes made
    /// ([`crate::talent::TalentMods::mana_cost`]: Frugal caster). What a check of whether it
    /// has the mana compares with, and what [`World::spend_mana`] takes.
    pub fn mana_cost(&self, content: &Content, base: i32) -> i32 {
        self.talent_mods(content).mana_cost(base)
    }
}

/// Mana back for `actor` over `seconds` at `rate` percent of the normal pace: its progress
/// ([`Actor::mana_regen`]) grows by `rate` a second, and every [`mana::regen_seconds`] × 100 of
/// it is a point. Nothing at its maximum (the progress is dropped) or dead.
pub(super) fn regain(actor: &mut Actor, content: &Content, seconds: u64, rate: u32) {
    let Some(need) = need(actor, content) else {
        actor.mana_regen = 0;
        return;
    };
    let max = actor.max_mana(content);
    let banked =
        u64::from(actor.mana_regen).saturating_add(seconds.saturating_mul(u64::from(rate)));
    let gained = i32::try_from(banked / need).unwrap_or(i32::MAX);
    actor.mana = actor.mana.saturating_add(gained).min(max);
    actor.mana_regen = if actor.mana >= max {
        0
    } else {
        u32::try_from(banked % need).unwrap_or(0)
    };
}

/// Seconds until `actor`'s next point of mana back at `rate` (at least 1): a moment of
/// `World::live`'s loop. `None` when none is coming: at its maximum, dead, or at no pace.
pub(super) fn until_regained(actor: &Actor, content: &Content, rate: u32) -> Option<u64> {
    let need = need(actor, content)?;
    let bank = u64::from(actor.mana_regen);
    (rate > 0).then(|| need.saturating_sub(bank).div_ceil(u64::from(rate)).max(1))
}

/// `actor` loses `amount` mana to its statuses' ticks (`mana_damage`): never below 0.
pub(super) fn drain(actor: &mut Actor, amount: i32) {
    if amount > 0 {
        actor.mana = actor.mana.saturating_sub(amount).max(0);
    }
}

/// The progress `actor` needs for a point of mana back ([`Actor::mana_regen`]); `None` when it
/// gets no more: at its maximum, or dead.
fn need(actor: &Actor, content: &Content) -> Option<u64> {
    let willpower = actor.effective_stats(content).score(Stat::Willpower);
    (actor.hp > 0 && actor.mana < actor.max_mana(content))
        .then(|| u64::from(mana::regen_seconds(willpower)) * 100)
}
