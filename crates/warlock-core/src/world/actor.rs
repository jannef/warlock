//! Actors: the player and creatures.

use crate::burden::Burden;
use crate::combat::{self, Attack, Category, Defence, Health, Profile};
use crate::content::{ActionKind, Content, CreatureId, FactionId, ObjectDef, ObjectId};
use crate::equipment::Equipment;
use crate::geom::{Dir, Pos};
use crate::noise::Heard;
use crate::object::{Stack, Weight, weight_of};
use crate::skills::{Effect, Skills};
use crate::stats::{Stat, Stats};
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
    /// The faction its level placed it in, if not its type's (`faction` in a level legend entry:
    /// peaceful creatures in the showcase). Read the one that counts with [`Actor::faction`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub faction_override: Option<FactionId>,
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

    /// Its maximum hit points: Toughness plus [`Actor::hp_base`], at least 1.
    pub fn max_hp(&self) -> i32 {
        combat::max_hp(self.stats.score(Stat::Toughness), self.hp_base)
    }

    /// How hurt it looks, in words.
    pub fn health(&self) -> Health {
        Health::of(self.hp, self.max_hp())
    }

    /// Its Defence (spec §8 "Combat"): 10 plus its Dexterity modifier (capped by the worn
    /// armour that allows the least and by its `burden`, the burden it suffers, spec §8
    /// "Burden": the lowest cap wins), the best armour, shield, and parry of what it wears and
    /// wields, its creature type's natural armour, and the Dodge its skills give (tumble; spec
    /// §8 "Skills").
    pub fn defence(&self, content: &Content, burden: Burden) -> Defence {
        let equipped: Vec<&ObjectDef> = (self.equipment.items().into_iter())
            .filter_map(|(_, item)| self.inventory.get(item))
            .map(|s| content.object(s.object))
            .collect();
        let dexterity = self.stats.modifier(Stat::Dexterity);
        let worn = equipped.iter().filter_map(|o| o.max_dex);
        let cap = worn.chain(burden.max_dex()).min();
        let dexterity = cap.map_or(dexterity, |cap| dexterity.min(cap));
        let worn = equipped.iter().flat_map(|o| {
            [
                (Category::Armour, o.armour),
                (Category::Shield, o.shield),
                (Category::Parry, o.parry),
            ]
        });
        let natural = (content.creature(self.creature).natural.as_ref())
            .map(|hide| (Category::Natural, hide.defence));
        let dodge = Some(self.skills.effect(content, Effect::Dodge))
            .filter(|&d| d != 0)
            .map(|d| (Category::Dodge, d));
        Defence::from_bonuses(
            std::iter::once((Category::Dexterity, dexterity))
                .chain(worn)
                .chain(natural)
                .chain(dodge),
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
        content.hand_slots().find_map(|slot| {
            let stack = self.inventory.get(self.equipment.get(slot)?)?;
            let def = content.object(stack.object);
            def.attack.as_ref().map(|a| (stack.object, a))
        })
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
    /// bare hands; Strength to hit and to damage.
    pub fn melee(&self, content: &Content) -> Profile {
        let natural = content.creature(self.creature).attack.as_ref();
        let attack = match (self.weapon(content), natural) {
            (Some((_, weapon)), _) => *weapon,
            (None, Some(natural)) => natural.attack,
            (None, None) => Attack::unarmed(),
        };
        Profile::melee(&attack, &self.stats)
    }

    /// What its shots do, with the missile weapon it has ready (`None` without one): Dexterity to
    /// hit, the launcher's own damage, and less of both for each point of Strength below what a
    /// bow needs.
    pub fn ranged(&self, content: &Content) -> Option<Profile> {
        let (_, launcher) = self.launcher(content)?;
        Some(Profile::ranged(launcher, &self.stats))
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
