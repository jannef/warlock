# Searching

Not everything is what it seems. A **secret door** looks like the wall around it, and a
**hidden trap** like the floor it lies on (see [Traps](traps.md)). Until you find them, they are
the wall and the floor: you cannot see through a secret door, walk through it, or tell it apart,
and neither can the creatures. You find them by **searching**, or **in passing**, if you are the
kind who notices such things. Both work the same way for every hidden thing.

## Searching

- `s` searches the cells around you, once: it takes a turn (6 seconds).
- `f` **searches for a while**: turn after turn, up to 10 turns, and it stops at once when you
  find something. Like a rest, it also stops when a creature comes into view or something
  happens worth a message.

Each turn of searching gives every hidden thing on the eight cells around you a fresh chance to
be found. How likely that is depends on how hard the thing is to find, on your **Intelligence**,
and on your **Search** skill (see [Skills](skills.md)). A master searcher also searches a cell
farther around them, as far as they can see. A search that finds nothing tells you nothing:
there may be nothing there, or you may have missed it. Search again; every turn is a new try,
and there is no limit to how often you may try. Only time is spent.

## Noticing in passing

Some characters notice hidden things without looking for them. After every turn you take,
walking, waiting, resting, or anything else, every hidden thing on the eight cells around you
gets a chance to catch your eye, a smaller one than a search would have. This chance depends on
your **Willpower** and your **Spot** skill. Who notices things in passing:

- **Elves** and **drow**: everything.
- **Dwarves** and **duergar**: what is hidden in **stone**, a secret door in a stone wall or a
  trap in a stone floor: most of a dungeon, but not a door in a timber wall, or a trap in the
  grass.
- **Kobolds**: hidden **traps**, anywhere, but never a secret door.
- **Anyone trained in Spot**: everything. Some races and classes train it (below).
- Everyone else notices nothing in passing, but can always search.

{{#include generated/search-races.md}}

Noticing something stops a rest, as any message does.

## What you find

- **A secret door** becomes the closed door it is: "You find a secret door!" From then on it is
  a door like any other.
- **A hidden trap** becomes known: "You notice a tripwire: an alarm trap." It shows as `^` in its
  own color from then on, and stepping onto it asks first (see [Traps](traps.md)).

Either way you remember it at once, even in the dark.

## How hard things are to find

Each hidden thing has its own difficulty. The table gives the chances for an average character
(Intelligence and Willpower 10, untrained): a single turn of searching, ten turns of it (one `f`),
and a single turn of noticing in passing. Better scores, a trained skill, and an elf's eye do
better.

{{#include generated/search-features.md}}

Deeper in a dungeon, everything hidden is harder to find. Generated dungeons hide a door or two on each
level of rooms, and only doors you can do without: the stairs and the keys are always within
reach without finding any of them. Behind a secret door there may be a room nobody has entered
in a long time.

{{#include generated/search-depths.md}}
