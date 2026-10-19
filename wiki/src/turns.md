# Turns and movement

The game is turn based. You act, and then time passes while the creatures around you act, each at
its own pace, until it is your turn again.

## What actions cost

What you do decides how long it takes. Every action has a **cost**: a normal action costs 1000
and takes 6 seconds at a normal pace, which is your pace. An action that costs more takes longer
in proportion, and one that costs less is quicker: an action of cost 1400 would take 8.4 seconds.

{{#include generated/actions.md}}

Every action takes the same time wherever you are, the overworld included: waiting there takes 6
seconds, and so do entering a place with `>` and walking off the edge of an outdoor area back to
the overworld. Only steps on the overworld are different: each is a journey (below). Picking up
or dropping takes one action however many stacks you take. Putting on or taking off
[equipment](equipment.md) takes its slot's time for each thing that moves, so a swap takes both.
Looking at your pack (`i`) or your equipment (`e`) takes no time, and neither does anything that
is refused, such as walking into a wall.

The clock counts whole seconds, so an action of 8.4 seconds shows as 8 or 9 seconds on it, but a
run of actions always adds up to their exact time: five such actions take exactly 42 seconds.

A heavy load slows every action you take, journeys included: a quarter longer when you are
burdened, half again as long when you are strained, so a normal action takes 7.5 or 9 seconds.
Overloaded, you cannot move at all. See [Burden](burden.md).

## Steps

On a dungeon or outdoor level a step costs more or less depending on the terrain you step onto. A
normal step takes 6 seconds, wading through water or pushing through a bush takes twice as long
(12 seconds), and moving through reeds, marsh, or rubble takes one and a half times as long (9
seconds). The full table is on the [Terrain](terrain.md) page.

On the overworld a step is a journey: it takes the time given for the terrain you step onto, from
20 minutes along a road to two and a half hours through a swamp. See [The overworld](overworld.md).

You can move in eight directions, diagonals included. You cannot step into a wall or other solid
terrain; trying takes no time. Walking into a closed door opens it, which costs the same as
opening it. Moving into a creature that is hostile to you attacks it (see
[Creatures](creatures.md#attacks)). Moving into any other creature swaps places with it, for
your step's time ("You swap places with the deer."), unless it will not make way, which takes no
time ("The sentry doesn't budge."); see [People and talk](npcs.md#getting-past).

## Creatures act at their own pace

Every creature has its own pace, and the game does not show it. A faster creature takes less time
over the same action, so it acts more often; a slower one acts less often. Creatures' actions
have costs too: a step, a wait, or an attack takes a creature as long as its cost and its pace
say, and a creature's load slows it as yours slows you. After your action, the creatures on the level act in the order their turns come up until it
is your turn again:

- A creature as quick as you acts once for each normal action of yours, right after you.
- A quicker creature may act twice or more before your next turn, and a slower one now and then
  lets one of your turns pass.
- The longer your action takes, the more the creatures do meanwhile. Wading through water takes
  you twice as long as a normal step, so creatures get about twice as many actions during it.
- When a creature's turn comes up at the same moment as yours, you go first.

Creatures on other levels do not act at all while you are away. See [Creatures](creatures.md).
