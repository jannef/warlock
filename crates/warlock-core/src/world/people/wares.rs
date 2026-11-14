//! A trader's one offer (spec §8 "Dialogue": "An offer"; the pedlar, #808,
//! docs/design/overworld-encounters.md §15 Q7): one item at a fixed price, take it or leave it.
//!
//! - **Its making**: the first time the player chats with or asks a creature whose dialogue has
//!   an [`TradeOffer`], its wares are made ([`World::stock_wares`]) from the stream
//!   `wares/<actor id>` (the stream is the trader's own, so nothing else moves): one
//!   `below(10_000)` against the offer's `unique_chance` when it names uniques; made, one
//!   `below` among those of its uniques not yet offered in the run
//!   ([`WorldState::uniques_offered`], saved), which is then recorded; else (or with none left)
//!   the favoured roll ([`crate::magic::loot::favoured`]) at the player's level plus the
//!   offer's bonus, at least its least. The item is identified: a trader names his wares.
//! - **Its price**, fixed (no favour, no haggling): a unique's is the offer's `unique_price`;
//!   anything else's its value plus `affix_price` per affix per item level.
//! - **The topic** ([`World::offer_topic`]): after the services, with the price and the item in
//!   words (`TopicView::ware`) while it is for sale; the dialogue screen asks first in the one
//!   dialog ("Buy it" / "Leave it"). Leaving it is nothing: no action, no time.
//! - **Buying** (`ask:E:wares`, [`World::ask_offer`]): without the gold, "You cannot pay." (no
//!   time); else the gold goes to the trader (`hand_over`), the item into the pack, the trader
//!   says the offer's `reply`, a deal's marks are made, and it takes `chat`'s time. Asked again
//!   after, the trader says `sold`: one offer per trader. A trader met again (a known party) is
//!   the same actor, with the same offer; another trader has one of his own.
//!
//! Saved: [`WorldState::wares`] (by trader) and [`WorldState::uniques_offered`], each left out
//! while empty, so runs that never ask a trader hash as before.
//!
//! [`WorldState::wares`]: crate::world::WorldState
//! [`WorldState::uniques_offered`]: crate::world::WorldState

use super::shop::count_of;
use crate::action::Event;
use crate::content::{ActionKind, DialogueId, MAX_ITEM_LEVEL, TradeOffer};
use crate::magic::describe::{self, LineKind};
use crate::object::{self, Stack};
use crate::rng::Rng;
use crate::world::{ActorId, Spent, World};
use serde::{Deserialize, Serialize};

/// What a trader offers: the item while it is for sale, and its price.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wares {
    /// The item; `None` once it is sold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<Stack>,
    /// What it costs, in the shops' currency.
    pub price: u32,
}

impl World {
    /// The offer of creature `id`'s dialogue `dialogue`, if any.
    fn offer_of(&self, dialogue: DialogueId) -> Option<&TradeOffer> {
        self.content.dialogue(dialogue).offer.as_ref()
    }

    /// Makes creature `id`'s wares if its dialogue `dialogue` has an offer and it has none yet
    /// (the module docs).
    pub(super) fn stock_wares(&mut self, id: ActorId, dialogue: DialogueId) {
        if self.state.wares.contains_key(&id) {
            return;
        }
        let Some(offer) = self.offer_of(dialogue).cloned() else {
            return;
        };
        let mut rng = Rng::stream(self.state.seed, &format!("wares/{}", id.0));
        let unique = offer.unique_chance > 0 && rng.below(10_000) < offer.unique_chance;
        let left: Vec<&String> = (offer.uniques.iter())
            .filter(|u| !self.state.uniques_offered.contains(*u))
            .collect();
        let (item, price) = match unique && !left.is_empty() {
            true => {
                let pick = left[rng.below(left.len() as u32) as usize].clone();
                let def = (self.content.unique(&pick)).expect("an offer names known uniques");
                let item = def.stack.clone();
                self.state.uniques_offered.insert(pick);
                (Some(item), offer.unique_price)
            }
            false => {
                let level = (self.player_level().saturating_add(offer.level_bonus))
                    .clamp(offer.level_least, MAX_ITEM_LEVEL);
                let item =
                    crate::magic::loot::favoured(&self.content, &mut rng, level, &offer.affixes);
                let price = item.as_ref().map_or(0, |s| self.favoured_price(&offer, s));
                (item, price)
            }
        };
        let item = item.map(|mut s| {
            s.identify();
            s
        });
        self.state.wares.insert(id, Wares { item, price });
    }

    /// What a favoured item costs: its value plus the offer's price per affix per item level.
    fn favoured_price(&self, offer: &TradeOffer, stack: &Stack) -> u32 {
        let (affixes, level) = stack
            .magic()
            .map_or((0, 0), |m| (m.affixes.len() as u64, u64::from(m.level)));
        let price = stack.value(&self.content) + u64::from(offer.affix_price) * affixes * level;
        u32::try_from(price).unwrap_or(u32::MAX)
    }

    /// The topic of creature `id`'s offer (its dialogue's `dialogue`), after the services: with
    /// its price and the item in words while it is for sale.
    pub(super) fn offer_topic(
        &self,
        id: ActorId,
        dialogue: DialogueId,
    ) -> Option<crate::world::TopicView> {
        let offer = self.offer_of(dialogue)?;
        let wares = self.state.wares.get(&id);
        let item = wares.and_then(|w| w.item.as_ref());
        Some(crate::world::TopicView {
            id: offer.id.clone(),
            name: offer.name.clone(),
            price: item.and(wares.map(|w| w.price)),
            items: None,
            ware: item.map(|s| self.ware_text(s)),
        })
    }

    /// An offered item in words as the player knows it: its name with its numbers, then what
    /// its magic does and its power ("a ring ... (+1 hit). Mystic: +12 to maximum mana. ...").
    fn ware_text(&self, stack: &Stack) -> String {
        let lore = self.lore();
        let magic = (describe::lines(stack, &self.content).into_iter())
            .filter(|l| matches!(l.kind, LineKind::Affix(_) | LineKind::Power))
            .map(|l| l.text);
        std::iter::once(crate::text::capitalized(&lore.listed(stack)))
            .chain(magic)
            .collect::<Vec<_>>()
            .join(". ")
    }

    /// Asks creature `id`, whose dialogue is `dialogue`, for its offer `topic`: buys it (the
    /// module docs). `None` when it offers nothing by that id.
    pub(super) fn ask_offer(
        &mut self,
        id: ActorId,
        dialogue: DialogueId,
        topic: &str,
        events: &mut Vec<Event>,
    ) -> Option<Spent> {
        let offer = self.offer_of(dialogue).filter(|o| o.id == topic)?.clone();
        self.stock_wares(id, dialogue);
        let wares = self.state.wares[&id].clone();
        let Some(item) = wares.item else {
            self.answer(id, offer.sold, Some(topic.to_string()), events);
            return Some(Spent::Nothing);
        };
        let currency = self.content.shops().map(|s| s.currency);
        let gold = currency.map_or(0, |c| count_of(&self.state.player().inventory, c));
        if gold < wares.price {
            self.say("You cannot pay.");
            return Some(Spent::Nothing);
        }
        let player = self.state.player;
        self.hand_over(player, id, wares.price);
        let who = self.called(&self.state.actors[&id]);
        let what = self.lore().describe(&item);
        self.say(format!("You pay {who} {} gold for {what}.", wares.price));
        let content = self.content.clone();
        object::add_to_inventory(self.state.inventory_mut(), item, &content);
        if let Some(w) = self.state.wares.get_mut(&id) {
            w.item = None;
        }
        self.train_deal();
        self.answer(id, offer.reply, Some(topic.to_string()), events);
        Some(Spent::Turn(self.price_of(player, ActionKind::Chat)))
    }
}
