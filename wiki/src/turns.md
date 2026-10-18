# Turns and movement

The game is turn based. You act, time passes, and then every creature acts once.

## Turn length

A level decides how long one turn takes:

| Kind of level | One turn |
|---|---|
| Dungeons and outdoor areas | 6 seconds |
| The overworld | 10 minutes |

A few things always take exactly one turn: waiting (`.`), opening or closing a door, lighting or
dousing the lantern, and taking stairs. The turn is the length of the level you start on, so
taking stairs off the overworld costs 10 minutes and walking off the edge of an outdoor area costs
6 seconds.

## Steps

On a dungeon or outdoor level a step takes one turn scaled by the terrain you step onto. A normal
step takes 6 seconds, wading through water or pushing through a bush takes twice as long (12
seconds), and moving through reeds, marsh, or rubble takes one and a half times as long (9
seconds). The full table is on the
[Terrain](terrain.md) page.

On the overworld a step is a journey: it takes the time given for the terrain you step onto, from
20 minutes along a road to two and a half hours through a swamp. See [The overworld](overworld.md).

You can move in eight directions, diagonals included. You cannot step into a wall or other solid
terrain, or onto a creature. Walking into a closed door opens it, and that takes a turn.

## Creatures act once per turn

After your action, each creature on the level acts once, however long your action took. Wading
through water takes you 12 seconds, but a creature still takes just one step. There is no speed
system yet.
