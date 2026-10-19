# Shops

As in ADOM, some rooms are shops: a room full of goods, and a shopkeeper standing by the door. A
**general store** sells a bit of everything; **specialty shops** sell one kind of thing. Every
shopkeeper has a name, and every shop a sign: "Grimbold's General Store".

You find a shop in the walled courtyard up the stairs from the hall, Grimbold's General Store,
and now and then one in the Depths: about one level in seven has one, in a room with a single
door (never in the caverns, never in the room you arrive in).

## Money

Gold coins (`$`) are the money. Every object is worth some gold (the "Worth" column of the
[objects table](objects.md#the-objects)); junk such as rocks is worth nothing, and nobody buys or
sells it. Prices are the only numbers a shop tells you.

## Buying

Step into a shop and its keeper greets you:

```
Welcome to Grimbold's General Store!
```

Everything lying on a shop's floor is for sale while its keeper trades with you.
[Look mode](controls.md#look-mode) and the message as you step onto it say the price:
"a dagger (for sale, 45 gold)". Picking something up does not buy it: it adds the price to what
you owe ("You pick up a dagger (unpaid, 45 gold)."), and your pack marks it "(unpaid, 45 gold)".

Pay with `p`. Paying takes no time; it pays for everything you owe, from the gold you carry, and
your goods are yours. If you do not have enough gold, put something back: drop it in the shop.

If you try to leave owing anything, you are asked first: `You owe Grimbold 45 gold. Pay before
you leave?` Answer `y` to pay; then you are free to go. Answer `n`, or if you cannot pay, you are
asked whether to leave without paying (see Theft, below); `n` or `Esc` there keeps you in the
shop. If you hold a movement key, you stop at the shop's edge instead of being asked.

## Selling

Drop something in a shop, and if the shop buys that kind of thing, its keeper makes an offer:
`Grimbold offers 15 gold for a scroll of light. Sell?` Answer `y` and you are paid from the
keeper's purse; the thing stays on the shop floor, for sale. Answer `n` and you keep it. A shop
has no use for what it does not buy, and you cannot leave it lying there. A shop pays only part
of what a thing is worth, and always less than it asks for it.

## Prices

Each kind of shop asks a part of an object's worth above it, and pays a part of it for what it
buys. Your **Presence** moves prices in your favour, or against you if it is low, and so does the
**Appraise** skill if you are trained in it: you pay less and are offered more. Neither is ever
shown as a number; you see only the prices.

{{#include generated/shops.md}}

## Theft

Leaving a shop owing anything is theft. The shopkeeper shouts "Stop, thief!", turns hostile, and
comes after you, and the goods you took are yours, stolen. A shopkeeper you attack turns on you
the same way. Shopkeepers are tough, quick, and well armed: stealing is always your choice, and
the courtyard stays safe unless you make it. An angry shopkeeper trades no more.

While you owe for something, you cannot shoot or throw it.

## A dead shopkeeper

When a shopkeeper dies, its shop is abandoned: what lies on its floor is free for the taking, and
so is its purse.
