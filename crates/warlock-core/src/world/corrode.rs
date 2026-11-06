//! Corroding gear (spec §8 "Creature gimmicks"; the dread design §5.8): a creature type's
//! `corrodes = { chance, what }` eats at the player's gear in melee, and leaves it **damaged**,
//! the smith's state ([`Worked`]: its improvements gone and a point worse until mended at an
//! anvil, `world/temper.rs`).
//!
//! - **`armour`** (the hezrou's filth): a melee hit of its that lands on the player rolls the
//!   chance; made, one piece the player wears (armour or a shield) whose material corrodes
//!   ([`crate::content::MaterialDef::corrodes`], by its main part) and is not damaged yet is
//!   drawn and damaged: "The acid eats at your chain mail."
//! - **`weapon`** (the ember beetle's acid blood): a melee hit of the player's that strikes it
//!   rolls the chance on what struck (the weapon in hand, or the shield that bashed): "Your long
//!   sword comes away pitted."
//! - The chance is halved for a player who resists acid at all, and a material that corrodes at
//!   `"half"` (bone, horn, ivory, chitin) takes a second roll at 50. Nothing is damaged twice and
//!   nothing is destroyed. A spirit weapon and a stack of several are left be.
//! - Made, but nothing the player has on corrodes (mithril, gold, wood, stone): nothing happens,
//!   and the first time in a life they are told why: "The acid runs off your mithril chain
//!   mail." (`Progress::once`, [`RUNS_OFF`]).
//!
//! Draws: the gameplay stream, one for the chance, then (made, with pieces to choose among) one
//! for the piece and one for a `"half"` material's second roll. None for a blow by or at
//! anything without `corrodes`.

use super::{ActorId, World};
use crate::action::AttackKind;
use crate::content::{CorrodesWhat, Corrosion, ObjectId};
use crate::magic::Worked;

/// The chance in percent that acid of `chance` eats at the gear of one who resists acid by
/// `resistance` percent: halved (rounded down) for any resistance at all.
pub(super) fn chance(chance: u8, resistance: i32) -> u32 {
    let chance = u32::from(chance);
    if resistance > 0 { chance / 2 } else { chance }
}

/// The once-in-a-life flag of the line that says why acid did nothing.
pub(super) const RUNS_OFF: &str = "acid_runs_off";

impl World {
    /// A melee hit by `attacker` landed on `target`, struck `with` what [`World::resolve`] names
    /// (none: the main hand's weapon): the acid of either eats at the player's gear, as the
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
        let acid =
            (self.actor_or_fallen(bearer)).and_then(|a| self.content.creature(a.creature).corrodes);
        let Some(acid) = acid.filter(|a| a.what == what) else {
            return;
        };
        let resistance = (self.content.damage_type_id("acid"))
            .map_or(0, |acid| self.resistances_of(player).get(Some(acid)));
        if !self.state.rng.chance(chance(acid.chance, resistance)) {
            return;
        }
        let exposed = self.exposed(what, with);
        let content = self.content.clone();
        let me = self.state.player();
        let corrodes = |item: usize| {
            let main = me.inventory[item]
                .parts(&content)
                .first()
                .map(|p| p.material);
            main.map_or(Corrosion::No, |m| content.material(m).corrodes())
        };
        let damaged = |item: usize| me.inventory[item].worked().is_some_and(|w| w.damaged);
        let eaten: Vec<(usize, Corrosion)> = (exposed.iter())
            .map(|&item| (item, corrodes(item)))
            .filter(|&(_, c)| c != Corrosion::No)
            .collect();
        let shown = exposed
            .first()
            .map(|&item| me.inventory[item].names(&content).0);
        let fresh: Vec<(usize, Corrosion)> = (eaten.iter().copied())
            .filter(|&(item, _)| !damaged(item))
            .collect();
        if eaten.is_empty() {
            if let Some(name) = shown
                && !self.state.progress.happened(RUNS_OFF)
            {
                self.state.progress.once.insert(RUNS_OFF.to_string());
                self.say(format!("The acid runs off your {name}."));
            }
            return;
        }
        if fresh.is_empty() {
            return;
        }
        let (item, corrosion) = fresh[self.state.rng.index(fresh.len())];
        if corrosion == Corrosion::Half && !self.state.rng.chance(50) {
            return;
        }
        let stack = &mut self.player_mut().inventory[item];
        let name = stack.names(&content).0;
        let mut worked = stack.worked().cloned().unwrap_or_else(Worked::default);
        worked.damage();
        stack.set_worked(worked);
        self.say(match what {
            CorrodesWhat::Armour => format!("The acid eats at your {name}."),
            CorrodesWhat::Weapon => format!("Your {name} comes away pitted."),
        });
    }

    /// Where in the player's pack lies what acid of `what` may reach, the body armour first: what
    /// they wear (armour and shields) for `armour`, what struck (`with`, else the main hand's
    /// weapon) for `weapon`. Never a spirit weapon or a stack of several.
    fn exposed(&self, what: CorrodesWhat, with: Option<ObjectId>) -> Vec<usize> {
        let content = &self.content;
        let me = self.state.player();
        let mut items: Vec<usize> = match what {
            CorrodesWhat::Armour => {
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
