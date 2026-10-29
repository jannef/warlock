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
//!   Toughness rise brings its hit point at once. A negative pile is out of play until the
//!   losses' slice (the drop is T3's). The daily rise is said at once, so it stops a rest;
//!   `WorldState::next_stat_check` holds the moment ([`schedule`]). Content without training
//!   rules checks nothing (and draws nothing).

use super::ice::Footing;
use super::{ActorId, World, WorldState};
use crate::action::AttackKind;
use crate::combat::{Blow, Category, Strike};
use crate::content::{Content, HAUL_GRAMS, ODDS_SCALE, SpellData, StatUse, TerrainId};
use crate::hunger::Hunger;
use crate::object::Stack;
use crate::rng::Rng;
use crate::stats::{PerStat, Stat};
use crate::status::Effect;
use crate::temperature::Temperature;
use crate::time::{DAWN, GameTime};

/// The least a step costs, in percent of a plain one, for its water to be waded through (deep
/// water: a pool, a stream; not a flooded floor).
const WADING_COST: u16 = 200;

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
        self.train_stats_of(used, named, count, PerStat::splat(true));
    }

    /// [`World::train_stat_times`] for the rules of `used` whose stat is one of `only`'s.
    fn train_stats_of(
        &mut self,
        used: StatUse,
        named: Option<Stat>,
        count: u16,
        only: PerStat<bool>,
    ) {
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
            let Some(stat) = rule.stat.or(named).filter(|&s| only[s]) else {
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

    /// The player cast `spell`, a spell they know: the `spell` use's rules of the stats its
    /// scales name ([`crate::content::SpellData::trained_stats`]).
    pub(super) fn train_spell(&mut self, spell: &SpellData) {
        self.train_stats_of(StatUse::Spell, None, 1, spell.trained_stats());
    }

    /// The player stepped onto `onto` with `footing`: a step through deep water (water that
    /// freezes and that a step wades slowly through, at least [`WADING_COST`]) trains
    /// Strength, unless its ice held them up.
    pub(super) fn train_wade(&mut self, onto: TerrainId, footing: Footing) {
        let terrain = self.content.terrain(onto);
        let deep = terrain.freeze.is_some() && terrain.move_cost >= WADING_COST;
        if deep && !matches!(footing, Footing::Held { .. }) {
            self.train_stat(StatUse::Waded);
        }
    }

    /// The player picked up `got`: one stack of [`HAUL_GRAMS`] or more trains Strength, once.
    pub(super) fn train_haul(&mut self, got: &[Stack]) {
        if got.iter().any(|s| s.weight(&self.content) >= HAUL_GRAMS) {
            self.train_stat(StatUse::Hauled);
        }
    }

    /// The player acted, not waiting: while Hungry (not yet Weak), Toughness may earn a mark.
    pub(super) fn train_hungry_step(&mut self) {
        if self.player_hunger() == Some(Hunger::Hungry) {
            self.train_stat(StatUse::HungryStep);
        }
    }

    /// Statuses wore off actor `id` by time (`ended`): each that harms as it ticks (hit points
    /// or mana) trains the player's Toughness.
    pub(super) fn train_shaking_off(&mut self, id: ActorId, ended: &[Effect]) {
        if id != self.state.player {
            return;
        }
        let content = self.content.clone();
        for e in ended {
            let def = content.status(e.status);
            if def.damage > 0 || def.mana_damage > 0 {
                self.train_stat(StatUse::ShookOff);
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
                AttackKind::Spell => self.train_stat(StatUse::Aimed),
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
        for stat in self.settle(Check::Level) {
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
        // A dawn below freezing hardens the body: its mark counts at this very check.
        if self.temperature() < Temperature::ZERO {
            self.train_stat(StatUse::Chilled);
        }
        for stat in self.settle(Check::Daily) {
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

    /// Settles each stat's pile in display order (module docs): `check`'s stream, six draws of
    /// [`ODDS_SCALE`] whatever the piles (a pile out of play still draws), none without
    /// training rules. A positive pile rises with its odds against the price and pays it, the
    /// surplus kept; a negative one is not moved yet (the drop is the losses' slice). Returns
    /// the stats that rose.
    pub(super) fn settle(&mut self, check: Check) -> Vec<Stat> {
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
            let price = rules.price_for(marks, score, potential);
            let odds = price.map_or(0, |price| rules.odds(marks.unsigned_abs(), price));
            let draw = match check {
                Check::Level => (self.state.levels_rng)
                    .get_or_insert_with(|| Rng::levels(seed))
                    .below(ODDS_SCALE),
                Check::Daily => self.state.training.below(ODDS_SCALE),
            };
            let Some(price) = price.filter(|_| marks > 0 && draw < odds) else {
                continue;
            };
            let actor = self.state.actors.get_mut(&player).expect("the player");
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
            risen.push(stat);
        }
        risen
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
