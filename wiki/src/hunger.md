# Hunger

You have to eat. Your stomach empties as **time** passes, whatever you do: walking,
[fighting](combat.md#hitting), [resting](resting.md#how-resting-works), and long journeys on the
[overworld](overworld.md) all count. The game measures your food in points of **nutrition**.
You never see the count, only how hungry you are.

{{#include generated/hunger-facts.md}}

## How hungry you are

{{#include generated/hunger-states.md}}

- **Hungry** is a warning: eat soon. Becoming hungry stops a [rest](resting.md#how-resting-works)
  and makes you read the [message](the-screen.md#messages) before you go on (see [What you
  see](#what-you-see)).
- **Weak**: your [Strength](stats.md#stat-strength) and [Dexterity](stats.md#stat-dexterity)
  count for less, both in a fight and in what you can carry (see [Stats](stats.md) and
  [Burden](burden.md)). The [character screen](the-screen.md#character) shows the lowered
  scores. Eating gives them back.
- **Fainting**: now and then you faint from hunger ("You faint from hunger.") and lose a few
  [turns](time.md#what-takes-time) while everything around you goes on.
- **Starving**: your wounds stop [healing](healing.md#how-fast), and you lose [hit
  points](hit-points.md#hit-points) as time passes until you eat or starve to death.
- **Satiated** does nothing. **Bloated** slows everything you do, as a
  [burden](burden.md#burden-1) does, so wait for it to pass. Eating more than your stomach can
  hold is **overeating**: you may vomit, which leaves you hungry and feeds you nothing.

You cannot choke to death on food.

## Races

Some [races](races.md) need more food than others:

{{#include generated/hunger-races.md}}

The [Lean](talents.md#talent-lean) talent makes you hungry more slowly (see
[Talents](talents.md)). So does magic that sustains you: its percent comes off how fast your
stomach empties, on top of the race's and the talent's (see [Magic items](magic-items.md)).
However much of it you wear, you still get a little hungry.

Fighting burns a little more. Each [blow](combat.md#hitting), [shot, or
throw](ranged.md#shooting-and-throwing) costs a few points (see the facts above).

## What you see

- The [status bar](the-screen.md#the-screen) shows how hungry you are unless you are not hungry
  at all: `Hungry`, `Weak`, `Fainting`, `Starving`, `Satiated`, or `Bloated`. From `Hungry` on,
  the word gets more urgent as it worsens: `Hungry` in mild yellow, `Weak` in strong orange,
  `Fainting` and `Starving` in alarming reds. On a narrow screen it is never dropped, and
  `Satiated` and `Bloated` are among the last things to go (see [The
  screen](the-screen.md#the-screen)).
- A [message](the-screen.md#messages) tells you when it changes: "You are getting hungry.",
  "You feel weak from hunger. You should eat soon.", "You feel faint from hunger. Eat now, or you
  will starve!", "You are starving! Eat, or you will die!", and when you eat, "You are no longer
  hungry.", "You feel satiated.", "You feel bloated."
- **Getting hungrier stops you.** When you become `Hungry`, `Weak`, `Fainting`, or `Starving`,
  the game waits at `--more--` until you have read it, even if the message fits on the line. A
  held key does not walk you past it.
- **A journey asks first.** A step on the [overworld](overworld.md#travel) takes much longer
  than a step anywhere else. While you are `Fainting` or `Starving`, every step there asks
  whether to go on. Staying takes no time, and `Esc` stays. The question depends on how hungry
  you are, not on whether this step would kill you. With [Long fast](talents.md#talent-long-fast)
  you are asked too.

To eat, see [Food and corpses](food.md).
