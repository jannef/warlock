//! Spirit weapon (spec §8 "Prestige classes"; #881): the Hunter's level-1 class ability,
//! `use:spirit_weapon:long_sword` ([`crate::action::Action::Shape`]), from the abilities menu's
//! Class group through the shape list. It calls a weapon of spirit-stuff into the main hand in
//! the shape chosen, one of the [`shapes`]: every melee weapon of the objects, by weapon skill
//! then id. The weapon is a [`Stack::spirit`] of that base object, made and weighing as it, and
//! known for what it is; it goes into the pack and is wielded through the ordinary equip path
//! (`World::equip_saying`), so what the hands held is put away into the pack as any swap does and
//! paid for as one; the call itself is priced as a wield. Nothing else is spent: no mana, no
//! cooldown, no check. A spirit weapon already held fades first, so calling again reshapes it.
//!
//! In the hand it strikes as its base with the talent's gifts ([`gifts`], read by `Actor::melee`
//! and `Actor::off_hand_blow`): more to hit, criticals threatened lower, and every point of
//! Protection ignored ([`super::weapon_active::ALL_PROTECTION`]: the soak draws nothing).
//!
//! **The invariant**: a spirit stack exists only in a hand slot. Whatever takes it out of the
//! hands while it stays in the pack makes it fade ([`World::fade_spirit`], at the end of
//! `World::equip_saying` and `World::remove`: a swap for steel, a move to the missile weapon slot,
//! `remove:SLOT`); a drop or a sale takes it off and lets it fade instead of letting it go
//! ([`World::release_spirit`]: the shop never sees it); a throw is refused ("It will not leave
//! your hand."); quests never count it (`shop::count_of`); what takes it out of the pack (an
//! altar's tithe) takes spirit-stuff, which is called again. Creatures never hold
//! one (no creature has the talent, and no gear names a spirit stack). Nothing here draws at
//! random.

use super::{Spent, World};
use crate::action::Positions;
use crate::combat::{FUMBLE, Profile};
use crate::content::{ActionKind, ClassActive, Content, ObjectId, TalentEffect};
use crate::object::{self, Stack};
use crate::talent::TalentMods;

/// The shapes a spirit weapon may be called in: every object of the `melee_weapons` class with
/// an attack that a hand holds, by weapon skill then id.
pub fn shapes(content: &Content) -> Vec<ObjectId> {
    let Some(class) = content.item_class_id("melee_weapons") else {
        return Vec::new();
    };
    let Some(hand) = content.hand_slots().next() else {
        return Vec::new();
    };
    let mut shapes: Vec<ObjectId> = (0..content.object_defs().len())
        .filter_map(|i| u16::try_from(i).ok().map(ObjectId))
        .filter(|&o| {
            let def = content.object(o);
            def.class == class && def.attack.is_some() && content.fits(o, hand)
        })
        .collect();
    shapes.sort_by(|&a, &b| {
        let key = |o: ObjectId| (content.object(o).skill, content.object(o).id.as_str());
        key(a).cmp(&key(b))
    });
    shapes
}

/// What a spirit weapon's description says of its power, and what it adds to its numbers: the
/// first `spirit_weapon` effect of the talents (one rank today), as `(to_hit, threat, ignores
/// Protection, phrase)`.
pub fn gift(content: &Content) -> Option<(i32, u8, bool, &str)> {
    (content.talents().iter())
        .flat_map(|(_, t)| &t.effects)
        .find_map(|e| match e {
            TalentEffect::SpiritWeapon {
                to_hit,
                threat,
                ignores_protection,
                phrase,
            } => Some((*to_hit, *threat, *ignores_protection, phrase.as_str())),
            _ => None,
        })
}

/// `profile` struck with `stack` sharpened by a spirit weapon's gifts, when it is one and the
/// striker has the talent: to hit, the threat lowered (never below the lowest roll that can hit),
/// and every point of Protection ignored.
pub(super) fn gifts(talents: &TalentMods, stack: &Stack, profile: &mut Profile) {
    let Some(gift) = talents.spirit_weapon.filter(|_| stack.is_spirit()) else {
        return;
    };
    profile.to_hit += gift.to_hit;
    let lowered = (i32::from(profile.threat) - i32::from(gift.threat)).max(FUMBLE + 1);
    profile.threat = u8::try_from(lowered).unwrap_or(profile.threat);
    if gift.ignores_protection {
        profile.penetration = super::weapon_active::ALL_PROTECTION;
    }
}

/// Whether `profile` ignores every point of Protection (a spirit weapon's): what the character
/// screen's attacks say as "pierces armour".
pub fn pierces_all(profile: &Profile) -> bool {
    profile.penetration >= super::weapon_active::ALL_PROTECTION
}

impl World {
    /// `use:ID:SHAPE`: the player uses class active `active` in the shape of object `shape`;
    /// only one that asks for a shape ([`ClassActive::aims_shape`]) may.
    pub(super) fn use_class_shape(&mut self, active: ClassActive, shape: &str) -> Spent {
        if !active.aims_shape() {
            self.say(format!("`{}` takes no shape.", active.id()));
            return Spent::Nothing;
        }
        if !self.may_use_class(active) {
            return Spent::Nothing;
        }
        match active {
            ClassActive::SpiritWeapon => self.call_weapon(shape),
            _ => Spent::Nothing,
        }
    }

    /// Calls a spirit weapon of `shape` into the main hand (module docs). Refused, with no time,
    /// for an id that is no shape's.
    fn call_weapon(&mut self, shape: &str) -> Spent {
        let content = self.content.clone();
        let object = content.object_id(shape);
        let (Some(object), Some(hand)) = (object, content.hand_slots().next()) else {
            self.say(format!("There is no weapon `{shape}` to call."));
            return Spent::Nothing;
        };
        if !shapes(&content).contains(&object) {
            let what = content.object(object).describe(1);
            self.say(format!(
                "You cannot call {what}: it is no weapon for the hand."
            ));
            return Spent::Nothing;
        }
        // One held already fades first: calling again reshapes it.
        self.fade_where(true);
        let inventory = self.state.inventory_mut();
        object::add_to_inventory(inventory, Stack::spirit(object), &content);
        let item = self.state.player().inventory.len() - 1;
        let def = content.object(object);
        let hands = if def.two_handed { "hands" } else { "hand" };
        let line = format!(
            "{} of pale light is in your {hands}.",
            crate::text::capitalized(&def.describe(1))
        );
        // Priced by the swap: a wield, and what the hands held put away as any swap's.
        let slot = content.slot(hand).id.clone();
        self.equip_saying(&slot, item, Some(line))
    }

    /// The spirit weapons the player holds out of the hands fade (module docs): taken out of the
    /// pack, the positions after them followed, each said. Returns the positions they had.
    pub(super) fn fade_spirit(&mut self) -> Positions {
        self.fade_where(false)
    }

    /// [`World::fade_spirit`]; with `held` every spirit weapon fades, in the hands too.
    fn fade_where(&mut self, held: bool) -> Positions {
        let content = self.content.clone();
        let hands: Vec<_> = content.hand_slots().collect();
        let player = self.state.player();
        let in_hand = |i: usize| {
            let slots = player.equipment.slots_of(i);
            !slots.is_empty() && slots.iter().all(|s| hands.contains(s))
        };
        let gone = Positions::of(
            (0..player.inventory.len())
                .filter(|&i| player.inventory[i].is_spirit() && (held || !in_hand(i))),
        );
        if gone.is_empty() {
            return gone;
        }
        let names: Vec<String> = (gone.iter())
            .map(|i| content.object(player.inventory[i].object).name.clone())
            .collect();
        let player = self.player_mut();
        for i in gone.iter().collect::<Vec<_>>().into_iter().rev() {
            player.inventory.remove(i);
        }
        player.equipment.after_removal(&gone);
        for name in names {
            self.say(format!("The {name} fades from your hand."));
        }
        gone
    }

    /// Of the inventory stacks `chosen` names (to drop, or to sell or leave in a shop), the
    /// spirit weapons are taken off and fade rather than leave the pack (module docs). Returns
    /// the rest, numbered for the pack without them, and what taking them off cost.
    pub(super) fn release_spirit(&mut self, chosen: &Positions) -> (Positions, Vec<ActionKind>) {
        let inventory = &self.state.player().inventory;
        let spirit: Vec<usize> = (chosen.iter())
            .filter(|&i| inventory.get(i).is_some_and(Stack::is_spirit))
            .collect();
        if spirit.is_empty() {
            return (chosen.clone(), Vec::new());
        }
        let kinds = self.take_off_before_dropping(spirit.iter().copied());
        let gone = self.fade_spirit();
        let rest = (chosen.iter())
            .filter(|&i| !gone.contains(i))
            .map(|i| i - gone.iter().filter(|&g| g < i).count());
        (Positions::of(rest), kinds)
    }
}
