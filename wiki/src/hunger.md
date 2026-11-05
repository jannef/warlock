# Hunger

You have to eat. As in ADOM, your stomach empties with **time**, whatever you do: walking,
[fighting](combat.md#hitting), [resting](resting.md#how-resting-works), and long journeys on the [overworld](overworld.md) alike. The game counts
your food in points of **nutrition**; you never see the count, only how hungry you are.

{{#include generated/hunger-facts.md}}

## How hungry you are

{{#include generated/hunger-states.md}}

- **Hungry** is a warning: eat soon. Becoming hungry stops a [rest](resting.md#how-resting-works) and
  makes you read the [message](controls.md#messages) before you go on (see [What you see](#what-you-see)).
- **Weak**: your [Strength](stats.md#stat-strength) and [Dexterity](stats.md#stat-dexterity) count for less, in a fight and in what you can carry
  (see [Stats](stats.md) and [Burden](burden.md)). The [character screen](controls.md#character) shows the lowered
  scores. Being fed again gives them back.
- **Fainting**: now and then you faint from hunger ("You faint from hunger.") and lose a few
  [turns](time.md#what-takes-time), while everything around you goes on.
- **Starving**: your wounds stop [healing](healing.md#how-fast), and you lose [hit points](combat.md#hit-points) by the clock until you eat,
  or starve to death.
- **Satiated** does nothing, but **Bloated** slows everything you do, as a [burden](burden.md#burden-1) does: wait for
  it to pass. Eating past the most your stomach holds is **overeating**: you may vomit, which
  leaves you hungry and feeds you nothing.

Choking to death on food, ADOM's famous end, can be turned on in the game's data; it is off.

## Races

Some [races](races.md) need more food than others:

{{#include generated/hunger-races.md}}

The **[Lean](talents.md#talent-lean)** talent makes your hunger come slower (see [Talents](talents.md)), and so does magic
that sustains you: its percent comes off how fast your stomach empties, on top of the race's
and the talent's (see [Magic items](magic-items.md)). However much you wear, you still hunger a
little.

Fighting burns a little more: each [blow](combat.md#hitting), [shot, or throw](ranged.md#shooting-and-throwing) costs a few points (see the facts
above).

## What you see

- The **[status bar](controls.md#the-screen)** shows how hungry you are, unless you are not hungry: `Hungry`, `Weak`,
  `Fainting`, `Starving`, `Satiated`, or `Bloated`. From `Hungry` down the word stands out as it
  worsens: `Hungry` mild yellow, `Weak` strong orange, `Fainting` and `Starving` alarming reds. It never gives way on a
  narrow screen; `Satiated` and `Bloated` are among the last things to (see [Controls](controls.md#the-screen)).
- A **[message](controls.md#messages)** says when it changes: "You are getting hungry.", "You feel weak from hunger. You should eat soon.",
  "You feel faint from hunger. Eat now, or you will starve!", "You are starving! Eat, or you will die!", and, eating, "You are no longer hungry.",
  "You feel satiated.", "You feel bloated."
- **Getting hungrier stops you.** Becoming `Hungry`, `Weak`, `Fainting`, or `Starving` waits at
  `--more--` until you have read it, even if the message fits, and a held key does not walk on past it.
- **A journey asks first.** A step on the [overworld](overworld.md#travel) takes much longer than one anywhere else. While
  you are `Fainting` or `Starving`, every step there asks whether to go on: staying takes no time, and is what `Esc` does.
  The question goes by how hungry you are, not by whether this step would kill you. [Long fast](talents.md#talent-long-fast) is asked too.

To eat, see [Food and corpses](food.md).
