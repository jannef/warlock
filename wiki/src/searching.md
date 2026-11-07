# Searching

A [secret door](doors.md#secret-doors) looks like the wall around it, and a
[hidden trap](traps.md#finding-traps) looks like the floor it lies on (see [Traps](traps.md)). Until you
find one, it *is* wall or floor to you. You cannot see through a secret door or walk through it,
and neither can the creatures. You find hidden things by **searching**, or
[in passing](#noticing-in-passing) if you are the kind who notices such things. Both work the
same way for every hidden thing.

The thin wall of a [greater treasure room](special-rooms.md#room-great-treasury) is hidden the
same way, and is as hard to find as a [secret door](doors.md#secret-doors). Once found, *the wall
here sounds hollow*. It stays a stone wall until you [dig](digging.md#digging) or
[blast](digging.md#explosions) through it.

## Searching

- `s` searches the cells around you, once: it takes a [turn](time.md#what-takes-time) (6 seconds).
- `S` **searches for a while**: turn after turn, up to 10 turns, and it stops at once when you
  find something. Like a [rest](resting.md#how-resting-works), it also stops when a creature comes into view or something
  happens worth a message.

Each turn of searching gives every hidden thing on the eight cells around you a fresh chance to
be found. The chance depends on how hard the thing is to find, your
[Intelligence](stats.md#stat-intelligence), and your [Search](skills.md#skill-search) skill (see
[Skills](skills.md)). For a hidden [trap](traps.md#finding-traps), whether on a floor, a door, or a
[chest](chests.md), your [Detect traps](skills.md#skill-detect-traps) skill counts instead of
Search. A master searcher also searches one cell farther out, as far as they can see.

A search that finds nothing tells you nothing. There may be nothing there, or you may have missed
it. Every turn is a new try, and you may search as often as you like. It only costs time.

## Noticing in passing

Some characters notice hidden things without looking for them. After every turn you take,
whatever you did, each hidden thing on the eight cells around you has a chance to catch your eye.
The chance is smaller than a search's. It depends on your
[Willpower](stats.md#stat-willpower) and your [Spot](skills.md#skill-spot) skill, or your
[Detect traps](skills.md#skill-detect-traps) skill for a hidden [trap](traps.md#finding-traps).
Who notices things in passing:

- **[Elves](races.md#race-elf)** and **[drow](races.md#race-drow)**: everything.
- **[Dwarves](races.md#race-dwarf)** and **[duergar](races.md#race-duergar)**: what is hidden in
  stone, such as a [secret door](doors.md#secret-doors) in a stone wall or a trap in a stone floor.
  That is most of a dungeon, but not a door in a timber wall or a [trap](traps.md#finding-traps) in
  the grass.
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

Each hidden thing has its own difficulty. The table gives the chances for an average, untrained
character with Intelligence and Willpower 10. It shows one turn of searching, ten turns (one `S`),
and one turn of noticing in passing. Better scores, a trained skill, and an
[elf's](races.md#race-elf) eye all do better.

{{#include generated/search-features.md}}

Deeper in a dungeon, everything hidden is harder to find. [Generated dungeons](dungeons.md) hide
a door or two on each level of rooms. They only hide [doors](doors.md) you can do without: you can
always reach the [stairs](terrain.md#stairs) and the [keys](doors.md#keys) without finding any.
Behind a [secret door](doors.md#secret-doors) there may be a room nobody has entered in a long
time.

{{#include generated/search-depths.md}}
