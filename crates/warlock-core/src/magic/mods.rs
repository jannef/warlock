//! What magic items do (spec §8 "Magic items"): their affixes' effects, summed.
//!
//! [`Mods::local`] is what an item's affixes do to itself and to the attacks made with it;
//! [`Mods::worn`] what everything an actor wears and wields does to the actor. Within one sum,
//! values add up, except Deflection and natural armour, of which the best counts (as within
//! their category of Defence), and light, of which the brightest (the largest radius) counts.
//! An unidentified item's effects count all the same: identifying only tells the player what
//! they are.
//!
//! The rules read them in one place each: combat (`Actor::melee`, `ranged`, `thrown`,
//! `defence`, `max_hp`), mana (`Actor::max_mana`, `World::mana_rate`), stats
//! (`Actor::effective_stats`), healing (`World::healing_rate`),
//! prices (attack and step speed),
//! burden (carrying limit and weight), noise (stealth), and light (a glowing item's wearer
//! carries its light).

use super::{Effect, Magic};
use crate::combat::{CRITS, Profile, THREATS};
use crate::content::{COSTS, Content, DamageTypeId, Group, LightId, MAX_SPEED, OnHit, Power};
use crate::damage::Rider;
use crate::dice::Dice;
use crate::equipment::Equipment;
use crate::object::Stack;
use crate::skills::SkillId;
use crate::stats::Stat;

/// Effects summed: one value per effect, the light carried, and the bonuses to skills.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mods {
    values: [i32; Effect::COUNT],
    /// The brightest light among them.
    pub light: Option<LightId>,
    /// What skill affixes add to each skill's value, by skill (spec §8 "Skills").
    skills: Vec<(SkillId, i32)>,
    /// The sides of the die of damage elemental affixes add, by damage type, in the order
    /// first met (spec §8 "Damage types").
    elemental: Vec<(DamageTypeId, i32)>,
    /// What resistance affixes add, by damage type, in percent.
    resistances: Vec<(DamageTypeId, i32)>,
    /// What its hits cause, from a unique item's Inflict power (local: the weapon that hits).
    on_hit: Vec<OnHit>,
}

impl Default for Mods {
    fn default() -> Self {
        Self {
            values: [0; Effect::COUNT],
            light: None,
            skills: Vec::new(),
            elemental: Vec::new(),
            resistances: Vec::new(),
            on_hit: Vec::new(),
        }
    }
}

/// Adds `value` to `kind`'s entry in `list`, or appends one.
fn add_by_kind(list: &mut Vec<(DamageTypeId, i32)>, kind: DamageTypeId, value: i32) {
    match list.iter_mut().find(|(k, _)| *k == kind) {
        Some((_, sum)) => *sum = sum.saturating_add(value),
        None => list.push((kind, value)),
    }
}

impl Mods {
    /// The sum of `effect` (the best, for Deflection and natural armour).
    pub fn get(&self, effect: Effect) -> i32 {
        self.values[effect.index()]
    }

    /// The bonus to `stat`.
    pub fn stat(&self, stat: Stat) -> i32 {
        self.get(Effect::Stat(stat))
    }

    /// What skill affixes add to each skill's value, by skill.
    pub fn skills(&self) -> &[(SkillId, i32)] {
        &self.skills
    }

    /// What resistance affixes add, in percent by damage type (spec §8 "Damage types").
    pub fn resistances(&self) -> &[(DamageTypeId, i32)] {
        &self.resistances
    }

    /// The damage elemental affixes add, one die per damage type: the sides add up, as the
    /// plain extra die's do.
    pub fn elemental(&self) -> impl Iterator<Item = Rider> + '_ {
        (self.elemental.iter()).filter_map(|&(kind, sides)| {
            let sides = u32::try_from(sides).ok().filter(|s| *s >= 2)?;
            Some(Rider {
                kind,
                dice: Dice::of(1, sides),
            })
        })
    }

    /// Whether nothing is changed.
    pub fn is_empty(&self) -> bool {
        *self == Mods::default()
    }

    /// Adds `value` of `effect` (a light's `light`, a skill affix's `skill`, a typed affix's
    /// `kind`).
    fn add(
        &mut self,
        (effect, skill, kind): Group,
        value: i32,
        light: Option<LightId>,
        content: &Content,
    ) {
        let slot = &mut self.values[effect.index()];
        match effect {
            Effect::Elemental => {
                if let Some(kind) = kind {
                    add_by_kind(&mut self.elemental, kind, value);
                }
            }
            Effect::Resistance => {
                if let Some(kind) = kind {
                    add_by_kind(&mut self.resistances, kind, value);
                }
            }
            Effect::Deflection | Effect::Natural => *slot = (*slot).max(value),
            Effect::Light => {
                let radius = |id: LightId| content.light(id).radius;
                if let Some(id) = light
                    && self.light.is_none_or(|best| radius(id) > radius(best))
                {
                    self.light = Some(id);
                }
            }
            Effect::Skill => {
                let Some(id) = skill else { return };
                match self.skills.iter_mut().find(|(s, _)| *s == id) {
                    Some((_, sum)) => *sum = sum.saturating_add(value),
                    None => self.skills.push((id, value)),
                }
            }
            _ => *slot = slot.saturating_add(value),
        }
    }

    /// Adds the effects of `magic`'s affixes on an item of `kind` that are `local` (or not).
    fn add_magic(&mut self, magic: &Magic, local: bool, stack: &Stack, content: &Content) {
        let Some(kind) = content.object(stack.object).kind else {
            return;
        };
        for rolled in magic.rolled(content) {
            if rolled.def.effect.is_local_on(kind) == local {
                let light = (rolled.def.tier(rolled.tier)).and_then(|t| t.light);
                self.add(rolled.def.group(), rolled.value, light, content);
            }
        }
    }

    /// What `stack`'s affixes do to itself and to the attacks made with it (its local effects),
    /// with what its material does to those attacks if it is made of another than its base item
    /// (to hit and damage, [`crate::variant::weapon`]; #193).
    pub fn local(stack: &Stack, content: &Content) -> Mods {
        let mut mods = Mods::default();
        if let Some(magic) = stack.magic() {
            mods.add_magic(magic, true, stack, content);
            // A unique weapon's Inflict power rides on the attacks made with it.
            let unique = (magic.unique.as_deref()).and_then(|id| content.unique(id));
            if let Some(Power::Inflict(hit)) = unique.and_then(|u| u.power.as_ref()) {
                mods.on_hit.push(*hit);
            }
        }
        if stack.own_parts().is_some() {
            let def = content.object(stack.object);
            let (hit, damage) = crate::variant::weapon(def, &stack.parts(content), content);
            mods.values[Effect::ToHit.index()] += hit;
            mods.values[Effect::Damage.index()] += damage;
        }
        mods
    }

    /// What `stack`'s affixes do to whoever wears or wields it (its global effects alone: what
    /// [`Mods::worn`] sums over everything equipped).
    pub fn global(stack: &Stack, content: &Content) -> Mods {
        let mut mods = Mods::default();
        if let Some(magic) = stack.magic() {
            mods.add_magic(magic, false, stack, content);
        }
        mods
    }

    /// What the things in `inventory` that `equipment` holds do to whoever wears and wields
    /// them (their global effects).
    pub fn worn(inventory: &[Stack], equipment: &Equipment, content: &Content) -> Mods {
        let mut mods = Mods::default();
        for (_, item) in equipment.items() {
            if let Some(stack) = inventory.get(item)
                && let Some(magic) = stack.magic()
            {
                mods.add_magic(magic, false, stack, content);
            }
        }
        mods
    }

    /// Changes an attack by these effects: to-hit, damage (added to every roll), extra damage
    /// dice, a wider critical threat range, a larger critical multiplier, and the damage of
    /// other kinds elemental affixes add (one rider per kind, after those the attack has); and
    /// what a unique weapon's Inflict power makes its hits cause (`Profile::on_hit`).
    pub fn sharpen(&self, profile: &mut Profile) {
        profile.to_hit += self.get(Effect::ToHit);
        profile.bonus += self.get(Effect::Damage);
        let threat = i32::from(profile.threat) - self.get(Effect::Threat);
        profile.threat = clamp_u8(threat, *THREATS.start(), *THREATS.end());
        let crit = i32::from(profile.crit) + self.get(Effect::Crit);
        profile.crit = clamp_u8(crit, *CRITS.start(), *CRITS.end());
        let sides = self.get(Effect::ExtraDamage);
        if sides >= 2 {
            let sides = u32::try_from(sides).unwrap_or(2);
            profile.extra = Some(match profile.extra {
                Some(more) => Dice::of(1, sides + more.max().unsigned_abs()),
                None => Dice::of(1, sides),
            });
        }
        for rider in self.elemental() {
            profile.add_rider(rider);
        }
        profile.on_hit.extend_from_slice(&self.on_hit);
    }

    /// Every effect with a value, in [`Effect::all`] order, and the light.
    pub fn iter(&self) -> impl Iterator<Item = (Effect, i32)> + '_ {
        (Effect::all().into_iter())
            .map(|e| (e, self.get(e)))
            .filter(|&(_, v)| v != 0)
    }
}

fn clamp_u8(value: i32, lo: u8, hi: u8) -> u8 {
    value.clamp(i32::from(lo), i32::from(hi)) as u8
}

/// What an action of `cost` costs `percent` faster: that much off (at most [`MAX_SPEED`]),
/// never below the cheapest action.
pub fn faster(cost: u32, percent: i32) -> u32 {
    let off = u32::try_from(percent.clamp(0, MAX_SPEED)).unwrap_or(0);
    (cost * (100 - off) / 100).max(*COSTS.start())
}

/// `value` less by `percent` (at most 90 %), rounded down: a lighter weight, a smaller chance to
/// break, quieter footsteps.
pub fn less(value: u64, percent: i32) -> u64 {
    let off = u64::try_from(percent.clamp(0, 90)).unwrap_or(0);
    value * (100 - off) / 100
}
