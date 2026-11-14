# Digging and walls

Walls are not forever. With a [pick-axe](#digging) and a lot of patience you can dig through
stone, and a big enough [explosion](#explosions) blows [walls](terrain.md#walls) apart. A wall
that gives way becomes [rubble](terrain.md#terrain-rubble): open ground you can walk on, a little
slowly, and [see across](sight.md#line-of-sight).

## Digging

1. **Learn to mine.** Only someone trained in the [Mining](skills.md#skill-mining) skill can dig.
   Every [dwarf](races.md#race-dwarf), [duergar](races.md#race-duergar), and
   [kobold](races.md#race-kobold) knows it, and some craftspeople and commoners do. Anyone else
   can learn it from [Alberich](npcs.md#alberich) in Fyrdelf, for gold.
2. **Get a digging tool.** A pick-axe or a mattock goes in the Tool slot of your
   [equipment](equipment.md): press `i`, then `m` ([Tool](equipment.md#what-fits-where)), then the
   tool's letter. Ælfric's General Store in Cēaptūn sells a pick-axe.
3. **Dig.** Press `u` (use a tool), then a direction: "Dig in which direction?". You start digging
   into the wall next to you ("You dig into the stone wall.").
4. **Keep at it.** Digging goes on by itself, turn after turn, as [resting](resting.md) does,
   until the wall gives way: "After 6 minutes, the stone wall crumbles."

Each turn of digging is a [normal action](turns.md#what-actions-cost), 6 seconds at a normal pace,
and it takes many turns. The harder the wall, the longer it takes. A better tool and more
[Strength](stats.md) get you through faster. A heavy load slows digging, as it slows everything
you do ([Burden](burden.md)). The [Tunneler](talents.md#talent-tunneler) talent does half again as
much work a turn (see [Talents](talents.md)). [Magic](magic-items.md) that digs faster, on a
pick-axe, gloves, or body armour, adds its percent of work a turn on top (see
[Magic items](magic-items.md)). So does the [Mining](skills.md#skill-mining) skill: 25 % more at 100, 40 % more at
200. Every wall you dig through trains it. Trained in Mining, a wall you dig through in a dungeon
may give you ore, stone, or a gem (see [Mining](mining.md)).

A pick-axe can itself be magic. Magic ones turn up rarely in the [loot](magic-items.md#finding-them)
of the deeper places (from [item level](affixes.md#base-items) 3) and in [shops](shops.md). Its
description gives its digging power: "Digs at 10". A mattock digs at 15. The pick-axes you start
with, find lying in a level, or take from a [dwarf](races.md#race-dwarf) are plain.

Digging times at average Strength (10) and a normal pace:

{{#include generated/digging.md}}

And with a pick-axe at other Strength scores:

{{#include generated/digging-strength.md}}

Without a tool ready, `u` tells you so: "You have no digging tool ready." Untrained in Mining, it
says "You do not know how to mine: that takes the Mining skill." Pointing it at open ground says
"There is nothing to dig there."

### Stopping and going on

Digging stops early, as [resting](resting.md#how-resting-works) does, when a
[hostile](creatures.md#factions) creature comes within 5 steps in [view](sight.md#line-of-sight)
("A kobold comes into view; you stop digging after 3 minutes.") or something worth a message
happens, such as a [sound you hear](hearing.md#what-you-hear), an attack on you, or the
[rain](weather.md) starting.

**What you dug stays dug.** The wall keeps the work you put into it, and the next `u` toward it
goes on from there ("You go on digging at the stone wall."). [Look mode](controls-prompts.md#look-mode)
calls a wall you have started on "partly dug". A dig also stops after a day of work, if it ever
runs that long.

### Noise

Digging is loud. Iron on stone carries a long way (see [Hearing](hearing.md)). Creatures nearby
hear it and some come to look, and when it is out of sight you hear it as "digging". Only you
dig; [creatures](creatures.md) do not.

## What breaks and what does not

- **[Packed earth](terrain.md#terrain-packed-earth)** gives way quickly,
  **[stone walls](terrain.md#terrain-wall)** take a while, and
  **[granite](terrain.md#terrain-granite)** takes longest. The table above gives each one's time.
- **[Doors](doors.md), [trees](terrain.md#terrain-tree), crystals, torches, and the like** never
  break: "You cannot dig through the closed door."
- **The [edge](terrain.md#walls) of a level** never breaks, whatever it is made of. Every level is
  closed in by walls you cannot dig out of.
- **Solid rock** deep in the ground shows as a stone wall once you dig next to it, so a tunnel you
  dig is walled like any corridor.

Only walls break. You cannot dig down or up, and you cannot dig on the
[overworld](overworld.md).

## Explosions

A [fireball trap](traps.md#trap-fireball)'s explosion (see [Traps](traps.md)) breaks, at once,
every wall in its blast that is softer than the explosion is strong: "The walls collapse!" The
table above says which walls a fireball breaks. It reaches only the walls it can touch, the cells
in line of sight of its centre and within its reach, so a blast never carries on through a wall it
breaks. Nothing is left to chance. [Granite](terrain.md#terrain-granite) and the edge of the level
hold.

## After a wall falls

The wall becomes rubble, and the rest follows at once. [Light](light.md#what-light-is) from beyond
the gap spills through, you see into what was behind it, and you [remember](sight.md#memory) it as
you remember everything you have seen ([Sight and memory](sight.md)). Dust rises where it fell.

## Where to dig

**[Fyrdelf](dungeon-guide.md#fyrdelf)**, the fire diggings under the hills of Ascdūn
([The Ascmearc](ascmearc.md)), is a mine and the place to try digging. Its foreman's cabin keeps a
spare [pick-axe](#digging) behind a plank wall.
