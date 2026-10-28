# Terrain

Every cell of the map has a terrain. It decides whether you can walk there, [how long a step takes](turns.md#what-actions-cost),
how [loud](hearing.md#how-loud) your steps are, whether it blocks [sight](sight.md#line-of-sight) and [sound](hearing.md), and how tall it stands for
[shadows](sun.md). The overworld has its own terrain, described on [its page](overworld.md).

## Doors

A closed door (`+`) blocks movement, [sight](sight.md#line-of-sight), and [light](light.md#what-light-is), and muffles [sound](hearing.md): you hear only what
happens close behind it. Walk into it to open it, which takes as long as opening it (6 seconds),
or press `o` and a direction. An open door (`'`) lets you, your light, and sound through. Press
`c` and a direction to close it again; you cannot close a door while something stands or lies in the doorway.
Opening and closing doors makes noise (see [Hearing](hearing.md)). Some creatures open doors too;
none closes them (see [Creatures](creatures.md)). Some doors are [locked](doors.md#locked-doors), and every closed door
can be [kicked](doors.md#kicking) in with `k`, leaving a [broken door](doors.md#kicking) (`,`) that stays open for good: see
[Doors](doors.md).

The game remembers a door as you last saw it: one you saw open stays drawn open until you see it
closed.

## Stairs

Stairs (`>` down, `<` up) join levels. Step onto them, then press `>` or `<` to use them. Using
stairs is a normal action (6 seconds). Each level you visit is kept as you left it, and [dungeon](dungeons.md) levels are generated
on your first visit. Levels you leave and return to are exactly as they were.

## Walls

Walls block movement, [sight](sight.md#line-of-sight), [light](light.md#what-light-is), and [sound](hearing.md). Most of them can be dug through with a [pick-axe](digging.md#digging),
slowly, and a big [explosion](digging.md#explosions) blows the softer ones apart; a broken wall becomes [rubble](#terrain-rubble). The edge of
a level never breaks. See [Digging and walls](digging.md).

## Water

Still water (ponds and pools) and running water (streams) are both wadeable but slow: a step into
either takes twice as long as a step on dry ground, and [splashes loudly](hearing.md#how-loud). Running water flows in a
direction you can see in the ripples, and foams where it meets a bank or rock. The current does
not push you yet. Water glints with nearby [light](light.md#what-light-is).

[Shallow water](#terrain-shallow-water), ankle-deep over a stone floor, covers the floor of a [flooded room](special-rooms.md#room-flooded) (see
[Special rooms](special-rooms.md)). It is paler than a pool. Wading through it is slower than
walking, though quicker than wading a pool, and it splashes just as loudly.

## Ice

Below [freezing](temperature.md), water grows ice: [thin ice](temperature.md#walking-on-ice), ice, and thick ice, by how thick it is. You walk on it
like dry ground, a little more carefully, but it holds only so much weight: thin ice gives way
under a person, and ice may under a heavy pack. [Falling through](temperature.md#falling-in) hurts a little, and climbing back
out takes a few turns. See [Temperature and ice](temperature.md).

## Plants and rock outdoors

[Trees](#terrain-tree), [pines](#terrain-pine), [dead trees](#terrain-dead-tree), [boulders](#terrain-boulder), crags (rock outcrops), and [standing stones](#terrain-standing-stone) block your way,
and all but dead trees block [sight](sight.md#line-of-sight) too. [Bushes](#terrain-bush) can be pushed through but slowly. [Reeds](#terrain-reeds), [marsh](#terrain-marsh), and
[rubble](#terrain-rubble) slow you a little. Their heights set how long their [shadows](sun.md) are.

Anything you can neither walk on nor see through blocks [sound](hearing.md), except a [closed door](doors.md), which only
muffles it: walls, solid rock, and the trees, boulders, crags, and stones above. Sound has to go
around them. [Water](#water), [bushes](#terrain-bush), [reeds](#terrain-reeds), [marsh](#terrain-marsh), [rubble](#terrain-rubble), [rocky ground](#terrain-rocky-ground), dry or tall grass, and creaking
boards make your steps louder; [grass](#terrain-grass), flowers, the [forest floor](#terrain-forest-floor), [dirt roads](#terrain-dirt-road), and [dusty floors](#terrain-dusty-floor)
soften them. See [Hearing](hearing.md).

## All terrain

Step times below are at a [normal pace](turns.md#what-actions-cost), which is yours: a normal step takes 6 seconds (see
[Turns and movement](turns.md)). "Walk on it" is yes for terrain you can enter, and for a [closed door](doors.md), which you open by walking into it.
"[Footsteps](hearing.md#what-makes-noise)" is how loud a step onto it is: quiet, normal, loud, or very loud.

{{#include generated/terrain-local.md}}
