# Terrain

Every cell of the map has a terrain. It decides whether you can walk there, how long a step takes,
how loud your steps are, whether it blocks sight and sound, and how tall it stands for
[shadows](sun.md). The overworld has its own terrain, described on [its page](overworld.md).

## Doors

A closed door (`+`) blocks movement, sight, and light, and muffles sound: you hear only what
happens close behind it. Walk into it to open it, which takes as long as opening it (6 seconds),
or press `o` and a direction. An open door (`'`) lets you, your light, and sound through. Press
`c` and a direction to close it again; you cannot close a door while something stands or lies in the doorway.
Opening and closing doors makes noise (see [Hearing](hearing.md)). Some creatures open doors too;
none closes them (see [Creatures](creatures.md)). Some doors are locked, and every closed door
can be kicked in with `k`, leaving a broken door (`,`) that stays open for good: see
[Doors](doors.md).

The game remembers a door as you last saw it: one you saw open stays drawn open until you see it
closed.

## Stairs

Stairs (`>` down, `<` up) join levels. Step onto them, then press `>` or `<` to use them. Using
stairs is a normal action (6 seconds). Each level you visit is kept as you left it, and dungeon levels are generated
on your first visit. Levels you leave and return to are exactly as they were.

## Walls

Walls block movement, sight, light, and sound. Most of them can be dug through with a pick-axe,
slowly, and a big explosion blows the softer ones apart; a broken wall becomes rubble. The edge of
a level never breaks. See [Digging and walls](digging.md).

## Water

Still water (ponds and pools) and running water (streams) are both wadeable but slow: a step into
either takes twice as long as a step on dry ground, and splashes loudly. Running water flows in a
direction you can see in the ripples, and foams where it meets a bank or rock. The current does
not push you yet. Water glints with nearby light.

Shallow water, ankle-deep over a stone floor, covers the floor of a flooded room (see
[Special rooms](special-rooms.md)). It is paler than a pool. Wading through it is slower than
walking, though quicker than wading a pool, and it splashes just as loudly.

## Ice

Below freezing, water grows ice: thin ice, ice, and thick ice, by how thick it is. You walk on it
like dry ground, a little more carefully, but it holds only so much weight: thin ice gives way
under a person, and ice may under a heavy pack. Falling through hurts a little, and climbing back
out takes a few turns. See [Temperature and ice](temperature.md).

## Plants and rock outdoors

Trees, pines, dead trees, boulders, crags (rock outcrops), and standing stones block your way,
and all but dead trees block sight too. Bushes can be pushed through but slowly. Reeds, marsh, and
rubble slow you a little. Their heights set how long their shadows are.

Anything you can neither walk on nor see through blocks sound, except a closed door, which only
muffles it: walls, solid rock, and the trees, boulders, crags, and stones above. Sound has to go
around them. Water, bushes, reeds, marsh, rubble, rocky ground, dry or tall grass, and creaking
boards make your steps louder; grass, flowers, the forest floor, dirt roads, and dusty floors
soften them. See [Hearing](hearing.md).

## All terrain

Step times below are at a normal pace, which is yours: a normal step takes 6 seconds (see
[Turns and movement](turns.md)). "Walk on it" is yes for terrain you can enter, and for a closed door, which you open by walking into it.
"Footsteps" is how loud a step onto it is: quiet, normal, loud, or very loud.

{{#include generated/terrain-local.md}}
