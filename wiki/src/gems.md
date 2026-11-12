# Gems

There are ten gems in the world, from humble smokestone to diamond. A gem either lies loose
(`*`, in its own colour) or is set in an item: a [ring](objects.md#class-rings), an
[amulet](objects.md#class-amulets), a weapon's pommel, or a [helm](objects.md#class-helmets)'s
studs. Either way, every gem has a kind, a **size**, and a **cut**, and its worth comes from all
three.

## The gems

The worth given below is what a small gem of fair cut fetches. That is the plain size and cut,
which a gem's name leaves out: "a ruby". How often each gem is found is its share of the gems
that [loot](magic-items.md#finding-them) turns up.

{{#include generated/gems.md}}

## Size and cut

A gem's worth is its kind's worth times its size's factor times its cut's factor. A large ruby
of fine cut is worth 300 × 15 × 1.6 = 7200 [gold](shops.md#money). An uncut tiny garnet is worth
60 × 0.3 × 0.25, about 5.

A gem's name gives its size and cut unless they are the plain ones: "a ruby", "a medium
amethyst", "a large fine emerald", "an uncut tiny garnet" (an uncut gem says so first). Looking
at a gem (its letter in your [pack](objects.md#your-pack)) always gives both: "Gem: tiny garnet,
uncut". A loose gem weighs what its size says.

{{#include generated/gem-sizes.md}}

Uncut gems are rough stones, drawn duller than cut ones. They are the most common find. The
deeper you go, the larger and finer the gems get. The columns in the table above and the one
below show how often each size and cut is found on a [dungeon](dungeons.md)'s first level and
on its eighth.

{{#include generated/gem-cuts.md}}

Gems of the same kind, size, and cut lie together in one stack ("3 rubies"). Any difference
makes another stack.

## Cutting

One trained in [Jewelcraft](skills.md#skill-jewelcraft) can cut a loose gem, or cut it again to
a finer cut. A good cut keeps the stone's size; a poor one, a failure, and every re-cut cost a
size, and a natural 2 shatters the stone. See [Gem cutting](gem-cutting.md) for the cuts, what a
large uncut ruby can become, and the odds. A cut gem can then be set in an item that has a
place for one (see [Setting a gem](gem-cutting.md#setting-a-gem)).

## Finding gems

Gems are rare. Now and then a find is a loose gem instead of an item. That can be one of the
pieces of [loot](magic-items.md#finding-them) on a level's floor, a vault's reward (but not one
that is sure to be [magic](magic-items.md)), or what a [creature](creatures.md) drops when it
dies. [Dungeons](dungeons.md) also hide gems in their vaults and secret rooms.

## Items set with a gem

Some items are found set with a gem. [Rings](objects.md#class-rings) and
[amulets](objects.md#class-amulets) often are, and [weapons](objects.md#class-melee-weapons),
[shields](objects.md#class-shields), and [helms](objects.md#class-helmets) sometimes. The gem is
a part of the item (see [Materials](materials.md#materials)), so you see it at once, whether the
item is [identified or not](magic-items.md#unidentified-items). The item's name shows it: "a gold
and ruby ring", "an emerald-pommelled long sword", "a garnet-studded iron helm", "a
topaz-tipped quarterstaff".

On a [magic item](magic-items.md), the [prefix](affixes.md#prefixes) goes before the gem and
the [suffix](affixes.md#suffixes) after: "a Sharp emerald-pommelled long sword of the Bear".
A [rare](magic-items.md#rarity) item keeps its own name, and what it is made of follows: "Grim
Bite, an emerald-pommelled long sword". [Looking at the item](magic-items.md#looking-at-an-item)
gives the gem's size and cut: "Pommel: small emerald, fine cut".

A gem adds its worth to its item. It does nothing else yet. A [ring](objects.md#class-rings) set
with a gem is drawn in the gem's colour. Uncut gems are never set in an item.

{{#include generated/ornaments.md}}

## Selling gems

General stores buy gems (see [Shops](shops.md)), but no shop sells them. A store pays its share
of the gem's worth, which depends on its kind, size, and cut. For a gem lying on its floor, it
asks that much more.
