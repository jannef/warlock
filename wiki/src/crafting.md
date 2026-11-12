# Crafting

A smith trained in [Metalcraft](skills.md#skill-metalcraft) can forge weapons at an anvil from
metal **stock**: ingots of steel, iron, mithril, and the rest, which you can buy, find, or get by
breaking things up. The recipes are on the [Metalcraft…](abilities.md#ability-metalcraft) list in
the [abilities menu](abilities.md#the-abilities-menu) (see [Professions](skill-uses.md#professions)).

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
