//! Outcomes in play (spec §7 "Special rooms"; the tables are `content/outcome.rs`): drawing one
//! from a table, leaned by a place's side, and applying it to the player. Each field of an
//! [`OutcomeEffect`] is one hook into a system that already exists (healing, statuses, hunger,
//! loot, traps, identification, harm, the lantern, spawning, diseases, stat damage, experience,
//! the teleport trap's move, the burst spell code, terrain), applied in the order the struct
//! lists them. Every draw is from the gameplay stream. The prayer that uses them is `pray.rs`,
//! the drink at a spring `drink.rs`.
//!
//! The outcome's line is said first (one of its `messages`, drawn when it has several), then its
//! hooks run; a hook that finds nothing to do (no unidentified item to make clear) says the
//! outcome's `otherwise` line instead, if it has one. A status an outcome gives is told by the
//! outcome's line, not the status's own start line. Harm an outcome does is the place's: its
//! kills are no one's and earn nothing ([`Harm::Place`]), as a trap's.

use super::fight::Harm;
use super::spell::Caster;
use super::travel::make_creature;
use super::{ActorId, World};
use crate::action::{Event, Positions};
use crate::alignment::Alignment;
use crate::combat::Blow;
use crate::content::{
    Amount, CreatureSpawn, Harmful, Near, OutcomeDef, OutcomeEffect, OutcomeKind, OutcomeSummon,
    OutcomeTable, Stance, danger,
};
use crate::damage::Damage;
use crate::geom::{Dir, Pos};
use crate::hunger::Hunger;
use crate::magic::loot::{self, LootGrade};
use crate::object::{Stack, add_to_pile};
use crate::stats::Stat;
use crate::variant::Source;

/// The kinds' weights of `table` for a player of side `player` at a place of side `place`
/// (spec §7 "Special rooms"): agreement (both good, or both evil) moves its `lean` from the bane
/// to the boon, opposition (good against evil) from the boon to the bane; a neutral place or
/// player leans nothing. Devout (`devout`) doubles an agreeing lean and takes an opposed one away.
pub fn leaned_kinds(
    table: &OutcomeTable,
    place: Alignment,
    player: Alignment,
    devout: bool,
) -> [u32; 3] {
    let lean = i32::try_from(table.lean).unwrap_or(0);
    let lean = match (place, player) {
        (Alignment::Neutral, _) | (_, Alignment::Neutral) => 0,
        (a, b) if a == b && devout => 2 * lean,
        (a, b) if a == b => lean,
        _ if devout => 0,
        _ => -lean,
    };
    table.leaned(lean)
}

impl World {
    /// Draws an outcome of `table` with the kinds' weights `kinds` ([`leaned_kinds`]): the kind
    /// first, then one of its outcomes by weight. `None` only for a table with nothing to draw.
    pub(super) fn draw_outcome<'t>(
        &mut self,
        table: &'t OutcomeTable,
        kinds: [u32; 3],
    ) -> Option<&'t OutcomeDef> {
        let total: u32 = kinds.iter().sum();
        if total == 0 {
            return None;
        }
        let mut at = self.state.rng.below(total);
        let mut kind = OutcomeKind::Bane;
        for (k, w) in OutcomeKind::ALL.into_iter().zip(kinds) {
            if at < w {
                kind = k;
                break;
            }
            at -= w;
        }
        self.draw_in(table, kind)
    }

    /// One of `table`'s outcomes of `kind`, by weight. `None` when the kind has none.
    pub(super) fn draw_in<'t>(
        &mut self,
        table: &'t OutcomeTable,
        kind: OutcomeKind,
    ) -> Option<&'t OutcomeDef> {
        let sum = table.total(kind);
        if sum == 0 {
            return None;
        }
        let mut at = self.state.rng.below(sum);
        for outcome in table.of(kind) {
            if at < outcome.weight {
                return Some(outcome);
            }
            at -= outcome.weight;
        }
        None
    }

    /// `outcome` befalls the player at the place whose feature is on `feature` (module docs).
    pub(super) fn befall(&mut self, outcome: &OutcomeDef, feature: Pos, events: &mut Vec<Event>) {
        let line = match outcome.messages.len() {
            0 => None,
            1 => outcome.messages.first().cloned(),
            n => {
                let i = self.state.rng.index(n);
                outcome.messages.get(i).cloned()
            }
        };
        // The line is said after the hooks when one of them may have nothing to do.
        let effect = &outcome.effect;
        if outcome.otherwise.is_none()
            && let Some(line) = &line
        {
            self.say(line.clone());
        }
        let done = self.hooks(effect, feature, events, |w| {
            if let (Some(line), Some(_)) = (&line, &outcome.otherwise) {
                w.say(line.clone());
            }
        });
        if !done && let Some(otherwise) = &outcome.otherwise {
            self.say(otherwise.clone());
        }
    }

    /// Runs `effect`'s hooks in order; `before` is called once a hook has found something to do,
    /// before it says anything of its own. Returns whether any did something.
    fn hooks(
        &mut self,
        effect: &OutcomeEffect,
        feature: Pos,
        events: &mut Vec<Event>,
        before: impl FnOnce(&mut Self),
    ) -> bool {
        let me = self.state.player;
        let mut before = Some(before);
        let mut found = |w: &mut Self| {
            if let Some(f) = before.take() {
                f(w);
            }
        };
        let mut done = false;
        if let Some(heal) = effect.heal {
            self.restore_hp(heal);
            done = true;
        }
        if !effect.cure.is_empty() {
            done |= self.cure_statuses(&effect.cure, events);
        }
        if let Some(means) = &effect.cure_by {
            let cured = self.diseased_or_cured_by(means);
            if cured {
                found(self);
                self.cure(me, means, events);
                done = true;
            }
        }
        if effect.mend {
            self.mend(me);
            done = true;
        }
        if let Some(mana) = effect.mana {
            self.restore_mana(mana);
            done = true;
        }
        if let Some(drain) = effect.drain {
            let amount = drain.roll(&mut self.state.rng);
            super::mana::drain(self.player_mut(), amount);
            done = true;
        }
        if let Some((status, degrees)) = effect.status {
            // The outcome's line tells it, not the status's own.
            self.afflict(me, status, degrees, None, &mut Vec::new());
            done = true;
        }
        if effect.hunger != 0 {
            self.feed_by_states(effect.hunger);
            done = true;
        }
        if !effect.objects.is_empty() {
            self.leave_objects(effect, feature);
            done = true;
        }
        if let Some(bonus) = effect.loot {
            self.leave_loot(bonus, effect.rarity, feature);
            done = true;
        }
        if let Some(reach) = effect.know_traps {
            self.sense_traps(reach);
            done = true;
        }
        if effect.identify {
            let unknown = self.unidentified();
            if !unknown.is_empty() {
                found(self);
                let i = unknown[self.state.rng.index(unknown.len())];
                self.identify(i);
                done = true;
            }
        }
        if effect.forget {
            done |= self.forget_one(found_once(&mut found));
        }
        if let Some(burn) = effect.burn {
            let taken = self.harm_taken(burn);
            self.hurt(me, taken, Harm::Spared, events);
            done = true;
        }
        if let Some(smite) = effect.smite {
            let taken = self.harm_taken(smite);
            if taken > 0 && !self.hurt(me, taken, Harm::Place, events) {
                self.wounded(me, &Blow::plain(taken, smite.kind), None, events);
            }
            done = true;
        }
        if effect.lose_coins > 0 {
            self.lose_coins(effect.lose_coins);
            done = true;
        }
        if effect.snuff {
            self.state.lantern_lit = false;
            self.state.lantern_hooded = false;
            done = true;
        }
        if let Some(summon) = effect.summon {
            self.summon_hostiles(summon, feature);
            done = true;
        }
        if effect.disease {
            let all: Vec<_> = self.content.disease_ids().map(|(id, _)| id).collect();
            if !all.is_empty() {
                let disease = all[self.state.rng.index(all.len())];
                self.infect(me, disease, None);
            }
            done = true;
        }
        if let Some(disease) = effect.catch {
            self.infect(me, disease, None);
            done = true;
        }
        if let Some(wither) = effect.wither {
            let stat = Stat::ALL[self.state.rng.index(Stat::ALL.len())];
            let points = wither.roll(&mut self.state.rng);
            let actor = self.player_mut();
            let score = actor.stats.score(stat);
            actor.damage.take(stat, points, score);
            done = true;
        }
        if effect.xp > 0 {
            self.grant_share_of_level(effect.xp);
            done = true;
        }
        if effect.tithe {
            done |= self.tithe(found_once(&mut found));
        }
        if effect.teleport {
            let here = self.player_pos();
            self.teleport(me, here, events);
            done = true;
        }
        if let Some(explosion) = &effect.explode {
            self.sound_at(explosion.noise, feature);
            let burst = explosion.burst.clone();
            self.cast_burst(Caster::Place, (feature, feature), &burst, events);
            done = true;
        }
        if let Some(terrain) = effect.becomes {
            self.state.current_mut().lights.remove(&feature);
            self.set_terrain(feature, terrain);
            done = true;
        }
        // The great boons and banes (`boons.rs`).
        if let Some(gift) = effect.gift {
            self.gift_stat(gift);
            done = true;
        }
        if let Some(wrath) = effect.wrath
            && self.wrath_stat(wrath)
        {
            found(self);
            done = true;
        }
        for _ in 0..effect.levels.unsigned_abs() {
            let changed = match effect.levels > 0 {
                true => self.gain_level(),
                false => self.lose_level(),
            };
            if changed {
                found(self);
            }
            done |= changed;
        }
        if effect.levels != 0 {
            // The level's lines, after the outcome's.
            self.say_news();
        }
        if effect.talent {
            done |= self.bestow_talent(found_once(&mut found));
        }
        if effect.flaw {
            done |= self.bestow_flaw(found_once(&mut found));
        }
        done
    }

    /// Hit points back: `heal`'s roll, half the maximum, or all of them, never past it.
    fn restore_hp(&mut self, heal: Amount) {
        let content = self.content.clone();
        let roll = match heal {
            Amount::Dice(d) => d.roll(&mut self.state.rng),
            Amount::Half => self.state.player().max_hp(&content) / 2,
            Amount::Full => i32::MAX,
        };
        let me = self.player_mut();
        let max = me.max_hp(&content);
        me.hp = me.hp.saturating_add(roll.max(0)).min(max).max(me.hp);
    }

    /// Mana back: `mana`'s roll, half the maximum, or all of it, never past it.
    fn restore_mana(&mut self, mana: Amount) {
        let content = self.content.clone();
        let roll = match mana {
            Amount::Dice(d) => d.roll(&mut self.state.rng),
            Amount::Half => self.state.player().max_mana(&content) / 2,
            Amount::Full => i32::MAX,
        };
        let me = self.player_mut();
        let max = me.max_mana(&content);
        me.mana = me.mana.saturating_add(roll.max(0)).min(max).max(me.mana);
    }

    /// The player's statuses among `statuses` end (and what follows them begins). Whether any
    /// did.
    fn cure_statuses(
        &mut self,
        statuses: &[crate::content::StatusId],
        events: &mut Vec<Event>,
    ) -> bool {
        let me = self.state.player;
        let actor = self.player_mut();
        let ended: Vec<_> = (statuses.iter())
            .filter_map(|&s| actor.effects.get(s).copied())
            .collect();
        for e in &ended {
            actor.effects.remove(e.status);
        }
        self.ended(me, &ended, events);
        !ended.is_empty()
    }

    /// Whether `means` cures any status or disease the player has.
    fn diseased_or_cured_by(&self, means: &str) -> bool {
        let actor = self.state.player();
        let cures = |by: &[String]| by.iter().any(|c| c == means);
        (actor.effects.iter()).any(|e| cures(&self.content.status(e.status).cured_by))
            || (actor.diseases.iter()).any(|i| cures(&self.content.disease(i.disease).cured_by))
    }

    /// The player's nutrition `states` states better (or worse), from the least of the state
    /// it lands in: never past Satiated, never into Starving; one already there is left as is.
    fn feed_by_states(&mut self, states: i8) {
        let content = self.content.clone();
        let (Some(def), Some(before)) = (content.hunger(), self.player_hunger()) else {
            return;
        };
        // Hunger::ALL is the most fed first.
        let at = Hunger::ALL.iter().position(|&h| h == before).unwrap_or(0) as i32;
        let (best, worst) = (
            Hunger::ALL
                .iter()
                .position(|&h| h == Hunger::Satiated)
                .unwrap_or(1) as i32,
            Hunger::ALL
                .iter()
                .position(|&h| h == Hunger::Fainting)
                .unwrap_or(5) as i32,
        );
        let to = (at - i32::from(states)).clamp(best.min(at), worst.max(at));
        let target = Hunger::ALL[usize::try_from(to).unwrap_or(0)];
        if target == before {
            return;
        }
        let Some(least) = def.least(target) else {
            return;
        };
        if let Some(fed) = self.player_mut().satiation.as_mut() {
            fed.nutrition = least;
            fed.banked = 0;
        }
        self.hunger_news(before);
    }

    /// Where something left by a place lies: the walkable cell beside its feature on `feature`
    /// that no one stands on, nearest the player (ties by cell order); else the player's own.
    fn foot_of(&self, feature: Pos) -> Pos {
        let level = self.state.current();
        let me = self.player_pos();
        let mut cells: Vec<Pos> = (Dir::ALL.iter().map(|&d| feature.step(d)))
            .filter(|&p| level.is_walkable(&self.content, p))
            .filter(|&p| p == me || self.state.actor_at(&level.id, p).is_none())
            .filter(|&p| p != me)
            .collect();
        cells.sort_by_key(|p| (p.dist2(me), *p));
        cells.first().copied().unwrap_or(me)
    }

    /// One of `effect.objects`, by weight, at the feature's foot.
    fn leave_objects(&mut self, effect: &OutcomeEffect, feature: Pos) {
        let total: u32 = effect.objects.iter().map(|o| o.2).sum();
        let mut at = self.state.rng.below(total.max(1));
        let Some(&(object, count, _)) = effect.objects.iter().find(|o| {
            let hit = at < o.2;
            at = at.saturating_sub(o.2);
            hit
        }) else {
            return;
        };
        let n = u32::try_from(count.roll(&mut self.state.rng).max(1)).unwrap_or(1);
        let at = self.foot_of(feature);
        self.lay(at, Stack::new(object, n));
    }

    /// An item of loot at the floor's item level plus `bonus`, at least of `grade` (a relic's
    /// `rare`: the top rarity its base allows), at the feature's foot.
    fn leave_loot(&mut self, bonus: u8, grade: LootGrade, feature: Pos) {
        let content = self.content.clone();
        let level = content
            .item_level(&self.state.current_level)
            .saturating_add(bonus);
        let luck = loot::Luck::of(&self.state.player().talent_mods(&content));
        let rng = &mut self.state.rng;
        let item = loot::roll_with(&content, rng, level, grade, Source::Loot, luck);
        if let Some(stack) = item {
            let at = self.foot_of(feature);
            self.lay(at, stack);
        }
    }

    /// `stack` lies on `at` of the current level, on top of its pile.
    fn lay(&mut self, at: Pos, stack: Stack) {
        let content = self.content.clone();
        let level = self.state.current_mut();
        add_to_pile(level.objects.entry(at).or_default(), &[stack], &content);
    }

    /// Every trap within `reach` cells of the player becomes known.
    fn sense_traps(&mut self, reach: i32) {
        let me = self.player_pos();
        let near: Vec<Pos> = (self.state.current().traps.keys())
            .copied()
            .filter(|p| p.chebyshev(me) <= reach)
            .collect();
        for p in near {
            self.know_trap(p);
        }
    }

    /// One identified magic item the player carries is unidentified again ([`Stack::forget`]).
    /// Whether one was; `found` before.
    fn forget_one(&mut self, found: impl FnOnce(&mut Self)) -> bool {
        let known: Vec<usize> = (self.state.player().inventory.iter().enumerate())
            .filter(|(_, s)| s.known_magic().is_some())
            .map(|(i, _)| i)
            .collect();
        if known.is_empty() {
            return false;
        }
        found(self);
        let i = known[self.state.rng.index(known.len())];
        if let Some(stack) = self.player_mut().inventory.get_mut(i) {
            stack.forget();
        }
        true
    }

    /// What of `harm`'s roll reaches the player, less what they resist.
    fn harm_taken(&mut self, harm: Harmful) -> i32 {
        let roll = harm.damage.roll(&mut self.state.rng).max(0);
        self.resisted(self.state.player, Damage::new(roll, harm.kind))
    }

    /// `percent` of every stack of coins the player carries vanishes (rounded down).
    fn lose_coins(&mut self, percent: u8) {
        let content = self.content.clone();
        let coins = content.item_class_id("coins");
        let me = self.player_mut();
        let mut emptied = Vec::new();
        for (i, stack) in me.inventory.iter_mut().enumerate() {
            if Some(content.object(stack.object).class) != coins {
                continue;
            }
            stack.count -= stack.count * u32::from(percent) / 100;
            if stack.count == 0 {
                emptied.push(i);
            }
        }
        remove_stacks(me, &emptied);
    }

    /// The most valuable stack the player carries (worn and wielded things too; the first of
    /// equal ones) vanishes. Whether one did; `found` before.
    fn tithe(&mut self, found: impl FnOnce(&mut Self)) -> bool {
        let content = self.content.clone();
        let best = (self.state.player().inventory.iter().enumerate())
            .map(|(i, s)| (s.value(&content), i))
            .filter(|&(v, _)| v > 0)
            .max_by_key(|&(v, i)| (v, std::cmp::Reverse(i)));
        let Some((_, i)) = best else {
            return false;
        };
        found(self);
        remove_stacks(self.player_mut(), &[i]);
        true
    }

    /// The share `percent` of the experience from the player's level to the next: the
    /// experience it takes, as a kill's would be given ([`World::grant_xp`]).
    fn grant_share_of_level(&mut self, percent: u8) {
        let rules = self.content.experience();
        let level = self.player_level();
        if level >= rules.max_level() {
            return;
        }
        let factor = crate::content::Factor::of(
            &self.content,
            self.state.character.as_ref().map(|c| &c.spec),
        );
        let span = rules
            .threshold_for(level + 1, factor)
            .saturating_sub(rules.threshold_for(level, factor));
        self.grant_xp(span * u32::from(percent) / 100);
    }

    /// `summon.count` hostile creatures of the floor's danger plus `summon.danger` come, awake,
    /// where `summon.near` says (module docs; spec §7 "Special rooms"). Nothing comes on a level
    /// without a danger, or where no cell is free.
    fn summon_hostiles(&mut self, summon: OutcomeSummon, feature: Pos) {
        let id = self.state.current_level.clone();
        let Some((_, _, spawning)) = self.content.spawning(&id) else {
            return;
        };
        let spawning = spawning.clone();
        let base = self.danger(&id);
        if base == 0 {
            return;
        }
        let danger = base.saturating_add(summon.danger);
        let player = self.content.creature_id("player");
        let hostile = |content: &crate::content::Content, c| {
            !content.creature(c).unique
                && player.is_some_and(|p| content.stance(c, p) == Stance::Hostile)
        };
        let once = self.state.progress.uniques.clone();
        let own: Vec<_> = (self
            .content
            .candidates(&spawning.population, &once)
            .into_iter())
        .filter(|&(c, _, _)| hostile(&self.content, c))
        .collect();
        let generic: Vec<_> = (self
            .content
            .candidates(&spawning.generic, &once)
            .into_iter())
        .filter(|&(c, _, _)| hostile(&self.content, c))
        .collect();
        let count = summon.count.roll(&mut self.state.rng).max(1);
        for _ in 0..count {
            let rng = &mut self.state.rng;
            let drawn = danger::draw(rng, &own, &generic, danger, &spawning.odds);
            // Too deep for the branch's population: its deepest type.
            let creature = drawn.or_else(|| {
                (own.iter().chain(&generic))
                    .max_by_key(|&&(c, level, _)| (level, std::cmp::Reverse(c)))
                    .map(|&(c, _, _)| c)
            });
            let Some(creature) = creature else {
                return;
            };
            let Some(pos) = self.summon_cell(summon.near, feature) else {
                return;
            };
            let spawn = CreatureSpawn {
                creature,
                faction: None,
                alignment: None,
                npc: None,
                pos,
                route: Vec::new(),
                carry: Vec::new(),
                equipment: Default::default(),
                rolls_gear: true,
                asleep: false,
            };
            make_creature(&mut self.state, &self.content, &id, &spawn);
            self.spawned += 1;
        }
    }

    /// Where a summoned creature comes: beside the feature (the free cell nearest it, ties by
    /// cell order), or at the room's edge (its walkable cells farthest from the feature, out of
    /// the player's view if any is; one drawn).
    fn summon_cell(&mut self, near: Near, feature: Pos) -> Option<Pos> {
        let level = self.state.current();
        let content = &self.content;
        let free = |p: Pos| {
            level.is_walkable(content, p)
                && self.state.actor_at(&level.id, p).is_none()
                && level.stairs_at(p).is_none()
                && !level.is_door(content, p)
        };
        match near {
            Near::Feature => {
                let mut ring: Vec<Pos> = (1..=3)
                    .flat_map(|r| {
                        (-r..=r).flat_map(move |dy| {
                            (-r..=r).map(move |dx| Pos::new(feature.x + dx, feature.y + dy))
                        })
                    })
                    .filter(|&p| free(p))
                    .collect();
                ring.sort_by_key(|p| (p.chebyshev(feature), *p));
                ring.first().copied()
            }
            Near::Edge => {
                let room = level.room_at(feature)?;
                let cells: Vec<Pos> = room.cells().into_iter().filter(|&p| free(p)).collect();
                let far = cells.iter().map(|p| p.chebyshev(feature)).max()?;
                let edge: Vec<Pos> = (cells.iter().copied())
                    .filter(|p| p.chebyshev(feature) == far)
                    .collect();
                let hidden: Vec<Pos> = (edge.iter().copied())
                    .filter(|&p| !self.perception.sees_actors_at(p))
                    .collect();
                let pool = if hidden.is_empty() { edge } else { hidden };
                let i = self.state.rng.index(pool.len());
                pool.get(i).copied()
            }
        }
    }

    /// A place (`Caster::Place`: an altar's fire) does `damage` to `target`, with no roll to hit,
    /// less what it resists (spec §7 "Special rooms"): the player is told when they are hit, or
    /// see who is; a kill is no one's.
    pub(super) fn place_hurts(&mut self, target: ActorId, damage: Damage, events: &mut Vec<Event>) {
        let Some(to) = self.state.actors.get(&target).map(|a| a.pos) else {
            return;
        };
        let taken = self.resisted(target, damage);
        let verb = (damage.kind).map_or_else(
            || "hits".to_string(),
            |k| self.content.damage_type(k).verb.clone(),
        );
        if target == self.state.player {
            self.say(match taken {
                t if t <= 0 => "The altar's fire washes over you harmlessly.".to_string(),
                _ => format!("The altar's fire {verb} you."),
            });
        } else if self.perception.sees_actors_at(to) {
            let name = self.creature_called(target).to_lowercase();
            self.say(format!("The altar's fire {verb} {name}."));
        }
        if taken <= 0 {
            return;
        }
        if !self.hurt(target, taken, Harm::Place, events) {
            self.wounded(target, &Blow::plain(taken, damage.kind), None, events);
        }
    }
}

/// A hook's `found` call, at most once.
fn found_once<'f, W>(found: &'f mut impl FnMut(&mut W)) -> impl FnOnce(&mut W) + 'f {
    move |w| found(w)
}

/// Takes the stacks at `positions` out of `actor`'s pack (and off them).
fn remove_stacks(actor: &mut super::Actor, positions: &[usize]) {
    let mut sorted = positions.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    for &i in sorted.iter().rev() {
        if i < actor.inventory.len() {
            actor.inventory.remove(i);
        }
    }
    actor.equipment.after_removal(&Positions::of(sorted));
}
