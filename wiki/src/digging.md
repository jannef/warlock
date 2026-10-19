# Digging and walls

Walls are not forever. With a pick-axe and a lot of patience you can dig through stone, and a big
enough explosion blows walls apart. A wall that gives way becomes rubble: open ground you can
walk on (a little slowly) and see across.

## Digging

1. **Get a digging tool.** A pick-axe or a mattock goes in the **Tool** slot of your
   [equipment](equipment.md): press `e`, then `m` (Tool), then the tool's letter. The hub's
   storeroom has a pick-axe, and the quarry has both.
2. **Dig.** Press `a` (apply), then a direction: "Dig in which direction?". You start digging into
   the wall next to you ("You dig into the stone wall.").
3. **Keep at it.** Digging goes on by itself, turn after turn, as [resting](resting.md) does, until
   the wall gives way: "After 6 minutes, the stone wall crumbles."

Digging takes a lot of effort. Each turn of digging is a normal action (6 seconds at a normal
pace), and it takes many of them: the harder the wall, the longer. A better tool and more
[Strength](stats.md) get through faster, and a heavy load slows you, as it slows everything you
do ([Burden](burden.md)).

At average Strength (10), at a normal pace:

{{#include generated/digging.md}}

And with a pick-axe at other Strength scores:

{{#include generated/digging-strength.md}}

Without a tool ready, `a` tells you so: "You have no digging tool ready." Pointing it at open
ground says "There is nothing to dig there."

### Stopping and going on

Digging stops early, just like resting, when a creature comes into view ("A kobold comes into
view; you stop digging after 3 minutes.") or something worth a message happens: a sound you hear,
an attack on you, the rain starting. **What you dug stays dug**: the wall keeps the work you put
into it, and the next `a` toward it goes on from there ("You go on digging at the stone wall.").
Look mode calls a wall you have started on "partly dug". A dig also stops after a day of work, if
it ever runs that long.

### Noise

Digging is loud: iron on stone carries a long way (see [Hearing](hearing.md)). Creatures nearby
hear it and some come to look, and you hear it as "digging" when it is out of sight. Only you dig;
creatures do not.

## What breaks and what does not

- **Packed earth** gives way quickly, **stone walls** take a while, and **granite** takes the
  better part of an hour.
- **Doors, trees, crystals, torches, and the like** never break: "You cannot dig through the closed
  door."
- **The edge of a level** never breaks, whatever it is made of. Every level is closed by walls you
  cannot dig out of.
- **Solid rock** deep in the ground shows its face as a stone wall once you dig next to it, so a
  tunnel you dig is walled like any corridor.

Only walls break. You cannot dig down or up, and digging does not work on the overworld.

## Explosions

Nothing in the game explodes yet; fireball traps are on their way. A big explosion breaks every
wall in its blast whose hardness is less than the explosion's strength, at once: "The wall
collapses!" It reaches only the walls it can touch, the cells in line of sight of its centre
within its reach, so a blast never goes through a wall it breaks. Nothing is left to chance.
Granite and the edge of the level hold.

## After a wall falls

The wall becomes rubble, and everything follows at once: light from beyond the gap spills through,
you see into what was behind it, and you remember it as you remember everything you have seen
([Sight and memory](sight.md)). Dust rises where it fell.

## The quarry

The stairs nook in the hub has a staircase down (`>`) to **the quarry**, an old stone quarry and
the place to try all this: a pick-axe and a mattock lie by the stairs, the pit is walled by a
granite face and a bank of packed earth, and a sealed chamber with a lamp and a little treasure
lies behind its east wall, with no door in. The **Quarry** scenario starts there.
