# Healing

Lost [hit points](hit-points.md#hit-points) come back by themselves over time. [Resting](resting.md)
(`b`) is the quickest way to let that time pass. A [potion of healing](potions.md#potion-of-healing)
brings some back at once (see [Potions](potions.md)).

The healing spells, [Mend](magic.md#spell-mend), [Cure wounds](magic.md#spell-cure-wounds), and
[Benison](magic.md#spell-benison), bring some back too. On the undead they
[harm instead](spellcasting.md#healing-harms-the-undead).

## How fast

At the usual pace you get a [hit point](hit-points.md#hit-points) back every so often. The tougher
you are, the faster it comes:

{{#include generated/regeneration.md}}

[Resting](resting.md#how-resting-works), [status effects](status-effects.md#how-they-work), and
starving (see [Hunger](hunger.md)) change the pace:

{{#include generated/healing.md}}

When several things change the pace at once:

- If several [status effects](status-effects.md#how-they-work) slow your healing, only the slowest counts.
- An effect that speeds healing up ([Renewal](magic.md#spell-renewal)'s, four times the pace) counts
  only while nothing slows it.
- A [disease](diseases.md#how-a-disease-goes) that slows healing more than being
  [Diseased](status-effects.md#status-diseased) does counts too (see [Diseases](diseases.md)).
- [Resting](resting.md#how-resting-works) makes up for some of it. Resting while
  [poisoned](status-effects.md#poisoned) heals at the usual pace.
- While you [bleed](status-effects.md#bleeding) or starve, nothing heals you, resting or not.
- In a [sanctuary](special-rooms.md#room-sanctuary) (a [special room](special-rooms.md)), wounds heal
  three times as fast on top of all that. This holds for anyone there, not just you.
- [Magic items](magic-items.md) that speed healing ("hit points come back 50% faster") add on top
  again.

## The fine print

- Healing runs on game time, whatever you are doing: waiting, walking, fighting, or making a long
  journey on the [overworld](overworld.md). Only the total time counts, not how it is split up.
- Progress toward the next [hit point](hit-points.md#hit-points) carries over from turn to turn. It is
  lost once you are unhurt.
- [Creatures](creatures.md) heal by the same rules, at their own [Toughness](stats.md#stat-toughness),
  while you are on their level. They never [rest](resting.md#how-resting-works).
- Some creatures **regenerate**. Most [trolls](creatures.md#creature-troll) heal four times as fast
  as their Toughness alone would give (the [creature numbers](creature-numbers.md) say which ones do).
  This counts as a status effect that speeds healing, so anything that slows healing slows theirs
  too. When you see one get a tenth of its hit points back at once, you are told: *The troll's
  wounds close.*
- **Fire and acid stop all healing** while they last. Nothing heals while
  [Burning](status-effects.md#status-burning) or [Corroded](status-effects.md#status-corroded): not a
  troll's regeneration, and not yours.

Your [mana](mana.md) comes back in the same way, paced by your [Willpower](stats.md#stat-willpower).

See also [Status effects](status-effects.md) and [Hit points](hit-points.md#hit-points).
