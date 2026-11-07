# Materials

Every object is made of one or more **parts**, each of one material: most are of one material
throughout (a steel dagger), some of several (a long sword's steel blade and leather grip, a gold
amulet's chain and the ruby in its setting). The first part is the main one: it is the material
the object is mostly made of, and the one the table above names first.

You can always see what something is made of, even a magic item you have not identified. An
item's name shows its **gems and precious metals** (the precious materials below) that it does
not mention already: a gold amulet set with a ruby is a *gold and ruby amulet*, and a long sword
with a ruby pommel is a *ruby-pommelled long sword*. Plain parts, like a leather grip, show
only when you look at the item (see [Magic items](magic-items.md#looking-at-an-item)), which lists
every part.

**Gems** are materials too, each with a size and a cut (see [Gems](gems.md)): a loose gem is made
of its gem alone, and an item found set with a gem has it as one of its parts, which adds the
gem's worth to the item's.

Every other material has its properties on one scale, steel's: its **weight** (how heavy an item
of it is against the same item of steel: metals by their real densities, the rest by what an
item of them weighs), what it does as a weapon's blade or head (**[to hit](combat.md#hitting)** and **[damage](combat.md#damage)**),
how much of an armour's [Protection](combat.md#protection) an armour of it keeps (**armour**), and its **worth**. A
material with no weapon figures cannot be a weapon's working part (no leather sword), one with
no armour figure cannot be armour (no flint mail). Materials come in **groups**: metal, wood,
bone (horn, ivory, chitin, and dragonbone with it), stone, hide, cloth, glass, and resin (amber).

{{#include generated/materials.md}}

## Corrosion

Some [creatures](creatures.md) carry acid. The [hezrou](creatures.md#creature-hezrou) and the [dretch](creatures.md#creature-dretch) eat at what you wear: each of their
blows that lands has a chance (25% for the hezrou, 10% for the dretch) to **damage** one piece of
armour or a shield you wear, drawn at random among those whose material corrodes. The [ember
beetle](creatures.md#creature-ember-beetle)'s blood eats at what strikes it: each of your blows that lands has a 25% chance to damage the
weapon (or the shield) it struck with. The [rust monster](creatures.md#creature-rust-monster) and the
[black pudding](creatures.md#creature-black-pudding) do both, each way at its own chance (50% for the
rust monster, 25% for the pudding). A damaged item loses its improvements and is a point worse
until it is mended, the same state as a failed attempt at an [anvil](altars-forges-springs.md#forges), where it is mended; acid
never damages an item twice and never destroys one.

The **Corrodes** column above says which materials acid eats: yes (the common metals, leather,
fur, and cloth), half (bone, horn, ivory, and chitin: at half the chance), or no (gold, silver,
electrum, chromium, nicrosil, mithril, adamantium, dragonscale, dragonbone, wood, stone, glass,
and the gems). Any [resistance](damage-types.md) to acid halves the chance. If nothing you wear (or the weapon
that struck) corrodes, nothing happens, and the first time you are told so: *The acid runs off
your mithril chain mail shirt.* Fighting bare-handed or with a spell risks nothing.

## Items of other materials

Most items can be found made of another material than their usual one: *a bronze long sword*,
*a duralumin chain mail shirt*, *a flint spear*, *a dragonbone quarterstaff*, *a silk cloak*.
What an item may be made of is its **construction list**, below, by group or by material; loot
and creatures never make anything else, so there are no leather swords or wooden chain mail.
Rings and amulets may be made of any material of a group (never paper or wax). Rocks, food,
[potions](potions.md), [scrolls](scrolls.md), coins, [gems](gems.md), and [keys](doors.md#keys) are only ever what they are.

Against its usual material, an item of another one:

- **weighs** more or less: each of its parts weighs its share of the item's weight times its
  material's weight over the usual one's (a bronze long sword's blade is 112% as heavy, its
  leather grip the same);
- **strikes** better or worse: [to hit](combat.md#hitting) and [damage](combat.md#damage) change by the difference between the two
  materials' weapon figures (a bronze long sword: +0 to hit, −1 damage against steel's), for
  every blow, throw, and shot made with it, [arrows and bolts](ammunition.md) by their heads;
- **protects** more or less: armour keeps its [Protection](combat.md#protection) times its material's armour figure
  over the usual one's, rounded to the nearest (an iron chain mail shirt's Protection 3 is 1
  in tin, 4 in chromium);
- is **worth** more or less, by its parts' shares and their materials' worth, as every item of
  other parts (a gold long sword is worth about nineteen steel ones);
- is drawn in its material's **colour** (a ring set with a gem keeps its gem's colour).

Rings and amulets change only in worth and colour. An item's name says its material when it is
not the usual one, after any gem and before the item: *a Sharp emerald-pommelled bronze long
sword of the Bear*; a material the name already says gives way (*an iron helm* of bone is *a
bone helm*, *a pair of iron-shod boots* of bronze *a pair of bronze-shod boots*). Like every
material, it shows at once, identified or not; its [damage](combat.md#damage) and [Protection](combat.md#protection) show when you [look at
the item](magic-items.md#looking-at-an-item), its to-hit does not.

{{#include generated/construction.md}}

**How often.** Loot draws an item's material from its list by how often each material is
found at its depth: primitive metals (copper and bronze, then iron and tin) are the common ones
near the surface, and steel becomes the usual metal only deeper. What a [creature](creatures.md) carries, and
so drops, is drawn by the creature's level instead: [kobolds](creatures.md#creature-kobold)' gear is mostly copper, bronze,
iron, bone, flint, and hide, and steel comes only with tougher creatures. The weights are
relative among the materials an item may be made of (its usual one too), and a material worse
in a fight than the item's usual one but dearer (gold, silver, aluminium on a sword; ivory on a
dagger) is found at most at weight 10, very rare, whatever its own weight says; rings and
amulets are worth only, so that never applies to them. Chromium, nicrosil, dragonbone, and
dragonscale appear only deep, mithril only from depth 20 and adamantium from depth 30 (on
creatures, from levels 30 and 40). Mithril is half the weight of steel and quicker in the hand
(+1 to hit); adamantium is a little heavier, bites deeper (+1 damage), and armour of it keeps
130% of its Protection. Neither is a precious metal, and acid runs off both. An [anvil](altars-forges-springs.md#forges) works
them as it does steel. Items placed by hand in the world's made places are what they are
placed as.

{{#include generated/material-finds.md}}

A long sword and a chain mail shirt in each metal:

{{#include generated/variants.md}}

## Pieces

Nothing lies about as raw material but **pieces**: what a broken thing falls to (an animated
thing that does not fall whole, see [Animated objects](animated.md)). A piece is a piece of one
material (*a piece of tin*, *3 pieces of silk*), shown as `,` in its material's colour, weighs
what 100 g of steel would in that material, and is worth what a piece of steel (1 gold) would,
by the material's worth (rounded: a piece of tin is worth nothing, one of gold 20 [gold](shops.md#money)).
Pieces of one material stack; they are miscellaneous goods, which no [shop](shops.md#selling) buys yet. A broken thing falls as the parts it was made of: a [gem](gems.md) set in it falls loose (its [size
and cut](gems.md#size-and-cut) kept), each other part as pieces of its material, half the part's weight in pieces
(one at least; parts of one material counted together), and paper, wax, and the like as
nothing. Its [magic](magic-items.md) is lost. A tin dagger with a ruby pommel breaks into 2 pieces of tin and
the ruby; a steel long sword into 8 pieces of steel and 3 of leather.

{{#include generated/pieces.md}}
