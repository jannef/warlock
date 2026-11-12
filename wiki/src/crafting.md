# Crafting

Three [skills](skills.md) are crafts that work materials: [Metalcraft](skills.md#skill-metalcraft),
[Woodcraft](skills.md#skill-woodcraft), and [Jewelcraft](skills.md#skill-jewelcraft). Each works
some [materials](materials.md#materials), and its checks use one [stat](stats.md). Everyone can
take things apart for their materials: **salvage**. A smith trained in Metalcraft can also forge
weapons at an anvil from metal **stock**: ingots of steel, iron, mithril, and the rest, which you
can buy, find, or get by salvaging. The recipes are on the
[Metalcraft…](abilities.md#ability-metalcraft) list in the
[abilities menu](abilities.md#the-abilities-menu) (see [Professions](skill-uses.md#professions)).
One trained in Jewelcraft can cut loose gems: see [Gem cutting](gem-cutting.md).

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
2. **A gem for the pommel**, if the weapon has one: a cut gem you carry, or **None** (press `-`).
   An uncut gem cannot be set.

Esc goes back a step and keeps what you chose there. After the last choice you are asked to
confirm, with the weapon named as it will be made: "Forge a ruby-pommelled mithril short sword? It
uses 2 mithril ingots and the ruby, at the anvil, and takes about 1 hour."

The weapon takes the metal you chose, so its name, weight, worth, and blows are that metal's: a
mithril blade is lighter. A set gem adds its worth. The weapon you make is
[identified](magic-items.md#unidentified-items): you know your own work.

Forging takes the recipe's time. Anything that would stop a [rest](resting.md#how-resting-works)
stops it; then nothing is spent but the time. Every finished piece of work may train
[Metalcraft](skills.md#skill-metalcraft), whatever came of it.

## The recipes

You know a recipe by your [Metalcraft](skills.md#skill-metalcraft), never by learning it: each is
listed from a skill value, and greyed out "beyond your skill" below it.

{{#include generated/recipes.md}}

## How well it comes out

When the time is up you make one check: 2d12, plus your [Strength](stats.md#stat-strength)
modifier, what [Metalcraft](skills.md#skill-metalcraft) gives (+8 at 100, +12 at 200), and your
[talents](talents.md) for Strength checks, against the recipe's difficulty. How far you beat it
(the **margin**) sets the grade:

{{#include generated/crafting-grades.md}}

A **Fine**, **Marvellous**, or **Masterwork** weapon is exactly a weapon the anvil has worked once,
twice, or three times (see [Forges](altars-forges-springs.md#forges)): each grade is one more
improvement to one of its numbers, and the forge counts as that many attempts. So you can temper
a Fine weapon further, but each attempt is as hard as on any once-tempered weapon, and a
Masterwork can be made no finer. A failure makes nothing: you break the blade up and get half the
metal back, and keep the gem.

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

The item itself is gone either way, with its [magic](magic-items.md) and any improvements from a
[forge](altars-forges-springs.md#forges). From a stack, only one is taken apart. Materials that no
craft works, like a cloth wrapping, give nothing. You learn nothing about an
[unidentified](magic-items.md#unidentified-items) item by taking it apart.

You cannot salvage:

- something you wear or wield (take it off first),
- something [protected](materials.md#protected-items): a quest's item, a
  [unique](magic-items.md#unique-items) item, or an item of Proof,
- goods you have not paid for in a [shop](shops.md#buying),
- a weapon of spirit,
- coins (they are never melted down),
- something with nothing a craft works, like a scroll or food.

Each skill you rolled earns a [training mark](skills.md#getting-better) if you are trained in it,
whatever came of the check.

The hardwood quarterstaff with its ruby gives 5 hardwood billets when the Woodcraft check is
made, and the ruby when the Jewelcraft check is made.

{{#include generated/salvage.md}}
