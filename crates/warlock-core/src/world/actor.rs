//! Actors: the player and creatures.

use crate::action::AttackKind;
use crate::burden::Burden;
use crate::combat::{self, Attack, Category, Defence, Health, Profile};
use crate::content::Effect as ItemEffect;
use crate::content::{
    ActionKind, Benefit, CombatStance, Content, CreatureId, FactionId, ObjectDef, ObjectId, Role,
    WeaponSkillId,
};
use crate::equipment::Equipment;
use crate::geom::{Dir, Pos};
use crate::magic::Mods;
use crate::noise::Heard;
use crate::object::{Stack, Weight, weight_of};
use crate::skills::{Effect, Skills};
use crate::stats::{Stat, Stats};
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
    /// player is shown only their own scores.
    pub stats: Stats,
    /// Hit points left (spec §8 "Combat"); at 0 or below it is dead.
    pub hp: i32,
    /// What its maximum hit points add to its Toughness: the creature type's `base_hp`, the
    /// player's race and class (the scenario's `hp`).
    pub hp_base: i32,
    /// Seconds of game time toward its next hit point back, while it is hurt.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub regen: u32,
    /// Its trained skills, with their values and training marks (spec §8 "Skills";
    /// [`crate::skills`]): its creature type's, the player's from their race and class. Every
    /// other skill is untrained: it gives nothing. Known to the rules (and dev tools) for every
    /// actor; the player is shown only their own trained skills' values.
    #[serde(default, skip_serializing_if = "Skills::is_empty")]
    pub skills: Skills,
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

    /// What everything it wears and wields does to it (spec §8 "Magic items"; their global
    /// effects, [`Mods::worn`]).
    pub fn mods(&self, content: &Content) -> Mods {
        Mods::worn(&self.inventory, &self.equipment, content)
    }

    /// Its stats as they are now: its own, each score raised by what it wears and wields
    /// (spec §8 "Magic items"). Every rule that reads a score reads these.
    pub fn scores(&self, content: &Content) -> Stats {
        self.stats_with(&self.mods(content))
    }

    fn stats_with(&self, mods: &Mods) -> Stats {
        self.stats.raised(|stat| mods.stat(stat))
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

    /// Its maximum hit points: Toughness plus [`Actor::hp_base`] plus what it wears and wields
    /// adds, at least 1.
    pub fn max_hp(&self, content: &Content) -> i32 {
        let mods = self.mods(content);
        let toughness = self.stats_with(&mods).score(Stat::Toughness);
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
    /// the wielded weapon's skill to its parry, shield blocking to a held shield's.
    pub fn defence(&self, content: &Content, burden: Burden) -> Defence {
        let mods = self.mods(content);
        let weapon = self.weapon_item(content);
        let parry_skill = self.benefit(content, self.melee_skill(content)).parry;
        let block = (content.weapon_skills().of_role(Role::ShieldBlock))
            .map_or(0, |id| self.benefit(content, Some(id)).shield);
        let equipped: Vec<(usize, &ObjectDef, Mods)> = (self.equipment.items().into_iter())
            .filter_map(|(_, item)| Some((item, self.inventory.get(item)?)))
            .map(|(item, s)| (item, content.object(s.object), Mods::local(s, content)))
            .collect();
        let dexterity = self.stats_with(&mods).modifier(Stat::Dexterity);
        let worn =
            (equipped.iter()).filter_map(|(_, o, m)| Some(o.max_dex? + m.get(ItemEffect::MaxDex)));
        let cap = worn.chain(burden.max_dex()).min();
        let dexterity = cap.map_or(dexterity, |cap| dexterity.min(cap));
        let worn = equipped.iter().flat_map(|(item, o, m)| {
            let wielded = Some(*item) == weapon;
            let shield = o.shield + m.get(ItemEffect::Shield);
            let parry = o.parry + m.get(ItemEffect::Parry);
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

    /// How far it throws inventory stack `item`, if that is a throwing weapon: its `range`, and
    /// what its affixes add.
    pub fn throw_range(&self, item: usize, content: &Content) -> Option<i32> {
        let stack = self.inventory.get(item)?;
        let def = content.object(stack.object);
        let range = def.range.filter(|_| def.is_thrown())?;
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

    /// The throwing weapon it has ready, if any: the first stack in a `ready` slot (the
    /// missiles) that is thrown (a quiver of rocks), and where in its inventory it is.
    pub fn ready_to_throw(&self, content: &Content) -> Option<usize> {
        content.slot_ids().find_map(|(slot, def)| {
            if def.action != ActionKind::Ready {
                return None;
            }
            let item = self.equipment.get(slot)?;
            let stack = self.inventory.get(item)?;
            content.object(stack.object).is_thrown().then_some(item)
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
    /// bare hands; Strength to hit and to damage, and what its weapon skill gives; then the
    /// weapon's own affixes and those of everything it wears and wields (spec §8 "Magic items");
    /// and its combat stance.
    pub fn melee(&self, content: &Content) -> Profile {
        let mods = self.mods(content);
        let natural = content.creature(self.creature).attack.as_ref();
        let weapon = self.weapon_item(content).map(|i| &self.inventory[i]);
        let (attack, local) = match (weapon, natural) {
            (Some(stack), _) => match &content.object(stack.object).attack {
                Some(attack) => (*attack, Mods::local(stack, content)),
                None => (Attack::unarmed(), Mods::default()),
            },
            (None, Some(natural)) => (natural.attack, Mods::default()),
            (None, None) => (Attack::unarmed(), Mods::default()),
        };
        let skill = self.melee_skill(content);
        let mut profile =
            Profile::melee(&attack, &self.stats_with(&mods)).with(&self.benefit(content, skill));
        local.sharpen(&mut profile);
        mods.sharpen(&mut profile);
        self.stanced(content, profile)
    }

    /// `profile` with the to-hit and damage of its combat stance.
    pub fn stanced(&self, content: &Content, profile: Profile) -> Profile {
        content.stance_def(self.stance).apply(profile)
    }

    /// What its shots do, with the missile weapon it has ready (`None` without one): Dexterity to
    /// hit, the launcher's own damage, and less of both for each point of Strength below what a
    /// bow needs; what its skill with the launcher gives; then the affixes of the launcher, of
    /// its ammunition, and of everything it wears and wields; and its combat stance.
    pub fn ranged(&self, content: &Content) -> Option<Profile> {
        let mods = self.mods(content);
        let launcher = &self.inventory[self.launcher_item(content)?];
        let def = content.object(launcher.object);
        let mut attack = def.attack?;
        let local = Mods::local(launcher, content);
        let draw = local.get(ItemEffect::Draw);
        attack.strength =
            (attack.strength).map(|s| u8::try_from((i32::from(s) - draw).max(1)).unwrap_or(1));
        let mut profile = Profile::ranged(&attack, &self.stats_with(&mods))
            .with(&self.benefit(content, def.skill));
        local.sharpen(&mut profile);
        if let Some(ammo) = self.ammunition_item(content) {
            Mods::local(&self.inventory[ammo], content).sharpen(&mut profile);
        }
        mods.sharpen(&mut profile);
        Some(self.stanced(content, profile))
    }

    /// What a throw of inventory stack `item` does (`None` if it is no throwing weapon):
    /// Dexterity to hit, Strength to damage, what its skill at throwing gives, its own affixes,
    /// those of everything it wears and wields, and its combat stance.
    pub fn thrown(&self, item: usize, content: &Content) -> Option<Profile> {
        let stack = self.inventory.get(item)?;
        let def = content.object(stack.object);
        let attack = def.attack.filter(|_| def.is_thrown())?;
        let mods = self.mods(content);
        let skill = content.weapon_skills().of_role(Role::Thrown);
        let mut profile =
            Profile::thrown(&attack, &self.stats_with(&mods)).with(&self.benefit(content, skill));
        Mods::local(stack, content).sharpen(&mut profile);
        mods.sharpen(&mut profile);
        Some(self.stanced(content, profile))
    }

    /// What a bash with the shield it holds does (`None` without one it can bash with): the
    /// shield's `bash` dice, Strength to hit and to damage as for any blow, what its skill at
    /// shield bashing gives, the affixes of everything it wears and wields, and its combat stance.
    pub fn bash(&self, content: &Content) -> Option<(ObjectId, Profile)> {
        let (shield, attack) = self.shield(content)?;
        let skill = content.weapon_skills().of_role(Role::ShieldBash);
        let mods = self.mods(content);
        let mut profile =
            Profile::melee(attack, &self.stats_with(&mods)).with(&self.benefit(content, skill));
        mods.sharpen(&mut profile);
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
