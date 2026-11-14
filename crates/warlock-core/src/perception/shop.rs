//! Shops as the player knows them: their signs, keepers, and prices.

use super::{CellView, PerceptionView};
use crate::content::Content;
use crate::content::Shops;
use crate::geom::Pos;
use crate::object::Stack;
use crate::time::GameTime;
use crate::world::Shop;

/// A shop as the player knows it (spec §7 "Shops"): its sign, its keeper's name, and its prices
/// for them, in gold. Never the value, the markup, or how their Presence or skills move the
/// prices.
#[derive(Clone, Copy, Debug)]
pub struct ShopView<'a> {
    pub(super) shop: &'a Shop,
    pub(super) shops: &'a Shops,
    pub(super) content: &'a Content,
    /// The player's favour (`world::favour`), hidden: only prices come of it.
    pub(super) favour: i32,
    /// The moment it is: rotten food is worth nothing (spec §8 "Hunger"), which only the
    /// prices tell.
    pub(super) now: GameTime,
}

impl<'a> ShopView<'a> {
    /// Whether `stack` is rotten food, which the shop neither sells nor buys.
    fn spoilt(&self, stack: &Stack) -> bool {
        crate::rot::stage(self.content, stack, self.now).is_some_and(|s| s.is_rotten())
    }

    /// What `stack` costs on the bill (what the player took is priced as it was).
    pub(super) fn bill(&self, stack: &Stack) -> u32 {
        (self.shops).price(self.content, self.shop.kind, stack, self.favour)
    }

    /// Its sign: "Grimbold's General Store".
    pub fn sign(&self) -> String {
        self.shop.sign(self.content)
    }

    /// Its keeper's name.
    pub fn keeper(&self) -> &'a str {
        &self.shop.name
    }

    /// Whether `p` is on its floor.
    pub fn contains(&self, p: Pos) -> bool {
        self.shop.contains(p)
    }

    /// Whether `p` is one of its ways out (its door, in plain sight): a step out there owing
    /// anything is turned back, any other way out is theft.
    pub fn is_exit(&self, p: Pos) -> bool {
        self.shop.is_exit(p)
    }

    /// What it asks for `stack` lying on its floor, if it is for sale (worth something, not the
    /// currency, not rotten food).
    pub fn price(&self, stack: &Stack) -> Option<u32> {
        let price = if self.spoilt(stack) {
            0
        } else {
            self.bill(stack)
        };
        (price > 0).then_some(price)
    }

    /// What it offers for `stack`, if it buys it (never rotten food).
    pub fn offer(&self, stack: &Stack) -> Option<u32> {
        if self.spoilt(stack) {
            return None;
        }
        let offer = (self.shops).offer(self.content, self.shop.kind, stack, self.favour);
        (offer > 0).then_some(offer)
    }

    /// What the player owes it.
    pub fn owed(&self) -> u32 {
        self.shop.owed(self.shops, self.content, self.favour)
    }
}

/// What a shop would pay for the player's own things among the stacks they chose to drop there
/// (spec §7 "Shops"): the offer and what it is for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sale {
    pub offer: u32,
    pub stacks: Vec<Stack>,
}

impl<'a> PerceptionView<'a> {
    /// The open shop the player stands in.
    pub fn shop_here(&self) -> Option<ShopView<'a>> {
        self.shop_at(self.snapshot.viewer)
    }

    /// The price of `stack` lying at `p`, a cell in sight, if it is for sale there (look mode:
    /// "a dagger (for sale, 45 gold)").
    pub fn price_tag(&self, p: Pos, stack: &Stack) -> Option<u32> {
        if !self.snapshot.sees_actors_at(p) {
            return None;
        }
        self.shop_at(p)?.price(stack)
    }

    /// What the player owes, and to whom, if a step onto `to` leaves the open shop they stand in
    /// (onto ground they know is walkable): by a way out ([`ShopView::is_exit`]) the keeper turns
    /// them back, any other way is theft, so presentation asks first.
    pub fn owed_on_leaving(&self, to: Pos) -> Option<(ShopView<'a>, u32)> {
        let view = self.shop_here()?;
        let walkable = match self.cell(to) {
            CellView::Visible(t) | CellView::Remembered(t) => self.content.terrain(t).walkable,
            CellView::Unknown => false,
        };
        let owed = view.owed();
        (!view.contains(to) && walkable && owed > 0).then_some((view, owed))
    }
}
