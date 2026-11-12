# Mining

**Dig through a wall in a dungeon, and you may break out ore, stone, or a gem.** Only if you are
trained in the [Mining](skills.md#skill-mining) skill. Untrained, a wall is just a wall. Every
[dwarf](races.md#race-dwarf) and [duergar](races.md#race-duergar) knows Mining, and so do
[kobolds](races.md#race-kobold).

Mining happens only in the dungeons: the generated floors under the hills, the endless stair,
the Pits. A town's walls, a hall's, or a rock on the [overworld](overworld.md) give nothing. A
wall an [explosion](digging.md#explosions) breaks gives nothing either. You have to
[dig](digging.md#digging) it yourself.

## Does the wall hold anything?

When the wall crumbles, there is a chance it held something. Harder rock holds more often, but
what it holds is harder to break out:

{{#include generated/mining-walls.md}}

If it did, you make a check: two twelve-sided dice, plus your [Strength](stats.md) modifier,
plus what Mining gives (+8 at 100, +12 at 200), against the difficulty. Snake eyes (a natural 2)
always fails, two twelves always succeed. Fail, and you find nothing, and the game says nothing:
you never knew it was there.

Succeed, and what you find lies on the rubble: "You break out a lump of iron ore." Ore and stone
come one lump at a time, one more for every 6 you beat the difficulty by, at most 3. A gem
comes alone.

## What comes out

What you find depends on how [dangerous](dungeons.md) the floor is. The precious metals and gems
are rare near the surface and grow common deeper down. **Mithril** and **adamantium** are found
only very deep: from danger 60 and 80, deeper than any dungeon but the endless stair of
Grundlēas goes.

Each find's share of what you break out, where a dungeon's rock holds every ore:

{{#include generated/mining-finds.md}}

A gem comes out uncut, its kind by how rare it is, its size larger the deeper you are, as
[gems](gems.md) are found anywhere. [Cut](gem-cutting.md) it yourself, or sell it as it is.

## Each dungeon's ores

Not every dungeon holds every metal. A fen holds only bog iron; the fey's own country holds no
iron at all. Where a dungeon names its ores, the others never come out there. Stone and gems
come out everywhere.

{{#include generated/mining-dungeons.md}}

## From ore to ingot

A lump of ore is no use to a smith as it is. **Smelt** it at an anvil: Metalcraft's *Smelt*
recipe, for anyone trained in [Metalcraft](skills.md#skill-metalcraft), turns one lump into one
ingot of its metal in ten minutes. The harder metals are harder to smelt, and a failed smelt
loses the lump. The ingot is ready for the forge (see [Crafting](crafting.md)).
