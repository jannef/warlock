# Turns and movement

The game is turn based. You act, and then time passes while the creatures around you act, each at
its own pace, until it is your turn again.

## Turn length

A level decides how long one turn takes:

| Kind of level | One turn |
|---|---|
| Dungeons and outdoor areas | 6 seconds |
| The overworld | 10 minutes |

A few things always take exactly one turn: waiting (`.`), opening or closing a door, lighting or
dousing the lantern, picking up or dropping objects (however many at once), and taking stairs.
Looking at your pack (`i`) takes no time. The turn is the length of the level you start on, so
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

## Creatures act at their own pace

Every creature has its own pace, and the game does not show it. After your action, the
creatures on the level act in the order their turns come up until it is your turn again:

- A creature as quick as you acts once for each of your turns, right after you.
- A quicker creature may act twice or more before your next turn, and a slower one now and then
  skips one of your turns.
- The longer your action takes, the more the creatures do meanwhile. Wading through water takes
  you twice as long as a normal step, so creatures get about twice as many actions during it.
- When a creature's turn comes up at the same moment as yours, you go first.

Creatures on other levels do not act at all while you are away. See [Creatures](creatures.md).
