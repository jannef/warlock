//! Miscasts (spec §8 "Spells": "Miscasts", #1271): every spell cast with the caster's own mana,
//! the player's (`cast.rs`) or a creature's (`creature_cast.rs`), makes a **Spellcraft check**
//! once it has gone off; a scroll, a wand, a flask, and a miscast's own echo make none.
//!
//! - **The check** ([`check::roll`]): 2d12 + [`Actor::cast_bonus`] (the modifier of
//!   `data/miscast.toml`'s `stat`, what talents add to its checks, and Spellcraft's `cast`
//!   points, [`Effect::Cast`]) against [`Actor::cast_difficulty`] (the rules' `base` +
//!   `per_level` × the spell's level + a point for each `armour_percent` of
//!   [`Actor::armour_spell_cost`]). A natural 2 always fails, so a cast always may miscast.
//! - **The table**: a failure rolls the table's die (`die`, rolled again and added on its highest
//!   face), adds `per_margin` × how far the check fell short, and takes off
//!   [`Actor::miscast_ward`] (Spellcraft's `ward` points, [`Effect::Ward`], and the stance's
//!   `miscast`); the total, at least 1, picks the one entry of `data/miscasts/` whose range
//!   holds it ([`crate::content::Miscast`]).
//! - **The entry**: its message (the player's own, or its `seen` line of a creature the player
//!   sees), then its effect, the mishaps' machinery generalized to any caster: statuses, mana,
//!   the teleport trap's [`World::teleport`], stat marks, a [`Drain`], a summon beside the
//!   caster, harm through [`World::resisted`] and [`World::hurt`], the mana refunded, the spell
//!   again ([`Cast::by_creature`]: already paid), a wild surge (a damage spell drawn among all
//!   with a roll, as a burst at someone near), a swap of places, and a boon on someone near.
//!   "Near" is within [`NEAR`] cells in a clear line ([`World::clear_shot`]), in actor-id order.
//!   A death of the player by their own miscast is [`Cause::Miscast`].
//!
//! Determinism: the check's two draws, the table's die, then the effect's own draws, all from the
//! gameplay stream, in that order; no draw without `data/miscast.toml`.

use super::cast::Cast;
use super::fight::Harm;
use super::spell::Caster;
use super::talk::capitalized;
use super::{Actor, ActorId, Cause, World};
use crate::action::{Event, ReadTarget};
use crate::check::{self, Checked};
use crate::content::{
    Burst, Content, CreatureId, CreatureSpawn, Drain, Miscast, MishapEffect, SpellData, SpellId,
    SpellPurpose,
};
use crate::damage::Damage;
use crate::dice::Roll;
use crate::geom::{Dir, Pos};
use crate::skills::Effect;
use crate::stats::Stat;

/// How far "near the caster" reaches for a surge, a swap, and a boon.
pub const NEAR: i32 = 6;

/// What a cast was aimed at, so a miscast's echo goes the same way.
#[derive(Clone, Copy, Debug)]
pub(super) enum Aim {
    /// The player's cast, at its cell or stack (or none).
    Player(Option<ReadTarget>),
    /// A creature's cast at actor `on`.
    Creature(ActorId),
}

impl Actor {
    /// The bonus of its Spellcraft check on a cast (module docs): 0 without the rules.
    pub fn cast_bonus(&self, content: &Content) -> i32 {
        let Some(rules) = content.miscast_rules() else {
            return 0;
        };
        self.effective_stats(content).modifier(rules.stat)
            + self.talent_mods(content).check(rules.stat)
            + self.skill_effect(content, Effect::Cast)
    }

    /// The difficulty of its Spellcraft check on a cast of `def`, in what it wears; `None`
    /// without the rules (no check).
    pub fn cast_difficulty(&self, content: &Content, def: &SpellData) -> Option<i32> {
        let rules = content.miscast_rules()?;
        Some(rules.difficulty(def.level, self.armour_spell_cost(content)))
    }

    /// What it takes off a miscast's total: Spellcraft's `ward` and its stance's `miscast`.
    pub fn miscast_ward(&self, content: &Content) -> i32 {
        self.skill_effect(content, Effect::Ward) + content.stance_def(self.stance).miscast
    }
}

/// The entry of `table` (ordered by its totals) that `total` falls in.
pub fn entry_for(table: &[Miscast], total: i32) -> Option<&Miscast> {
    table.iter().find(|m| m.covers(total))
}

impl World {
    /// `caster`'s cast `cast` went off, aimed by `aim`: its Spellcraft check, and on a failure
    /// the miscast table's entry (module docs). Nothing for a dead caster.
    pub(super) fn after_cast(
        &mut self,
        caster: ActorId,
        cast: &Cast<'_>,
        aim: Aim,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(rules) = content.miscast_rules().copied() else {
            return;
        };
        if self.state.death.is_some() && caster == self.state.player {
            return;
        }
        let Some(me) = self.state.actors.get(&caster).filter(|a| a.hp > 0) else {
            return;
        };
        let (bonus, ward) = (me.cast_bonus(&content), me.miscast_ward(&content));
        let Some(difficulty) = me.cast_difficulty(&content, cast.def) else {
            return;
        };
        let Checked::Failed { by } = check::roll(&mut self.state.rng, bonus, difficulty) else {
            return;
        };
        let total = self.miscast_total(rules.die, rules.per_margin * by, ward);
        let Some(entry) = entry_for(content.miscasts(), total).cloned() else {
            return;
        };
        self.miscast(caster, cast, aim, &entry, events);
    }

    /// The table's total: the die, rolled again and added on its highest face, + `margin` −
    /// `ward`, at least 1.
    pub(super) fn miscast_total(&mut self, die: u32, margin: i32, ward: i32) -> i32 {
        let mut rolled: i32 = 0;
        loop {
            let face = self.state.rng.below(die) + 1;
            rolled = rolled.saturating_add(i32::try_from(face).unwrap_or(i32::MAX));
            if face < die {
                break;
            }
        }
        rolled.saturating_add(margin).saturating_sub(ward).max(1)
    }

    /// Miscast `entry` befalls `caster` after `cast` (module docs): said, then done.
    pub(super) fn miscast(
        &mut self,
        caster: ActorId,
        cast: &Cast<'_>,
        aim: Aim,
        entry: &Miscast,
        events: &mut Vec<Event>,
    ) {
        let player = self.state.player;
        let Some(me) = self.state.actors.get(&caster) else {
            return;
        };
        let here = me.pos;
        if caster == player {
            self.say(entry.message.clone());
        } else if self.perception.sees_actors_at(here) {
            let who = capitalized(&self.called(me));
            self.say(entry.seen.replace("{who}", &who));
        }
        let alive = self.state.death.is_none();
        let content = self.content.clone();
        match entry.effect {
            MishapEffect::Nothing | MishapEffect::Learn => {}
            MishapEffect::Status { status, severity } => {
                self.afflict(caster, status, severity, None, events);
            }
            MishapEffect::Backfire { full } => self.miscast_backfire(caster, cast, full, events),
            MishapEffect::Mana { percent } => {
                if let Some(me) = self.state.actors.get_mut(&caster) {
                    let lost = (me.mana.max(0) * i32::from(percent) + 99) / 100;
                    me.mana = (me.mana - lost).max(0);
                }
            }
            MishapEffect::Teleport => self.teleport(caster, here, events),
            MishapEffect::LoseMark => {
                let Some(me) = self.state.actors.get(&caster) else {
                    return;
                };
                let marked: Vec<Stat> = (Stat::ALL.iter().copied())
                    .filter(|&s| me.stat_marks[s] > 0)
                    .collect();
                if !marked.is_empty() {
                    let i = self.state.rng.below(marked.len() as u32) as usize;
                    if let Some(me) = self.state.actors.get_mut(&caster) {
                        me.stat_marks[marked[i]] -= 1;
                    }
                }
            }
            MishapEffect::Mark { stat } => {
                if let Some(me) = self.state.actors.get_mut(&caster) {
                    me.stat_marks[stat] = me.stat_marks[stat].saturating_add(1);
                }
            }
            MishapEffect::Drain {
                stat,
                points,
                difficulty,
            } => {
                let drain = Drain {
                    stat,
                    points,
                    check: Stat::Willpower,
                    difficulty,
                };
                self.drain(caster, drain, None, events);
            }
            MishapEffect::Summon { creature } => self.miscast_summon(here, creature),
            MishapEffect::Harm { dice, damage } => {
                let amount = dice.roll(&mut self.state.rng).max(1);
                let kind = damage.or(cast.def.damage_type);
                let amount = self.resisted(caster, Damage::new(amount, kind));
                if amount > 0 {
                    self.hurt(caster, amount, Harm::Place, events);
                }
            }
            MishapEffect::Refund => {
                if let Some(me) = self.state.actors.get(&caster) {
                    let back = me.spell_cost(&content, cast.def);
                    let most = me.max_mana(&content);
                    if let Some(me) = self.state.actors.get_mut(&caster) {
                        me.mana = (me.mana + back).min(most.max(me.mana));
                    }
                }
            }
            MishapEffect::Twice => self.miscast_echo(caster, cast, aim, events),
            MishapEffect::Surge => self.miscast_surge(caster, events),
            MishapEffect::Swap => {
                // A rooted one never swaps (`rooted.rs`).
                let mut near = self.near_caster(caster);
                near.retain(|&(id, _)| !self.is_rooted(id));
                if !near.is_empty() && !self.is_rooted(caster) {
                    let i = self.state.rng.below(near.len() as u32) as usize;
                    let (other, there) = near[i];
                    self.teleport_to(caster, (here, there), events);
                    self.teleport_to(other, (there, here), events);
                }
            }
            MishapEffect::Boon { status, severity } => {
                let mut near: Vec<ActorId> = vec![caster];
                near.extend(self.near_caster(caster).into_iter().map(|(id, _)| id));
                let i = self.state.rng.below(near.len() as u32) as usize;
                self.afflict(near[i], status, severity, None, events);
            }
        }
        if caster == player
            && alive
            && let Some(death) = &mut self.state.death
        {
            death.by = None;
            death.cause = Some(Cause::Miscast(cast.spell));
        }
    }

    /// Everyone but `caster` on its level within [`NEAR`] cells of it in a clear line, with
    /// where they stand, in actor-id order.
    fn near_caster(&self, caster: ActorId) -> Vec<(ActorId, Pos)> {
        let Some(me) = self.state.actors.get(&caster) else {
            return Vec::new();
        };
        let (level, here) = (me.level.clone(), me.pos);
        (self.state.actors.values())
            .filter(|a| a.id != caster && a.level == level && a.hp > 0)
            .filter(|a| a.pos.chebyshev(here) <= NEAR)
            .filter(|a| self.clear_shot(here, a.pos, caster, a.id))
            .map(|a| (a.id, a.pos))
            .collect()
    }

    /// The spell of `cast` goes off on its caster: a damage spell as a burst around them (its
    /// radius, else 1) at its base roll, or the roll it was cast with when `full`; a hex on
    /// them at its degrees; any other kind does nothing more than the entry's words.
    fn miscast_backfire(
        &mut self,
        caster: ActorId,
        cast: &Cast<'_>,
        full: bool,
        events: &mut Vec<Event>,
    ) {
        let def = cast.def;
        match def.kind {
            SpellPurpose::Damage => {
                let roll = if full { cast.roll } else { def.damage };
                if let Some(roll) = roll {
                    let Some(here) = self.state.actors.get(&caster).map(|a| a.pos) else {
                        return;
                    };
                    self.miscast_burst(caster, def, (here, roll), events);
                }
            }
            SpellPurpose::Hex => {
                if let (Some((status, _)), Some(degrees)) = (def.effect.status, cast.degrees) {
                    self.afflict(caster, status, degrees, None, events);
                }
            }
            _ => {}
        }
    }

    /// `def` goes off as a burst of `roll` centred on `center`, by `caster` (its radius, else
    /// 1), said as any spell's blows are.
    fn miscast_burst(
        &mut self,
        caster: ActorId,
        def: &SpellData,
        (center, roll): (Pos, Roll),
        events: &mut Vec<Event>,
    ) {
        let burst = Burst {
            spell: def.look.effect.clone().unwrap_or_default(),
            radius: def.radius.map_or(1, i32::from),
            damage: roll,
            reroll: 0,
            kind: def.damage_type,
            blast: 0,
            ice: None,
        };
        let mut made = Vec::new();
        self.sound_at(def.look.noise, center);
        self.cast_burst(Caster::Actor(caster), (center, center), &burst, &mut made);
        self.spell_lands(caster, (def, &[]), made, None, events);
    }

    /// The spell goes off again at what it was aimed at, already paid for, with no check of its
    /// own.
    fn miscast_echo(
        &mut self,
        caster: ActorId,
        cast: &Cast<'_>,
        aim: Aim,
        events: &mut Vec<Event>,
    ) {
        let content = self.content.clone();
        let Some(me) = self.state.actors.get(&caster) else {
            return;
        };
        let echo = Cast::by_creature(me, &content, (cast.spell, cast.def));
        match aim {
            Aim::Player(target) => {
                self.perform(&echo, target, events);
            }
            Aim::Creature(on) => {
                let here = me.pos;
                let Some(at) = self.state.actors.get(&on).map(|a| a.pos) else {
                    return;
                };
                self.creature_spell_does((caster, here), &echo, (on, at), events);
            }
        }
    }

    /// A wild surge: a damage spell drawn among all that roll one goes off as a burst around
    /// someone near the caster, drawn the same way, at its base roll; no one near, nothing.
    fn miscast_surge(&mut self, caster: ActorId, events: &mut Vec<Event>) {
        let content = self.content.clone();
        let near = self.near_caster(caster);
        if near.is_empty() {
            return;
        }
        let spells: Vec<SpellId> = (content.spells().iter().enumerate())
            .filter(|(_, s)| s.kind == SpellPurpose::Damage && s.damage.is_some())
            .map(|(i, _)| SpellId(u16::try_from(i).unwrap_or(u16::MAX)))
            .collect();
        if spells.is_empty() {
            return;
        }
        let spell = spells[self.state.rng.below(spells.len() as u32) as usize];
        let (_, at) = near[self.state.rng.below(near.len() as u32) as usize];
        let def = content.spell(spell);
        let Some(roll) = def.damage else {
            return;
        };
        self.miscast_burst(caster, def, (at, roll), events);
    }

    /// A `creature` comes to the first free cell around `here`, of its own kind's faction.
    fn miscast_summon(&mut self, here: Pos, creature: CreatureId) {
        let level = self.state.current();
        let free = (Dir::ALL.iter().map(|&d| here.step(d))).find(|&p| {
            level.is_walkable(&self.content, p) && self.state.actor_at(&level.id, p).is_none()
        });
        let Some(pos) = free else {
            return;
        };
        let id = level.id.clone();
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
            status: Vec::new(),
        };
        super::travel::make_creature(&mut self.state, &self.content, &id, &spawn);
        self.spawned += 1;
    }
}
