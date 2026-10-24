//! Shops (`assets/data/shops.toml`, spec §7 "Shops"), ADOM-like: kinds of shop (a general store
//! and specialty shops), what each sells and buys by item class ([`ItemClassId`]), its prices,
//! the shopkeeper every shop has, and how often generated levels get one.
//!
//! Prices are a stack's value (`Stack::value`: its object's `value`, made of its parts) times a
//! shop's `markup` (what it asks) or `pays` (what it offers for what it buys), in percent, moved
//! in the player's favour (or against it) by their *favour*: their Presence and their Appraise
//! skill ([`Shops::price`], [`Shops::offer`]; the favour itself is `world/shop.rs`'s). A shop
//! never offers more for something than it asks for it.
//!
//! Where a shop lies on a level is a [`ShopPlan`]: from a level file's `[[shop]]` (a hand-made
//! shop, its goods placed like any objects) or from the generator (`mapgen/shop.rs`). Its
//! keeper is spawned at its post from the `[keeper]` template, so every shopkeeper is alike.
//!
//! Without the file there are no shops, and level files may not mark any.

use super::creature::CreatureId;
use super::faction::FactionId;
use super::item_class::ItemClassId;
use super::level::{CreatureSpawn, equip_ids};
use super::object::{ObjectId, parse_stacks};
use super::{Content, ContentError, Stance};
use crate::geom::Pos;
use crate::object::Stack;
use serde::{Deserialize, Serialize};

/// Index of a kind of shop in [`Shops::defs`]; assigned in file order (in the save fingerprint).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ShopId(pub u8);

/// The most kinds of shop there can be.
pub const MAX_SHOPS: usize = 32;
/// The most stacks of goods a generated shop lays out.
pub const MAX_ITEMS: u8 = 52;
/// The most the player's favour moves a price, in percent either way.
pub const MAX_FAVOUR: i32 = 50;
/// The most favour there can be once talents lift the cap (spec §8 "Class abilities": the
/// merchant's Magnate and Golden tongue), either way.
pub const MOST_FAVOUR: i32 = 90;
/// The placeholder for the keeper's name in a shop's `title`.
pub const KEEPER: &str = "{keeper}";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShopFile {
    currency: String,
    outraged: String,
    #[serde(default)]
    presence: u8,
    #[serde(default)]
    chance: u8,
    #[serde(default)]
    names: Vec<String>,
    bundle: [u32; 2],
    keeper: KeeperSpec,
    #[serde(default)]
    shop: Vec<ShopSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeeperSpec {
    creature: String,
    #[serde(default)]
    carry: Vec<String>,
    #[serde(default)]
    wield: Vec<String>,
    #[serde(default)]
    wear: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShopSpec {
    id: String,
    name: String,
    title: String,
    stock: Vec<String>,
    #[serde(default)]
    buys: Vec<String>,
    items: [u8; 2],
    markup: u16,
    pays: u8,
}

/// A kind of shop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopDef {
    pub id: String,
    /// What it is, without an article: "general store".
    pub name: String,
    /// Its sign, with [`KEEPER`] for the keeper's name: "{keeper}'s General Store".
    pub title: String,
    /// The item classes of what generated shops of this kind lay out.
    pub stock: Vec<ItemClassId>,
    /// The item classes of what it buys.
    pub buys: Vec<ItemClassId>,
    /// How many stacks of goods a generated shop lays out, least and most.
    pub items: (u8, u8),
    /// What it asks, in percent of the value.
    pub markup: u16,
    /// What it pays for what it buys, in percent of the value.
    pub pays: u8,
}

impl ShopDef {
    /// Its sign, kept by `keeper`: "Grimbold's General Store".
    pub fn sign(&self, keeper: &str) -> String {
        self.title.replace(KEEPER, keeper)
    }
}

/// What every shopkeeper is: its creature type, and what it carries (its purse among it), wears,
/// and wields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keeper {
    pub creature: CreatureId,
    pub carry: Vec<Stack>,
    pub equipment: crate::equipment::Equipment,
}

/// Where a shop lies on a level and who keeps it: a level file's `[[shop]]`, or a generated one.
/// Its keeper is spawned at `post` with the level's other creatures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShopPlan {
    pub kind: ShopId,
    /// The keeper's name.
    pub keeper: String,
    /// The shop floor's corners, inclusive: everything lying there is for sale.
    pub min: Pos,
    pub max: Pos,
    /// Where the keeper stands, beside the way in.
    pub post: Pos,
}

impl ShopPlan {
    pub fn contains(&self, p: Pos) -> bool {
        (self.min.x..=self.max.x).contains(&p.x) && (self.min.y..=self.max.y).contains(&p.y)
    }

    /// Whether it shares a cell with `other`.
    pub fn overlaps(&self, other: &ShopPlan) -> bool {
        self.min.x <= other.max.x
            && other.min.x <= self.max.x
            && self.min.y <= other.max.y
            && other.min.y <= self.max.y
    }
}

/// The shops of `data/shops.toml`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shops {
    /// What shops are paid in: gold coins.
    pub currency: ObjectId,
    /// The faction a robbed or attacked shopkeeper joins: hostile to the player.
    pub outraged: FactionId,
    /// Percent better (or worse) prices for each point of the Presence modifier.
    pub presence: u8,
    /// Percent of generated rooms-and-corridors levels with a shop.
    pub chance: u8,
    /// Shopkeepers' names for generated shops.
    pub names: Vec<String>,
    /// How many of a stackable object lie in one stack of generated stock, least and most.
    pub bundle: (u32, u32),
    pub keeper: Keeper,
    defs: Vec<ShopDef>,
}

impl Shops {
    /// Every kind of shop, in file order (by [`ShopId`]).
    pub fn defs(&self) -> &[ShopDef] {
        &self.defs
    }

    pub fn get(&self, id: ShopId) -> &ShopDef {
        &self.defs[usize::from(id.0)]
    }

    pub fn id(&self, name: &str) -> Option<ShopId> {
        let i = self.defs.iter().position(|d| d.id == name)?;
        Some(ShopId(u8::try_from(i).ok()?))
    }

    /// Whether generated levels get shops at all.
    pub fn generated(&self) -> bool {
        self.chance > 0 && !self.defs.is_empty() && !self.names.is_empty()
    }

    /// Whether `stack` is traded at all: it is worth something (by what it is made of: a loose
    /// gem by its gem), and it is not the currency.
    pub fn trades(&self, content: &Content, stack: &Stack) -> bool {
        stack.object != self.currency && stack.value(content) > 0
    }

    /// Whether a shop of kind `kind` stocks `object` when it is generated: it is worth
    /// something as it is made, of a class the shop stocks.
    pub fn stocks(&self, content: &Content, kind: ShopId, object: ObjectId) -> bool {
        let def = content.object(object);
        object != self.currency
            && def.value > 0
            && def.burning.is_none()
            && self.get(kind).stock.contains(&def.class)
    }

    /// Whether a shop of kind `kind` buys `stack`.
    pub fn buys(&self, content: &Content, kind: ShopId, stack: &Stack) -> bool {
        let def = content.object(stack.object);
        self.trades(content, stack)
            && def.burning.is_none()
            && self.get(kind).buys.contains(&def.class)
    }

    /// What a shop of kind `kind` asks for `stack`, in gold, from a buyer with `favour` (percent
    /// in their favour, [`MAX_FAVOUR`] at most either way): its value times the markup, less the
    /// favour, rounded up; at least 1 for anything traded, 0 for anything that is not.
    pub fn price(&self, content: &Content, kind: ShopId, stack: &Stack, favour: i32) -> u32 {
        if !self.trades(content, stack) {
            return 0;
        }
        let base = self.base(content, stack) * u64::from(self.get(kind).markup);
        let percent = u64::from((100 - favour.clamp(-MOST_FAVOUR, MOST_FAVOUR)).unsigned_abs());
        let price = (base * percent).div_ceil(10_000).max(1);
        u32::try_from(price).unwrap_or(u32::MAX)
    }

    /// What a shop of kind `kind` offers for `stack`, in gold, to a seller with `favour`: its
    /// value times what the shop pays, plus the favour, rounded down; never more than it would
    /// ask for it, and 0 for anything it does not buy.
    pub fn offer(&self, content: &Content, kind: ShopId, stack: &Stack, favour: i32) -> u32 {
        self.offer_more(content, kind, stack, favour, 0)
    }

    /// [`Shops::offer`] to a seller shops pay `more` percent more (spec §8 "Class abilities":
    /// Insured), still never more than it would ask.
    pub fn offer_more(
        &self,
        content: &Content,
        kind: ShopId,
        stack: &Stack,
        favour: i32,
        more: i32,
    ) -> u32 {
        if !self.buys(content, kind, stack) {
            return 0;
        }
        let base = self.base(content, stack) * u64::from(self.get(kind).pays);
        let percent = u64::from((100 + favour.clamp(-MOST_FAVOUR, MOST_FAVOUR)).unsigned_abs());
        let more = u64::from((100 + more).max(0).unsigned_abs());
        let offer = u32::try_from(base * percent * more / 1_000_000).unwrap_or(u32::MAX);
        offer.min(self.price(content, kind, stack, favour))
    }

    /// The value of `stack`: each one's, by what it is made of, times how many.
    fn base(&self, content: &Content, stack: &Stack) -> u64 {
        stack.value(content)
    }

    /// The keeper of a shop, to spawn at `post` with the level's creatures.
    pub fn keeper_spawn(&self, post: Pos) -> CreatureSpawn {
        CreatureSpawn {
            creature: self.keeper.creature,
            faction: None,
            alignment: None,
            pos: post,
            route: Vec::new(),
            npc: None,
            carry: self.keeper.carry.clone(),
            equipment: self.keeper.equipment.clone(),
        }
    }

    /// Parses `data/shops.toml` once objects, creatures, and factions are known.
    pub(super) fn parse(file: &str, text: &str, content: &Content) -> Result<Self, ContentError> {
        let err = |m: String| ContentError::new(file, m);
        let parsed: ShopFile = toml::from_str(text).map_err(|e| err(e.to_string()))?;
        let currency = content
            .object_id(&parsed.currency)
            .ok_or_else(|| err(format!("unknown currency object `{}`", parsed.currency)))?;
        if !content.object(currency).stackable {
            return Err(err(format!(
                "the currency `{}` must be stackable",
                parsed.currency
            )));
        }
        let factions = content.factions();
        let outraged = factions
            .id(&parsed.outraged)
            .ok_or_else(|| err(format!("unknown faction `{}`", parsed.outraged)))?;
        let player = factions.id(super::PLAYER_FACTION);
        if player.is_none_or(|p| factions.stance(outraged, p) != Stance::Hostile) {
            return Err(err(format!(
                "the outraged faction `{}` must be hostile to the player",
                parsed.outraged
            )));
        }
        if parsed.presence > 10 || parsed.chance > 100 {
            return Err(err(
                "`presence` must be 0 to 10 and `chance` a percent, 0 to 100".into(),
            ));
        }
        if parsed.names.iter().any(|n| n.trim().is_empty()) {
            return Err(err("`names` must not be empty".into()));
        }
        let [least, most] = parsed.bundle;
        if least == 0 || least > most || most > super::MAX_PLACED {
            return Err(err(format!(
                "`bundle` must be [least, most] from 1 to {}",
                super::MAX_PLACED
            )));
        }
        let keeper = keeper(&parsed.keeper, content).map_err(|m| err(format!("keeper: {m}")))?;
        if parsed.shop.len() > MAX_SHOPS {
            return Err(err(format!("at most {MAX_SHOPS} kinds of shop")));
        }
        let mut defs: Vec<ShopDef> = Vec::new();
        for spec in parsed.shop {
            let bad = |m: String| err(format!("shop `{}`: {m}", spec.id));
            if defs.iter().any(|d| d.id == spec.id) {
                return Err(err(format!("duplicate shop id `{}`", spec.id)));
            }
            if spec.name.trim().is_empty() || !spec.title.contains(KEEPER) {
                return Err(bad(format!(
                    "`name` must not be empty and `title` must name the keeper with `{KEEPER}`"
                )));
            }
            if spec.stock.is_empty() {
                return Err(bad("`stock` must name at least one class".into()));
            }
            let [least, most] = spec.items;
            if least == 0 || least > most || most > MAX_ITEMS {
                return Err(bad(format!(
                    "`items` must be [least, most] from 1 to {MAX_ITEMS}"
                )));
            }
            if !(100..=1000).contains(&spec.markup) || spec.pays > 100 {
                return Err(bad(
                    "`markup` must be 100 to 1000 and `pays` 0 to 100 (percent of the value)"
                        .into(),
                ));
            }
            let classes = |ids: &[String]| -> Result<Vec<ItemClassId>, ContentError> {
                (ids.iter())
                    .map(|c| {
                        (content.item_class_id(c)).ok_or_else(|| {
                            bad(format!("unknown item class `{c}` (data/item_classes.toml)"))
                        })
                    })
                    .collect()
            };
            let (stock, buys) = (classes(&spec.stock)?, classes(&spec.buys)?);
            defs.push(ShopDef {
                id: spec.id,
                name: spec.name,
                title: spec.title,
                stock,
                buys,
                items: (least, most),
                markup: spec.markup,
                pays: spec.pays,
            });
        }
        let shops = Self {
            currency,
            outraged,
            presence: parsed.presence,
            chance: parsed.chance,
            names: parsed.names,
            bundle: (least, most),
            keeper,
            defs,
        };
        if parsed.chance > 0 && shops.names.is_empty() {
            return Err(err(
                "generated shops (`chance`) need `names` for their keepers".into(),
            ));
        }
        for (i, def) in shops.defs.iter().enumerate() {
            let kind = ShopId(i as u8);
            let any = (0..content.object_defs().len())
                .any(|o| shops.stocks(content, kind, ObjectId(o as u16)));
            if !any {
                return Err(err(format!(
                    "shop `{}`: no object worth anything is of a class it stocks",
                    def.id
                )));
            }
        }
        Ok(shops)
    }
}

/// The shopkeeper template: a creature type (not the player), what it carries, and what of that
/// it wields and wears.
fn keeper(spec: &KeeperSpec, content: &Content) -> Result<Keeper, String> {
    let creature = content
        .creature_id(&spec.creature)
        .filter(|_| spec.creature != "player")
        .ok_or_else(|| format!("unknown creature `{}`", spec.creature))?;
    let carry = parse_stacks(&spec.carry, content).map_err(|m| format!("carry: {m}"))?;
    let equipment = equip_ids(&spec.wield, &spec.wear, &carry, content)?;
    Ok(Keeper {
        creature,
        carry,
        equipment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> Content {
        Content::builtin().expect("built-in content")
    }

    #[test]
    fn builtin_shops_load() {
        let c = content();
        let shops = c.shops().expect("shops.toml");
        let ids: Vec<&str> = shops.defs().iter().map(|d| d.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "general_store",
                "weaponsmith",
                "armourer",
                "provisioner",
                "fletcher"
            ]
        );
        assert_eq!(c.object(shops.currency).id, "gold_coin");
        assert!(shops.generated());
        let store = shops.id("general_store").expect("a general store");
        assert_eq!(
            shops.get(store).sign("Grimbold"),
            "Grimbold's General Store"
        );
        let keeper = shops.keeper_spawn(Pos::new(3, 4));
        assert_eq!(c.creature(keeper.creature).id, "shopkeeper");
        assert!(
            !keeper.equipment.is_empty(),
            "a keeper is armed and armoured"
        );
    }

    #[test]
    fn shops_trade_by_class() {
        let c = content();
        let shops = c.shops().expect("shops");
        let id = |name: &str| c.object_id(name).expect(name);
        let kind = |name: &str| shops.id(name).expect(name);
        assert!(shops.stocks(&c, kind("weaponsmith"), id("dagger")));
        assert!(!shops.stocks(&c, kind("weaponsmith"), id("chain_mail")));
        assert!(shops.buys(&c, kind("armourer"), &Stack::one(id("chain_mail"))));
        assert!(shops.stocks(&c, kind("fletcher"), id("long_bow")));
        // Nobody trades the currency, junk, or what burns.
        for name in ["gold_coin", "rock", "burning_arrow"] {
            for k in 0..shops.defs().len() {
                assert!(
                    !shops.buys(&c, ShopId(k as u8), &Stack::one(id(name))),
                    "{name}"
                );
                assert!(!shops.stocks(&c, ShopId(k as u8), id(name)), "{name}");
            }
        }
    }

    #[test]
    fn prices_follow_value_markup_and_favour() {
        let c = content();
        let shops = c.shops().expect("shops");
        let store = shops.id("general_store").expect("store");
        let dagger = Stack::one(c.object_id("dagger").expect("dagger"));
        // A dagger is worth 30: the store asks 150 % and pays 30 %.
        assert_eq!(shops.price(&c, store, &dagger, 0), 45);
        assert_eq!(shops.offer(&c, store, &dagger, 0), 9);
        // In the buyer's favour: less to pay, more offered; against them, the other way.
        assert_eq!(shops.price(&c, store, &dagger, 10), 41);
        assert_eq!(shops.offer(&c, store, &dagger, 10), 9);
        assert_eq!(shops.price(&c, store, &dagger, -10), 50);
        // The favour is at most MOST_FAVOUR either way (`world::favour` caps it at half unless
        // talents lift the cap).
        assert_eq!(
            shops.price(&c, store, &dagger, 120),
            shops.price(&c, store, &dagger, MOST_FAVOUR)
        );
        assert_eq!(shops.price(&c, store, &dagger, MAX_FAVOUR), 23);
        // A stack is priced whole, then rounded: 12 arrows worth 1 each.
        let arrows = Stack::new(c.object_id("arrow").expect("arrow"), 12);
        assert_eq!(shops.price(&c, store, &arrows, 0), 18);
        // Anything traded costs at least 1; junk is not for sale.
        let apple = Stack::one(c.object_id("apple").expect("apple"));
        assert_eq!(shops.price(&c, store, &apple, MAX_FAVOUR), 1);
        assert_eq!(shops.offer(&c, store, &apple, 0), 0);
        let rock = Stack::one(c.object_id("rock").expect("rock"));
        assert_eq!(shops.price(&c, store, &rock, 0), 0);
    }

    #[test]
    fn a_shop_never_offers_more_than_it_asks() {
        let c = content();
        let shops = c.shops().expect("shops");
        for k in 0..shops.defs().len() {
            let kind = ShopId(k as u8);
            for o in 0..c.object_defs().len() {
                let stack = Stack::new(ObjectId(o as u16), 7);
                for favour in [-MAX_FAVOUR, -10, 0, 25, MAX_FAVOUR] {
                    let (ask, offer) = (
                        shops.price(&c, kind, &stack, favour),
                        shops.offer(&c, kind, &stack, favour),
                    );
                    assert!(offer <= ask, "{kind:?} {o} {favour}: {offer} > {ask}");
                }
            }
        }
    }

    #[test]
    fn bad_shops_are_refused() {
        let c = content();
        let good = include_str!("../../../../assets/data/shops.toml");
        for (from, to, why) in [
            (
                "currency = \"gold_coin\"",
                "currency = \"dagger\"",
                "stackable",
            ),
            (
                "currency = \"gold_coin\"",
                "currency = \"mithril\"",
                "unknown currency",
            ),
            (
                "outraged = \"outraged\"",
                "outraged = \"merchants\"",
                "hostile to the player",
            ),
            ("chance = 70", "chance = 101", "`chance`"),
            ("bundle = [6, 15]", "bundle = [0, 15]", "`bundle`"),
            ("markup = 150", "markup = 50", "`markup`"),
            ("pays = 30", "pays = 101", "`pays`"),
            ("items = [6, 10]", "items = [10, 6]", "`items`"),
            (
                "title = \"{keeper}'s General Store\"",
                "title = \"General Store\"",
                "`title`",
            ),
            (
                "creature = \"shopkeeper\"",
                "creature = \"player\"",
                "unknown creature",
            ),
            (
                "wear = [\"chain_mail\", \"iron_helm\"]",
                "wear = [\"ruby\"]",
                "ruby",
            ),
            (
                "stock = [\"melee_weapons\", \"missiles\"]",
                "stock = [\"coins\"]",
                "no object worth",
            ),
            (
                "stock = [\"melee_weapons\", \"missiles\"]",
                "stock = [\"swords\"]",
                "unknown item class `swords`",
            ),
        ] {
            assert!(good.contains(from), "{from}");
            let text = good.replacen(from, to, 1);
            let e = Shops::parse("shops.toml", &text, &c).unwrap_err();
            assert!(e.message.contains(why), "{why}: {}", e.message);
        }
        let twice = format!(
            "{good}\n[[shop]]\nid = \"fletcher\"\nname = \"x\"\ntitle = \"{{keeper}}\"\n\
             stock = [\"missiles\"]\nitems = [1, 2]\nmarkup = 100\npays = 0\n"
        );
        let e = Shops::parse("shops.toml", &twice, &c).unwrap_err();
        assert!(e.message.contains("duplicate shop id"), "{}", e.message);
    }
}
