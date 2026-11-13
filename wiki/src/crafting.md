# Crafting

Three [skills](skills.md) are crafts that work materials: [Metalcraft](skills.md#skill-metalcraft),
[Woodcraft](skills.md#skill-woodcraft), and [Jewelcraft](skills.md#skill-jewelcraft). Each works
some [materials](materials.md#materials), and its checks use one [stat](stats.md). Everyone can
take things apart for their materials: **salvage**. A smith trained in Metalcraft can also forge
weapons, armour, and tools at an anvil from metal **stock**: ingots of steel, iron, mithril, and the rest, which you
can buy, find, or get by salvaging. Someone trained in Woodcraft can make knives, spears, clubs,
staves, bows, crossbows, slings, a lute, and a pipe by hand, with a knife, from billets of wood,
shards of bone and horn, hides, sinew, and blocks of flint or stone, and armour, shields, and
clothes from hides. The recipes are on the [Metalcraft…](abilities.md#ability-metalcraft) and
[Woodcraft…](abilities.md#ability-woodcraft) lists in the
[abilities menu](abilities.md#the-abilities-menu) (see [Professions](skill-uses.md#professions)).
An alchemist brews potions from herbs: see [Brewing potions](recipes-alchemy.md). An enchanter
binds magic from crystals into what they carry: see [Enchanting](enchanting.md).
A recipe counts a potion, flask, or scroll you carry only once you know what it is
([unknown potions](potions.md#unknown-potions), [unknown scrolls](scrolls.md#unknown-scrolls)):
an unknown one is no use to it.
One trained in Jewelcraft can cut loose gems and set them (see [Gem cutting](gem-cutting.md)),
and make rings and amulets (see [Making jewellery](#making-jewellery)) on the
[Jewelcraft…](abilities.md#ability-jewelcraft) list.

## Who works what

Each material group is worked by one skill, with one stat:

{{#include generated/crafting-groups.md}}

Flint, stone, and obsidian are the exceptions: they are knapped by hand, so
[Woodcraft](skills.md#skill-woodcraft) works them, with Dexterity, not Jewelcraft.

Cloth and glass have no craft yet. Nothing made of them can be saved.

Some materials are harder to work than others. A material's **work** is added to the difficulty
of every check that works it:

{{#include generated/crafting-work.md}}

Each craft skill gives **craft** points to its own checks: +8 at 100, +12 at 200 (see
[what skills give](skills.md#what-skills-give)). They count only for the materials that skill
works. Metalcraft never helps you save wood, even if you know both.

## Forging a weapon

Stand beside an [anvil](terrain.md#terrain-anvil), open
[Metalcraft…](abilities.md#ability-metalcraft), and choose a weapon. You then choose, one list at
a time:

1. **The metal**: the stacks of ingots you carry that the weapon may be made of, each with how many
   it takes. A stack with too few is greyed out "too few".
2. **The haft**, for a spear or a javelin: billets of wood. A smith needs wood for these too, so
   the recipe is greyed out "no wood to hand" without it.
3. **A gem for the ornament** (a pommel, an inlay), if the recipe has one: a cut gem you carry, or
   **None** (press `-`). An uncut gem cannot be set.

Esc goes back a step and keeps what you chose there. After the last choice you are asked to
confirm, with the weapon named as it will be made: "Forge a ruby-pommelled mithril short sword? It
uses 2 mithril ingots and the ruby, at the anvil, and takes about 1 hour."

The weapon takes the metal you chose, so its name, weight, worth, and blows are that metal's: a
mithril blade is lighter. A set gem adds its worth. The weapon you make is
[identified](magic-items.md#unidentified-items): you know your own work.

Forging takes the recipe's time. Anything that would stop a [rest](resting.md#how-resting-works)
stops it; then nothing is spent but the time. Every finished piece of work may train the craft's
skill, [Metalcraft](skills.md#skill-metalcraft) or [Woodcraft](skills.md#skill-woodcraft),
whatever came of it.

## Making a weapon by hand

Some weapons need no forge. With [Woodcraft](skills.md#skill-woodcraft) and a knife in your pack,
you can make them anywhere: open [Woodcraft…](abilities.md#ability-woodcraft) and choose one. The
lists are the same as at the anvil: the stock for each part (a knife's flint or bone, a spear's
head and then its haft of wood), then a gem if the weapon has an ornament. Without a knife the
recipes are greyed out "no knife".

What you make depends on the recipe, not the material. A flint knife is Woodcraft's, made with
Dexterity; a steel knife is Metalcraft's, forged with Strength. A smith's spear still takes a
wooden haft, but it is a Metalcraft recipe all the same.

## Bows, crossbows, and instruments

[Bows, crossbows, and slings](objects.md#class-missile-weapons) are made by hand too, never at an
anvil. A bow is carved from wood,
horn, or dragonbone and strung with a length of **sinew** (a [piece](materials.md#pieces) of it); a composite bow takes two. Sinew is not
a part of the bow: it is used up, like the stock. After the bow's material you choose the sinew to
use. A composite bow of horn is the true one; one of wood is a cheaper copy, named plain
"composite bow".

A crossbow is made of wood, but its lock and trigger take one ingot of any metal. After the wood
you choose which ingot to use. Without sinew or an ingot the recipe is greyed out "no sinew to
hand" or "no metal to hand".

A lute (strung with sinew) and a long-stemmed pipe are made the same way. They have nothing to
improve, so they always come out plain, however well the work goes.

Some weapons are never made by hand. A battle axe or a great axe of stone would shatter, a
stiletto is a slim steel point, and a sword of dragonbone is only ever found.

## Making armour and clothes by hand

[Woodcraft](skills.md#skill-woodcraft) also works hides. With a knife in your pack you can make
shields, a leather jerkin, studded leather, scale mail of bone, caps and helms, cloaks, gloves,
boots, and plain clothes and shoes, anywhere. Choose the stock for the main part, as for a weapon:
a shield may be of wood, hide, chitin, or dragonscale; scale mail of bone or dragonscale.

A **hide** weighs 1.4 kg, so it goes much further than a billet: a small shield takes 13 billets
of wood but only 2 hides. A leather jerkin takes 3 hides, a fur cloak 2 fur pelts, and a pair of
shoes 1 hide. The tables below give every count.

Studded leather and hobnailed boots also take one ingot of any metal for the studs or nails. You
choose which ingot after the stock; it is used up and is no part of what you make.

Armour, shields, helmets, cloaks, gloves, and boots come out
[Fine, Marvellous, or a Masterwork](#how-well-it-comes-out) like a weapon. Clothes and shoes have
nothing to improve, so they always come out plain.

Some things are never made yet: the padded coat, the wool cloak, and anything else of cloth or
wool wait for a craft that weaves; a reed kilt is only bought or found.

## Forging armour and tools

At the [anvil](terrain.md#terrain-anvil) a smith also forges [armour](combat.md#defence): mail and
plate, metal shields and helmets, gauntlets, and iron-shod boots. The smith forges tools too: a
[lantern](lantern.md) of iron or brass, a [pick-axe](digging.md#digging), and a mattock. You choose
the metal and, for a buckler, a tower shield, or a helm, a gem, just as for a weapon. The leather
of a boot, a lantern's glass, and a pick's haft cost nothing.

Metal is priced **by weight**, so mail is costly. A chain mail shirt weighs 11 kg and takes 22
ingots and two hours. A suit of plate weighs 22 kg and takes 44 ingots and four hours, and it is
listed only from Metalcraft 60. A [salvaged](#salvage) suit gives back half: a chain shirt gives
11 ingots.

A stone pick or mattock needs no forge. It is a digger's first tool, knapped by hand from blocks
of stone or flint with [Woodcraft](skills.md#skill-woodcraft) and a knife, like the
[hand-made weapons](#making-a-weapon-by-hand).

Some things are never made. Candles are bought, since nobody works wax. Rope is cloth, which no
craft works yet. Keys only open the locks they were made for. Household goods can only be
salvaged.

## Making jewellery

[Jewelcraft](skills.md#skill-jewelcraft) makes rings and amulets anywhere, with no tools, using
Dexterity. Open [Jewelcraft…](abilities.md#ability-jewelcraft) and choose one. The lists work as
at the anvil: first the stock, then a gem if the piece takes one.

- **Rings** are each of their own metal: an iron ring takes iron, a gold ring gold. A ring is
  light, but it still takes a whole ingot. You may set a cut gem in it as you make it, or choose
  None and set one later with [Set](gem-cutting.md#setting-a-gem).
- **Amulets**: a jade amulet takes a block of jade, an amber amulet a lump of amber (its cord
  costs nothing). A gold amulet takes a gold ingot and a cut gem for its setting, which it cannot
  do without: without a cut gem it is greyed out "no gem to hand". The gem names it: "a gold and
  sapphire amulet".
- **A bone amulet** is carved by hand, so it is [Woodcraft](skills.md#skill-woodcraft)'s, with a
  knife in your pack. It takes a shard of bone, horn, ivory, or dragonbone; its cord costs
  nothing.

Rings and amulets are never Fine or better: they come out plain, however well you do. A failure
gives back half the stock (at least one piece) and keeps the gem.

## The recipes

You know a recipe by your skill in its craft, never by learning it: each is listed from a skill
value, and greyed out "beyond your skill" below it. Only those trained in the craft see these
recipes.

### Jewelcraft

{{#include generated/recipes-jewelcraft.md}}

### Metalcraft

{{#include generated/recipes-metalcraft.md}}

### Woodcraft

{{#include generated/recipes-woodcraft.md}}

## How well it comes out

When the time is up you make one check: 2d12, plus the craft's stat modifier
([Strength](stats.md#stat-strength) for Metalcraft, [Dexterity](stats.md#stat-dexterity) for
Woodcraft), what the craft's skill gives (+8 at 100, +12 at 200), and your [talents](talents.md)
for checks of that stat, against the recipe's difficulty plus the main material's work. How far you beat it
(the **margin**) sets the grade:

{{#include generated/crafting-grades.md}}

A **Fine**, **Marvellous**, or **Masterwork** weapon is exactly a weapon the anvil has worked once,
twice, or three times (see [Forges](altars-forges-springs.md#forges)): each grade is one more
improvement to one of its numbers, and the forge counts as that many attempts. So you can temper
a Fine weapon further, but each attempt is as hard as on any once-tempered weapon, and a
Masterwork can be made no finer. Armour improves in its [Defence](combat.md#defence) or
[Protection](combat.md#protection). Something with no numbers to improve, like a lantern or a
pick-axe, always comes out plain. A failure makes nothing: you break the work up and get half the
stock back, and keep the gem.

## Batches: arrowheads, arrows, and stones

Some recipes make a **batch** of many: [arrowheads](#recipe-arrowheads) by the score at an anvil,
[arrows](#recipe-arrows) and [crossbow bolts](#recipe-bolts) (see
[Fletching](skill-uses.md#fletching)), [sling stones](#recipe-sling-stones) chipped from a block
of stone (no knife needed), and [flaming arrows](#recipe-flaming-arrows), arrows wrapped in rag
and soaked in a flask of oil. A batch has no grade. Your check sets how many you get:

- beat the difficulty, and you get the whole count;
- fall up to 3 short, half of it, rounded up;
- fall up to 6 short, one;
- fall further, none: "You spoil the whole batch."

A natural 2 always spoils the batch; a natural 24 always makes the whole count.

An **arrowhead** is the one thing a smith makes for someone else: one ingot of any metal makes
twenty, of that metal. A fletcher puts one on each arrow, and the arrow takes its metal. Bone and
flint need no smith: a shard or a block heads a whole batch of arrows by itself.

A batch only uses what it makes. One billet of wood is enough for up to 11 arrows or 4 bolts; a
batch makes no more than your billets and arrowheads cover. Flaming arrows take one arrow each, so
four arrows make at most four. If the batch is spoiled you keep your arrowheads and arrows, and
get half of the wood or the metal back (a single billet or ingot always comes back whole).

## Salvage

**[Salvage](abilities.md#ability-salvage)** is in everyone's
[abilities menu](abilities.md#the-abilities-menu), after the skills' own uses. Choose something
you carry. You need no tools and can do it anywhere. It takes ten minutes. If something comes
into view, you stop, the item is untouched, and the time is lost.

When the time is up, you make one check for each material in the item that a craft works, using
that material's skill. A ruby-tipped hardwood quarterstaff takes two checks: Woodcraft for the
hardwood and Jewelcraft for the ruby. You make each check even if you are not trained in the
skill. Your stat's [modifier](stats.md#modifiers) is then all you add.

- **Made:** the material comes back as pieces, half its weight, and at least one piece. A
  [gem](gems.md) comes back whole, keeping its [size and cut](gems.md#size-and-cut).
- **Failed:** the material is lost. A gem only loses a step: its cut drops one (down to crude),
  and if it cannot, its size drops one (down to tiny). A natural 2 shatters it. So does a failed
  check on a tiny gem with no cut left to lose.

The item itself is gone either way, with any improvements from a
[forge](altars-forges-springs.md#forges). From a stack, only one is taken apart. Materials that no
craft works, like a cloth wrapping, give nothing.

Its [magic](magic-items.md) can be caught. Before the material checks, a magic or rare item
makes one more check for each of its affixes, on [Willpower](stats.md#stat-willpower) and
[Enchanting](skills.md#skill-enchanting). Each one made leaves a
[magic crystal](crystals.md) of the affix's colour. A deep rare item may also leave a rare
material. This works on an [unidentified](magic-items.md#unidentified-items) item too, but the
messages do not name its affixes. [Magic crystals](crystals.md#catching-crystals-in-salvage) has the
numbers.

You cannot salvage:

- something you wear or wield (take it off first),
- something [protected](materials.md#protected-items): a quest's item, a
  [unique](magic-items.md#unique-items) item, or an item of Proof,
- goods you have not paid for in a [shop](shops.md#buying),
- a weapon of spirit,
- coins (they are never melted down),
- something with nothing a craft works, like a scroll or food.

Each skill you rolled earns a [training mark](skills.md#getting-better) if you are trained in it,
whatever came of the check. So does Enchanting, if any affix was checked.

The hardwood quarterstaff with its ruby gives 5 hardwood billets when the Woodcraft check is
made, and the ruby when the Jewelcraft check is made.

{{#include generated/salvage.md}}

## The craftsperson

A [craftsperson](classes.md#craftsperson) works their trades better than anyone, and their class
abilities change the rules above.

### Waste not

From level 6 a craftsperson has **Waste not**: nothing is lost in their hands but time.

- **Salvage** gives back three quarters of each material's weight instead of half (still at least
  one piece). The hardwood quarterstaff above gives 8 billets instead of 5. A gem's fate is
  unchanged: a failed check still chips it, and a natural 2 still shatters it.
- **A failed recipe** gives back all of its stock and inputs instead of half. A failed batch gives
  back all its wood and metal. A failed [brew](recipes-alchemy.md#how-a-brew-goes) is still spent on its murky draught.
- **A failed temper** at [the anvil](altars-forges-springs.md#forges), or a failed
  [whittle](skill-uses.md#whittling), damages nothing, even on a natural 2: the item stays as it
  was, and a damaged item is never destroyed. The attempt still counts, so the next one is as hard
  as it would have been.

### Quick hands

From level 40 a craftsperson has **Quick hands**: every craft takes half its time. That is every
[recipe](#the-recipes) (an [enchanting](enchanting.md) too), a temper at [the anvil](altars-forges-springs.md#forges), a
[whittle](skill-uses.md#whittling), [cutting and setting a gem](gem-cutting.md),
[salvage](#salvage), [Make do and mend](classes.md#make-do-and-mend), and
[Field forge](classes.md#field-forge). A long sword takes half as long to forge, and salvage five
minutes instead of ten. A ranger's [Fletcher](talents.md#talent-fletcher) halves arrows too, and
the two together make them in a quarter of the time. Gathering herbs, foraging, scribing, and
deciphering are not crafts and take as long as ever. The times the game shows are the halved
ones.

### Master's hand

From level 50 a craftsperson has **Master's hand**: when a check is made, **4** is added to its
margin before the grade or the cut is read. A failed check still fails, and a natural 2 still
fails or shatters the gem. So a [Fine](#how-well-it-comes-out) piece comes from a margin of 2,
Marvellous from 8, and a Masterwork from 16, and every [cut](gem-cutting.md#how-well-it-comes-out)
comes 4 sooner. The grade tables above and the cut tables under
[Gem cutting](gem-cutting.md#how-well-it-comes-out) have a "With Master's hand" column. Setting
a gem, batches, and brews have no grade and are unchanged.
