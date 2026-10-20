//! Actors: the player and creatures.

use super::Talk;
use crate::action::AttackKind;
use crate::burden::Burden;
use crate::combat::{self, Attack, Category, Defence, Health, Profile, Target};
use crate::content::Effect as ItemEffect;
use crate::content::{
    ActionKind, Benefit, CombatStance, Content, CreatureId, DamageTypeId, FactionId, Grip,
    ObjectDef, ObjectId, Role, TwoWeapons, WeaponSkillId,
};
use crate::damage::{Protection, Resistances};
use crate::disease::Diseases;
use crate::door::{Key, Lock};
use crate::equipment::Equipment;
use crate::geom::{Dir, Pos};
use crate::hunger::{Hunger, Satiation};
use crate::magic::Mods;
use crate::noise::Heard;
use crate::object::{Stack, Weight, weight_of};
use crate::skills::{Effect, Skills};
use crate::stats::{PerStat, Stat, StatDamage, Stats};
use crate::status::Effects;
use crate::weapon_skills::WeaponSkills;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable actor identity. Ids are never reused within a run; actors ready at the same moment act
/// in ascending id order, after the player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ActorId(pub u32);

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    Player,
    Creature,
}

/// A fixed, looping patrol route: what a creature does while no hostile is about.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Patrol {
    pub route: Vec<Dir>,
    /// Index of the next step to take.
    pub step: usize,
}

/// What a creature remembers between its actions (spec §8 "Creature behavior"); all `None` while
/// it goes about its routine. The player has none.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mind {
    /// Where it last saw a hostile it has since lost sight of: it goes there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<Pos>,
    /// Where it heard a noise whose source it could not see: it goes to look.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise: Option<Pos>,
    /// Where it left its routine (its patrol route, or the cell it waited on) to fight or look:
    /// it walks back there once nothing is about, then carries on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post: Option<Pos>,
}

impl Mind {
    /// Whether it is going about its routine.
    pub fn is_calm(&self) -> bool {
        *self == Mind::default()
    }
}

/// What an attack draws on and trains (spec §8 "Weapon skills", "Fighting styles"): a weapon
/// skill and a fighting style, each if any ([`Actor::attack_skills`]). A kick draws on unarmed
/// fighting alone: it is made whatever the hands hold, so it is fought in no style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AttackSkills {
    pub skill: Option<WeaponSkillId>,
    pub style: Option<WeaponSkillId>,
}

impl AttackSkills {
    /// A weapon skill alone, in no style.
    pub fn only(skill: Option<WeaponSkillId>) -> Self {
        Self { skill, style: None }
    }

    /// Both, in the order they train (and their news is said): the weapon skill, then the style.
    pub fn trained(self) -> impl Iterator<Item = WeaponSkillId> {
        [self.skill, self.style].into_iter().flatten()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub id: ActorId,
    pub kind: ActorKind,
    pub creature: CreatureId,
    pub level: String,
    pub pos: Pos,
    pub patrol: Option<Patrol>,
    /// Scheduler energy (spec §8 "Speed", `world/schedule.rs`): the actor may act while this is
    /// at least 0. It gains the creature type's speed per second of game time while the actor is
    /// on the current level; an action costs 1000 per second it takes.
    pub energy: i64,
    /// Noises the actor heard since it last acted, in the order heard (spec §5 "Hearing"; see
    /// [`crate::noise`]): where each was made, its kind, and how loud it was here. A creature's
    /// list is what it may react to in its next action ([`crate::World`]'s `creature_turn`, where
    /// the AI reads it) and is cleared as it acts; the player's covers the turn that last passed
    /// and becomes the perceived sounds.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub heard: Vec<Heard>,
    /// What a creature has in mind (`world/ai.rs`).
    #[serde(default, skip_serializing_if = "Mind::is_calm")]
    pub mind: Mind,
    /// What it carries, lettered `a` on (see [`crate::object`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inventory: Vec<Stack>,
    /// What of its inventory it wears and wields, by slot ([`crate::equipment`]). Others see
    /// what a creature wears and wields; never what else it carries.
    #[serde(default, skip_serializing_if = "Equipment::is_empty")]
    pub equipment: Equipment,
    /// Its scores and potentials (spec §8 "Stats"; [`crate::stats`]): from its creature type,
    /// the player's from the scenario. Known to the rules (and dev tools) for every actor; the
    /// player is shown only their own scores. Rules read them through
    /// [`Actor::effective_stats`], which magic items raise and status effects and stat damage
    /// may lower.
    pub stats: Stats,
    /// Hit points left (spec §8 "Combat"); at 0 or below it is dead.
    pub hp: i32,
    /// What its maximum hit points add to its Toughness: the creature type's `base_hp`, the
    /// player's race and class (the scenario's `hp`).
    pub hp_base: i32,
    /// Progress toward its next hit point back, while it is hurt, in hundredths of a second of
    /// game time at the normal pace (spec §8 "Healing"): each second adds its healing rate in
    /// percent (100 normally, 200 resting, less or none while a status slows it), and every
    /// [`combat::regen_seconds`] × 100 of it is a hit point.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub regen: u32,
    /// Its trained skills, with their values and training marks (spec §8 "Skills";
    /// [`crate::skills`]): its creature type's, the player's from their race and class. Every
    /// other skill is untrained: it gives nothing. Known to the rules (and dev tools) for every
    /// actor; the player is shown only their own trained skills' values.
    #[serde(default, skip_serializing_if = "Skills::is_empty")]
    pub skills: Skills,
    /// The status effects it suffers (spec §8 "Status effects"; [`crate::status`]).
    #[serde(default, skip_serializing_if = "Effects::is_empty")]
    pub effects: Effects,
    /// The diseases it carries, incubating or showing (spec §8 "Diseases";
    /// [`crate::disease`]).
    #[serde(default, skip_serializing_if = "Diseases::is_empty")]
    pub diseases: Diseases,
    /// The points its diseases took off its scores, mending once none shows (spec §8 "Stats";
    /// [`StatDamage`]).
    #[serde(default, skip_serializing_if = "StatDamage::is_none")]
    pub damage: StatDamage,
    /// Its rank and marks in every weapon skill (spec §8 "Weapon skills";
    /// [`crate::weapon_skills`]): a creature's from its type, the player's from their race and
    /// class. Only the player trains. The player is shown their own ranks; what a rank gives
    /// stays hidden.
    #[serde(default, skip_serializing_if = "WeaponSkills::is_empty")]
    pub weapon_skills: WeaponSkills,
    /// The faction its level placed it in, if not its type's (`faction` in a level legend entry:
    /// peaceful creatures in the showcase). Read the one that counts with [`Actor::faction`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faction_override: Option<FactionId>,
    /// How fed it is (spec §8 "Hunger"): the player's, when the content has hunger; creatures
    /// do not hunger (`None`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub satiation: Option<Satiation>,
    /// Its combat stance (spec §8 "Stances"): the player chooses theirs; creatures stay neutral.
    #[serde(default, skip_serializing_if = "CombatStance::is_neutral")]
    pub stance: CombatStance,
    /// The unique character its level made it (`npc` in a level legend entry), whose dialogue it
    /// speaks rather than its type's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub npc: Option<String>,
    /// How far its talk has got (spec §8 "Dialogue"): which line comes next, which mutter, and
    /// whether the player has met it.
    #[serde(default, skip_serializing_if = "Talk::is_new")]
    pub talk: Talk,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl Actor {
    /// The faction it fights for: the one its level placed it in, else its creature type's.
    pub fn faction(&self, content: &Content) -> Option<FactionId> {
        self.faction_override
            .or(content.creature(self.creature).faction)
    }

    /// What everything it wears and wields does to it (spec §8 "Magic items"; their global
    /// effects, [`Mods::worn`]).
    pub fn mods(&self, content: &Content) -> Mods {
        Mods::worn(&self.inventory, &self.equipment, content)
    }

    /// How hungry it is (spec §8 "Hunger"); `None` for an actor that does not hunger.
    pub fn hunger(&self, content: &Content) -> Option<Hunger> {
        let def = content.hunger()?;
        self.satiation.map(|s| def.state(s.nutrition))
    }

    /// Its stats as the rules read them: its own scores, each raised by what it wears and
    /// wields (spec §8 "Magic items"), then less what its status effects take from them (spec
    /// §8 "Status effects": strong poison lowers Strength and Dexterity), what hunger takes
    /// while it is Weak or worse (spec §8 "Hunger": `[weak]` in `data/hunger.toml`, Strength and
    /// Dexterity), and its stat damage (spec §8 "Diseases"). The penalties add up; no score goes
    /// below 1. Every rule that reads a score reads these.
    pub fn effective_stats(&self, content: &Content) -> Stats {
        self.stats_with(content, &self.mods(content))
    }

    /// Its own scores raised by what it wears and wields, before what its status effects,
    /// hunger, and stat damage take (the character screen marks the scores those lower).
    pub fn raised_stats(&self, content: &Content) -> Stats {
        self.raised_with(&self.mods(content))
    }

    fn raised_with(&self, mods: &Mods) -> Stats {
        self.stats.raised(|stat| mods.stat(stat))
    }

    fn stats_with(&self, content: &Content, mods: &Mods) -> Stats {
        let raised = self.raised_with(mods);
        match self.penalty(content) {
            Some(by) => raised.adjusted_by(&by),
            None => raised,
        }
    }

    /// What its status effects, hunger (Weak or worse), and stat damage take off its scores, if
    /// anything.
    fn penalty(&self, content: &Content) -> Option<PerStat<i32>> {
        let mut by = self.effects.penalty(content);
        if let (Some(def), Some(hunger)) = (content.hunger(), self.hunger(content))
            && hunger.is_weak()
        {
            let by = by.get_or_insert_with(|| PerStat::splat(0));
            for stat in Stat::ALL {
                by[stat] += def.weak[stat];
            }
        }
        if self.damage.is_damaged() {
            let sum = by.get_or_insert_with(|| PerStat::splat(0));
            for (stat, points) in self.damage.by().iter() {
                sum[stat] += points;
            }
        }
        by
    }

    /// What its skills give of `effect` (spec §8 "Skills"), with what the skill affixes of what
    /// it wears and wields add to their values (spec §8 "Magic items"). Every rule that reads a
    /// skill's benefit reads this.
    pub fn skill_effect(&self, content: &Content, effect: Effect) -> i32 {
        if self.equipment.is_empty() {
            return self.skills.effect(content, effect);
        }
        (self.skills).effect_with(content, effect, self.mods(content).skills())
    }

    /// Its maximum hit points: its (effective) Toughness plus [`Actor::hp_base`] plus what it
    /// wears and wields adds, at least 1. Toughness damage lowers it, as in D&D 3.5 (spec §8
    /// "Diseases").
    pub fn max_hp(&self, content: &Content) -> i32 {
        let mods = self.mods(content);
        let toughness = self.stats_with(content, &mods).score(Stat::Toughness);
        combat::max_hp(toughness, self.hp_base + mods.get(ItemEffect::MaxHp))
    }

    /// Its maximum hit points as they will be once its penalties pass: its Toughness raised by
    /// what it wears and wields, not lowered by its statuses, hunger, or stat damage. Taking off
    /// something that raised them leaves it no more hit points than this (a penalty passes; the
    /// hit points it keeps under one stay, spec §8 "Status effects").
    pub fn lasting_max_hp(&self, content: &Content) -> i32 {
        let mods = self.mods(content);
        let toughness = self.raised_with(&mods).score(Stat::Toughness);
        combat::max_hp(toughness, self.hp_base + mods.get(ItemEffect::MaxHp))
    }

    /// How hurt it looks, in words.
    pub fn health(&self, content: &Content) -> Health {
        Health::of(self.hp, self.max_hp(content))
    }

    /// Its Defence (spec §8 "Combat"): 10 plus its Dexterity modifier (capped by the worn
    /// armour that allows the least and by its `burden`, the burden it suffers, spec §8
    /// "Burden": the lowest cap wins), the best armour, shield, and parry of what it wears and
    /// wields (each with what its own affixes add), its creature type's natural armour, the Dodge
    /// its skills give (tumble; spec §8 "Skills"), and the Dodge, Deflection, and natural armour
    /// its magic items give (spec §8 "Magic items"). Weapon skills add (spec §8 "Weapon skills"):
    /// the wielded weapon's skill to its parry, shield blocking to a held shield's; and so does
    /// the fighting style its hands are in (spec §8 "Fighting styles": a duelist's parry, sword
    /// and board's shield).
    pub fn defence(&self, content: &Content, burden: Burden) -> Defence {
        let mods = self.mods(content);
        let weapon = self.weapon_item(content);
        let style = self.benefit(content, self.style(content, AttackKind::Melee, None));
        let parry_skill = self.benefit(content, self.melee_skill(content)).parry + style.parry;
        let block = (content.weapon_skills().of_role(Role::ShieldBlock))
            .map_or(0, |id| self.benefit(content, Some(id)).shield)
            + style.shield;
        let equipped: Vec<(usize, &ObjectDef, Mods, bool)> = (self.equipment.items().into_iter())
            .filter_map(|(slot, item)| {
                Some((item, content.slot(slot).hand, self.inventory.get(item)?))
            })
            .map(|(item, hand, s)| {
                (
                    item,
                    content.object(s.object),
                    Mods::local(s, content),
                    hand,
                )
            })
            .collect();
        let dexterity = self.stats_with(content, &mods).modifier(Stat::Dexterity);
        let worn = (equipped.iter())
            .filter_map(|(_, o, m, _)| Some(o.max_dex? + m.get(ItemEffect::MaxDex)));
        let cap = worn.chain(burden.max_dex()).min();
        let dexterity = cap.map_or(dexterity, |cap| dexterity.min(cap));
        // A weapon readied to throw does not parry: only what is held in a hand.
        let worn = equipped.iter().flat_map(|(item, o, m, hand)| {
            let wielded = Some(*item) == weapon;
            let shield = o.shield + m.get(ItemEffect::Shield);
            let parry = if *hand {
                o.parry + m.get(ItemEffect::Parry)
            } else {
                0
            };
            [
                (Category::Armour, o.armour + m.get(ItemEffect::Armour)),
                (
                    Category::Shield,
                    shield + if o.shield > 0 { block } else { 0 },
                ),
                (
                    Category::Parry,
                    parry + if wielded { parry_skill } else { 0 },
                ),
            ]
        });
        let natural = (content.creature(self.creature).natural.as_ref())
            .map(|hide| (Category::Natural, hide.defence));
        let dodge = Some(self.skill_effect(content, Effect::Dodge))
            .filter(|&d| d != 0)
            .map(|d| (Category::Dodge, d));
        let magic = [
            (Category::Natural, mods.get(ItemEffect::Natural)),
            (Category::Dodge, mods.get(ItemEffect::Dodge)),
            (Category::Deflection, mods.get(ItemEffect::Deflection)),
        ];
        let stance = content.stance_def(self.stance).defence();
        Defence::from_bonuses(
            std::iter::once((Category::Dexterity, dexterity))
                .chain(worn)
                .chain(natural)
                .chain(dodge)
                .chain(magic.into_iter().filter(|&(_, b)| b != 0))
                .chain(stance),
        )
    }

    /// Its Protection (spec §8 "Combat"): the damage it soaks off every physical hit, the sum
    /// of what it wears (each piece's `protection` with what its own affixes add; a shield has
    /// none) and its creature type's natural armour; against each physical kind, more or less
    /// by what the pieces say (spec §8 "Damage types": chain mail +1 against slashing, −1
    /// against piercing), and nothing against an element.
    pub fn protection(&self, content: &Content) -> Protection {
        let worn: Vec<&Stack> = (self.equipment.items().into_iter())
            .filter_map(|(_, item)| self.inventory.get(item))
            .collect();
        let base: i32 = worn.iter().map(|s| s.protection(content)).sum();
        let natural =
            (content.creature(self.creature).natural.as_ref()).map_or(0, |h| h.protection);
        let adjustments = (worn.iter()).flat_map(|s| s.protects(content).iter().copied());
        if content.damage_types().is_empty() {
            return Protection::plain(base + natural);
        }
        Protection::new(content, base + natural, adjustments)
    }

    /// What it resists (spec §8 "Damage types"), in percent per kind: what its creature type
    /// is, plus what it wears and wields (each piece's `resist`, and the resistance affixes of
    /// all of it), capped ([`Resistances::sum`]).
    pub fn resistances(&self, content: &Content) -> Resistances {
        let count = content.damage_types().len();
        let innate = content.creature(self.creature).resist.iter().copied();
        let worn = (self.equipment.items().into_iter())
            .filter_map(|(_, item)| self.inventory.get(item))
            .flat_map(|s| s.resists(content).iter().copied());
        let magic = self.mods(content);
        let gear: Vec<(DamageTypeId, i32)> =
            worn.chain(magic.resistances().iter().copied()).collect();
        Resistances::sum(count, innate, gear)
    }

    /// The side of it an attack strikes (spec §8 "Combat"): its Defence (its `burden` capping
    /// its Dexterity), its Protection, and its resistances.
    pub fn target(&self, content: &Content, burden: Burden) -> Target {
        Target {
            defence: self.defence(content, burden),
            protection: self.protection(content),
            resistances: self.resistances(content),
        }
    }

    /// What a blow that its Protection soaked to nothing glanced off, in messages: the piece it
    /// wears with the most Protection (the first in slot order of the best), else its natural
    /// armour, if either has any.
    pub(crate) fn soaked_by<'c>(&self, content: &'c Content) -> Option<&'c str> {
        let mut best: Option<(i32, &'c str)> = None;
        for (_, item) in self.equipment.items() {
            let Some(stack) = self.inventory.get(item) else {
                continue;
            };
            let pv = stack.protection(content);
            if pv > 0 && best.is_none_or(|(b, _)| pv > b) {
                best = Some((pv, content.object(stack.object).name.as_str()));
            }
        }
        let hide = (content.creature(self.creature).natural.as_ref()).filter(|h| h.protection > 0);
        match (best, hide) {
            (Some((pv, _)), Some(hide)) if hide.protection > pv => Some(hide.name.as_str()),
            (Some((_, name)), _) => Some(name),
            (None, hide) => hide.map(|h| h.name.as_str()),
        }
    }

    /// The faintest loudness it hears (spec §5 "Hearing"): its creature type's `hearing`, lower
    /// by what its skills give (listen; spec §8 "Skills"), never below 1; `None`: deaf.
    pub fn hearing(&self, content: &Content) -> Option<u16> {
        let threshold = content.creature(self.creature).hearing?;
        let keener = self.skill_effect(content, Effect::Hearing);
        Some((i32::from(threshold) - keener).max(1) as u16)
    }

    /// Where in its inventory the weapon it wields is, if any: the first hand that holds one.
    pub fn weapon_item(&self, content: &Content) -> Option<usize> {
        content.hand_slots().find_map(|slot| {
            let item = self.equipment.get(slot)?;
            let stack = self.inventory.get(item)?;
            content.object(stack.object).attack.as_ref().map(|_| item)
        })
    }

    /// The weapon it wields, if any: the first hand that holds one, and what it is.
    pub fn weapon<'c>(&self, content: &'c Content) -> Option<(ObjectId, &'c Attack)> {
        let stack = self.inventory.get(self.weapon_item(content)?)?;
        let def = content.object(stack.object);
        def.attack.as_ref().map(|a| (stack.object, a))
    }

    /// Where in its inventory the shield it can bash with lies: the first hand that holds one.
    pub fn shield_item(&self, content: &Content) -> Option<usize> {
        content.hand_slots().find_map(|slot| {
            let item = self.equipment.get(slot)?;
            let stack = self.inventory.get(item)?;
            content.object(stack.object).bash.as_ref().map(|_| item)
        })
    }

    /// The shield it holds that it can bash with, and what a bash with it does (spec §8
    /// "Weapon skills"): the first hand that holds one.
    pub fn shield<'c>(&self, content: &'c Content) -> Option<(ObjectId, &'c Attack)> {
        let stack = self.inventory.get(self.shield_item(content)?)?;
        let def = content.object(stack.object);
        def.bash.as_ref().map(|a| (stack.object, a))
    }

    /// What its rank in weapon skill `skill` gives (nothing for no skill).
    pub fn benefit(&self, content: &Content, skill: Option<WeaponSkillId>) -> Benefit {
        skill.map_or_else(Benefit::default, |id| {
            content
                .weapon_skills()
                .benefit(id, self.weapon_skills.rank(id))
        })
    }

    /// The weapon skill of its melee blows: the wielded weapon's, else unarmed (bare hands and
    /// natural attacks alike).
    pub fn melee_skill(&self, content: &Content) -> Option<WeaponSkillId> {
        match self.weapon(content) {
            Some((weapon, _)) => content.object(weapon).skill,
            None => content.weapon_skills().of_role(Role::Unarmed),
        }
    }

    /// The weapon skill an attack of `kind` made `with` (a missile, a shield for a bash, the
    /// off hand's weapon for its blow, or nothing for a blow) draws on and trains: a blow's
    /// ([`Actor::melee_skill`]), a bash's shield bashing, an off-hand blow's its weapon's, a
    /// shot's the launcher's, a throw's thrown; a spell none.
    pub fn attack_skill(
        &self,
        content: &Content,
        kind: AttackKind,
        with: Option<ObjectId>,
    ) -> Option<WeaponSkillId> {
        let skills = content.weapon_skills();
        match (kind, with) {
            (AttackKind::Melee, None) => self.melee_skill(content),
            (AttackKind::Melee, Some(with)) => match content.object(with) {
                shield if shield.bash.is_some() => skills.of_role(Role::ShieldBash),
                weapon => weapon.skill,
            },
            (AttackKind::Ranged, _) => self
                .launcher(content)
                .and_then(|(launcher, _)| content.object(launcher).skill),
            (AttackKind::Thrown, _) => skills.of_role(Role::Thrown),
            (AttackKind::Spell, _) => None,
        }
    }

    /// What is in its hands: each thing once (a two-handed weapon is in both), in hand order,
    /// with where in its inventory it is.
    fn in_hands<'c>(&self, content: &'c Content) -> Vec<(usize, &'c ObjectDef)> {
        let mut held: Vec<usize> = Vec::new();
        for slot in content.hand_slots() {
            let Some(item) = self.equipment.get(slot) else {
                continue;
            };
            if self.inventory.get(item).is_some() && !held.contains(&item) {
                held.push(item);
            }
        }
        (held.into_iter())
            .map(|item| (item, content.object(self.inventory[item].object)))
            .collect()
    }

    /// The grip its hands are in for a blow (spec §8 "Fighting styles"), and the weapon skills
    /// of the weapons in them: nothing (unarmed), one one-handed weapon (the other hand empty),
    /// a weapon in each hand, a two-handed weapon, or a one-handed weapon and a shield; `None`
    /// for anything else (a shield alone).
    pub fn grip(&self, content: &Content) -> Option<(Grip, Vec<WeaponSkillId>)> {
        let held = self.in_hands(content);
        let weapon = |o: &ObjectDef| o.attack.is_some();
        let skills = held.iter().filter_map(|(_, o)| o.skill).collect();
        let grip = match held.as_slice() {
            [] => Grip::Unarmed,
            [(_, w)] if weapon(w) && w.two_handed => Grip::TwoHanded,
            [(_, w)] if weapon(w) => Grip::OneWeapon,
            [(_, a), (_, b)] if weapon(a) && weapon(b) => Grip::TwoWeapons,
            [(_, a), (_, b)] if (weapon(a) && b.shield > 0) || (a.shield > 0 && weapon(b)) => {
                Grip::WeaponAndShield
            }
            _ => return None,
        };
        Some((grip, skills))
    }

    /// The fighting style an attack of `kind` made `with` (as for [`Actor::attack_skill`]) is
    /// fought in, whatever its rank (spec §8 "Fighting styles"): a blow's, a bash's, or an
    /// off-hand blow's by the grip of its hands; a shot's by its launcher; a throw's by what is
    /// thrown. A spell has none.
    pub fn style(
        &self,
        content: &Content,
        kind: AttackKind,
        with: Option<ObjectId>,
    ) -> Option<WeaponSkillId> {
        let table = content.weapon_skills();
        let skill_of = |object: ObjectId| content.object(object).skill;
        match kind {
            AttackKind::Melee => {
                let (grip, weapons) = self.grip(content)?;
                table.style_for(grip, &weapons)
            }
            AttackKind::Ranged => {
                let (launcher, _) = self.launcher(content)?;
                let weapons: Vec<WeaponSkillId> = skill_of(launcher).into_iter().collect();
                table.style_for(Grip::Launcher, &weapons)
            }
            AttackKind::Thrown => {
                let weapons: Vec<WeaponSkillId> = with.and_then(skill_of).into_iter().collect();
                table.style_for(Grip::Thrown, &weapons)
            }
            AttackKind::Spell => None,
        }
    }

    /// The fighting styles it fights in now, in id order: its hands' (blows and Defence), its
    /// readied missile weapon's, and throwing's while it has something ready to throw.
    pub fn styles_in_use(&self, content: &Content) -> Vec<WeaponSkillId> {
        let thrown = self
            .ready_to_throw(content)
            .map(|i| self.inventory[i].object);
        let mut styles: Vec<WeaponSkillId> = [
            self.style(content, AttackKind::Melee, None),
            self.launcher(content)
                .and_then(|_| self.style(content, AttackKind::Ranged, None)),
            thrown.and_then(|t| self.style(content, AttackKind::Thrown, Some(t))),
        ]
        .into_iter()
        .flatten()
        .collect();
        styles.sort_unstable();
        styles.dedup();
        styles
    }

    /// The weapon skill and the fighting style an attack of `kind` made `with` (as for
    /// [`Actor::attack_skill`]) draws on and trains.
    pub fn attack_skills(
        &self,
        content: &Content,
        kind: AttackKind,
        with: Option<ObjectId>,
    ) -> AttackSkills {
        AttackSkills {
            skill: self.attack_skill(content, kind, with),
            style: self.style(content, kind, with),
        }
    }

    /// What its ranks in `skills` give: the weapon skill's benefit and the fighting style's,
    /// together.
    pub fn benefit_of(&self, content: &Content, skills: AttackSkills) -> Benefit {
        (self.benefit(content, skills.skill)).plus(self.benefit(content, skills.style))
    }

    /// What its ranks give an attack of `kind` made `with` (as for [`Actor::attack_skill`]): its
    /// weapon skill's benefit and its fighting style's, together.
    pub fn attack_benefit(
        &self,
        content: &Content,
        kind: AttackKind,
        with: Option<ObjectId>,
    ) -> Benefit {
        self.benefit_of(content, self.attack_skills(content, kind, with))
    }

    /// The weapon in its off hand while it holds a weapon in each of two hands (spec §8
    /// "Fighting styles"): the second, in hand order, where in its inventory it is, and what it
    /// is.
    pub fn off_hand<'c>(&self, content: &'c Content) -> Option<(usize, &'c Attack)> {
        match self.in_hands(content).as_slice() {
            [(_, main), (off, def)] if main.attack.is_some() => Some((*off, def.attack.as_ref()?)),
            _ => None,
        }
    }

    /// Where in its inventory the missile weapon it has ready is: the object in a slot of
    /// action `ready` that launches something.
    pub fn launcher_item(&self, content: &Content) -> Option<usize> {
        content.slot_ids().find_map(|(slot, def)| {
            if def.action != ActionKind::Ready {
                return None;
            }
            let item = self.equipment.get(slot)?;
            let object = content.object(self.inventory.get(item)?.object);
            object.launches.as_ref()?;
            object.attack.as_ref().map(|_| item)
        })
    }

    /// The missile weapon it has ready and what it is.
    pub fn launcher<'c>(&self, content: &'c Content) -> Option<(ObjectId, &'c Attack)> {
        let stack = self.inventory.get(self.launcher_item(content)?)?;
        let object = content.object(stack.object);
        object.attack.as_ref().map(|a| (stack.object, a))
    }

    /// How far its missile weapon shoots, in cells: its `range`, and what its affixes add.
    pub fn launcher_range(&self, content: &Content) -> Option<i32> {
        let stack = self.inventory.get(self.launcher_item(content)?)?;
        let range = content.object(stack.object).range?;
        Some(range + Mods::local(stack, content).get(ItemEffect::Range))
    }

    /// How far it throws inventory stack `item`, if it can be thrown: a throwing weapon's
    /// `range`, any other weapon's [`crate::combat::IMPROVISED_RANGE`], and what its affixes add.
    pub fn throw_range(&self, item: usize, content: &Content) -> Option<i32> {
        let stack = self.inventory.get(item)?;
        let range = content.object(stack.object).throw_range()?;
        Some(range + Mods::local(stack, content).get(ItemEffect::Range))
    }

    /// The ammunition it has ready for its launcher: the first ready stack the launcher fires.
    pub fn ammunition(&self, content: &Content) -> Option<ObjectId> {
        self.ammunition_item(content)
            .map(|item| self.inventory[item].object)
    }

    /// Where in its inventory the ammunition it has ready for its launcher is (see
    /// [`Actor::ammunition`]).
    pub fn ammunition_item(&self, content: &Content) -> Option<usize> {
        let (launcher, _) = self.launcher(content)?;
        content.slot_ids().find_map(|(slot, def)| {
            if def.action != ActionKind::Ready {
                return None;
            }
            let item = self.equipment.get(slot)?;
            let stack = self.inventory.get(item)?;
            content.fires(launcher, stack.object).then_some(item)
        })
    }

    /// What it has ready to throw, if anything: the first stack in a `ready` slot (the missile
    /// weapon slot) that can be thrown (a stack of rocks or daggers, or a sword), and where in
    /// its inventory it is.
    pub fn ready_to_throw(&self, content: &Content) -> Option<usize> {
        content.slot_ids().find_map(|(slot, def)| {
            if def.action != ActionKind::Ready {
                return None;
            }
            let item = self.equipment.get(slot)?;
            let stack = self.inventory.get(item)?;
            content.object(stack.object).can_throw().then_some(item)
        })
    }

    /// The digging tool it has ready (spec §8 "Digging"): the first thing it wears or wields, in
    /// slot order, that digs (a pick-axe in the Tool slot), and its power.
    pub fn dig_tool(&self, content: &Content) -> Option<(ObjectId, u16)> {
        self.equipment.items().into_iter().find_map(|(_, item)| {
            let stack = self.inventory.get(item)?;
            (content.object(stack.object).dig).map(|power| (stack.object, power))
        })
    }

    /// The key it carries that opens `lock` (spec §8 "Doors"): the first one made for it, else
    /// the first skeleton key; `None` if it carries none that fits.
    pub fn key_for(&self, content: &Content, lock: &Lock) -> Option<ObjectId> {
        let fitting = (self.inventory.iter())
            .map(|s| s.object)
            .filter_map(|o| Some((o, content.object(o).key.as_ref()?)))
            .filter(|(_, key)| key.opens(lock));
        let mut skeleton = None;
        for (object, key) in fitting {
            match key {
                Key::Lock(_) => return Some(object),
                Key::Skeleton => {
                    skeleton.get_or_insert(object);
                }
            }
        }
        skeleton
    }

    /// What its melee attacks do: with the weapon it wields, else its natural attack, else its
    /// bare hands; Strength to hit and to damage, and what its weapon skill and fighting style
    /// give; then the weapon's own affixes and those of everything it wears and wields (spec §8
    /// "Magic items"); and its combat stance. With a weapon in each hand this is the main hand's
    /// blow, at the main hand's penalty, less what the style eases ([`Actor::off_hand_blow`] is
    /// the other).
    pub fn melee(&self, content: &Content) -> Profile {
        let mods = self.mods(content);
        let natural = content.creature(self.creature).attack.as_ref();
        let weapon = self.weapon_item(content).map(|i| &self.inventory[i]);
        let unarmed = || Attack::unarmed(content.unarmed_damage());
        let (attack, local, penetration, added) = match (weapon, natural) {
            (Some(stack), _) => match &content.object(stack.object).attack {
                Some(attack) => {
                    let def = content.object(stack.object);
                    (
                        *attack,
                        Mods::local(stack, content),
                        def.penetration,
                        def.added,
                    )
                }
                None => (unarmed(), Mods::default(), 0, None),
            },
            (None, Some(natural)) => (natural.attack, Mods::default(), 0, None),
            (None, None) => (unarmed(), Mods::default(), 0, None),
        };
        let benefit = self.attack_benefit(content, AttackKind::Melee, None);
        let mut profile = Profile::melee(&attack, &self.stats_with(content, &mods)).with(&benefit);
        profile.penetration = penetration;
        if self.off_hand(content).is_some() {
            let main = content.weapon_skills().two_weapons().main;
            profile.to_hit += TwoWeapons::eased(main, benefit.ease_main);
        }
        // The weapon's own added damage first, then what its affixes add, then the worn ones'.
        profile.riders.extend(added);
        local.sharpen(&mut profile);
        mods.sharpen(&mut profile);
        self.stanced(content, profile)
    }

    /// `profile` with the to-hit and damage of its combat stance.
    pub fn stanced(&self, content: &Content, profile: Profile) -> Profile {
        content.stance_def(self.stance).apply(profile)
    }

    /// The off hand's blow while it holds a weapon in each hand (spec §8 "Fighting styles"):
    /// that weapon, Strength to hit and to damage, what its own weapon skill and the fighting
    /// style give, at the off hand's penalty less what the style eases; then that weapon's own
    /// affixes and those of everything it wears and wields (spec §8 "Magic items"); and its
    /// combat stance. `None` without one.
    pub fn off_hand_blow(&self, content: &Content) -> Option<(ObjectId, Profile)> {
        let (item, attack) = self.off_hand(content)?;
        let stack = &self.inventory[item];
        let mods = self.mods(content);
        let benefit = self.attack_benefit(content, AttackKind::Melee, Some(stack.object));
        let mut profile = Profile::melee(attack, &self.stats_with(content, &mods)).with(&benefit);
        profile.penetration = content.object(stack.object).penetration;
        let penalty = content.weapon_skills().two_weapons().off;
        profile.to_hit += TwoWeapons::eased(penalty, benefit.ease_off);
        profile.riders.extend(content.object(stack.object).added);
        Mods::local(stack, content).sharpen(&mut profile);
        mods.sharpen(&mut profile);
        Some((stack.object, self.stanced(content, profile)))
    }

    /// What its shots do, with the missile weapon it has ready (`None` without one): Dexterity to
    /// hit, the launcher's own damage, and less of both for each point of Strength below what a
    /// bow needs; what its skill with the launcher and its fighting style give; then the affixes
    /// of the launcher, of its ammunition, and of everything it wears and wields; and its combat
    /// stance.
    pub fn ranged(&self, content: &Content) -> Option<Profile> {
        let mods = self.mods(content);
        let launcher = &self.inventory[self.launcher_item(content)?];
        let def = content.object(launcher.object);
        let mut attack = def.attack?;
        let local = Mods::local(launcher, content);
        let draw = local.get(ItemEffect::Draw);
        attack.strength =
            (attack.strength).map(|s| u8::try_from((i32::from(s) - draw).max(1)).unwrap_or(1));
        let benefit = self.attack_benefit(content, AttackKind::Ranged, None);
        let mut profile = Profile::ranged(&attack, &self.stats_with(content, &mods)).with(&benefit);
        profile.penetration = def.penetration;
        // Added damage: the launcher's own, its affixes', the ammunition's own, its affixes'.
        profile.riders.extend(def.added);
        local.sharpen(&mut profile);
        if let Some(ammo) = self.ammunition_item(content) {
            let ammo = &self.inventory[ammo];
            let ammo_def = content.object(ammo.object);
            profile.penetration += ammo_def.penetration;
            profile.riders.extend(ammo_def.added);
            Mods::local(ammo, content).sharpen(&mut profile);
        }
        mods.sharpen(&mut profile);
        Some(self.stanced(content, profile))
    }

    /// What a throw of inventory stack `item` does (`None` if it cannot be thrown): Dexterity
    /// to hit, Strength to damage (a weapon not made for throwing thrown whole, at
    /// [`Profile::improvised`]), what its skill at throwing and its fighting style give, its own
    /// affixes, those of everything it wears and wields, and its combat stance.
    pub fn thrown(&self, item: usize, content: &Content) -> Option<Profile> {
        let stack = self.inventory.get(item)?;
        let def = content.object(stack.object);
        let attack = def.attack.filter(|_| def.can_throw())?;
        let mods = self.mods(content);
        let benefit = self.attack_benefit(content, AttackKind::Thrown, Some(stack.object));
        let mut profile = Profile::thrown(&attack, &self.stats_with(content, &mods));
        if def.is_improvised() {
            profile = profile.improvised();
        }
        let mut profile = profile.with(&benefit);
        profile.penetration = def.penetration;
        profile.riders.extend(def.added);
        Mods::local(stack, content).sharpen(&mut profile);
        mods.sharpen(&mut profile);
        Some(self.stanced(content, profile))
    }

    /// What a bash with the shield it holds does (`None` without one it can bash with): the
    /// shield's `bash` dice, Strength to hit and to damage as for any blow, what its skill at
    /// shield bashing and its fighting style give, the affixes of everything it wears and
    /// wields, and its combat stance.
    pub fn bash(&self, content: &Content) -> Option<(ObjectId, Profile)> {
        let (shield, attack) = self.shield(content)?;
        let benefit = self.attack_benefit(content, AttackKind::Melee, Some(shield));
        let mods = self.mods(content);
        let mut profile = Profile::melee(attack, &self.stats_with(content, &mods)).with(&benefit);
        mods.sharpen(&mut profile);
        Some((shield, self.stanced(content, profile)))
    }

    /// What its kicks at a creature do (spec §8 "Doors"): an unarmed blow whatever it wields
    /// (bare hands' damage, Strength to hit and to damage), with what its unarmed fighting gives,
    /// the affixes of everything it wears and wields (spec §8 "Magic items"), and its combat
    /// stance.
    pub fn kick(&self, content: &Content) -> Profile {
        let skill = content.weapon_skills().of_role(Role::Unarmed);
        let mods = self.mods(content);
        let fists = Attack::unarmed(content.unarmed_damage());
        let mut profile = Profile::melee(&fists, &self.stats_with(content, &mods))
            .with(&self.benefit(content, skill));
        mods.sharpen(&mut profile);
        self.stanced(content, profile)
    }
}

impl Actor {
    /// Its creature's base weight plus everything it carries, in grams. Known to the rules (and
    /// dev tools) for every actor; the player is shown only their own.
    pub fn weight(&self, content: &Content) -> Weight {
        Weight {
            base: u64::from(content.creature(self.creature).weight),
            carried: weight_of(&self.inventory, content),
        }
    }
}
