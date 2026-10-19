# Healing

Lost hit points come back by themselves, with time. There is nothing to do but let the time pass,
and [resting](resting.md) (`R`) is the quickest way to do that.

## How fast

At the usual pace, a hit point comes back every so often, faster the tougher you are:

{{#include generated/regeneration.md}}

Resting, status effects, and later other things change the pace:

{{#include generated/healing.md}}

When several status effects slow your healing, the slowest counts; a disease that slows it more
than being Diseased does counts too (see [Diseases](diseases.md)). Resting makes up for some of
it: resting while poisoned heals at the usual pace. While you bleed, nothing heals you, rest or
not.

## The fine print

- Healing runs on game time, whatever you do: waiting, walking, fighting, or a long journey on
  the [overworld](overworld.md). Only the time counts, and it does not matter how it is cut up.
- Progress toward the next hit point carries over from turn to turn, and is lost once you are
  unhurt.
- Creatures heal by the same rules, at their own Toughness, while you are on their level; they
  never rest.

See also [Status effects](status-effects.md) and [Combat](combat.md#hit-points).
