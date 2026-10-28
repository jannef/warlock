# Hunger

You have to eat. As in ADOM, your stomach empties with **time**, whatever you do: walking,
fighting, resting, and long journeys on the [overworld](overworld.md) alike. The game counts
your food in points of **nutrition**; you never see the count, only how hungry you are.

{{#include generated/hunger-facts.md}}

## How hungry you are

{{#include generated/hunger-states.md}}

- **Hungry** is a warning: eat soon. Becoming hungry stops a rest (see [Resting](resting.md)).
- **Weak**: your Strength and Dexterity count for less, in a fight and in what you can carry
  (see [Stats](stats.md) and [Burden](burden.md)). The character screen shows the lowered
  scores. Being fed again gives them back.
- **Fainting**: now and then you faint from hunger ("You faint from hunger.") and lose a few
  turns, while everything around you goes on.
- **Starving**: your wounds stop healing, and you lose hit points by the clock until you eat,
  or starve to death.
- **Satiated** does nothing, but **Bloated** slows everything you do, as a burden does: wait for
  it to pass. Eating past the most your stomach holds is **overeating**: you may vomit, which
  leaves you hungry and feeds you nothing.

Choking to death on food, ADOM's famous end, can be turned on in the game's data; it is off.

## Races

Some races need more food than others:

{{#include generated/hunger-races.md}}

The **Lean** talent makes your hunger come slower (see [Talents](talents.md)), and so does magic
that sustains you: its percent comes off how fast your stomach empties, on top of the race's
and the talent's (see [Magic items](magic-items.md)). However much you wear, you still hunger a
little.

Fighting burns a little more: each blow, shot, or throw costs a few points (see the facts
above).

## What you see

- The **status bar** shows how hungry you are, unless you are not hungry: `Hungry`, `Weak`,
  `Fainting`, `Starving`, `Satiated`, or `Bloated`. It is one of the last things to give way on
  a narrow screen (see [Controls](controls.md#the-screen)).
- A **message** says when it changes: "You are getting hungry.", "You feel weak from hunger.",
  "You feel faint from hunger.", "You are starving!", and, eating, "You are no longer hungry.",
  "You feel satiated.", "You feel bloated."

To eat, see [Food and corpses](food.md).
