# Enchanting

An enchanter binds magic into things. Trained in [Enchanting](skills.md#skill-enchanting), you
have its recipes on the [Enchanting…](abilities.md#ability-enchanting) list in the
[abilities menu](abilities.md#the-abilities-menu). You can work anywhere: no anvil, no tools, an
hour's quiet work with the thing in your lap. The magic comes from
[magic crystals](crystals.md), and the colour of a crystal decides what kind of magic it can
give.

## The recipes

- **Kindle** makes a plain item magic, with one [affix](affixes.md#prefixes) (a prefix or a
  [suffix](affixes.md#suffixes)). It takes one crystal.
- **Quicken** gives a magic item of one affix the other kind: a suffix beside its prefix, or a
  prefix beside its suffix. It takes one crystal, and the item must have room for one more
  (its [capacity](magic-items.md#capacity)). It does not work on a rare item.
- **Unbind** takes every affix off a magic or rare item. It is plain again, and you get nothing
  back. It takes no crystal.

Each recipe works one item you carry. You must take it off first if you wear or wield it, and
you must know what its magic is ([identify](magic-items.md#unidentified-items) it first). A
[unique item](magic-items.md#unique-items) or something a quest asks for will not take your
will. Arrows, bolts, and other [ammunition](ammunition.md) are worked a whole stack at once, for
one crystal.

{{#include generated/recipes-enchanting.md}}

## The colour decides the magic

A crystal gives only magic of its colour's families (the [colours table](crystals.md#the-colours)
lists them). A red crystal never gives mana, a blue one never gives life. If nothing of that
colour can be added to the item (it already has every kind the colour gives, or the item cannot
take them), the work is refused before it starts: "No more of that nature will take."

## How strong the magic is

The affix is rolled as [loot](magic-items.md#finding-them) is, at an item level: the greater of
the item's own and your **reach**. Your reach grows with Enchanting, from item level 1 to 12
(the table above). So a beginner kindles weak magic, and a master rolls the strongest tiers. A
deep find you quicken keeps its own item level if it is higher than your reach. The item takes
the roll's item level.

## The check

When the work is done you make one check: 2d12 plus your Willpower
[modifier](stats.md#modifiers) plus Enchanting's craft bonus, against the recipe's difficulty
plus one for every three item levels of the roll (Unbind: of the item's own item level). A
natural 2 always fails and a natural 24 always succeeds. If you are interrupted, you put down
your work and nothing is spent.

- **Made**: the item is enchanted and you know its magic. "The long sword takes the red
  crystal's glow. It is now a Sharp long sword."
- **Failed**: the crystal goes dark and is spent. The item is as it was. A failed Unbind leaves
  the magic where it was.
- **A natural 2**: the crystal is spent and the item **warps**: it is
  [damaged](materials.md#item-damage), as a failed temper damages it, and an item already damaged
  falls apart. An item of Proof is spared.

Every finished work, made or not, can [train](skills.md#getting-better) Enchanting.
