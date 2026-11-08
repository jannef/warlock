# Prayer

The [priest](classes.md#priest) has one prayer, **[Orison](abilities.md#ability-orison)**, in the
[abilities menu](abilities.md#the-abilities-menu). It comes at level 1 and deepens with the
priest's [levels](experience.md#levels). It costs no [mana](mana.md#spending-it) and takes the
[time of a spell](turns.md#what-actions-cost). It answers what ails you most, as far as your
prayers reach. Pray too often and nothing answers for a while.

You cannot pray while [raging](status-effects.md#status-raging) or
[silenced](status-effects.md#status-silenced). A prayer with nothing to answer is refused ("You
have nothing to pray for.") and costs nothing: no time and no strain. A [disease](diseases.md) still
brewing in you does not count yet, so a prayer tells you nothing of one.

The menu names the prayer by its highest tier: Orison, then Prayer, Litany, Benediction, and
Miracle, as the [table below](#how-it-grows) gives them, with that tier's description.

## What it answers

Each prayer answers the first thing that ails you, in this order, among the uses your prayers have
reached. From level 40 it answers the first two, and from 60 all of them at once. A use not yet
reached is skipped, even if it ails you.

{{#include generated/prayer-uses.md}}

A prayer trains nothing: no [stat](stats.md#training) and no [caster style](schools.md#caster-styles). It does
not move your [alignment](alignment.md#what-moves-you).

## How it grows

The prayer talents come with the priest's levels (see [Classes](classes.md#levels)). Each adds uses
or answers. The prayer's name changes with each new tier, to the dark name when the dark powers
answer you.

{{#include generated/prayer-levels.md}}

At level 60, the [Miracle](talents.md#talent-miracle) answers everything that ails you in one
prayer. It also fills every [resource](resources.md) pool you hold
([rage](resources.md#rage), [ki](resources.md#resource-ki), [focus](resources.md#resource-focus),
and the rest) to its most.

## Strain

Every prayer adds a hidden **strain**, answered or not, and the strain passes with time. The more
strain a prayer finds, the likelier it **fizzles** ("You pray, but nothing answers."). A fizzle
takes its time and answers nothing. It also locks the prayer: every prayer after it fizzles, with
no chance at all, until the strain has passed to nothing.

{{#include generated/prayer-strain.md}}

The chance a prayer fizzles, if you pray again and again with the same time between and none of the
earlier prayers has fizzled:

{{#include generated/prayer-fizzle.md}}

So a prayer now and then is always answered. A priest who prays every few minutes in a long fight
starts to fail, and once a prayer has fizzled, the rest of that fight goes without.

## The holy and the dark

Who answers depends on your [alignment](alignment.md) at the moment you pray. If you are Neutral or
better, the holy powers answer; if Callous or worse, the dark ones. The game never names them, but
the prayer's words tell you ("You kneel and pray." or "You whisper to the dark."). So does its name
in the menu: a Whisper, an Invocation, a Rite, a Pact, a Miracle.

The dark powers answer the same things in the same order, with the same strain. But each answer
has a twist (the table above gives each):

- The diseases they take from you, the brewing ones too, are caught by the nearest
  [creature](creatures.md) you see, as if it had failed to resist them. A creature immune to disease
  catches none.
- The [poison](status-effects.md#status-poisoned) they take is laid on that creature, at the same
  degrees. A creature immune or resistant to [poison](damage-types.md#damage-poison) takes less or
  none.
- A hex or hold they release is laid back on whoever laid it on you, if you see them, at the
  degrees it had. Whoever held you is caught [in a web](status-effects.md#status-webbed) instead.
- They feed you to the brim, one short of [Bloated](hunger.md#how-hungry-you-are). The holy powers
  feed you to [Satiated](hunger.md#how-hungry-you-are).

A creature that dies of a sickness or poison passed on is your kill. A twist that reaches a
creature breaks the [hallowed ground](cursed-and-hallowed.md#hallowed-ground) you stand on, as a blow
struck from it would.

## Communion and Sanctuary

Besides the prayer, the priest's table has three rites of holy and unholy ground.

- [Consecrate](abilities.md#ability-consecrate) answers only a priest of Neutral or better.
- Its dark twin, [Communion with the dark](talents.md#talent-communion), works only for a priest the
  dark powers answer. In an unholy place their prayers add no strain at all, and the place's drain
  never touches them. The chance of a fizzle and its lock still go by the strain already there.
- [Sanctuary](abilities.md#ability-sanctuary) works for either. The dark powers' ground reads
  "dreaded" instead of "hallowed", and works the same (see
  [Special rooms](cursed-and-hallowed.md#hallowed-ground)).

{{#include generated/prayer-rites.md}}
