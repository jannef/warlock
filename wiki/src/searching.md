# Searching

Not everything is what it seems. A **[secret door](doors.md#secret-doors)** looks like the wall around it, and a
**[hidden trap](traps.md#finding-traps)** like the floor it lies on (see [Traps](traps.md)). Until you find them, they are
the wall and the floor: you cannot see through a secret door, walk through it, or tell it apart,
and neither can the creatures. You find them by **searching**, or **[in passing](searching.md#noticing-in-passing)**, if you are the
kind who notices such things. Both work the same way for every hidden thing.

The thin wall of a [greater treasure room](special-rooms.md#room-great-treasury) is hidden the same way, at a [secret
door's](doors.md#secret-doors) difficulty: found, *the wall here sounds hollow*, and it stays a hollow stone wall to you
until you [dig](digging.md#digging) or [blast](digging.md#explosions) through it.

## Searching

- `s` searches the cells around you, once: it takes a [turn](time.md#what-takes-time) (6 seconds).
- `S` **searches for a while**: turn after turn, up to 10 turns, and it stops at once when you
  find something. Like a [rest](resting.md#how-resting-works), it also stops when a creature comes into view or something
  happens worth a message.

Each turn of searching gives every hidden thing on the eight cells around you a fresh chance to
be found. How likely that is depends on how hard the thing is to find, on your **[Intelligence](stats.md#stat-intelligence)**,
and on your **[Search](skills.md#skill-search)** skill (see [Skills](skills.md)); for a hidden **[trap](traps.md#finding-traps)**, on a floor, a door,
or a [chest](chests.md), your **[Detect traps](skills.md#skill-detect-traps)** skill counts instead of [Search](skills.md#skill-search). A master searcher also searches
a cell farther around them, as far as they can see. A search that finds nothing tells you nothing:
there may be nothing there, or you may have missed it. Search again; every turn is a new try,
and there is no limit to how often you may try. Only time is spent.

## Noticing in passing

Some characters notice hidden things without looking for them. After every turn you take,
walking, waiting, resting, or anything else, every hidden thing on the eight cells around you
gets a chance to catch your eye, a smaller one than a search would have. This chance depends on
your **[Willpower](stats.md#stat-willpower)** and your **[Spot](skills.md#skill-spot)** skill, or for a hidden [trap](traps.md#finding-traps) your **[Detect traps](skills.md#skill-detect-traps)** skill.
Who notices things in passing:

- **[Elves](races.md#race-elf)** and **[drow](races.md#race-drow)**: everything.
- **[Dwarves](races.md#race-dwarf)** and **[duergar](races.md#race-duergar)**: what is hidden in **stone**, a [secret door](doors.md#secret-doors) in a stone wall or a
  trap in a stone floor: most of a dungeon, but not a door in a timber wall, or a [trap](traps.md#finding-traps) in the
  grass.
- **[Kobolds](races.md#race-kobold)**: hidden **[traps](traps.md#finding-traps)**, anywhere, but never a [secret door](doors.md#secret-doors).
- **Anyone trained in [Spot](skills.md#skill-spot)**: everything but [traps](traps.md#finding-traps). Some races and classes train it (below).
- **Anyone trained in [Detect traps](skills.md#skill-detect-traps)**: hidden **[traps](traps.md#finding-traps)**. [Thieves](classes.md#thief) and [kobolds](races.md#race-kobold) start trained in it.
- Everyone else notices nothing in passing, but can always search.

{{#include generated/search-races.md}}

Noticing something stops a [rest](resting.md#how-resting-works), as any message does.

## What you find

- **A [secret door](doors.md#secret-doors)** becomes the closed door it is: "You find a secret door!" From then on it is
  a door like any other.
- **A [hidden trap](traps.md#finding-traps)** becomes known: "You notice a tripwire: an alarm trap." It shows as `^` in its
  own color from then on, and stepping onto it asks first (see [Traps](traps.md)).

Either way you remember it at once, even in the dark.

## How hard things are to find

Each hidden thing has its own difficulty. The table gives the chances for an average character
(Intelligence and Willpower 10, untrained): a single turn of searching, ten turns of it (one `S`),
and a single turn of noticing in passing. Better scores, a trained skill, and an [elf's](races.md#race-elf) eye do
better.

{{#include generated/search-features.md}}

Deeper in a dungeon, everything hidden is harder to find. [Generated dungeons](dungeons.md) hide a door or two on each
level of rooms, and only [doors](doors.md) you can do without: the [stairs](terrain.md#stairs) and the [keys](doors.md#keys) are always within
reach without finding any of them. Behind a [secret door](doors.md#secret-doors) there may be a room nobody has entered
in a long time.

{{#include generated/search-depths.md}}
