# Healing

Lost [hit points](combat.md#hit-points) come back by themselves, with time, and [resting](resting.md) (`b`) is the
quickest way to let the time pass. A [potion of healing](potions.md#potion-of-healing) brings some back at once (see
[Potions](potions.md)).

## How fast

At the usual pace, a [hit point](combat.md#hit-points) comes back every so often, faster the tougher you are:

{{#include generated/regeneration.md}}

[Resting](resting.md#how-resting-works), [status effects](status-effects.md#how-they-work), starving (see [Hunger](hunger.md)), and later other things change the
pace:

{{#include generated/healing.md}}

When several [status effects](status-effects.md#how-they-work) slow your healing, the slowest counts; a [disease](diseases.md#how-a-disease-goes) that slows it more
than being [Diseased](status-effects.md#status-diseased) does counts too (see [Diseases](diseases.md)). [Resting](resting.md#how-resting-works) makes up for some of
it: resting while [poisoned](status-effects.md#poisoned) heals at the usual pace. While you [bleed](status-effects.md#bleeding) or starve, nothing heals you,
[rest](resting.md#how-resting-works) or not. In a [sanctuary](special-rooms.md#room-sanctuary) (a [special room](special-rooms.md)) wounds heal three times as fast,
on top of all that, yours and anyone's, and [magic items](magic-items.md) that speed healing
("hit points come back 50% faster") add on top again.

## The fine print

- Healing runs on game time, whatever you do: waiting, walking, fighting, or a long journey on
  the [overworld](overworld.md). Only the time counts, and it does not matter how it is cut up.
- Progress toward the next [hit point](combat.md#hit-points) carries over from turn to turn, and is lost once you are
  unhurt.
- [Creatures](creatures.md) heal by the same rules, at their own [Toughness](stats.md#stat-toughness), while you are on their level; they
  never [rest](resting.md#how-resting-works).

Your [mana](mana.md) comes back the same way, by your [Willpower](stats.md#stat-willpower).

See also [Status effects](status-effects.md) and [Combat](combat.md#hit-points).
