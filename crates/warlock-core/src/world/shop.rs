//! Shops in play (spec §7 "Shops"; the kinds and prices are `content/shop.rs`), ADOM-like.
//!
//! A level keeps its shops ([`Shop`]): the shop floor, the keeper and where it stands, what the
//! player has picked up there and not paid for, and how the keeper stands toward them. While the
//! keeper trades ([`ShopStatus::Open`]), everything worth something lying on the shop floor
//! (but the currency) is for sale:
//!
//! - **Entering** the shop floor the keeper greets the player with the shop's sign.
//! - **Picking up** goods adds them to what the player owes (`unpaid`, counted by object and
//!   what it is made of, so a gem is priced by its gem, size, and cut), and
//!   says the price; the player **pays** with [`crate::Action::Pay`] (no time), from the gold
//!   they carry into the keeper's purse. Without enough gold they put things back by dropping
//!   them, which takes the drop's time.
//! - **Using up** goods is buying them: the bill counts by object, so unpaid goods eaten (or used
//!   up any other way) stay owed, and food eaten off the shop floor is owed for
//!   ([`World::ate_goods`]).
//! - **Dropping** in the shop puts back what is unpaid; of the player's own things it offers a
//!   price for what it buys ([`crate::Action::Drop`] says the offer and keeps them;
//!   [`crate::Action::Sell`] takes it, from the keeper's purse), and has no use for the rest.
//! - **Rotten food** (spec §8 "Hunger") is worth nothing: lying in the shop it is not for sale,
//!   and the keeper will not buy it ("Grimbold sniffs at a loaf of bread and says: \"Not at any
//!   price.\""). What is on the bill stays priced as it was.
//! - **Leaving** by the shop's way out (its `exits`: the door) owing anything, the keeper steps
//!   into the way and the player is turned back one step into the shop ([`World::turned_back`]):
//!   walking out of the door never steals. [`crate::Action::Pay`] with a direction pays and
//!   steps out at once.
//! - **Theft**: leaving the shop floor owing anything any other way (through a wall dug open) is
//!   theft: the keeper joins the outraged faction (hostile to the player) and comes after them,
//!   and so does a keeper the player attacks ([`ShopStatus::Outraged`]): it trades no more. A
//!   keeper's death leaves its goods free to take ([`ShopStatus::Abandoned`]). Unpaid goods
//!   cannot be shot or thrown in the shop.
//!
//! Prices follow the player's favour ([`favour`]): their Presence and their Appraise skill, never
//! shown but as gold.

use super::{ActorId, Deed, Spent, World};
use crate::action::{Event, Meal, Positions};
use crate::content::StatUse;
use crate::content::{
    ActionKind, Content, MAX_FAVOUR, MAX_FAVOUR_LIFT, ShopId, ShopPlan, Shops, TerrainId,
};
use crate::geom::{Dir, Pos};
use crate::grid::Grid;
use crate::object::{self, Stack};
use crate::skills::Effect;
use crate::stats::Stat;
use serde::{Deserialize, Serialize};

/// How a shop's keeper stands toward the player.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShopStatus {
    /// It trades: everything worth something lying in the shop is for sale.
    #[default]
    Open,
    /// It was robbed or attacked: hostile to the player, it trades no more.
    Outraged,
    /// It is dead: what lies in the shop is free to take.
    Abandoned,
}

impl ShopStatus {
    fn is_open(&self) -> bool {
        *self == ShopStatus::Open
    }
}

/// A shop on a level (spec §7 "Shops"), kept in the level's state (saved, hashed).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shop {
    pub kind: ShopId,
    pub keeper: ActorId,
    /// The keeper's name.
    pub name: String,
    /// The shop floor's corners, inclusive.
    pub min: Pos,
    pub max: Pos,
    /// Where the keeper stands.
    pub post: Pos,
    /// The ways out, as the level was made: the walkable cells (and doors) around the shop floor.
    /// A step onto one owing anything is turned back; leaving any other way is theft.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exits: Vec<Pos>,
    /// What the player has picked up here and not paid for, by object (one stack each).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unpaid: Vec<Stack>,
    #[serde(default, skip_serializing_if = "ShopStatus::is_open")]
    pub status: ShopStatus,
}

impl Shop {
    /// The shop `plan` lays out on `terrain`, kept by actor `keeper`. Its ways out are the cells
    /// around its floor that can be walked onto, or are doors.
    pub(crate) fn from_plan(
        plan: &ShopPlan,
        keeper: ActorId,
        terrain: &Grid<TerrainId>,
        content: &Content,
    ) -> Self {
        let (min, max) = (plan.min, plan.max);
        let exits = (min.y - 1..=max.y + 1)
            .flat_map(|y| (min.x - 1..=max.x + 1).map(move |x| Pos::new(x, y)))
            .filter(|&p| !plan.contains(p))
            .filter(|&p| {
                (terrain.get(p)).is_some_and(|&t| {
                    let def = content.terrain(t);
                    def.walkable || def.is_closed_door()
                })
            })
            .collect();
        Self {
            kind: plan.kind,
            keeper,
            name: plan.keeper.clone(),
            min,
            max,
            post: plan.post,
            exits,
            unpaid: Vec::new(),
            status: ShopStatus::Open,
        }
    }

    /// Whether `p` is one of its ways out.
    pub fn is_exit(&self, p: Pos) -> bool {
        self.exits.contains(&p)
    }

    /// Whether `p` is on the shop floor.
    pub fn contains(&self, p: Pos) -> bool {
        (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
    }

    /// Whether its keeper trades.
    pub fn is_open(&self) -> bool {
        self.status.is_open()
    }

    /// Its sign: "Grimbold's General Store".
    pub fn sign(&self, content: &Content) -> String {
        content
            .shops()
            .map_or_else(|| self.name.clone(), |s| s.get(self.kind).sign(&self.name))
    }

    /// What the player owes for what they took, in gold, at `favour`.
    pub fn owed(&self, shops: &Shops, content: &Content, favour: i32) -> u32 {
        (self.unpaid.iter())
            .map(|s| shops.price(content, self.kind, s, favour))
            .sum()
    }

    /// Whether everything is well formed for `content` (a guard for loaded saves).
    pub(crate) fn is_valid_for(&self, content: &Content, width: i32, height: i32) -> bool {
        let inside = |p: Pos| (0..width).contains(&p.x) && (0..height).contains(&p.y);
        content
            .shops()
            .is_some_and(|s| usize::from(self.kind.0) < s.defs().len())
            && inside(self.min)
            && inside(self.max)
            && self.exits.iter().all(|&p| inside(p))
            // The bill counts goods (an object and what it is made of): no stacks lying
            // anywhere, so no magic and no age.
            && self.unpaid.iter().all(|s| {
                usize::from(s.object.0) < content.object_defs().len()
                    && s.count >= 1
                    && s.magic().is_none()
                    && s.died.is_none()
                    && (s.own_parts()).is_none_or(|own| {
                        crate::item_parts::check(content.object(s.object), own, content).is_ok()
                    })
            })
    }

    /// Adds `stack` to what the player owes, counted by goods: its object and what it is made
    /// of (a magic item is priced as its base item, food whatever its age, a gem by its gem).
    fn owe(&mut self, stack: &Stack) {
        match self.unpaid.iter_mut().find(|s| s.same_goods(stack)) {
            Some(same) => same.count += stack.count,
            None => self.unpaid.push(stack.goods(stack.count)),
        }
    }

    /// Takes `stack` off what the player owes: they put it back.
    fn put_back(&mut self, stack: &Stack) {
        if let Some(same) = self.unpaid.iter_mut().find(|s| s.same_goods(stack)) {
            same.count = same.count.saturating_sub(stack.count);
        }
        self.unpaid.retain(|s| s.count > 0);
    }
}

/// `count` of `stack`: the same object, with the same magic.
pub fn part_of(stack: &Stack, count: u32) -> Stack {
    let mut part = stack.clone();
    part.count = count;
    part
}

/// How much of each stack of `inventory` is unpaid: the count of the goods `unpaid` names,
/// given to the stacks of those goods from the last one back (the latest picked up).
pub fn unpaid_parts(inventory: &[Stack], unpaid: &[Stack]) -> Vec<u32> {
    let mut parts = vec![0; inventory.len()];
    for owed in unpaid {
        let mut left = owed.count;
        for (i, stack) in inventory.iter().enumerate().rev() {
            if stack.same_goods(owed) && left > 0 {
                parts[i] = stack.count.min(left);
                left -= parts[i];
            }
        }
    }
    parts
}

/// How far prices move in `actor`'s favour, in percent (spec §7 "Shops"): the shops' `presence`
/// for each point of their Presence modifier, plus what their skills give (Appraise's
/// bargaining), plus what a good keeper makes of their alignment (spec §8 "Alignment"), plus
/// what their talents give (spec §8 "Talents": Silver tongue, Haggler), at most [`MAX_FAVOUR`]
/// either way, the most in their favour lifted by their talents (Haggler; at most
/// [`MAX_FAVOUR_LIFT`]). Never shown.
pub fn favour(content: &Content, shops: &Shops, actor: &super::Actor) -> i32 {
    let presence = actor.stats.modifier(Stat::Presence) * i32::from(shops.presence);
    let bargain = actor.skills.effect(content, Effect::Bargain);
    let alignment = super::alignment::keeper_favour(content, shops, actor);
    let talents = actor.talent_mods(content);
    let most = MAX_FAVOUR + talents.favour_cap.clamp(0, MAX_FAVOUR_LIFT);
    (presence + bargain + alignment + talents.favour).clamp(-MAX_FAVOUR, most)
}

/// How many of `object` `stacks` hold.
pub fn count_of(stacks: &[Stack], object: crate::content::ObjectId) -> u32 {
    (stacks.iter())
        .filter(|s| s.object == object)
        .map(|s| s.count)
        .sum()
}

/// Takes `amount` of `object` out of `stacks`, from the last stack back. Returns the positions
/// of the stacks that emptied and are gone (to follow with the equipment).
fn take(stacks: &mut Vec<Stack>, object: crate::content::ObjectId, amount: u32) -> Positions {
    let mut left = amount;
    for stack in stacks.iter_mut().rev() {
        if stack.object == object && left > 0 {
            let taken = stack.count.min(left);
            stack.count -= taken;
            left -= taken;
        }
    }
    let gone = Positions::of((0..stacks.len()).filter(|&i| stacks[i].count == 0));
    stacks.retain(|s| s.count > 0);
    gone
}

/// What a drop or a sale in a shop does with the stacks chosen, worked out before anything
/// changes.
#[derive(Default)]
struct Deal {
    /// Unpaid goods going back on the shop floor.
    back: Vec<Stack>,
    /// The player's own things sold, and what they fetch.
    sold: Vec<Stack>,
    payment: u32,
    /// The player's own things the shop would buy (a drop only says the offer).
    offered: Vec<Stack>,
    offer: u32,
    /// The player's own things the shop has no use for.
    refused: Vec<Stack>,
    /// The player's own food the shop would buy but for its being rotten.
    spoilt: Vec<Stack>,
    /// How many of each inventory stack leave the player's pack.
    leaving: Vec<u32>,
}

impl World {
    /// The shops of the content, which a level with shops always has.
    fn shop_kinds(&self) -> &Shops {
        self.content
            .shops()
            .expect("levels have shops only when content has data/shops.toml")
    }

    /// The open shop the player stands in on the current level: its index.
    pub(super) fn shop_here(&self) -> Option<usize> {
        let here = self.player_pos();
        (self.state.current().shops.iter()).position(|s| s.is_open() && s.contains(here))
    }

    /// The player's favour in prices now.
    fn player_favour(&self) -> i32 {
        favour(&self.content, self.shop_kinds(), self.state.player())
    }

    /// Whether `stack` is rotten food, which is worth nothing (spec §7 "Shops").
    fn spoilt(&self, stack: &Stack) -> bool {
        self.stage(stack).is_some_and(|s| s.is_rotten())
    }

    /// What `stack` lying at `p` costs, if it is for sale there: in an open shop, traded, and
    /// not rotten food.
    pub(super) fn asking_price(&self, p: Pos, stack: &Stack) -> Option<u32> {
        let shops = self.content.shops()?;
        let shop = (self.state.current().shops.iter()).find(|s| s.is_open() && s.contains(p))?;
        if self.spoilt(stack) {
            return None;
        }
        let price = shops.price(&self.content, shop.kind, stack, self.player_favour());
        (price > 0).then_some(price)
    }

    /// The player picked up `got` in the open shop they stand in: what is for sale is now unpaid.
    /// Returns the price of each stack (`None` for what is free to take: rotten food too).
    pub(super) fn picked_up_goods(&mut self, got: &[Stack]) -> Vec<Option<u32>> {
        let Some(i) = self.shop_here() else {
            return vec![None; got.len()];
        };
        let (content, favour) = (self.content.clone(), self.player_favour());
        let shops = content.shops().expect("a shop is here");
        let spoilt: Vec<bool> = got.iter().map(|s| self.spoilt(s)).collect();
        let level = self.state.current_mut();
        let shop = &mut level.shops[i];
        (got.iter().zip(spoilt))
            .map(|(stack, spoilt)| {
                let price = match spoilt {
                    true => 0,
                    false => shops.price(&content, shop.kind, stack, favour),
                };
                (price > 0).then(|| {
                    shop.owe(stack);
                    price
                })
            })
            .collect()
    }

    /// The player ate `food` (one of it), from the pack or the floor as `meal` says, in the open
    /// shop they stand in: eaten goods are bought (spec §7 "Shops"). Food that lay there for sale
    /// is now owed for; unpaid food from the pack stays owed, as the bill counts by object and
    /// nothing that uses goods up takes them off it. The keeper says so. (Rotten food lying
    /// there was worth nothing; unpaid food from the pack is priced as the bill prices it.)
    pub(super) fn ate_goods(&mut self, meal: Meal, food: &Stack) {
        let Some(i) = self.shop_here() else {
            return;
        };
        let price = match meal {
            Meal::Floor(_) => self.asking_price(self.player_pos(), food),
            Meal::Pack(_) => {
                let kind = self.state.current().shops[i].kind;
                let price =
                    (self.shop_kinds()).price(&self.content, kind, food, self.player_favour());
                (price > 0).then_some(price)
            }
        };
        let Some(price) = price else {
            return;
        };
        let what = &self.content.object(food.object).name;
        let line = match meal {
            Meal::Floor(_) => {
                self.state.current_mut().shops[i].owe(food);
                format!("You eat it, you buy it: {price} gold for the {what}.")
            }
            Meal::Pack(_) => {
                let unpaid = count_of(&self.state.current().shops[i].unpaid, food.object);
                if count_of(&self.state.player().inventory, food.object) >= unpaid {
                    return;
                }
                format!("Eaten or not, the {what} is still {price} gold.")
            }
        };
        let (name, owed) = self.owing().expect("eaten goods are owed for");
        let mut text = format!("{name} says: \"{line}\"");
        if owed != price {
            text.push_str(&format!(" You owe {name} {owed} gold."));
        }
        self.say(text);
    }

    /// Whom the player owes how much in the open shop they stand in, if anything.
    pub(super) fn owing(&self) -> Option<(String, u32)> {
        let shop = &self.state.current().shops[self.shop_here()?];
        let owed = shop.owed(self.shop_kinds(), &self.content, self.player_favour());
        (owed > 0).then(|| (shop.name.clone(), owed))
    }

    /// The player pays for what they took in the open shop they stand in (spec §7 "Shops"): all
    /// of it, from the gold they carry into the keeper's purse. Takes no time. Refused (and
    /// nothing changes) outside a shop, owing nothing, or without enough gold.
    pub(super) fn pay(&mut self) -> Spent {
        let Some(i) = self.shop_here() else {
            self.say("There is nobody here to pay.");
            return Spent::Nothing;
        };
        let content = self.content.clone();
        let shops = content.shops().expect("a shop is here");
        let shop = self.state.current().shops[i].clone();
        let owed = shop.owed(shops, &content, self.player_favour());
        if owed == 0 {
            self.say(format!("You owe {} nothing.", shop.name));
            return Spent::Nothing;
        }
        let gold = count_of(&self.state.player().inventory, shops.currency);
        if gold < owed {
            let have = match gold {
                0 => "none".to_string(),
                n => format!("only {n}"),
            };
            self.say(format!(
                "You owe {} {owed} gold, but you have {have}.",
                shop.name
            ));
            return Spent::Nothing;
        }
        self.hand_over(self.state.player, shop.keeper, owed);
        self.state.current_mut().shops[i].unpaid.clear();
        self.say(format!("You pay {} {owed} gold.", shop.name));
        self.train_stat(StatUse::Trade);
        self.player_did(Deed::PaidDebt);
        Spent::NoTime
    }

    /// Moves `amount` gold from actor `from`'s pack into actor `to`'s.
    fn hand_over(&mut self, from: ActorId, to: ActorId, amount: u32) {
        let currency = self.shop_kinds().currency;
        if let Some(giver) = self.state.actors.get_mut(&from) {
            let gone = take(&mut giver.inventory, currency, amount);
            giver.equipment.after_removal(&gone);
        }
        let content = self.content.clone();
        if let Some(taker) = self.state.actors.get_mut(&to) {
            let gold = Stack::new(currency, amount);
            object::add_to_inventory(&mut taker.inventory, gold, &content);
        }
    }

    /// Drops (`sell` false) or sells (`sell` true) the inventory stacks `positions` names in the
    /// open shop the player stands in (spec §7 "Shops"): what of them is unpaid goes back; of
    /// the player's own things, a drop says what the shop offers for what it buys and keeps
    /// them, a sale takes the offer from the keeper's purse. Returns `None` outside an open
    /// shop.
    pub(super) fn trade(&mut self, positions: &Positions, sell: bool) -> Option<Spent> {
        let i = self.shop_here()?;
        let inventory = self.state.player().inventory.clone();
        if inventory.is_empty() {
            self.say("You are not carrying anything.");
            return Some(Spent::Nothing);
        }
        let Some(chosen) = self.chosen(positions, inventory.len(), "You carry no such thing.")
        else {
            return Some(Spent::Nothing);
        };
        let content = self.content.clone();
        let shop = self.state.current().shops[i].clone();
        let deal = self.deal(&shop, &inventory, &chosen, sell);
        let name = shop.name.as_str();
        if !deal.refused.is_empty() {
            let what = object::phrase(&deal.refused, &content);
            self.say(format!("{name} has no use for {what}."));
        }
        // Rotten food is worth nothing: the keeper's nose is the only hint of it the player gets.
        if !deal.spoilt.is_empty() {
            let what = object::phrase(&deal.spoilt, &content);
            self.say(format!(
                "{name} sniffs at {what} and says: \"Not at any price.\""
            ));
        }
        if !deal.offered.is_empty() {
            let what = object::phrase(&deal.offered, &content);
            self.say(format!("{name} offers {} gold for {what}.", deal.offer));
        }
        if deal.back.is_empty() && deal.sold.is_empty() {
            return Some(Spent::Nothing);
        }
        // What leaves whole is taken off first, if worn or wielded.
        let gone =
            Positions::of((0..inventory.len()).filter(|&k| deal.leaving[k] == inventory[k].count));
        let mut kinds = self.take_off_before_dropping(gone.iter());
        kinds.push(ActionKind::Drop);
        // Priced at the load the player has as they begin.
        let spent = self.price_for(&kinds);
        let kept: Vec<Stack> = (inventory.iter().zip(&deal.leaving))
            .map(|(s, &n)| part_of(s, s.count - n))
            .filter(|s| s.count > 0)
            .collect();
        *self.state.inventory_mut() = kept;
        self.player_mut().equipment.after_removal(&gone);
        let here = self.player_pos();
        let level = self.state.current_mut();
        let mut pile = level.objects_at(here).to_vec();
        let laid: Vec<Stack> = deal.back.iter().chain(&deal.sold).cloned().collect();
        object::add_to_pile(&mut pile, &laid, &content);
        level.set_pile(here, pile);
        for stack in &deal.back {
            level.shops[i].put_back(stack);
        }
        if !deal.back.is_empty() {
            let what = object::phrase(&deal.back, &content);
            self.say(format!("You put back {what}."));
        }
        if !deal.sold.is_empty() {
            self.hand_over(shop.keeper, self.state.player, deal.payment);
            let what = object::phrase(&deal.sold, &content);
            self.say(format!(
                "You sell {what} to {name} for {} gold.",
                deal.payment
            ));
            self.train_stat(StatUse::Trade);
        }
        Some(spent)
    }

    /// What dropping (or selling) the stacks `chosen` of `inventory` in `shop` does (see
    /// [`World::trade`]). A sale the keeper's purse cannot pay for does not happen.
    fn deal(&mut self, shop: &Shop, inventory: &[Stack], chosen: &Positions, sell: bool) -> Deal {
        let content = self.content.clone();
        let shops = content.shops().expect("a shop is here");
        let favour = self.player_favour();
        let parts = unpaid_parts(inventory, &shop.unpaid);
        let mut deal = Deal {
            leaving: vec![0; inventory.len()],
            ..Deal::default()
        };
        let mut selling: Vec<usize> = Vec::new();
        for k in chosen.iter() {
            let (stack, unpaid) = (&inventory[k], parts[k]);
            if unpaid > 0 {
                deal.back.push(part_of(stack, unpaid));
                deal.leaving[k] += unpaid;
            }
            let own = part_of(stack, stack.count - unpaid);
            if own.count == 0 {
                continue;
            }
            let offer = shops.offer(&content, shop.kind, &own, favour);
            if offer == 0 {
                deal.refused.push(own);
            } else if self.spoilt(&own) {
                deal.spoilt.push(own);
            } else if sell {
                deal.sold.push(own);
                deal.payment += offer;
                selling.push(k);
            } else {
                deal.offered.push(own);
                deal.offer += offer;
            }
        }
        if deal.sold.is_empty() {
            return deal;
        }
        let purse = (self.state.actors.get(&shop.keeper))
            .map_or(0, |k| count_of(&k.inventory, shops.currency));
        if purse < deal.payment {
            let what = object::phrase(&deal.sold, &content);
            self.say(format!("{} cannot afford {what}.", shop.name));
            deal.sold.clear();
            deal.payment = 0;
            return deal;
        }
        for k in selling {
            deal.leaving[k] = inventory[k].count;
        }
        deal
    }

    /// Whether the player may shoot or throw inventory stack `item` here: not unpaid goods in the
    /// shop they stand in. Says why not.
    pub(super) fn may_loose(&mut self, item: usize) -> bool {
        let Some(i) = self.shop_here() else {
            return true;
        };
        let inventory = &self.state.player().inventory;
        let parts = unpaid_parts(inventory, &self.state.current().shops[i].unpaid);
        if parts.get(item).copied().unwrap_or(0) == 0 {
            return true;
        }
        let what = inventory[item].describe(&self.content);
        self.say(format!("You have not paid for {what}."));
        false
    }

    /// Where the player ends up trying to step from `from` onto `to`, if the keeper of the open
    /// shop they stand in turns them back (spec §7 "Shops"): `to` is one of its ways out and they
    /// owe it something. They are pushed one cell away from the way out, the other way from the
    /// step, or else onto the nearest free cell of the shop floor (the first in row order among
    /// the nearest), or else they stay where they are. Says why.
    pub(super) fn turned_back(&mut self, from: Pos, to: Pos) -> Option<Pos> {
        let shops = &self.state.current().shops;
        let shop = shops
            .iter()
            .find(|s| s.is_open() && s.contains(from) && s.is_exit(to) && !s.unpaid.is_empty())?
            .clone();
        let level = self.state.current();
        let free = |p: Pos| {
            p != from
                && shop.contains(p)
                && (level.terrain.get(p)).is_some_and(|&t| self.content.terrain(t).walkable)
                && self.state.actor_at(&level.id, p).is_none()
        };
        let back = Pos::new(2 * from.x - to.x, 2 * from.y - to.y);
        let floor = (shop.min.y..=shop.max.y)
            .flat_map(|y| (shop.min.x..=shop.max.x).map(move |x| Pos::new(x, y)));
        let land = if free(back) {
            back
        } else {
            floor
                .filter(|&p| free(p))
                .min_by_key(|p| p.dist2(from))
                .unwrap_or(from)
        };
        self.say(format!(
            "{} steps into your way: you haven't paid.",
            shop.name
        ));
        Some(land)
    }

    /// The player stepped from `from` to `to` on the current level: entering an open shop the
    /// keeper greets them; leaving one owing anything (any way but its ways out, which turn them
    /// back) is theft (spec §7 "Shops").
    pub(super) fn shop_step(&mut self, from: Pos, to: Pos) {
        let content = self.content.clone();
        for i in 0..self.state.current().shops.len() {
            let shop = &self.state.current().shops[i];
            if !shop.is_open() {
                continue;
            }
            let (was, is) = (shop.contains(from), shop.contains(to));
            if !was && is {
                self.say(format!("Welcome to {}!", shop.sign(&content)));
            } else if was && !is && !shop.unpaid.is_empty() {
                let name = shop.name.clone();
                self.outrage(i);
                self.say(format!(
                    "You leave without paying. \"Stop, thief!\" shouts {name}."
                ));
                self.player_did(Deed::Theft);
            }
        }
    }

    /// The keeper of shop `i` on the current level turns on the player: it joins the outraged
    /// faction and trades no more; what the player owed is theirs now, stolen.
    fn outrage(&mut self, i: usize) {
        let outraged = self.shop_kinds().outraged;
        let shop = &mut self.state.current_mut().shops[i];
        shop.status = ShopStatus::Outraged;
        shop.unpaid.clear();
        let keeper = shop.keeper;
        if let Some(actor) = self.state.actors.get_mut(&keeper) {
            actor.faction_override = Some(outraged);
        }
    }

    /// The player attacked actor `target`: a keeper of an open shop on the current level is
    /// outraged (said after the attack's own messages).
    pub(super) fn provoke(&mut self, target: ActorId) {
        let shops = &self.state.current().shops;
        let Some(i) = shops.iter().position(|s| s.is_open() && s.keeper == target) else {
            return;
        };
        let name = shops[i].name.clone();
        self.outrage(i);
        self.news
            .push(format!("\"You will pay for that!\" shouts {name}."));
        self.player_did(Deed::AttackedShopkeeper);
    }

    /// Actor `id` on level `level` died: a shop it kept is abandoned, its goods free to take.
    pub(super) fn keeper_died(&mut self, id: ActorId, level: &str) {
        let Some(level) = self.state.levels.get_mut(level) else {
            return;
        };
        for shop in level.shops.iter_mut().filter(|s| s.keeper == id) {
            shop.status = ShopStatus::Abandoned;
            shop.unpaid.clear();
        }
    }

    /// [`crate::action::Action::Pay`]: pays ([`World::pay`]) and, with a direction, steps that
    /// way at once. Paid, the player steps out at once; a step that fails leaves them paid.
    pub(super) fn pay_then(&mut self, then: Option<Dir>, events: &mut Vec<Event>) -> Spent {
        let paid = self.pay();
        match (paid, then) {
            (Spent::NoTime, Some(dir)) => match self.player_moves(dir, events) {
                Spent::Nothing => Spent::NoTime,
                spent => spent,
            },
            _ => paid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpaid_goods_are_the_latest_stacks_of_their_object() {
        let (a, b) = (crate::content::ObjectId(1), crate::content::ObjectId(2));
        let s = Stack::new;
        let inventory = [s(a, 1), s(b, 5), s(a, 1), s(b, 3)];
        // Two daggers, one unpaid: the later one. Six arrows of eight unpaid: the later stack
        // whole, then three of the first.
        let unpaid = [s(a, 1), s(b, 6)];
        assert_eq!(unpaid_parts(&inventory, &unpaid), [0, 3, 1, 3]);
        assert_eq!(unpaid_parts(&inventory, &[]), [0, 0, 0, 0]);
        let mut pack = inventory.to_vec();
        let gone = take(&mut pack, b, 4);
        assert_eq!(pack, [s(a, 1), s(b, 4), s(a, 1)]);
        assert_eq!(gone, Positions::of([3]));
        assert_eq!(count_of(&pack, b), 4);
    }
}
