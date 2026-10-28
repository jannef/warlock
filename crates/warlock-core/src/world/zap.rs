//! Using a wand (spec §8 "Magic items": "Wands", the magic design §9): a wand in the missile
//! weapon slot is used as a missile weapon is (`t`, `fire:X,Y`), and casts the spell it holds
//! through the one cast path ([`World::wand_cast`]):
//!
//! - **Caster level**: the wand's own (`[object.wand] caster_level`, by default its tier's
//!   comfortable level), plus Wand mastery's (`TalentMods::wand_level`) and the worn items'
//!   (`wand_level`, of Wands, unidentified too); never the user's, so a
//!   warrior fires a wand as well as a wizard does. Its dice grow with that level and are held
//!   to the spell's cap as every cast's are.
//! - **Resolution**: the spell's own: an `attack` spell rolls to hit (the user's Dexterity, as
//!   every spell's attack roll), `auto` and `area` ones do not. Reach is the user's (Willpower
//!   counted), as a scroll read is.
//! - **Charges**: each cast that goes off uses one, instead of mana; one refused (out of reach,
//!   nobody to touch) uses none and takes no time. An empty wand does nothing ("Nothing
//!   happens.", no time) and stays an item; the player then knows it is empty. Wands are never
//!   recharged.
//! - **Identification**: a wand type is unknown ("an oak wand", [`crate::scroll_label`]) until
//!   one is used or identified; its charges stay unknown until it is identified (the scroll of
//!   identify, Connoisseur's carrying).
//! - **Time**: the `cast` action's, as many times as the spell's `casts`.
//!
//! Determinism: nothing here draws; the cast draws what the spell's rolls need.

use super::{Actor, Spent, World};
use crate::action::Event;
use crate::content::{ActionKind, Content, Effect, Wand};
use crate::geom::Pos;

impl Actor {
    /// Where in its inventory the wand it has ready is: the first stack in a `ready` slot (the
    /// missile weapon slot) that is a wand.
    pub fn wand_item(&self, content: &Content) -> Option<usize> {
        content.slot_ids().find_map(|(slot, def)| {
            if def.action != ActionKind::Ready {
                return None;
            }
            let item = self.equipment.get(slot)?;
            let stack = self.inventory.get(item)?;
            content.object(stack.object).wand.map(|_| item)
        })
    }

    /// The caster level `wand` casts at in its hands: the wand's own, plus Wand mastery's and
    /// the worn items' `wand_level` (module docs), at least 1.
    pub fn wand_caster_level(&self, content: &Content, wand: &Wand) -> i32 {
        let worn = self.mods(content).get(Effect::WandLevel);
        (i32::from(wand.caster_level) + self.talent_mods(content).wand_level + worn).max(1)
    }
}

impl World {
    /// The player uses the wand in inventory stack `item` at cell `at` (module docs).
    pub(super) fn zap(&mut self, item: usize, at: Pos, events: &mut Vec<Event>) -> Spent {
        let content = self.content.clone();
        let me = self.state.player();
        let Some(stack) = me.inventory.get(item) else {
            self.say("You carry no such thing.");
            return Spent::Nothing;
        };
        let Some(wand) = content.object(stack.object).wand else {
            return Spent::Nothing;
        };
        if stack.charges_left(&content) == Some(0) {
            self.say("Nothing happens.");
            // Now the player knows it is empty.
            self.player_mut().inventory[item].identify_charges(&content);
            return Spent::Nothing;
        }
        let level = me.wand_caster_level(&content, &wand);
        let casts = i64::from(content.spell(wand.spell).casts);
        let price = self.price_of(self.state.player, ActionKind::Cast) * casts;
        if self.wand_cast(item, (wand.spell, level), at, events) {
            Spent::Turn(price)
        } else {
            Spent::Nothing
        }
    }

    /// Uses up a charge of the wand in inventory stack `item` as its spell goes off, and says
    /// so: "You zap the oak wand." A wand type used for the first time is identified: "It is a
    /// wand of fireball."
    pub(super) fn use_charge(&mut self, item: usize) {
        let Some(stack) = self.player_mut().inventory.get_mut(item) else {
            return;
        };
        stack.use_charge();
        let wand = stack.clone();
        let title = self.lore().title(&wand);
        self.say(format!("You zap the {title}."));
        if self.learn_scroll(wand.object) {
            let what = self.content.object(wand.object).describe(1);
            self.say(format!("It is {what}."));
        }
    }
}
