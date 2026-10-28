# Shops

As in ADOM, some rooms are shops: a room full of goods, and a shopkeeper standing by the door. A
**general store** sells a bit of everything; **specialty shops** sell one kind of thing. Every
shopkeeper has a name, and every shop a sign: "Ælfric's General Store".

Cēaptūn, the market town of the Ascmearc, has four on its square: Ælfric's General Store,
Eadric's Forge (weapons and missiles), Beornwyn's Armoury, and Cyneburg's Provisions (food for
the road, candles, rope, and tools). Dungeons have one now and then: about one level in six has one, in a room with a single
door (never in the caverns, never in the room you arrive in). Each dungeon has its own odds,
kinds of shop, and keepers (the table under [Special rooms](special-rooms.md#by-dungeon)).

## Money

Gold coins (`$`) are the money. Every object is worth some gold (the "Worth" column of the
[objects table](objects.md#the-objects)), and a gem, or an item set with one, by what the gem is
worth (see [Gems](gems.md)); junk such as rocks is worth nothing, and nobody buys or sells it. Prices are the only numbers a shop tells you.

## Buying

Step into a shop and its keeper greets you:

```
Welcome to Ælfric's General Store!
```

Everything lying on a shop's floor is for sale while its keeper trades with you.
[Look mode](controls.md#look-mode) and the message as you step onto it say the price:
"a dagger (for sale, 45 gold)". Picking something up does not buy it: it adds the price to what
you owe ("You pick up a dagger (unpaid, 45 gold)."), and your pack marks it "(unpaid, 45 gold)".

Pay with `p`. Paying takes no time; it pays for everything you owe, from the gold you carry, and
your goods are yours. If you do not have enough gold, put something back: drop it in the shop.

What you use up, you have bought. Eat food you have not paid for and you still owe for it; eat
food lying on a shop's floor and its price goes on what you owe:

```
Ælfric says: "You eat it, you buy it: 47 gold for the angelica root."
```

You cannot put back what you have eaten: pay for it, or sell the shopkeeper something first.

## Leaving

A shopkeeper does not let you out of the door owing money. Step toward the door while you owe
anything and a small window asks first:

- If you have the gold: `You owe Ælfric 47 gold.` Choose **Pay 47 gold and leave** (`y`) to
  pay and step out at once, in one move. Choose **Don't pay** (`n`, or `Esc`) and Ælfric steps
  into your way: "Ælfric steps into your way: you haven't paid." You are pushed back one step
  into the shop.
- If you do not: it says what you owe and how much you have, and **Go back** is the only answer:
  you are pushed back the same way. Put something back, or bring more gold.

Being turned back takes as long as a step. If you hold a movement key, you stop at the shop's
edge instead of being asked.

A merchant's **Credit** (and an aristocrat's **Commanding**, see
[Classes](classes.md#levels)) lets you walk out owing up to 100 (50) gold: the keeper nods
("you can pay next time"), and the debt stays on their bill until you come back and pay.

## Selling

Drop something in a shop, and if the shop buys that kind of thing, its keeper makes an offer:
`Ælfric offers 15 gold for a scroll of light.` Choose **Sell for 15 gold** (`y`) and you are
paid from the keeper's purse; the thing stays on the shop floor, for sale. Choose **Keep it**
(`n` or `Esc`) and you keep it. A shop
has no use for what it does not buy, and you cannot leave it lying there. A shop pays only part
of what a thing is worth, and always less than it asks for it.

Rotten food is worth nothing (see [Food](food.md#food-spoils)): a keeper sniffs at it and will
not buy it at any price, and what rots on a shop's floor is no longer for sale.

## Prices

Each kind of shop asks a part of an object's worth above it, and pays a part of it for what it
buys. Your **Presence** moves prices in your favour, or against you if it is low, and so does the
**Negotiation** skill if you are trained in it: you pay less and are offered more. Shopkeepers are
good folk, so your [alignment](alignment.md) moves their prices too: a saint pays a little less,
an evil character more. Some class abilities do too (Well dressed, Golden tongue),
and a merchant's Magnate and Golden tongue let prices move further than anyone else's; Insured
makes shops pay half again as much (never more than they ask). So do the [talents](talents.md)
Silver tongue and Haggler. None of it is ever shown as a
number; you see only the prices. Shops sell and buy by [item
class](objects.md#item-classes):

{{#include generated/shops.md}}

## Theft

Walking out of the door can never steal, but leaving a shop owing anything any other way is
theft: through a wall you have dug open, say. You are asked first (pay and leave, leave without
paying, or stay). Leave without paying and the shopkeeper shouts "Stop, thief!", turns hostile,
and comes after you, and the goods you took are yours, stolen. A shopkeeper you attack turns on you
the same way, and so does one who catches you picking their pocket with the Steal skill ("Ælfric
catches you stealing. \"Stop, thief!\""; see [Stealing](skills.md#stealing)). A purse lifted
unnoticed leaves the shop open, though the shopkeeper cannot pay for what you sell until they
have gold again. Shopkeepers are tough, quick, and well armed: stealing is always your choice,
and the town stays safe unless you make it. An angry shopkeeper trades no more.

While you owe for something, you cannot shoot or throw it.

## A dead shopkeeper

When a shopkeeper dies, its shop is abandoned: what lies on its floor is free for the taking, and
so is its purse.
