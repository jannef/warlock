# Materials

Every [object](objects.md) is made of something, and what it is made of matters. A bronze sword
hits differently from a steel one, mithril mail is lighter than steel, gold is worth more than
tin, and acid eats some materials but not others. This page explains how.

An object is made of one or more **parts**, each of one material. Most objects are one material
throughout, such as a steel dagger. Some have several parts: a long sword has a steel blade and
a leather grip, and a gold amulet has its chain and the ruby in its setting. The first part is
the main one: the material the object is mostly made of, and the one the [objects
table](objects.md#the-objects) names first.

You can always see what something is made of, even a magic item you have not identified. An
item's name shows its **gems and precious metals** (the precious materials in the table below)
when it does not already mention them. A gold amulet set with a ruby is a *gold and ruby
amulet*, and a long sword with a ruby pommel is a *ruby-pommelled long sword*. Plain parts, like
a leather grip, show only when you look at the item, which lists every part (see [Magic
items](magic-items.md#looking-at-an-item)).

**Gems** are materials too, each with a size and a cut (see [Gems](gems.md)). A loose gem is
made of its gem alone. An item found set with a gem has it as one of its parts, and the gem's
worth is added to the item's.

Every other material is measured against steel:

- **weight**: how heavy an item of it is compared with the same item of steel. Metals go by
  their real densities, the rest by what an item of them weighs.
- **[to hit](combat.md#hitting)** and **[damage](combat.md#damage)**: what it does as a weapon's
  blade or head.
- **armour**: how much of an armour's [Protection](combat.md#protection) an armour of it keeps.
- **worth**.

A material with no weapon figures cannot be a weapon's working part, so there is no leather
sword. One with no armour figure cannot be armour, so there is no flint mail.

Materials come in **groups**: metal, wood, bone (with horn, ivory, chitin, and dragonbone),
stone, hide, cloth, glass, and resin (amber).

{{#include generated/materials.md}}

## Corrosion

Some [creatures](creatures.md) carry acid, and acid **damages** gear:

- The [hezrou](creatures.md#creature-hezrou) and the [dretch](creatures.md#creature-dretch) eat
  at what you wear. Each of their blows that lands has a chance (25% for the hezrou, 10% for the
  dretch) to damage one piece of armour or a shield you wear, picked at random among those whose
  material corrodes.
- The [ember beetle](creatures.md#creature-ember-beetle)'s blood eats at what strikes it. Each of
  your blows that lands has a 25% chance to damage the weapon or shield you struck with.
- The [rust monster](creatures.md#creature-rust-monster) and the [black
  pudding](creatures.md#creature-black-pudding) do both, each way at its own chance: 50% for the
  rust monster, 25% for the pudding.

A damaged item loses its improvements and is a point worse until it is mended. That is the same
state a failed attempt at an [anvil](altars-forges-springs.md#forges) leaves it in, and an anvil
is also where it is mended. Acid never damages an item twice and never destroys one.

The **Corrodes** column above says which materials acid eats:

- **yes**: the common metals, leather, fur, and cloth;
- **half**: bone, horn, ivory, and chitin, at half the chance;
- **no**: gold, silver, electrum, chromium, nicrosil, mithril, adamantium, dragonscale,
  dragonbone, wood, stone, glass, and the gems.

Any [resistance](damage-types.md) to acid halves the chance. If nothing you wear (or the weapon
that struck) corrodes, nothing happens, and the first time you are told so: *The acid runs off
your mithril chain mail shirt.* Fighting bare-handed or with a spell risks nothing.

## Items of other materials

Most items can be found made of a material other than their usual one: *a bronze long sword*,
*a duralumin chain mail shirt*, *a flint spear*, *a dragonbone quarterstaff*, *a silk cloak*.
What an item may be made of is its **construction list**, shown below by group or by material.
Loot and creatures never make anything outside it, so there are no leather swords or wooden
chain mail. Rings and amulets may be made of any material of a group, but never paper or wax.
Rocks, food, [potions](potions.md), [scrolls](scrolls.md), coins, [gems](gems.md), and
[keys](doors.md#keys) are only ever what they are.

Compared with its usual material, an item of another one:

- **weighs** more or less. Each part weighs its share of the item's weight times its material's
  weight over the usual one's. A bronze long sword's blade is 112% as heavy, and its leather grip
  stays the same.
- **strikes** better or worse. [To hit](combat.md#hitting) and [damage](combat.md#damage) change
  by the difference between the two materials' weapon figures: a bronze long sword is +0 to hit
  and −1 damage against steel. This counts for every blow, throw, and shot made with it, and for
  [arrows and bolts](ammunition.md) by their heads.
- **protects** more or less. Armour keeps its [Protection](combat.md#protection) times its
  material's armour figure over the usual one's, rounded to the nearest. An iron chain mail
  shirt's Protection 3 is 1 in tin and 4 in chromium.
- is **worth** more or less, by its parts' shares and their materials' worth, like any item with
  several parts. A gold long sword is worth about nineteen steel ones.
- is drawn in its material's **colour**. A ring set with a gem keeps its gem's colour.

Rings and amulets change only in worth and colour.

An item's name gives its material when it is not the usual one, after any gem and before the
item: *a Sharp emerald-pommelled bronze long sword of the Bear*. If the name already names a
material, the new one replaces it: *an iron helm* made of bone is *a bone helm*, and *a pair of
iron-shod boots* made of bronze is *a pair of bronze-shod boots*. Like every material, it shows
at once, identified or not. The item's [damage](combat.md#damage) and
[Protection](combat.md#protection) show when you [look at the
item](magic-items.md#looking-at-an-item), but its to-hit does not.

{{#include generated/construction.md}}

### How often each material is found

Loot picks an item's material from its list by how often each material is found at that depth.
Primitive metals are the common ones near the surface (copper and bronze, then iron and tin),
and steel becomes the usual metal only deeper. What a [creature](creatures.md) carries, and so
drops, is picked by the creature's level instead. [Kobolds](creatures.md#creature-kobold)' gear
is mostly copper, bronze, iron, bone, flint, and hide, and steel comes only with tougher
creatures.

The weights in the table are relative among the materials an item may be made of, its usual one
included. Some materials are worse in a fight than an item's usual one but dearer, such as gold,
silver, or aluminium on a sword, or ivory on a dagger. Such a material is found at weight 10 at
most, which is very rare, whatever its own weight says. Rings and amulets are only about worth,
so this never applies to them.

Chromium, nicrosil, dragonbone, and dragonscale appear only deep. Mithril appears only from
depth 20 and adamantium from depth 30 (on creatures, from levels 30 and 40).

- **Mithril** is half the weight of steel and quicker in the hand (+1 to hit).
- **Adamantium** is a little heavier than steel and bites deeper (+1 damage), and armour of it
  keeps 130% of its Protection.

Neither is a precious metal, and acid runs off both. An [anvil](altars-forges-springs.md#forges)
works them as it does steel. Items placed by hand in the world's made places are always what
they were placed as.

{{#include generated/material-finds.md}}

A long sword and a chain mail shirt in each metal:

{{#include generated/variants.md}}

## Pieces

Raw material never lies about by itself, except as **pieces**: what a broken thing falls to (an
animated thing that does not fall whole, for example; see [Animated objects](animated.md)). A
piece is a piece of one material, such as *a piece of tin* or *3 pieces of silk*, shown as `,`
in its material's colour.

- A piece weighs what 100 g of steel would weigh in that material.
- It is worth what a piece of steel (1 gold) would be worth in that material, rounded. A piece of
  tin is worth nothing, and one of gold 20 [gold](shops.md#money).
- Pieces of one material stack. They are miscellaneous goods, which no [shop](shops.md#selling)
  buys yet.

A broken thing falls apart into the parts it was made of. A [gem](gems.md) set in it falls
loose, keeping its [size and cut](gems.md#size-and-cut). Each other part becomes pieces of its
material: half the part's weight in pieces, at least one, with parts of the same material counted
together. Paper, wax, and the like leave nothing. Any [magic](magic-items.md) is lost. A tin
dagger with a ruby pommel breaks into 2 pieces of tin and the ruby. A steel long sword breaks
into 8 pieces of steel and 3 of leather.

{{#include generated/pieces.md}}
