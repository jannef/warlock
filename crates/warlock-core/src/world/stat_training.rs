//! Stat training in the world (spec §8 "Stats"; the rules are
//! [`crate::content::StatTraining`]): the player's uses of their stats earn marks, and each level
//! gained, and every day at 06:00, checks every stat once.
//!
//! - **Marks**: a use ([`StatUse`]) is reported where it happens, through `World::train` for the
//!   uses skills share (a dodge, a kick, a step within earshot, a burdened action, a spot) and
//!   [`World::train_stat`] for the rest; each rule for it gives its stat a mark with the rule's
//!   chance, one draw of the **training** stream per rule below 100 (never gameplay). Only the
//!   player trains, and only while alive. Marks are never said.
//! - **Checks** ([`World::check_stats`], from `World::level_up`, level by level): each stat in
//!   display order gets one draw of the **levels** stream, always (so the stream stays in step
//!   whatever the marks), and rises by 1 with the odds of
//!   [`StatTraining::odds`](crate::content::StatTraining::odds); a rise uses up the stat's marks
//!   and says "You feel stronger.", nothing more. A Toughness rise brings the hit point it adds
//!   at once, as a level's do. Content without training rules checks nothing (and draws
//!   nothing).
//! - **The daily check** ([`World::daily_stat_check`], #562): once a game day at 06:00, a
//!   moment of `World::live` ([`World::until_stat_check`]), each stat gets the same check with
//!   the need times `daily_factor` (or Hard training's), on the **training** stream, six draws
//!   always. A rise is said at once, so it stops a rest. `WorldState::next_stat_check` holds
//!   the moment ([`schedule`]).

use super::{ActorId, World, WorldState};
use crate::action::AttackKind;
use crate::combat::{Blow, Category, Strike};
use crate::content::{Content, StatUse};
use crate::hunger::Hunger;
use crate::rng::Rng;
use crate::stats::Stat;
use crate::time::{DAWN, GameTime};

/// What a rise in `stat` says.
fn rise_news(stat: Stat) -> &'static str {
    match stat {
        Stat::Strength => "You feel stronger.",
        Stat::Dexterity => "You feel quicker.",
        Stat::Toughness => "You feel tougher.",
        Stat::Intelligence => "You feel sharper.",
        Stat::Willpower => "You feel more resolute.",
        Stat::Presence => "You feel more confident.",
    }
}

impl World {
    /// Whether any stat earns marks on `used`.
    pub(super) fn trains_stat(&self, used: StatUse) -> bool {
        self.content.stat_training().rules(used).next().is_some()
    }

    /// The player made use `used` of their stats: each rule for it gives its stat a mark with
    /// the rule's chance (a draw of the training stream below 100). Nothing is said.
    pub(super) fn train_stat(&mut self, used: StatUse) {
        self.train_stat_times(used, None, 1);
    }

    /// [`World::train_stat`] `count` times over (a potion's marks): each rule for `used` gives
    /// its stat, or `named` for a use that names its own (a potion's), up to `count` marks, each
    /// with the rule's chance: one draw of the training stream per mark below 100, none at 100
    /// (as a single use). Marks saturate at `u16::MAX`.
    pub(super) fn train_stat_times(&mut self, used: StatUse, named: Option<Stat>, count: u16) {
        if self.state.death.is_some() || !self.trains_stat(used) {
            return;
        }
        let content = self.content.clone();
        // Quick study of the body: a potion's marks, more of them.
        let count = match used.names_stat() {
            true => {
                let percent = self
                    .state
                    .player()
                    .talent_mods(&content)
                    .potion_marks_percent;
                let more = (u32::from(count) * percent.clamp(-100, 1000) as u32 / 100) as u16;
                if percent >= 0 {
                    count.saturating_add(more)
                } else {
                    count
                }
            }
            false => count,
        };
        for rule in content.stat_training().rules(used) {
            let Some(stat) = rule.stat.or(named) else {
                continue;
            };
            let won = match rule.chance {
                100.. => count,
                chance => (0..count)
                    .filter(|_| self.state.training.chance(u32::from(chance)))
                    .count() as u16,
            };
            let player = self.state.player;
            if let Some(actor) = self.state.actors.get_mut(&player) {
                let marks = &mut actor.stat_marks[stat];
                *marks = marks.saturating_add(won);
            }
        }
    }

    /// Actor `attacker`'s attack of `kind` killed `target`: a kill of the player's in melee of
    /// a creature of their level or above trains Toughness.
    pub(super) fn train_kill(&mut self, (attacker, target): (ActorId, ActorId), kind: AttackKind) {
        let player = self.state.player;
        if attacker != player || kind != AttackKind::Melee {
            return;
        }
        let creature = self.actor_or_fallen(target).map(|a| a.creature);
        let level = creature.map(|c| self.content.creature(c).level);
        if level.is_some_and(|l| l >= self.player_level()) {
            self.train_stat(StatUse::Killed);
        }
    }

    /// An attack by `attacker` on `target` of `kind` came to `blow` (before its damage is
    /// dealt): what it trains of the player's stats. A blow of theirs that hits up close
    /// trains Strength (a shot or a throw Dexterity), a critical up close Strength again, and
    /// one that lands while they are Hungry or worse Willpower; a blow on them that costs hit
    /// points trains Toughness, a big hit or a critical more; one their shield blocked
    /// Strength, one their Parry turned Dexterity. (A dodge is the skills' report,
    /// `World::train`.)
    pub(super) fn train_fight(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        kind: AttackKind,
        blow: &Blow,
    ) {
        let player = self.state.player;
        if attacker == player && blow.strike.is_hit() {
            match kind {
                AttackKind::Melee => {
                    self.train_stat(StatUse::Blow);
                    if blow.strike == (Strike::Hit { critical: true }) {
                        self.train_stat(StatUse::Critical);
                    }
                }
                AttackKind::Ranged | AttackKind::Thrown => self.train_stat(StatUse::Shot),
                AttackKind::Spell => {}
            }
            let hungry = self.player_hunger().is_some_and(|h| h <= Hunger::Hungry);
            if hungry && kind != AttackKind::Spell {
                self.train_stat(StatUse::HungryFight);
            }
        }
        if target == player && blow.damage > 0 {
            self.train_stat(StatUse::Hurt);
            let max = self.state.player().max_hp(&self.content);
            let critical = matches!(blow.strike, Strike::Hit { critical: true });
            if critical || crate::combat::big_hit(blow.damage, max) {
                self.train_stat(StatUse::BigHit);
            }
        }
        if target == player {
            match blow.strike {
                Strike::Missed {
                    by: Some(Category::Shield),
                } => self.train_stat(StatUse::Block),
                Strike::Missed {
                    by: Some(Category::Parry),
                } => self.train_stat(StatUse::Parry),
                _ => {}
            }
        }
    }

    /// The player won a check against a disease made with `stat`: Willpower's trains it, any
    /// other's Toughness (spec §8 "Diseases").
    pub(super) fn train_resisting(&mut self, id: ActorId, stat: Stat) {
        if id != self.state.player {
            return;
        }
        match stat {
            Stat::Willpower => self.train_stat(StatUse::ResistedWill),
            _ => self.train_stat(StatUse::Resisted),
        }
    }

    /// A level came: each stat in display order gets its check (module docs). Draws six times
    /// from the levels stream whatever the marks, none without training rules.
    pub(super) fn check_stats(&mut self) {
        for stat in self.stat_checks(Check::Level) {
            self.news.push(rise_news(stat).into());
        }
    }

    /// Seconds from `now` until the player's daily stat check, if one is scheduled (a moment of
    /// [`World::live`]; 0 when it is due).
    pub(super) fn until_stat_check(&self, id: ActorId, now: GameTime) -> Option<u64> {
        let next = self
            .state
            .next_stat_check
            .filter(|_| id == self.state.player)?;
        Some(next.seconds().saturating_sub(now.seconds()))
    }

    /// The daily stat check, if it is due at `when` (module docs): each stat in display order
    /// gets its check on the training stream, six draws whatever the marks, and a rise is said
    /// at once (it stops a rest). The next comes at the next 06:00.
    pub(super) fn daily_stat_check(&mut self, id: ActorId, when: GameTime) {
        let Some(next) = self.state.next_stat_check else {
            return;
        };
        if id != self.state.player || next > when {
            return;
        }
        self.state.next_stat_check = Some(next.max(when).next_at(DAWN));
        for stat in self.stat_checks(Check::Daily) {
            self.say(rise_news(stat));
        }
    }

    /// The dev skip to `to` passes no moment ([`World::skip_time_dev`]): a daily check it jumps
    /// over or lands on is not made, and the next comes at the first 06:00 after `to`.
    pub(super) fn skip_stat_check(&mut self, to: GameTime) {
        if let Some(next) = &mut self.state.next_stat_check
            && *next <= to
        {
            *next = to.next_at(DAWN);
        }
    }

    /// Checks each stat in display order (module docs): `check`'s stream, six draws whatever
    /// the marks, none without training rules. Returns the stats that rose.
    fn stat_checks(&mut self, check: Check) -> Vec<Stat> {
        let content = self.content.clone();
        let rules = content.stat_training();
        let mut risen = Vec::new();
        if !rules.is_active() {
            return risen;
        }
        let seed = self.state.seed;
        let player = self.state.player;
        for stat in Stat::ALL {
            let Some(actor) = self.state.actors.get(&player) else {
                return risen;
            };
            let (score, potential) = (actor.stats.score(stat), actor.stats.potential(stat));
            let marks = actor.stat_marks[stat];
            let (marks, out_of) = match check {
                Check::Level => rules.odds(marks, score, potential),
                Check::Daily => {
                    let talents = actor.talent_mods(&content);
                    let factor = (talents.daily_factor)
                        .map_or(rules.daily_factor, |f| f.min(rules.daily_factor));
                    rules.odds_by(marks, score, potential, factor)
                }
            };
            let draw = match check {
                Check::Level => (self.state.levels_rng)
                    .get_or_insert_with(|| Rng::levels(seed))
                    .below(out_of),
                Check::Daily => self.state.training.below(out_of),
            };
            if draw >= marks {
                continue;
            }
            let actor = self.state.actors.get_mut(&player).expect("the player");
            let before = actor.max_hp(&content);
            if !actor.stats.train(stat) {
                continue;
            }
            actor.stat_marks[stat] = 0;
            // A Toughness rise adds a hit point to the maximum: it comes at once.
            let grown = actor.max_hp(&content) - before;
            if grown > 0 {
                actor.hp += grown;
            }
            risen.push(stat);
        }
        risen
    }
}

/// Which check a stat gets: a level's (the levels stream, the plain need) or the daily one at
/// 06:00 (the training stream, the need times `daily_factor`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Check {
    Level,
    Daily,
}

/// Schedules the daily stat check for a new or loaded run: the next 06:00 after now when the
/// content has training rules and none is scheduled (a run just begun, a save made before the
/// check), none without the rules.
pub(super) fn schedule(state: &mut WorldState, content: &Content) {
    if !content.stat_training().is_active() {
        state.next_stat_check = None;
    } else if state.next_stat_check.is_none() {
        state.next_stat_check = Some(state.time.next_at(DAWN));
    }
}
