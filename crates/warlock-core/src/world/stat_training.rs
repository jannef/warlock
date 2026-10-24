//! Stat training in the world (spec §8 "Stats"; the rules are
//! [`crate::content::StatTraining`]): the player's uses of their stats earn marks, and each level
//! gained checks every stat once.
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

use super::{ActorId, World};
use crate::action::AttackKind;
use crate::combat::{Blow, Strike};
use crate::content::StatUse;
use crate::hunger::Hunger;
use crate::rng::Rng;
use crate::stats::Stat;

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
        if self.state.death.is_some() || !self.trains_stat(used) {
            return;
        }
        let content = self.content.clone();
        for rule in content.stat_training().rules(used) {
            if rule.chance < 100 && !self.state.training.chance(u32::from(rule.chance)) {
                continue;
            }
            let player = self.state.player;
            if let Some(actor) = self.state.actors.get_mut(&player) {
                let marks = &mut actor.stat_marks[rule.stat];
                *marks = marks.saturating_add(1);
            }
        }
    }

    /// An attack by `attacker` on `target` of `kind` came to `blow` (before its damage is
    /// dealt): what it trains of the player's stats. A blow of theirs that hits up close
    /// trains Strength (a shot or a throw Dexterity), and one that lands while they are Hungry
    /// or worse Willpower; a blow on them that costs hit points trains Toughness, a big hit or
    /// a critical more. (A dodge is the skills' report, `World::train`.)
    pub(super) fn train_fight(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        kind: AttackKind,
        blow: &Blow,
    ) {
        let player = self.state.player;
        if attacker == player && blow.strike.is_hit() {
            match kind {
                AttackKind::Melee => self.train_stat(StatUse::Blow),
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
        let content = self.content.clone();
        let rules = content.stat_training();
        if !rules.is_active() {
            return;
        }
        let seed = self.state.seed;
        let player = self.state.player;
        for stat in Stat::ALL {
            let Some(actor) = self.state.actors.get(&player) else {
                return;
            };
            let (score, potential) = (actor.stats.score(stat), actor.stats.potential(stat));
            let (marks, out_of) = rules.odds(actor.stat_marks[stat], score, potential);
            let rng = self
                .state
                .levels_rng
                .get_or_insert_with(|| Rng::levels(seed));
            if rng.below(out_of) >= marks {
                continue;
            }
            let before = actor.max_hp(&content);
            let actor = self.state.actors.get_mut(&player).expect("the player");
            if !actor.stats.train(stat) {
                continue;
            }
            actor.stat_marks[stat] = 0;
            // A Toughness rise adds a hit point to the maximum: it comes at once.
            let grown = actor.max_hp(&content) - before;
            if grown > 0 {
                actor.hp += grown;
            }
            self.news.push(rise_news(stat).into());
        }
    }
}
