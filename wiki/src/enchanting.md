# Enchanting

An enchanter binds magic into things. Trained in [Enchanting](skills.md#skill-enchanting), you
have its recipes on the [Enchanting…](abilities.md#ability-enchanting) list in the
[abilities menu](abilities.md#the-abilities-menu). You can work anywhere: no anvil, no tools, an
hour's quiet work with the thing in your lap. The magic comes from
[magic crystals](crystals.md), and the colour of a crystal decides what kind of magic it can
give.

## Enchanting an item

- **Kindle** makes a plain item magic, with one [affix](affixes.md#prefixes) (a prefix or a
  [suffix](affixes.md#suffixes)). It takes one crystal.
- **Quicken** gives a magic item of one affix the other kind: a suffix beside its prefix, or a
  prefix beside its suffix. It takes one crystal, and the item must have room for one more
  (its [capacity](magic-items.md#capacity)). On a rare item it adds one affix more, of either
  kind, if the item has room, and it takes a [heart-crystal](#the-rare-materials) too.
- **Recast** rolls a magic item's affixes anew: as many as it had, of the same kinds (a prefix
  for its prefix, a suffix for its suffix), all of the crystal's colour. It takes one crystal.
  It does not work on a rare item: unbind it, then imbue it. With a
  [soul-fetter](#the-rare-materials) you keep one affix you choose, and only the rest are rolled.
- **Imbue** makes a plain item [rare](magic-items.md#rarity), with a name and three to six
  affixes, as many as a found rare item rolls but never more than the item's
  [capacity](magic-items.md#capacity). It takes **three crystals**, and the order you choose them
  in matters: the first affix is of the first crystal's colour, the second of the second's, the
  third of the third's, and any more of any of the three. The item must hold three affixes at
  least ("It would not hold so much.").
- **Ennoble** makes a magic item rare: one affix more, of the crystal's colour, and a name. It
  takes one crystal, and the item must hold three affixes at least and have room for one more.
- **Unbind** takes every affix off a magic or rare item. It is plain again, and you get nothing
  back. It takes no crystal.

Each recipe works one item you carry. You must take it off first if you wear or wield it, and
you must know what its magic is ([identify](magic-items.md#unidentified-items) it first). A
[unique item](magic-items.md#unique-items) or something a quest asks for will not take your
will. Arrows, bolts, and other [ammunition](ammunition.md) are worked a whole stack at once, for
the same crystals as one item. Imbue's three crystals may all come from one stack.

{{#include generated/recipes-enchanting.md}}

## Which magic a crystal gives

A crystal gives only magic of its colour's families (the [colours table](crystals.md#the-colours)
lists them). A red crystal never gives mana, a blue one never gives life. If nothing of that
colour can be added to the item (it already has every kind the colour gives, or the item cannot
take them), the work is refused before it starts: "No more of that nature will take."

## Keeping magic out with seals

A **seal** keeps one family of magic out of the roll. After you choose the crystals, the work
asks for up to three seals: a list of the families your crystal's colour gives, in words
("fire", "the critical", "mana"). Choose one by its letter, or None (`-`) for no more seals.
The rest of the roll is drawn as before, only never from a sealed family. So a red crystal
sealed against fire and the critical can still give damage, strength, or life, but never fire
or the critical.

Each seal costs **one more crystal of the same colour**, taken from the crystals you chose (for
Imbue, from the first of its three whose colour has that family), and makes the work longer. A
seal holds for that one work only: it leaves nothing on the item. You cannot seal a family the
crystal's colour does not give ("The red crystal holds nothing of mana."), the same family
twice, or a family the item already has or could never take ("There is nothing to seal there.").

{{#include generated/enchanting-seals.md}}

## The rare materials

Two [rare materials](crystals.md#rare-materials) go further than any crystal:

- A **heart-crystal** lets Quicken add an affix to a rare item, one at a time, as long as the
  item has room for it. Each Quicken on a rare item takes one heart-crystal beside its crystal.
- A **soul-fetter** lets Recast keep one affix exactly as it is. The work asks which affix to
  keep after the seals (only when you carry a soul-fetter); the other affixes are rolled anew.

Both are spent whether the work takes or not, as the crystals are. The list above says where
they come from, and how often.

## How strong the magic is

The affix is rolled as [loot](magic-items.md#finding-them) is, at an item level: the greater of
the item's own and your **reach** (a plain item's, Kindle's and Imbue's, is your reach). Your
reach grows with Enchanting, from item level 1 to 12 (the table above). So a beginner kindles
weak magic, and a master rolls the strongest tiers. A deep find you quicken, recast, or ennoble
keeps its own item level if it is higher than your reach. The item takes the roll's item level.

## The check

When the work is done you make one check: 2d12 plus your Willpower
[modifier](stats.md#modifiers) plus Enchanting's craft bonus, against the recipe's difficulty
plus one for every three item levels of the roll (Unbind: of the item's own item level). A
natural 2 always fails and a natural 24 always succeeds. If you are interrupted, you put down
your work and nothing is spent.

- **Made**: the item is enchanted and you know its magic. "The long sword takes the red
  crystal's glow. It is now a Sharp long sword." An imbued or ennobled item wakes with its
  name: "The long sword drinks the three crystals and wakes as Grim Bite."
- **Failed**: the crystals go dark and are spent, with any heart-crystal or soul-fetter. The
  item is as it was. A failed Unbind leaves the magic where it was.
- **A natural 2**: the crystals are spent and the item **warps**: it is
  [damaged](materials.md#item-damage), as a failed temper damages it, and an item already damaged
  falls apart. An item of Proof is spared.

Every finished work, made or not, can [train](skills.md#improving-a-skill) Enchanting.

See also: [Magic crystals](crystals.md), [Magic items](magic-items.md), and
[Prefixes and suffixes](affixes.md).
