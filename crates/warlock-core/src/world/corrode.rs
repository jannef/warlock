//! Harm to gear (spec §8 "Creature gimmicks"; the dread design §5.8): a creature type's
//! `corrodes = { chance, what, kind }` damages the player's gear in melee by item damage of
//! `kind` (acid, rust, or fire; `item_damage.rs`, the one damaged state the smith mends at an
//! anvil, `world/temper.rs`).
//!
//! - **`armour`** (the hezrou's filth): a melee hit of its that lands on the player rolls the
//!   chance; made, one piece the player wears (armour or a shield) is drawn: "The acid eats at
//!   your chain mail."
//! - **`weapon`** (the ember beetle's acid blood): a melee hit of the player's that strikes it
//!   rolls the chance on what struck (the weapon in hand, or the shield that bashed): "Your long
//!   sword comes away pitted."
//! - **`both`** (the rust monster's feelers and the black pudding, #1138): each of the two, at
//!   the chance.
//! - The chance is halved for a player who resists the damage type of the kind's id at all
//!   (acid, fire; rust has none). A spirit weapon and a stack of several are left be.
//! - The piece drawn then takes the harm ([`World::harm_item`]): its material withstands it by
//!   its `resists` chance of the kind (mithril and gold nearly always, iron seldom), else it is
//!   damaged, and one damaged already is destroyed: "The acid eats your chain mail away."
//!   Withstood, the first time in a life for each kind the player is told why: "The acid runs
//!   off your mithril chain mail." (`Progress::once`, [`Lines::once`]).
//!
//! Draws: the gameplay stream, one for the chance, then (made, with pieces to choose among) one
//! for the piece and one for the material's chance unless it is 0 or 100. None for a blow by or
//! at anything without `corrodes`.

use super::item_damage::Harmed;
use super::{ActorId, World};
use crate::action::AttackKind;
use crate::content::{CorrodesWhat, ItemHarm, ObjectId};

/// The chance in percent that harm of `chance` reaches the gear of one who resists its damage
/// type by `resistance` percent: halved (rounded down) for any resistance at all.
pub(super) fn chance(chance: u8, resistance: i32) -> u32 {
    let chance = u32::from(chance);
    if resistance > 0 { chance / 2 } else { chance }
}

/// What is said of harm of one kind to a piece named `{}`: worn, struck with, destroyed, and
/// withstood (that once in a life, by the flag `once`).
pub(super) struct Lines {
    pub(super) worn: &'static str,
    pub(super) struck: &'static str,
    pub(super) gone: &'static str,
    pub(super) withstood: &'static str,
    pub(super) once: &'static str,
}

/// The lines of harm `kind` (never [`ItemHarm::Work`], refused at load for a creature).
pub(super) const fn lines(kind: ItemHarm) -> Lines {
    match kind {
        ItemHarm::Rust => Lines {
            worn: "Rust spreads over your {}.",
            struck: "Your {} comes away rusted.",
            gone: "Your {} rusts away to nothing.",
            withstood: "Your {} takes no rust.",
            once: "rust_takes_none",
        },
        ItemHarm::Fire => Lines {
            worn: "The heat scorches your {}.",
            struck: "Your {} comes away scorched.",
            gone: "Your {} burns away.",
            withstood: "The heat leaves your {} unmarked.",
            once: "heat_leaves_unmarked",
        },
        ItemHarm::Acid | ItemHarm::Work => Lines {
            worn: "The acid eats at your {}.",
            struck: "Your {} comes away pitted.",
            gone: "The acid eats your {} away.",
            withstood: "The acid runs off your {}.",
            once: "acid_runs_off",
        },
    }
}

impl World {
    /// A melee hit by `attacker` landed on `target`, struck `with` what [`World::resolve`] names
    /// (none: the main hand's weapon): the harm of either reaches the player's gear, as the
    /// module docs say.
    pub(super) fn corrode(
        &mut self,
        (attacker, target): (ActorId, ActorId),
        (kind, with): (AttackKind, Option<ObjectId>),
    ) {
        let player = self.state.player;
        if kind != AttackKind::Melee || self.state.death.is_some() {
            return;
        }
        let (bearer, what) = if target == player {
            (attacker, CorrodesWhat::Armour)
        } else if attacker == player {
            (target, CorrodesWhat::Weapon)
        } else {
            return;
        };
        let harm =
            (self.actor_or_fallen(bearer)).and_then(|a| self.content.creature(a.creature).corrodes);
        let Some(harm) = harm.filter(|a| a.what.eats(what)) else {
            return;
        };
        let resistance = (self.content.damage_type_id(harm.kind.id()))
            .map_or(0, |t| self.resistances_of(player).get(Some(t)));
        if !self.state.rng.chance(chance(harm.chance, resistance)) {
            return;
        }
        let exposed = self.exposed(what, with);
        if exposed.is_empty() {
            return;
        }
        let item = exposed[self.state.rng.index(exposed.len())];
        let name = self.state.player().inventory[item].names(&self.content).0;
        let lines = lines(harm.kind);
        let line = match self.harm_item(item, harm.kind) {
            Harmed::Withstood if self.state.progress.happened(lines.once) => return,
            Harmed::Withstood => {
                self.state.progress.once.insert(lines.once.to_string());
                lines.withstood
            }
            Harmed::Damaged => match what {
                CorrodesWhat::Armour | CorrodesWhat::Both => lines.worn,
                CorrodesWhat::Weapon => lines.struck,
            },
            Harmed::Destroyed => lines.gone,
        };
        self.say(line.replacen("{}", &name, 1));
    }

    /// Where in the player's pack lies what acid of `what` may reach, the body armour first: what
    /// they wear (armour and shields) for `armour`, what struck (`with`, else the main hand's
    /// weapon) for `weapon`. Never a spirit weapon or a stack of several.
    fn exposed(&self, what: CorrodesWhat, with: Option<ObjectId>) -> Vec<usize> {
        let content = &self.content;
        let me = self.state.player();
        let mut items: Vec<usize> = match what {
            // `what` is the side struck, never `Both` (that is the creature's, `corrode`).
            CorrodesWhat::Armour | CorrodesWhat::Both => {
                let mut worn: Vec<usize> = Vec::new();
                for (_, item) in me.equipment.items() {
                    let kind = (me.inventory.get(item)).and_then(|s| content.object(s.object).kind);
                    let protects = kind
                        .is_some_and(|k| k.is_armour() || k == crate::content::ItemKind::Shield);
                    if protects && !worn.contains(&item) {
                        worn.push(item);
                    }
                }
                // The body armour first: the piece the runoff line names.
                let body = |i: &usize| {
                    let kind = content.object(me.inventory[*i].object).kind;
                    kind != Some(crate::content::ItemKind::Body)
                };
                worn.sort_by_key(body);
                worn
            }
            CorrodesWhat::Weapon => match with {
                None => me.weapon_item(content).into_iter().collect(),
                Some(object) => (me.in_hands(content).into_iter())
                    .map(|(item, _)| item)
                    .rfind(|&item| me.inventory[item].object == object)
                    .into_iter()
                    .collect(),
            },
        };
        items.retain(|&item| {
            let stack = &me.inventory[item];
            stack.count == 1 && !stack.is_spirit()
        });
        items
    }
}
