# Crafting

Three [skills](skills.md) are crafts that work materials: [Metalcraft](skills.md#skill-metalcraft),
[Woodcraft](skills.md#skill-woodcraft), and [Jewelcraft](skills.md#skill-jewelcraft). Each works
some [materials](materials.md#materials), and its checks use one [stat](stats.md). Everyone can
take things apart for their materials: **[salvage](salvage.md)**. A smith trained in Metalcraft can also [forge](forging.md)
weapons, armour, and tools at an anvil from metal **stock**: ingots of steel, iron, mithril, and the rest, which you
can buy, find, or get by salvaging. Someone trained in Woodcraft can make knives, spears, clubs,
staves, bows, crossbows, slings, a lute, and a pipe [by hand](handcraft.md), with a knife, from billets of wood,
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
and make rings and amulets (see [Making jewellery](jewellery.md)) on the
[Jewelcraft…](abilities.md#ability-jewelcraft) list.

## Which skill makes what

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

## The recipes

You know a recipe by your skill in its craft, never by learning it: each is listed from a skill
value, and greyed out "beyond your skill" below it. Only those trained in the craft see these
recipes. Each craft's recipes are listed on its page:

- [Forging at the anvil](forging.md): Metalcraft's weapons, armour, and tools.
- [Making things by hand](handcraft.md): Woodcraft's weapons, bows, armour, clothes, and
  batches of arrows and stones.
- [Making jewellery](jewellery.md): Jewelcraft's rings and amulets.
- [Salvage](salvage.md): taking things apart for their materials, which anyone can do.

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

A [craftsperson](class-craftsperson.md) works their trades better than anyone, and their class
abilities change these rules. See also: [Enchanting](enchanting.md),
[Gem cutting](gem-cutting.md), and [Mining](mining.md).
