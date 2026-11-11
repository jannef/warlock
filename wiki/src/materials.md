# Materials

Every [object](objects.md) is made of something, and what it is made of matters. A bronze sword
hits differently from a steel one, mithril mail is lighter than steel, gold is worth more than
tin, and some materials stand acid, rust, and fire better than others. This page explains how.

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

## Item damage

An item can be **damaged** in several ways, and it is always the same state. A damaged item loses
its improvements and is a point worse until it is mended, at an
[anvil](altars-forges-springs.md#forges) or, for wood, with a knife
([Whittle](abilities.md#ability-whittle)). That is the state a failed attempt at an anvil leaves it
in. An item damaged again, by anything, is **destroyed**. Only its name tells what did it: *a
corroded*, *a rusted*, *a scorched*, or *a damaged long sword*.

Some [creatures](creatures.md) harm your gear in melee:

- The [hezrou](creatures.md#creature-hezrou) and the [dretch](creatures.md#creature-dretch) carry
  acid that eats at what you wear. Each of their blows that lands has a chance (25% for the
  hezrou, 10% for the dretch) to reach one piece of armour or a shield you wear, picked at random.
- The [ember beetle](creatures.md#creature-ember-beetle)'s acid blood eats at what strikes it.
  Each of your blows that lands has a 25% chance to reach the weapon or shield you struck with.
- The [black pudding](creatures.md#creature-black-pudding)'s acid does both, each way at 25%, and
  the [rust monster](creatures.md#creature-rust-monster) rusts both ways at 50%.

Any [resistance](damage-types.md) to acid halves the chance of acid. Rust has no damage type to
resist.

What is reached then **withstands** it by its main material's chance, in the **Withstands acid**,
**Withstands rust**, and **Withstands fire** columns above:

- Withstood, nothing happens. The first time for each kind you are told so: *The acid runs off
  your mithril chain mail shirt.*
- Otherwise the item is damaged, or destroyed if it was damaged already.

Gold, silver, mithril, adamantium, dragonscale, and the gems nearly always withstand everything.
Iron and steel rust and corrode easily but stand heat. Copper and its alloys corrode but hardly
rust. The soft metals melt. Wood and cloth burn. Nothing but metal rusts. A failed attempt at an
anvil is never withstood. Fighting bare-handed or with a spell risks nothing.

### Protected items

Some items no harm reaches at all: not acid, rust, or fire, not a failed attempt at an
[anvil](altars-forges-springs.md#forges) or with the knife, and not a spell's
[ruin](spellcasting.md#what-lies-in-its-way) of what lies on the floor. They are:

- [unique items](magic-items.md#unique-items);
- items of the [of Proof](affixes.md#suffixes) affix, whether or not you have identified it;
- anything a [quest](quests.md) asks you to bring or deliver, until that quest is done for good.
  What a quest gives you as its reward is an ordinary item, unless another quest asks for it.

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

Most items are made of their usual material: a long sword is steel, a cloak wool. At depth 1, an
item that could be made of something else is its usual material 90% of the time. The chance of
another material grows by half a point with each depth, to 20% from depth 21 on. A
[creature](creatures.md)'s gear, and what it drops, goes by the creature's level instead of the
depth.

When an item is made of another material, that material is picked from the item's list by the
weights in the table further down. The weights compare only the other materials, not the usual one.
Primitive metals are the common ones near the surface (copper and bronze, then iron and tin),
and steel becomes the common one only deeper. Some materials are worse in a fight than an item's
usual one but dearer, such as gold, silver, or aluminium on a sword, or ivory on a dagger. Such
a material's weight is 10 at most, which is very rare, whatever its own weight says. Rings and
amulets are only about worth, so this never applies to them.

A material's weight also depends on how well it **suits** the item. The table gives each
material's fit for weapons (anything that strikes or is shot), armour (shields too), jewellery
(rings and amulets), and other items:

- **good**: its weight as given;
- **poor**: a tenth of it. Copper and the precious metals make poor weapons and armour, hides
  make poor weapons and jewellery, and wood makes poor armour;
- **odd**: a hundredth of it. Tin, pewter, zinc, and cadmium make odd blades. Cloth, wool, and
  fur make odd weapons and odd rings.

{{#include generated/material-finds.md}}

An odd material is never ruled out, but it is very rare: a wool ring or a tin long sword is
found well under once in a thousand of those items. How often a few items turn up in each kind
of material, at depths 1, 10, and 21:

{{#include generated/material-odds.md}}

Some [factions](creatures.md#factions) **favour** materials for the gear they carry and drop.
Each favoured material takes its share of the picks first, for any item whose list holds it.
The usual picking described above covers the rest. An orc's spear is
bone more often than not, the dead of the barrows carry bronze, and
[kobolds](creatures.md#creature-kobold) like flint, bone, and copper. A favourite never makes
an item of a material it could not otherwise be made of: no tribe carries a bone long sword.

{{#include generated/material-tribes.md}}

Chromium, nicrosil, dragonbone, and dragonscale appear only deep. Mithril appears only from
depth 20 and adamantium from depth 30 (on creatures, from levels 30 and 40).

- **Mithril** is half the weight of steel and quicker in the hand (+1 to hit).
- **Adamantium** is a little heavier than steel and bites deeper (+1 damage), and armour of it
  keeps 130% of its Protection.

Neither is a precious metal, and acid runs off both. An [anvil](altars-forges-springs.md#forges)
works them as it does steel. Items placed by hand in the world's made places are always what
they were placed as.

A long sword and a chain mail shirt in each metal:

{{#include generated/variants.md}}

## Pieces

Raw material comes in **pieces**: what a broken thing falls to (an animated thing that does not
fall whole, for example; see [Animated objects](animated.md)), and the stock smiths and other
crafters work with. A piece is one material, shown as `,` in its material's colour, and named by
its group: *a steel ingot*, *3 hardwood billets*, *a leather hide*, *a bone shard*, *a block of
jade*, *a lump of amber*, *a bolt of wool*, *a lump of glass*.

- A piece weighs what 500 g of steel would weigh in that material: a wood billet 200 g, a gold
  ingot 1230 g.
- It is worth what a steel ingot (1 gold) would be worth in that material, rounded. A tin ingot
  is worth nothing, and a gold one 20 [gold](shops.md#money).
- Pieces of one material stack. They are listed under **Materials**. A smithy and a general
  store sell them, and a smithy, a weaponsmith, an armourer, a fletcher, and a general store buy
  them (see [Shops](shops.md#selling)), unless they are worth nothing, like a wood billet. In
  Cēaptūn the smithy has steel and iron ingots.

How many pieces a thing holds goes by weight: its weight in a material divided by one piece of
it, rounded, and at least one. A dagger's steel is one ingot, a long sword's blade three, a great
sword seven, and a quarterstaff ten wood billets.

A broken thing falls apart into the parts it was made of. A [gem](gems.md) set in it falls
loose, keeping its [size and cut](gems.md#size-and-cut). Each other part becomes pieces of its
material: half the part's weight in pieces, at least one, with parts of the same material counted
together. Paper, wax, and the like leave nothing. Any [magic](magic-items.md) is lost. A tin
dagger with a ruby pommel breaks into a tin ingot and the ruby. A steel long sword breaks into
2 steel ingots and a leather hide.

Each group is worked by one skill: metal by [Metalcraft](skills.md#skill-metalcraft), with
Strength; wood, bone, and hide by [Woodcraft](skills.md#skill-woodcraft), with Dexterity; stone,
amber, and gems by [Jewelcraft](skills.md#skill-jewelcraft), with Dexterity. Cloth and glass have
no crafting skill yet. Some materials are harder to work than others: the **work** column of the
materials table is added to the difficulty of working them (steel 0, tin −3, adamantium +8).
[Salvage](crafting.md#salvage) uses both: it takes an item apart into these pieces.

{{#include generated/pieces.md}}
