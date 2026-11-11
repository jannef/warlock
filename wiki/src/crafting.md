# Crafting

Three [skills](skills.md) are crafts that work materials: [Metalcraft](skills.md#skill-metalcraft),
[Woodcraft](skills.md#skill-woodcraft), and [Jewelcraft](skills.md#skill-jewelcraft). Each works
some [materials](materials.md#materials), and its checks use one [stat](stats.md). For now the one
craft everyone can do is taking things apart: **salvage**. Making things from the
[pieces](materials.md#pieces) it gives is still to come.

## Who works what

Each material group is worked by one skill, with one stat:

{{#include generated/crafting-groups.md}}

Cloth and glass have no craft yet. Nothing made of them can be saved.

Some materials are harder to work than others. A material's **work** is added to the difficulty
of every check that works it:

{{#include generated/crafting-work.md}}

Each craft skill gives **craft** points to its own checks: +8 at 100, +12 at 200 (see
[what skills give](skills.md#what-skills-give)). They count only for the materials that skill
works. Metalcraft never helps you save wood, even if you know both.

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
