# Magic crystals

A **magic crystal** is magic caught in a stone. There are six colours, and each is a nature: red
is the blood and the fire, blue the mind and the frost, and so on. An [enchanter](enchanting.md) binds them
into arms and armour. The colour decides what kinds of magic a crystal can give.

You get crystals by [salvaging](crafting.md#salvage) magic items, identified or not, and from
the creatures you kill with [Soulglass](#soulglass). No shop sells them and none lie on the
floor.
Anyone can [consume](#consuming-a-crystal) one.

## The colours

Each colour comes from the creatures whose nature it is, and gives magic of its families. A
creature's colour is the first row below with a [tag](quests.md#sorts-of-creatures) it carries, so a goblin shaman (a goblin and
a caster) is blue, not green. The [creatures table](creatures.md) gives every creature's colour.

A kind of magic (an affix's family) can belong to two colours. Life is red and white, never
blue. Mana is blue, never red.

{{#include generated/crystal-colours.md}}

## Catching crystals in salvage

When you [salvage](crafting.md#salvage) a magic or rare item, each of its affixes
gets a check before the materials do. You make it even if you are not trained in
[Enchanting](skills.md#skill-enchanting). A made check leaves one crystal of the affix's colour:
the first colour in the table whose families hold it. A failed one leaves nothing. Deeper magic
(a higher tier) is harder to catch.

An [unidentified](magic-items.md#unidentified-items) item gives crystals the same way. The
messages do not name its affixes ("You catch some of its magic in a red crystal."), but the
colours you get hint at what it was.

## Soulglass

[Soulglass](magic.md#spell-soulglass) is a ray of necromancy. The [witch](classes.md#witch) learns
it at level 16, and anyone can learn it from its grimoire. A living creature it kills turns to
glass: it leaves no corpse and none of the crafting stock its body would give, but crystals of its
colour lie where it fell. What it carried falls as usual.

A stronger creature leaves more crystals, and a very strong one may leave a
[rare material](#rare-materials) too (how many and how often is under [The numbers](#the-numbers)).
A creature with no corpse (a spirit, a fire elemental), an animated thing, a summoned creature,
a creature of no colour, and a [keeper of Grundlēas](food.md#corpses) die as they would from any
other spell. The undead take no
[necrotic](spellcasting.md#spell-types) harm, so Soulglass does nothing to them.

## Consuming a crystal

Crystals are in the Consume list (`e`, the same list as [food](food.md) and
[potions](potions.md)), whether they are in your pack or on the floor where you stand. Choose
one: you crush it and drink what is in it. This takes a turn and needs no check, and you need not
be hungry. It gives:

- **mana**, the same for every colour, never past your maximum,
- several [training marks](stats.md#raising-your-stats) in Willpower (how many is under
  [The numbers](#the-numbers)),
- a step toward evil on your [alignment](alignment.md). A crystal is what is left of a soul.

It is never refused, even when your mana is full. The rare materials are not crystals and cannot
be consumed.

## Rare materials

Two rare materials come with very powerful magic:

- the **heart-crystal**, which will add an affix to a rare item beyond its roll, and
- the **soul-fetter**, which will keep one affix while the rest are made again.

Salvaging a rare item of a high enough item level can leave one, and so can a powerful creature
killed by [Soulglass](#soulglass). A dragon always leaves a heart-crystal.

Morwenna, a deep gnome on the cutting floor of Gimmwīc, reads crystals and will tell you what each
colour holds, if you stop to listen (see [People](npcs.md)).

## The numbers

{{#include generated/crystal-rules.md}}

See also: [Enchanting](enchanting.md), [Crafting](crafting.md#salvage), and
[Magic items](magic-items.md).
