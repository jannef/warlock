# Shops

Some rooms are shops: a room full of goods, with a shopkeeper standing by the door. A **general
store** sells a bit of everything, and a **specialty shop** sells one kind of thing. Every
shopkeeper has a name, and every shop has a sign: "Ælfric's General Store".

Cēaptūn, the market town of the [Ascmearc](ascmearc.md#the-places), has two shops on its
square:

- Ælfric's General Store sells everything but arms and armour: food for the road, angelica root,
  candles, rope, a pick-axe, potions, scrolls, and cloaks.
- Eadric's Smithy sells weapons, bows and slings, arrows and bolts, armour, helms, shields,
  gloves, and boots. The smith's [anvil](altars-forges-springs.md#forges) stands in its corner.

These two kinds of shop are found only in the town. The table below lists them as the town store
and the smithy.

[Dungeons](dungeons.md) have a shop now and then. About one level in six has one, in a room with
a single door. It is never in the caverns and never in the room you arrive in. Each dungeon has
its own odds, kinds of shop, and keepers (see the table under [Special
rooms](special-rooms.md#by-dungeon)).

## Money

[Gold](objects.md#class-coins) coins (`$`) are the money. Every [object](objects.md#what-an-object-is)
is worth some gold, given in the "Worth" column of the [objects table](objects.md#the-objects).
A gem, or an item set with one, is also worth what the gem is worth (see [Gems](gems.md)). Junk
such as rocks is worth nothing, and nobody buys or sells it. Prices are the only numbers a shop
tells you.

## Buying

Step into a shop and its keeper greets you:

```
Welcome to Ælfric's General Store!
```

Everything lying on a shop's floor is for sale while its keeper trades with you. [Look
mode](controls-prompts.md#look-mode), and the message when you step onto something, give the price: "a
dagger (for sale, 45 gold)".

[Picking something up](objects.md#picking-up) does not buy it. It adds the price to what you
owe ("You pick up a dagger (unpaid, 45 gold)."), and your [pack](objects.md#your-pack) marks it
"(unpaid, 45 gold)".

Pay with `p`. Paying takes no time. It pays for everything you owe from the gold you carry, and
then the goods are yours. If you do not have enough gold, put something back by dropping it in
the shop.

What you use up, you have bought. If you [eat](food.md#eating) food you have not paid for, you
still owe for it. If you eat food lying on a shop's floor, its price goes on what you owe:

```
Ælfric says: "You eat it, you buy it: 47 gold for the angelica root."
```

You cannot put back what you have eaten. Pay for it, or sell the shopkeeper something first.

## Leaving

A shopkeeper does not let you out of the door while you owe money. Step toward the door while
you owe anything and a small window asks first:

- If you have the gold: `You owe Ælfric 47 gold.` Choose **Pay 47 gold and leave** (`y`) to pay
  and step out in one move. Choose **Don't pay** (`n`, or `Esc`) and Ælfric steps into your
  way: "Ælfric steps into your way: you haven't paid." You are pushed back one step into the
  shop.
- If you do not have the gold, the window says what you owe and how much you have. **Go back**
  is the only answer, and you are pushed back the same way. Put something back, or bring more
  gold.

Being turned back takes as long as a step. If you are holding a movement key, you stop at the
shop's edge instead of being asked.

## Selling

Drop something in a shop, and if the shop buys that kind of thing, its keeper makes an offer:
`Ælfric offers 15 gold for a scroll of light.` Choose **Sell for 15 gold** (`y`) to be paid
from the keeper's purse. The thing stays on the shop floor, for sale. Choose **Keep it** (`n`
or `Esc`) to keep it. A shop has no use for what it does not buy, and you cannot leave such
things lying there. A shop pays only part of what a thing is worth, and always less than it
asks for it.

An item worked at an [anvil](altars-forges-springs.md#forges) is worth more for each
improvement, and half as much while it is damaged.

Rotten food is worth nothing (see [Food](food.md#food-spoils)). A keeper sniffs at it and will
not buy it at any price. Food that rots on a shop's floor is no longer for sale.

## Prices

Each kind of shop asks a share above an object's worth, and pays a share of the worth for what
it buys. Several things move prices:

- Your [Presence](stats.md#stat-presence) moves prices in your favour, or against you if it is
  low.
- The [Negotiation](skills.md#skill-negotiation) skill, if you are trained in it, makes you pay
  less and get offered more.
- Shopkeepers are good folk, so your [alignment](alignment.md) moves their prices too. A saint
  pays a little less, an evil character more.
- A [merchant](class-merchant.md)'s [Haggler](class-merchant.md#talent-haggler) is the one [class
  ability](classes.md#levels) that changes prices.

None of this is ever shown as a number. You see only the prices. Shops sell and buy by [item
class](objects.md#item-classes):

{{#include generated/shops.md}}

## Theft

Walking out of the door can never be theft. Leaving a shop owing anything any other way is, for
example through a wall you have [dug](digging.md#digging) open. You are asked first: pay and
leave, leave without paying, or stay. If you leave without paying, the shopkeeper shouts "Stop,
thief!", turns [hostile](creatures.md#factions), and comes after you. The goods you took are
yours, stolen.

A shopkeeper you attack turns on you the same way. So does one who catches you picking their
pocket with the [Steal](skills.md#skill-steal) skill ("Ælfric catches you stealing. \"Stop,
thief!\""; see [Stealing](skill-uses.md#stealing)). A purse lifted unnoticed leaves the shop
open, but the shopkeeper cannot pay for what you sell until they have gold again.

Shopkeepers are tough, quick, and well armed. Stealing is always your choice, and the town
stays safe unless you make it otherwise. An angry shopkeeper trades no more.

While you owe for something, you cannot [shoot or throw](ranged.md#shooting-and-throwing) it.

## A dead shopkeeper

When a shopkeeper dies, the shop is abandoned. What lies on its floor is free for the taking,
and so is the keeper's purse.

See also: [People and talk](npcs.md), [Objects and your pack](objects.md), and
[the merchant](class-merchant.md).
