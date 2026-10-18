# Terrain

Every cell of the map has a terrain. It decides whether you can walk there, how long a step takes,
whether it blocks sight, and how tall it stands for [shadows](sun.md). The overworld has its own
terrain, described on [its page](overworld.md).

## Doors

A closed door (`+`) blocks movement, sight, and light. Walk into it to open it, which takes a turn,
or press `o` and a direction. An open door (`'`) lets you and your light through. Press `c` and a
direction to close it again; you cannot close a door while something stands in the doorway.

The game remembers a door as you last saw it: one you saw open stays drawn open until you see it
closed.

## Stairs

Stairs (`>` down, `<` up) join levels. Step onto them, then press `>` or `<` to use them. Using
stairs takes a turn. Each level you visit is kept as you left it, and dungeon levels are generated
on your first visit from the run's seed. Levels you leave and return to are exactly as they were.

## Water

Still water (ponds and pools) and running water (streams) are both wadeable but slow: a step into
either takes twice as long as a step on dry ground. Running water flows in a direction you can see
in the ripples, and foams where it meets a bank or rock. The current does not push you yet. Water
glints with nearby light.

## Plants and rock outdoors

Trees, pines, boulders, crags, and standing stones block your way, and the tall ones block sight
too (dead trees do not). Bushes can be pushed through but slowly. Reeds, marsh, and rubble slow you
a little. Their heights set how long their shadows are.

## All terrain

Step times below are for a 6-second turn, which is what dungeons and outdoor areas use. "Walk on
it" is yes for terrain you can enter, and for a closed door, which you open by walking into it.

{{#include generated/terrain-local.md}}
