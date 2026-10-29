//! The balance probe's stat training (#727; `docs/design/stat-training.md` §6.2, §13 T6): the
//! days of a trained career, lived through the game's own training hooks. Each level's kills (the
//! experience curve's, `Experience::kills_for`) are fought blow by blow with `combat::strike`
//! against the monsters of the player's level, and every blow goes through `World::train_fight`
//! (blow, critical, block, parry, hurt, big hit), a dodged one through the Dodge use, a kill
//! through `killed` and `discovered`, a cast through `World::spend_mana` (cast) and
//! `World::train_spell` (spell), an attack spell's roll through `train_fight` (aimed). What a
//! day holds besides the fights is [`COMMON`] and the class's [`Build::extras`] (first guesses,
//! the design's "3 to 10 marks a level" region); those go through `World::train_stat_times`, the
//! uses' own chances. Each day ends with `World::daily_stat_check`; the level's check is
//! `levelup:auto`'s.

use super::{Build, Gear, Monster, Side, casts, dress, side_of};
use crate::action::AttackKind;
use crate::burden::Burden;
use crate::combat::{self, Category, Profile, Strike, Target};
use crate::content::{CreatureId, Resolution, StatUse};
use crate::rng::Rng;
use crate::stats::Stat;
use crate::world::{ActorId, World};
use std::collections::BTreeSet;

/// The game days level `level` takes to reach (the design's §3.1: one for levels 2 to 5, two to
/// 15, three to 30, four beyond: 189 days to level 60).
pub(super) fn days(level: u8) -> u32 {
    match level {
        0..=5 => 1,
        6..=15 => 2,
        16..=30 => 3,
        _ => 4,
    }
}

/// The longest fight measured, in rounds: a monster the player cannot kill by then is left
/// alone (it still counts as the kill, so the level's kills stay the curve's).
const MAX_ROUNDS: u32 = 40;

/// Steps a day that are actions (for a burden that slows them).
const STEPS: f64 = 300.0;

/// What every character does in a day besides fighting: a use and how many times a day.
pub(super) const COMMON: &[(StatUse, f64)] = &[
    // A rest and two meals.
    (StatUse::Endured, 3.0),
    (StatUse::Chat, 2.0),
    (StatUse::Greeted, 0.3),
    (StatUse::Trade, 1.0),
    (StatUse::Swapped, 1.0),
    // Steps within earshot of something that could hear them.
    (StatUse::Sneak, 30.0),
    (StatUse::Searched, 20.0),
    (StatUse::Found, 0.3),
    (StatUse::Spot, 0.5),
    (StatUse::Read, 0.3),
    (StatUse::Identified, 0.1),
    (StatUse::Explored, 0.3),
    (StatUse::Marched, 10.0),
    (StatUse::Hauled, 0.5),
    (StatUse::Kick, 0.5),
    (StatUse::Climb, 0.1),
    (StatUse::Waded, 2.0),
    (StatUse::Picked, 0.2),
    (StatUse::Disarmed, 0.1),
    (StatUse::KeptWits, 0.5),
    (StatUse::ShookOff, 0.5),
    (StatUse::Appraised, 0.2),
    (StatUse::Foraged, 0.2),
];

/// The `[training]` knobs swept (`price_per_point`, `ramp_divisor`), the data's pair among them.
pub(super) const SWEEP: &[(u16, u16)] = &[
    (10, 2),
    (10, 3),
    (10, 4),
    (10, 6),
    (15, 2),
    (15, 4),
    (20, 4),
];

/// One swept pair: each trained career's scores at level 60, by class.
pub(super) struct Swept {
    pub per_point: u16,
    pub ramp: u16,
    pub finals: Vec<(&'static str, [u8; 6])>,
}

/// How many times a use at `rate` a day comes on day `day` (counted from 0 over the career):
/// whole counts that add up to the rate, no state.
fn on_day(rate: f64, day: u32) -> u16 {
    let at = |d: u32| (rate * f64::from(d)).floor();
    (at(day + 1) - at(day)) as u16
}

/// A career's training as it went: what the career module reads at each measured level.
#[derive(Default)]
pub(super) struct Log {
    /// Game days lived.
    pub day: u32,
    /// The day each stat first rose (1-based), if it has.
    pub first_rise: [Option<u32>; 6],
    /// Marks earned in each stat over the last level, and its days.
    pub earned: [i64; 6],
    pub earned_days: u32,
    /// The creature types killed (the `discovered` use's first kills).
    killed: BTreeSet<CreatureId>,
}

impl Log {
    /// Notes the rises between `before` and `w`'s scores now.
    pub fn rises(&mut self, w: &World, before: [u8; 6]) {
        let now = scores(w);
        for i in 0..6 {
            if now[i] > before[i] && self.first_rise[i].is_none() {
                self.first_rise[i] = Some(self.day.max(1));
            }
        }
    }
}

/// The player's stored scores, `Stat::ALL` order.
pub(super) fn scores(w: &World) -> [u8; 6] {
    Stat::ALL.map(|s| w.state.player().stats.score(s))
}

fn piles(w: &World) -> [i64; 6] {
    Stat::ALL.map(|s| i64::from(w.state.player().stat_marks[s]))
}

/// The monsters a player of `level` fights: those within two levels, else the nearest below.
fn foes(monsters: &[Monster], level: u8) -> Vec<&Monster> {
    let near = |lo: i32, hi: i32| -> Vec<&Monster> {
        let l = i32::from(level);
        (monsters.iter())
            .filter(|m| (l + lo..=l + hi).contains(&i32::from(m.level)))
            .collect()
    };
    let mut out = near(-2, 2);
    if out.is_empty() {
        out = near(-10, 0);
    }
    out
}

/// The spell `w`'s player fights with, if their build casts: the damage spell of the most mean
/// damage a cast among those costing at most a fifth of the pool (else the cheapest).
fn weapon_spell(w: &World) -> Option<super::Cast> {
    let pool = w.state.player().max_mana(&w.content);
    let (cheap, dear): (Vec<_>, Vec<_>) = (casts(w).into_iter())
        .filter(|c| c.cost > 0)
        .partition(|c| c.cost * 5 <= pool);
    let mean = |c: &super::Cast| super::mean(c.dice.0).min(f64::from(c.lifted));
    match cheap.is_empty() {
        false => cheap.into_iter().max_by(|a, b| mean(a).total_cmp(&mean(b))),
        true => dear.into_iter().min_by_key(|c| c.cost),
    }
}

/// Lives the days that bring `w`'s player (of `build`, at level `level`) to the next level:
/// each day's share of the level's kills, fought through the hooks, its other uses, and the
/// daily check.
pub(super) fn live_level(
    w: &mut World,
    build: &Build,
    level: u8,
    monsters: &[Monster],
    seed: u64,
    log: &mut Log,
) {
    let content = w.content.clone();
    let player = w.state.player;
    let mut dressed = w.clone();
    dress(&mut dressed, build, level, Gear::Plain, seed);
    let me = side_of(&dressed, player);
    // The burden of the dressed player without the kit's pieces the gear replaced (they would be
    // left behind).
    let mut load = dressed.load_of(player);
    let (kit, worn) = (w.state.player(), dressed.state.player());
    for (i, stack) in worn.inventory.iter().enumerate() {
        if kit.equipment.holds(i) && !worn.equipment.holds(i) {
            load.carried = load.carried.saturating_sub(stack.weight(&content));
        }
    }
    let slowed = load.penalty() != Burden::Unburdened;
    let spell = (build.class == "wizard")
        .then(|| weapon_spell(&dressed))
        .flatten();
    let foes = foes(monsters, level);
    let kills = content.experience().kills_for(level + 1);
    let days = days(level + 1);
    let mut rng = Rng::stream(seed * 1000 + u64::from(level), "stat training");
    let foe = ActorId(u32::MAX);
    log.earned = [0; 6];
    log.earned_days = days;
    let mut fought = 0u32;
    for d in 0..days {
        let start = piles(w);
        let today = kills * (d + 1) / days - kills * d / days;
        let mut rounds = 0u32;
        for _ in 0..today {
            let Some(m) = foes.get(fought as usize % foes.len().max(1)) else {
                break;
            };
            let them = &m.geared[(fought as usize + seed as usize) % m.geared.len()];
            fought += 1;
            rounds += fight(w, &me, them, spell.as_ref(), build, &mut rng, foe);
            if m.level >= level {
                w.train_stat(StatUse::Killed);
            }
            if log.killed.insert(m.creature) {
                w.train_stat(StatUse::Discovered);
            }
            if build.casts_a_kill {
                cast(w, None);
            }
        }
        let day = log.day;
        for &(used, rate) in COMMON.iter().chain(build.extras) {
            let n = on_day(rate, day);
            if n > 0 {
                w.train_stat_times(used, None, n);
            }
        }
        if slowed {
            let actions = u16::try_from(rounds + STEPS as u32).unwrap_or(u16::MAX);
            w.train_stat_times(StatUse::Burdened, None, actions);
        }
        // A quest done every other level.
        if d == 0 && level.is_multiple_of(2) {
            w.train_stat(StatUse::Quest);
        }
        let end = piles(w);
        for i in 0..6 {
            log.earned[i] += end[i] - start[i];
        }
        log.day += 1;
        let before = scores(w);
        let now = w.state.time;
        w.state.next_stat_check = Some(now);
        w.daily_stat_check(player, now);
        log.rises(w, before);
    }
}

/// Spends a cast's mana as `World::spend_mana` does (the pool refilled first: the rests between
/// fights are `COMMON`'s), and trains the spell's stats when it is one of the player's.
fn cast(w: &mut World, spell: Option<&super::Cast>) {
    let player = w.state.player;
    let content = w.content.clone();
    let max = w.state.player().max_mana(&content);
    w.player_mut().mana = max;
    let cost = spell.map_or(1, |c| c.cost.max(1));
    w.spend_mana(player, cost);
    if let Some(c) = spell {
        w.train_spell(content.spell(c.spell));
    }
}

/// One fight to the kill (or [`MAX_ROUNDS`]): the player's blows, or casts of `spell`, and the
/// monster's, each through the training hooks. Returns the rounds.
fn fight(
    w: &mut World,
    me: &Side,
    them: &Side,
    spell: Option<&super::Cast>,
    build: &Build,
    rng: &mut Rng,
    foe: ActorId,
) -> u32 {
    let player = w.state.player;
    let mut hp = them.hp;
    let (mut mine, mut theirs) = (0.0, 0.0);
    let mut first = true;
    for round in 1..=MAX_ROUNDS {
        if let Some(c) = spell {
            cast(w, Some(c));
            let amount = match c.resolution {
                Resolution::Attack | Resolution::Touch => {
                    let p = Profile {
                        damage: c.dice.0,
                        ..c.aim.clone()
                    };
                    let blow = combat::strike(&p, &Target::bare(them.missile), rng);
                    w.train_fight((player, foe), AttackKind::Spell, &blow);
                    blow.damage
                }
                Resolution::Auto | Resolution::Area => c.dice.0.roll(rng).max(0),
            };
            hp -= crate::world::spell::capped(amount, Some(c.lifted));
        } else {
            mine += me.blows;
            while mine >= 1.0 && hp > 0 {
                mine -= 1.0;
                for p in std::iter::once(&me.blow).chain(&me.off) {
                    let blow = combat::strike(p, &them.target, rng);
                    w.train_fight((player, foe), AttackKind::Melee, &blow);
                    if first && build.class == "thief" && blow.strike.is_hit() {
                        w.train_stat(StatUse::SneakAttack);
                    }
                    first = false;
                    hp -= blow.damage;
                }
            }
        }
        if hp <= 0 {
            return round;
        }
        theirs += them.blows;
        while theirs >= 1.0 {
            theirs -= 1.0;
            let blow = combat::strike(&them.blow, &me.target, rng);
            let dodged = matches!(
                blow.strike,
                Strike::Missed {
                    by: Some(Category::Dexterity | Category::Dodge)
                }
            );
            if dodged {
                w.train_stat(StatUse::Dodge);
            }
            w.train_fight((foe, player), AttackKind::Melee, &blow);
        }
    }
    MAX_ROUNDS
}
