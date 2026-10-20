# Diseases

Some harm comes slowly. A filthy bite, a night in the marshes, or a bad meal can leave you with a
**disease**: you will not know it at first, but after a while you fall ill, and it gets worse day
by day until your body fights it off. Some diseases never go by themselves, and one of them can
kill.

Whatever the disease, it shows the same way: as **Diseased**, after your hit points in the
[status bar](controls.md#the-screen) and among your status effects on the
[character screen](character.md). Which disease it is, you learn from its symptoms, and once it
has run its course for a day, the character screen names it (`Diseased  Filth fever`).

{{#include generated/diseases.md}}

"Resisted with" is the stat you fight the disease with, and how often an average score of it
(10) wins a check against it. "—" under "Caught by": nothing gives it yet.

## How a disease goes

- **Catching it.** When a disease comes near you (a bite, the marsh, rotten food), your body
  tries to fight it off at once: a **check**. Win it and nothing happens. Lose it and you have
  caught the disease. Nothing tells you either way.
- **Incubation.** For a few days nothing shows. Then it breaks out, with a message of its own
  ("You feel feverish."), and from then on you are **Diseased**.
- **Every day** after that, your body fights it again: another check. Losing one costs you: the
  disease takes points off some of your [stats](stats.md) (**stat damage**), and you are told how
  it worsens ("Fever wracks you."). Winning one costs nothing.
- **Shaking it off.** Win two checks in a row and the disease is gone ("Your fever breaks.").
  A loss starts the count over.
- **Some never go.** Grave rot does not go by itself, whatever your checks; only a cure will end
  it (below).

You can carry several diseases at once. Each runs its own course; you are Diseased while any of
them shows.

## Checks

A check is rolled like an attack (see [Combat](combat.md)): two twelve-sided dice, plus a bonus
from your score in the disease's stat, against how hard the disease is. Most diseases are fought
with **Toughness**, brain fever with **Willpower**. A roll of 2 always loses, a roll of 24 always
wins. The game never shows the dice, the bonus, or how hard a disease is: you see what comes of
it.

## Stat damage

The points a disease takes off your stats count everywhere your stats do: a weaker Strength hits
softer and carries less, a clumsier Dexterity dodges worse. The character screen shows your
stats with the damage taken off, in red.

- **Toughness damage lowers your most hit points**, as Toughness gives them (see
  [Combat](combat.md#hit-points)). If you have more hit points than your new most, you are left
  with the most.
- **It never kills you by itself**: no stat goes below 1, and a disease's damage never takes your
  last hit point, except for a disease that kills.
- **It mends slowly**: once no disease shows, a point of each damaged stat comes back every day.
  While you are still Diseased, nothing mends.

## Healing while Diseased

Your hit points come back at half the pace while you are Diseased, and marsh ague slows them even
more (see [Healing](healing.md)). Resting still doubles the pace.

## Where diseases come from

- **Bites**: some creatures carry a disease in their bite. Not every bite gives it to you, and
  you may still fight it off.
- **Marshes and swamps**: crossing the marshes of a swamp, or a swamp on the
  [overworld](overworld.md), may give you marsh ague. The longer you spend in them, the likelier
  it gets.
- **Rotten food**: eating rotten food (of any kind) may give you grave rot.

## Grave rot

Grave rot never goes by itself, and it can kill: when its damage takes your Toughness to nothing,
you die of it, and the death screen says "Died of disease". A grave rot caught is a race against
the days: to a shop, for a cure.

## Cures

An **angelica root** (`%`) cures every disease you have the moment you eat it (`E`), grave rot
too, and even one that has not broken out yet: "The rot is gone from your flesh." It feeds you a
little, too. Shops that sell food sell it (a general store or a provisioner, found in the
Depths), and Grimbold's General Store in the courtyard has one. Potions and priests will come.

## In the message line

A disease breaking out and a disease worsening stop a [rest](resting.md), like any news. A disease
leaving you ("Your fever breaks.") is written to the message line but lets you rest on.

See also [Status effects](status-effects.md) and [Stats](stats.md).
