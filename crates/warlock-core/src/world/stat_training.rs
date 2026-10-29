//! Stat training in the world (spec §8 "Stats"; the rules are
//! [`crate::content::StatTraining`]): the player's uses of their stats earn marks, and each level
//! gained, and every day at 06:00, checks every stat once.
//!
//! - **Marks**: a use ([`StatUse`]) is reported where it happens, through `World::train` for the
//!   uses skills share (a dodge, a kick, a step within earshot, a burdened action, a spot) and
//!   [`World::train_stat`] for the rest; each rule for it gives its stat a mark with the rule's
//!   chance, one draw of the **training** stream per rule below 100 (never gameplay). Only the
//!   player trains, and only while alive. Marks are never said.
//! - **The pile**: `Actor::stat_marks`, a signed sum per stat, is the whole state: no cap, no
//!   decay, no daily counter (the design, `docs/design/stat-training.md` §4.5).
//! - **Checks** ([`World::settle`]): at every level gained ([`World::check_stats`], from
//!   `World::level_up`, level by level, on the **levels** stream) and once a game day at 06:00
//!   ([`World::daily_stat_check`], #562, a moment of `World::live`
//!   ([`World::until_stat_check`]), on the **training** stream), each stat in display order
//!   draws once, always (so the stream stays in step whatever the piles), and moves by 1 with
//!   [`StatTraining::odds`](crate::content::StatTraining::odds) of its pile against its price.
//!   A rise pays the price (a pile short of it goes to zero), keeps the surplus for the next
//!   check, never lifts the potential, and says "You feel stronger.", nothing more; a
//!   Toughness rise brings its hit point at once. A negative pile (deeds',
//!   [`super::stat_deeds`]) drops its stat the same way against the drop's price
//!   ([`Stats::untrain`](crate::stats::Stats::untrain): never below 3 nor the stat damage on
//!   it plus one, the potential untouched), pays that price back toward zero (no further), cuts
//!   hit points and mana to the new maxima (never below 1), and says "You feel weaker.". The
//!   daily check's hungry dawn is dealt before its draws. The daily move is said at once, so it stops a rest;
//!   `WorldState::next_stat_check` holds the moment ([`schedule`]). Content without training
//!   rules checks nothing (and draws nothing).

use super::stat_deeds::drop_news;
use super::{ActorId, World, WorldState};
use crate::action::AttackKind;
use crate::combat::{Blow, Category, Strike};
use crate::content::{Content, ODDS_SCALE, StatDeed, StatUse};
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
    /// with the rule's chance (Hard training's `TalentMods::train_percent` more, never above
    /// 100): one draw of the training stream per mark for a rule below 100 (whatever the talent
    /// makes of it), none at 100 (as a single use). The pile saturates at `i32::MAX`.
    pub(super) fn train_stat_times(&mut self, used: StatUse, named: Option<Stat>, count: u16) {
        if self.state.death.is_some() || !self.trains_stat(used) {
            return;
        }
        let content = self.content.clone();
        let talents = self.state.player().talent_mods(&content);
        // Quick study of the body: a potion's marks, more of them.
        let count = match used.names_stat() {
            true => {
                let percent = talents.potion_marks_percent;
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
                chance => {
                    let chance = i32::from(chance) * (100 + talents.train_percent.max(-100)) / 100;
                    let chance = chance.clamp(0, 100) as u32;
                    (0..count)
                        .filter(|_| self.state.training.chance(chance))
                        .count() as u16
                }
            };
            let player = self.state.player;
            if let Some(actor) = self.state.actors.get_mut(&player) {
                let marks = &mut actor.stat_marks[stat];
                *marks = marks.saturating_add(i32::from(won));
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
        for moved in self.settle(Check::Level) {
            self.news.push(moved.news().into());
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
        // Greeting the dawn starved: the deed lands before the draws.
        if self.player_hunger().is_some_and(|h| h <= Hunger::Weak) {
            self.stat_deed(StatDeed::HungryDawn);
        }
        for moved in self.settle(Check::Daily) {
            self.say(moved.news());
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

    /// Settles each stat's pile in display order (module docs): `check`'s stream, six draws of
    /// [`ODDS_SCALE`] whatever the piles (a pile out of play still draws), none without
    /// training rules. A positive pile rises with its odds against the price and pays it, the
    /// surplus kept; a negative one drops with its odds against the drop's price and pays that
    /// back, no further than zero. Returns the stats that moved.
    pub(super) fn settle(&mut self, check: Check) -> Vec<Moved> {
        let content = self.content.clone();
        let rules = content.stat_training();
        let mut moved = Vec::new();
        if !rules.is_active() {
            return moved;
        }
        let seed = self.state.seed;
        let player = self.state.player;
        for stat in Stat::ALL {
            let Some(actor) = self.state.actors.get(&player) else {
                return moved;
            };
            let (score, potential) = (actor.stats.score(stat), actor.stats.potential(stat));
            let marks = actor.stat_marks[stat];
            let price = rules.price_for(marks, score, potential);
            let odds = price.map_or(0, |price| rules.odds(marks.unsigned_abs(), price));
            let draw = match check {
                Check::Level => (self.state.levels_rng)
                    .get_or_insert_with(|| Rng::levels(seed))
                    .below(ODDS_SCALE),
                Check::Daily => self.state.training.below(ODDS_SCALE),
            };
            let Some(price) = price.filter(|_| draw < odds) else {
                continue;
            };
            let actor = self.state.actors.get_mut(&player).expect("the player");
            if marks < 0 {
                // A score always carries the stat damage on it.
                let floor = actor.damage.get(stat).saturating_add(1);
                if !actor.stats.untrain(stat, floor) {
                    continue;
                }
                // A drop pays its price back; a pile short of it goes to zero, no credit.
                actor.stat_marks[stat] = (i64::from(marks) + i64::from(price)).min(0) as i32;
                // The most hit points and mana may have fallen: never below 1, a drop never
                // kills.
                actor.hp = actor.hp.min(actor.max_hp(&content)).max(1);
                actor.mana = actor.mana.min(actor.max_mana(&content));
                moved.push(Moved::Fell(stat));
                continue;
            }
            let before = actor.max_hp(&content);
            if !actor.stats.train(stat) {
                continue;
            }
            // A rise pays its price; a pile short of it goes to zero, no debt.
            actor.stat_marks[stat] = (i64::from(marks) - i64::from(price)).max(0) as i32;
            // A Toughness rise adds a hit point to the maximum: it comes at once.
            let grown = actor.max_hp(&content) - before;
            if grown > 0 {
                actor.hp += grown;
            }
            moved.push(Moved::Rose(stat));
        }
        moved
    }
}

/// A stat a check moved, and which way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Moved {
    Rose(Stat),
    Fell(Stat),
}

impl Moved {
    /// What the player is told of it.
    fn news(self) -> &'static str {
        match self {
            Moved::Rose(stat) => rise_news(stat),
            Moved::Fell(stat) => drop_news(stat),
        }
    }
}

/// Which check settles the piles: a level's (the levels stream) or the daily one at 06:00 (the
/// training stream); the rule is the same.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Check {
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
