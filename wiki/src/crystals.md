# Magic crystals

A **magic crystal** is magic caught in a stone. There are six colours, and each is a nature: red
is the blood and the fire, blue the mind and the frost, and so on. An enchanter will bind them
into arms and armour. The colour decides what kinds of magic a crystal can give.

You get crystals by [salvaging](crafting.md#salvage) magic items you have
[identified](magic-items.md#unidentified-items). No shop sells them and none lie on the floor.
Anyone can [consume](#consuming-a-crystal) one.

## The colours

Each colour comes from the creatures whose nature it is, and gives magic of its families. A
creature's colour is the first row below with a [tag](quests.md#sorts-of-creatures) it carries, so a goblin shaman (a goblin and
a caster) is blue, not green. The [creatures table](creatures.md) gives every creature's colour.

A kind of magic (an affix's family) can belong to two colours. Life is red and white, never
blue. Mana is blue, never red.

{{#include generated/crystal-colours.md}}

## Catching crystals in salvage

When you [salvage](crafting.md#salvage) an identified magic or rare item, each of its affixes
gets a check before the materials do. You make it even if you are not trained in
[Enchanting](skills.md#skill-enchanting). A made check leaves one crystal of the affix's colour:
the first colour in the table whose families hold it. A failed one leaves nothing. Deeper magic
(a higher tier) is harder to catch.

An unidentified item gives nothing: its magic goes out unread.

## Consuming a crystal

**[Consume crystal](abilities.md#ability-consume)** is in everyone's
[abilities menu](abilities.md#the-abilities-menu), under Items. Choose a crystal you carry. You
crush it and drink what is in it. This takes a turn and needs no check. It gives:

- **mana**, the same for every colour, never past your maximum,
- a [training mark](stats.md#training) in Willpower,
- a step toward evil on your [alignment](alignment.md). A crystal is what is left of a soul.

It is never refused, even when your mana is full. The rare materials are not crystals and cannot
be consumed.

## Rare materials

Two rare materials come with very powerful magic:

- the **heart-crystal**, which will add an affix to a rare item beyond its roll, and
- the **soul-fetter**, which will keep one affix while the rest are made again.

Salvaging an identified rare item of a high enough item level can leave one.

## The numbers

{{#include generated/crystal-rules.md}}
