//! Actors: the player and creatures.

use crate::action::AttackKind;
use crate::burden::Burden;
use crate::combat::{self, Attack, Category, Defence, Health, Profile};
use crate::content::{
    ActionKind, Benefit, CombatStance, Content, CreatureId, FactionId, ObjectDef, ObjectId, Role,
    WeaponSkillId,
};
use crate::equipment::Equipment;
use crate::geom::{Dir, Pos};
use crate::hunger::{Hunger, Satiation};
use crate::noise::Heard;
use crate::object::{Stack, Weight, weight_of};
use crate::skills::{Effect, Skills};
use crate::stats::{PerStat, Stat, Stats};
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
    /// [`Actor::effective_stats`], which status effects may lower.
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

    /// How hungry it is (spec §8 "Hunger"); `None` for an actor that does not hunger.
    pub fn hunger(&self, content: &Content) -> Option<Hunger> {
        let def = content.hunger()?;
        self.satiation.map(|s| def.state(s.nutrition))
    }

    /// Its stats as the rules read them: its scores less what its status effects take from
    /// them (spec §8 "Status effects": strong poison lowers Strength and Dexterity) and what
    /// hunger takes while it is Weak or worse (spec §8 "Hunger": `[weak]` in `data/hunger.toml`,
    /// Strength and Dexterity). The penalties add up.
    pub fn effective_stats(&self, content: &Content) -> Stats {
        let mut by = self.effects.penalty(content);
        if let (Some(def), Some(hunger)) = (content.hunger(), self.hunger(content))
            && hunger.is_weak()
        {
            let by = by.get_or_insert_with(|| PerStat::splat(0));
            for stat in Stat::ALL {
                by[stat] += def.weak[stat];
            }
        }
        match by {
            Some(by) => self.stats.adjusted_by(&by),
            None => self.stats,
        }
    }

    /// Its maximum hit points: its (effective) Toughness plus [`Actor::hp_base`], at least 1.
    pub fn max_hp(&self, content: &Content) -> i32 {
        let toughness = self.effective_stats(content).score(Stat::Toughness);
        combat::max_hp(toughness, self.hp_base)
    }

    /// How hurt it looks, in words.
    pub fn health(&self, content: &Content) -> Health {
        Health::of(self.hp, self.max_hp(content))
    }

    /// Its Defence (spec §8 "Combat"): 10 plus its Dexterity modifier (capped by the worn
    /// armour that allows the least and by its `burden`, the burden it suffers, spec §8
    /// "Burden": the lowest cap wins), the best armour, shield, and parry of what it wears and
    /// wields, its creature type's natural armour, and the Dodge its skills give (tumble; spec
    /// §8 "Skills"). Weapon skills add (spec §8 "Weapon skills"): the wielded weapon's skill to
    /// its parry, shield blocking to a held shield's.
    pub fn defence(&self, content: &Content, burden: Burden) -> Defence {
        let weapon = self.weapon_item(content);
        let parry = self.benefit(content, self.melee_skill(content)).parry;
        let block = (content.weapon_skills().of_role(Role::ShieldBlock))
            .map_or(0, |id| self.benefit(content, Some(id)).shield);
        let equipped: Vec<(usize, &ObjectDef, bool)> = (self.equipment.items().into_iter())
            .filter_map(|(slot, item)| {
                Some((item, content.slot(slot).hand, self.inventory.get(item)?))
            })
            .map(|(item, hand, s)| (item, content.object(s.object), hand))
            .collect();
        let dexterity = self.effective_stats(content).modifier(Stat::Dexterity);
        let worn = equipped.iter().filter_map(|(_, o, _)| o.max_dex);
        let cap = worn.chain(burden.max_dex()).min();
        let dexterity = cap.map_or(dexterity, |cap| dexterity.min(cap));
        // A weapon readied to throw does not parry: only what is held in a hand.
        let worn = equipped.iter().flat_map(|&(item, o, hand)| {
            let wielded = Some(item) == weapon;
            [
                (Category::Armour, o.armour),
                (
                    Category::Shield,
                    o.shield + if o.shield > 0 { block } else { 0 },
                ),
                (
                    Category::Parry,
                    (if hand { o.parry } else { 0 }) + if wielded { parry } else { 0 },
                ),
            ]
        });
        let natural = (content.creature(self.creature).natural.as_ref())
            .map(|hide| (Category::Natural, hide.defence));
        let dodge = Some(self.skills.effect(content, Effect::Dodge))
            .filter(|&d| d != 0)
            .map(|d| (Category::Dodge, d));
        let stance = content.stance_def(self.stance).defence();
        Defence::from_bonuses(
            std::iter::once((Category::Dexterity, dexterity))
                .chain(worn)
                .chain(natural)
                .chain(dodge)
                .chain(stance),
        )
    }

    /// The faintest loudness it hears (spec §5 "Hearing"): its creature type's `hearing`, lower
    /// by what its skills give (listen; spec §8 "Skills"), never below 1; `None`: deaf.
    pub fn hearing(&self, content: &Content) -> Option<u16> {
        let threshold = content.creature(self.creature).hearing?;
        let keener = self.skills.effect(content, Effect::Hearing);
        Some((i32::from(threshold) - keener).max(1) as u16)
    }

    /// The weapon it wields, if any: the first hand that holds one, and what it is.
    pub fn weapon<'c>(&self, content: &'c Content) -> Option<(ObjectId, &'c Attack)> {
        let item = self.weapon_item(content)?;
        let object = self.inventory[item].object;
        Some((object, content.object(object).attack.as_ref()?))
    }

    /// Where in its inventory the weapon it wields is (see [`Actor::weapon`]).
    fn weapon_item(&self, content: &Content) -> Option<usize> {
        content.hand_slots().find_map(|slot| {
            let item = self.equipment.get(slot)?;
            let stack = self.inventory.get(item)?;
            content.object(stack.object).attack.as_ref().map(|_| item)
        })
    }

    /// The shield it holds that it can bash with, and what a bash with it does (spec §8
    /// "Weapon skills"): the first hand that holds one.
    pub fn shield<'c>(&self, content: &'c Content) -> Option<(ObjectId, &'c Attack)> {
        content.hand_slots().find_map(|slot| {
            let stack = self.inventory.get(self.equipment.get(slot)?)?;
            let def = content.object(stack.object);
            def.bash.as_ref().map(|a| (stack.object, a))
        })
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

    /// The weapon skill an attack of `kind` made `with` (a missile, a shield for a bash, or
    /// nothing for a blow) draws on and trains: a blow's ([`Actor::melee_skill`]), a bash's
    /// shield bashing, a shot's the launcher's, a throw's thrown; a spell none.
    pub fn attack_skill(
        &self,
        content: &Content,
        kind: AttackKind,
        with: Option<ObjectId>,
    ) -> Option<WeaponSkillId> {
        let skills = content.weapon_skills();
        match (kind, with) {
            (AttackKind::Melee, None) => self.melee_skill(content),
            (AttackKind::Melee, Some(_)) => skills.of_role(Role::ShieldBash),
            (AttackKind::Ranged, _) => self
                .launcher(content)
                .and_then(|(launcher, _)| content.object(launcher).skill),
            (AttackKind::Thrown, _) => skills.of_role(Role::Thrown),
            (AttackKind::Spell, _) => None,
        }
    }

    /// The missile weapon it has ready and what it is: the object in a slot of action `ready`
    /// that launches something.
    pub fn launcher<'c>(&self, content: &'c Content) -> Option<(ObjectId, &'c Attack)> {
        content.slot_ids().find_map(|(slot, def)| {
            if def.action != ActionKind::Ready {
                return None;
            }
            let stack = self.inventory.get(self.equipment.get(slot)?)?;
            let object = content.object(stack.object);
            object.launches.as_ref()?;
            object.attack.as_ref().map(|a| (stack.object, a))
        })
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

    /// What its melee attacks do: with the weapon it wields, else its natural attack, else its
    /// bare hands; Strength to hit and to damage, and what its weapon skill gives.
    pub fn melee(&self, content: &Content) -> Profile {
        let natural = content.creature(self.creature).attack.as_ref();
        let attack = match (self.weapon(content), natural) {
            (Some((_, weapon)), _) => *weapon,
            (None, Some(natural)) => natural.attack,
            (None, None) => Attack::unarmed(),
        };
        let skill = self.melee_skill(content);
        let profile = Profile::melee(&attack, &self.effective_stats(content))
            .with(&self.benefit(content, skill));
        self.stanced(content, profile)
    }

    /// `profile` with the to-hit and damage of its combat stance.
    pub fn stanced(&self, content: &Content, profile: Profile) -> Profile {
        content.stance_def(self.stance).apply(profile)
    }

    /// What its shots do, with the missile weapon it has ready (`None` without one): Dexterity to
    /// hit, the launcher's own damage, and less of both for each point of Strength below what a
    /// bow needs; and what its skill with the launcher gives.
    pub fn ranged(&self, content: &Content) -> Option<Profile> {
        let (launcher, attack) = self.launcher(content)?;
        let skill = content.object(launcher).skill;
        let profile = Profile::ranged(attack, &self.effective_stats(content))
            .with(&self.benefit(content, skill));
        Some(self.stanced(content, profile))
    }

    /// What a throw of a throwing weapon making `attack` does: Dexterity to hit, Strength to
    /// damage, what its skill at throwing gives, and its combat stance.
    pub fn thrown(&self, content: &Content, attack: &Attack) -> Profile {
        let skill = content.weapon_skills().of_role(Role::Thrown);
        let profile = Profile::thrown(attack, &self.effective_stats(content))
            .with(&self.benefit(content, skill));
        self.stanced(content, profile)
    }

    /// What a bash with the shield it holds does (`None` without one it can bash with): the
    /// shield's `bash` dice, Strength to hit and to damage as for any blow, what its skill at
    /// shield bashing gives, and its combat stance.
    pub fn bash(&self, content: &Content) -> Option<(ObjectId, Profile)> {
        let (shield, attack) = self.shield(content)?;
        let skill = content.weapon_skills().of_role(Role::ShieldBash);
        let profile = Profile::melee(attack, &self.effective_stats(content))
            .with(&self.benefit(content, skill));
        Some((shield, self.stanced(content, profile)))
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
