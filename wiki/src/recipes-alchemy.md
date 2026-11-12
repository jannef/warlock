# Brewing potions

If you are trained in [Alchemy](skills.md#skill-alchemy), you can brew
[potions](potions.md) from [herbs](herbs.md). Open
[Alchemy…](abilities.md#ability-alchemy) in the [abilities menu](abilities.md#the-abilities-menu)
and choose a potion. You need no station and no tools, and the flask is free: an alchemist always
has bottles. You can brew anywhere.

## How a brew goes

There is nothing to choose. The herbs are taken from your pack, and you are asked to confirm:
"Brew a potion of healing? It uses 2 sprigs of moonwort, and takes about 20 minutes."

A brew takes 20 minutes. Anything that would stop a [rest](resting.md#how-resting-works) stops it;
then nothing is spent but the time.

When the time is up you make one check: 2d12, plus your
[Intelligence](stats.md#stat-intelligence) [modifier](stats.md#modifiers), what Alchemy gives
(+8 at 100, +12 at 200), and your [talents](talents.md) for Intelligence checks, against the
recipe's difficulty.

- **Made:** you have the potion, and you know it for what it is, even if you had never seen one
  before: you know your own work. Potions have no grades: a brew either works or it does not.
- **Failed:** the herbs are spent and the brew goes murky. You are left with a
  [murky draught](potions.md#potion-murky-draught).
- **A natural 2** always fails, and worse: you are left with a
  [brewer's blunder](potions.md#potion-brewers-blunder), a gamble to drink.
- **A natural 24** always brews the potion.

Every finished brew may earn Alchemy a [training mark](skills.md#getting-better), whatever came
of it.

## The recipes

You know a recipe by your Alchemy, never by learning it: each is listed from a skill value, and
greyed out "beyond your skill" below it. A recipe whose herbs you lack is greyed out "not
enough" of the herb that is short.

The last three columns are the chance that a brew makes the potion, for someone of average
Intelligence with no talents (a dash: not yet listed at that value).

{{#include generated/recipes-alchemy.md}}

Some potions are never brewed. The elixirs, the gamble draughts (the draught of chance and
troll's blood), holy water, and the deeper potions (restoration, quickness, and the like) come
only from loot and shops. Oil and tar are bought.
