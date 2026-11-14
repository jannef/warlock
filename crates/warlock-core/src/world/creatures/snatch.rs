//! Theft on a cooldown (docs/design/monster-dread.md §5.6, G6a; #1130): a creature type with
//! `steals` (`content/theft.rs`: the pixie, the nymph, the imp, the succubus and the incubus)
//! takes things from the player's pack as it fights them. Enemy-only: no creature robs another,
//! and the player's own Steal is `steal.rs`.
//!
//! **Up close** ([`World::snatches_at`], asked in `ai.rs::fight` before a cast): a thief beside
//! the player with its cooldown off strikes in melee rather than cast, so a caster thief (the
//! pixie, the nymph) steals too; on its cooldown it fights as its type does.
//!
//! **When** ([`World::snatch`], hooked in `fight.rs` after a blow's riders): a melee hit of the
//! thief's lands on the player and leaves them alive, and the thief does not carry its cooldown
//! status (`theft`). Then the chance, one gameplay draw; made, one stack of what the player
//! carries and neither wears nor wields, of the classes its `what` allows (`small`: coins, gems,
//! potions, rings, scrolls; `any`: anything), picked uniformly (one more draw with two or more to
//! pick from, the Steal ability's way). With nothing to take, nothing happens and no cooldown is
//! given.
//!
//! **What**: the whole stack moves into the thief's pack (it joins a stack there of the same
//! goods), the thief takes the cooldown status, and it **fights on** as it was: no flight, no
//! blink away (the owner's word, design §17 D1). What it took falls where it dies, as anything a
//! creature carries does (`fight.rs::dies`).
//!
//! **Words**: "The nymph snatches a scroll of frost from you!" (by the type's name, as a seen
//! creature is said; "Something" for one the player does not see: they feel the hand). The
//! message history keeps it, so the player knows what to look for in the drop.

use crate::action::{AttackKind, Event, Positions};
use crate::content::{SMALL_LOOT, Takes};
use crate::object;
use crate::world::{ActorId, World};

impl World {
    /// Whether creature `thief` would rather strike its target `target` to steal than do
    /// anything else (`ai.rs::fight`, before a cast): it is a thief, the target is the player
    /// beside it, and its cooldown is off. A caster thief so steps out of its spells up close.
    pub(super) fn snatches_at(&self, thief: ActorId, target: ActorId) -> bool {
        let me = &self.state.actors[&thief];
        let Some(steals) = self.content.creature(me.creature).steals.as_ref() else {
            return false;
        };
        let Some(victim) = self.state.actors.get(&target) else {
            return false;
        };
        target == self.state.player
            && me.pos.chebyshev(victim.pos) <= 1
            && !me.effects.has(steals.status)
    }

    /// Creature `thief`'s melee blow hit `victim` and left them alive: it may steal (module
    /// docs).
    pub(in crate::world) fn snatch(
        &mut self,
        thief: ActorId,
        victim: ActorId,
        kind: AttackKind,
        events: &mut Vec<Event>,
    ) {
        let player = self.state.player;
        if kind != AttackKind::Melee || victim != player || thief == player {
            return;
        }
        let content = self.content.clone();
        let Some(me) = self.state.actors.get(&thief) else {
            return;
        };
        let Some(steals) = content.creature(me.creature).steals.clone() else {
            return;
        };
        if me.effects.has(steals.status) || !self.state.actors.contains_key(&victim) {
            return;
        }
        if !self.state.rng.chance(u32::from(steals.chance)) {
            return;
        }
        let small: Vec<_> = (SMALL_LOOT.iter())
            .filter_map(|c| content.item_class_id(c))
            .collect();
        let pack = self.state.player();
        let loose: Vec<usize> = (0..pack.inventory.len())
            .filter(|&i| !pack.equipment.holds(i))
            .filter(|&i| match steals.what {
                Takes::Any => true,
                Takes::Small => small.contains(&content.object(pack.inventory[i].object).class),
            })
            .collect();
        let pick = match loose.len() {
            0 => return,
            1 => loose[0],
            n => loose[self.state.rng.index(n)],
        };
        let me = self.player_mut();
        let stack = me.inventory.remove(pick);
        me.equipment.after_removal(&Positions::of([pick]));
        let Some(robber) = self.state.actors.get_mut(&thief) else {
            return;
        };
        object::add_to_inventory(&mut robber.inventory, stack.clone(), &content);
        let pos = robber.pos;
        let name = content.creature(robber.creature).name.clone();
        self.afflict(thief, steals.status, steals.degrees, None, events);
        let what = object::phrase(std::slice::from_ref(&stack), self.lore());
        let who = if self.perception.sees_actors_at(pos) {
            crate::content::the_start(&name)
        } else {
            "Something".to_string()
        };
        self.say(format!("{who} snatches {what} from you!"));
    }
}
