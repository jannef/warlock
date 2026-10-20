# Mana

Mana is your second pool beside your [hit points](combat.md#hit-points), and it works the same way:
you have some, it has a most, and it comes back by itself with time. Nothing spends it yet: magic
will.

The status bar shows it right after your hit points, as `MP 5/12`: what you have left of your
most. The [character screen](character.md) (`@`) shows it beside your hit points. Creatures may
have mana too, but you never see theirs.

## How much

Your most mana is what your [race](races.md) and your [class](classes.md) give, plus what your
Intelligence, Willpower, and Presence add: high scores add to it, low ones take from it, and it
never goes below nothing. Wizards and witches have the most; warriors and barbarians start with
none of their own, and a half-orc's race gives none.

{{#include generated/mana.md}}

Each level up to 20 raises it (and the mana you have now) by half your class's mana plus what your
mental stats add again (see [Experience](experience.md#what-a-level-gives)). A wizard keeps
growing it; a warrior with ordinary wits gains none. [Magic items](magic-items.md) that add to it
("+5 to maximum mana") raise your most, not what you have.

## How fast it comes back

At the usual pace, a point of mana comes back every so often, faster the stronger your Willpower:

{{#include generated/mana-regeneration.md}}

Resting, some status effects and diseases, starving, and special rooms change the pace:

{{#include generated/mana-paces.md}}

When several status effects slow it, the slowest counts; a disease that slows it more than being
Diseased does counts too. Resting makes up for some of it, as it does for healing, and so does a
sanctuary (a [special room](special-rooms.md)). Magic items that speed it ("mana comes back 50%
faster") add on top of all that. Nothing else changes it: not your burden, not the cold, not your
combat stance.

## When the most falls

- Taking off a magic item that raised it lowers what you have down to your new most, if need be.
- A [disease](diseases.md) that damages your Intelligence, Willpower, or Presence lowers your most,
  and what you have with it.
- A status effect that lowers one of those stats lowers your most only while it lasts: you keep
  the mana you had, but none comes back above the lower most until it passes.

## The fine print

- Mana comes back on game time, whatever you do, like hit points: waiting, walking, fighting, or a
  long journey. Only the time counts, and it does not matter how it is cut up.
- Progress toward the next point carries over from turn to turn, and is lost once you are full.
- You never see how fast it comes back as a number, only your mana rising.

See also [Healing](healing.md) and [Stats](stats.md).
