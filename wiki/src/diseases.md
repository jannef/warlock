# Diseases

Some harm comes slowly. A filthy bite, a night in the marshes, or a bad meal can leave you with a
**disease**. You will not know it at first, but after a while you fall ill, and it gets worse day
by day until your body fights it off. Some diseases never go by themselves, and one of them can
kill.

Every disease shows the same way: as **[Diseased](status-list.md#status-diseased)**, after your
[hit points](hit-points.md#hit-points) in the [status bar](the-screen.md#the-screen), and among
your [status effects](status-effects.md#how-they-work) on the [character screen](character.md). You
learn which disease it is from its symptoms. Once it has run for a day, the character screen names
it (`Diseased  Filth fever`).

{{#include generated/diseases.md}}

"Resisted with" is the [stat](stats.md) you fight the disease with, and how often an average score
(10) wins a check against it. A "—" under "Caught by" means nothing gives it yet.

## How a disease goes

- **Catching it.** When a disease comes near you (a bite, the marsh,
  [rotten food](food.md#food-spoils)), your body tries to fight it off at once. This is a
  **check**. Win it and nothing happens. Lose it and you have caught the disease. You are not told
  either way.
- **Incubation.** For a few days nothing shows. Then it breaks out, with a message of its own ("You
  feel feverish."), and from then on you are **[Diseased](status-list.md#status-diseased)**.
- **Every day** after that, your body fights it again with another check. Winning costs nothing.
  Losing means the disease takes points off some of your [stats](stats.md)
  (**[stat damage](#stat-damage)**), and you are told how it worsens ("Fever wracks you.").
- **Shaking it off.** Win two checks in a row and the disease is gone ("Your fever breaks."). A
  loss starts the count over.
- **Some never go.** [Grave rot](#disease-grave-rot) does not go by itself, whatever your checks.
  Only a [cure](#curing-a-disease) ends it (below).

You can carry several diseases at once. Each runs its own course, and you are Diseased while any
of them shows.

## Checks

A check is rolled like an attack (see [Combat](combat.md#hitting)). You roll two twelve-sided dice
and add the modifier of your score in the disease's stat (see [Stats](stats.md#modifiers)). You win
if the sum reaches the disease's **difficulty**. A roll of 2 always loses, and a roll of 24 always
wins.

Most diseases are fought with **[Toughness](stats.md#stat-toughness)**;
[brain fever](#disease-brain-fever) is fought with **[Willpower](stats.md#stat-willpower)**.

Some things add to every check, both when you catch a disease and when you fight it off:

- [Ratfolk](races.md#race-ratfolk) add 4 (see [Races](races.md#the-evil-races)).
- Talents such as [Strong constitution](talents.md#talent-strong-constitution) add theirs (see
  [Talents](talents.md)).
- Magic that guards your health adds its bonus (see [Magic items](magic-items.md)).

The game never shows the dice, the bonus, or the difficulty: you only see what comes of it. Here
are the exact odds by your score in the stat, before any race's bonus:

{{#include generated/disease-checks.md}}

## Stat damage

The points a disease takes off your [stats](stats.md) count everywhere your stats do. A weaker
[Strength](stats.md#stat-strength) hits softer and carries less; a clumsier
[Dexterity](stats.md#stat-dexterity) dodges worse. The [character screen](the-screen.md#character)
shows your stats with the damage taken off, in red.

- **[Toughness](stats.md#stat-toughness) damage lowers your maximum
  [hit points](hit-points.md#hit-points)**, since Toughness gives them. If you have more hit points
  than your new maximum, you drop to the maximum.
- **It never kills you by itself.** No stat goes below 1, and a disease's damage never takes your
  last hit point, unless it is a disease that kills.
- **It mends slowly.** Once no disease shows, one point of each damaged stat comes back every day.
  While you are still [Diseased](status-list.md#status-diseased), nothing mends.

## Healing while Diseased

While you are [Diseased](status-list.md#status-diseased), your
[hit points](hit-points.md#hit-points) come back at half the pace, and
[marsh ague](#disease-marsh-ague) slows them even more (see [Healing](healing.md)).
[Resting](resting.md#how-resting-works) still doubles the pace.

## Where diseases come from

- **Bites.** Some [creatures](creatures.md) carry a disease in their bite. Not every bite passes it
  on, and you may still fight it off. The table above says whose bite, and how often a hit passes
  it on.
- **Marshes and swamps.** Crossing the [marshes](terrain.md#terrain-marsh) of a swamp, or a swamp
  on the [overworld](overworld.md), may give you [marsh ague](#disease-marsh-ague). The longer you
  stay, the likelier it gets: the table above gives the chance for each hour spent there.
- **Rotten food.** Eating [rotten food](food.md#food-spoils) of any kind may give you
  [grave rot](#disease-grave-rot).

## Grave rot

Grave rot never goes by itself, and it can kill. When its damage takes your
[Toughness](stats.md#stat-toughness) to nothing, you die, and the death screen says "Died of
disease". Once you have caught it, you are racing the days to reach a [shop](shops.md) for a
[cure](#curing-a-disease).

## Mummy rot

The touch of a [mummy](creatures.md#creature-mummy) may give you mummy rot. Like grave rot, it
never goes by itself, but it is slower: every few days it dries a little more
[Toughness](stats.md#stat-toughness) out of you. It does not kill, but the Toughness it takes stays
gone until a [cure](#curing-a-disease) ends it.

## Burrowing worms

The fist of a [wormhost](creatures.md#creature-wormhost) may leave its green worms in you. They are
quick: a day later they are burrowing, and every day they take
[Toughness](stats.md#stat-toughness), wits, and blood. Win two checks in a row and you are rid of
them. But they can kill, as [grave rot](#disease-grave-rot) can, so a [cure](#curing-a-disease) is the safer
way.

## Curing a disease

An **angelica root** (`%`) cures every disease you have the moment you eat it (`e`). That includes
[grave rot](#disease-grave-rot), and even a disease that has not broken out yet: "The rot is gone
from your flesh." It feeds you a little, too. [Shops](shops.md) that sell food sell it (a general
store or a provisioner, found in the dungeons), and Ælfric's General Store in
[Cēaptūn](ascmearc.md#the-places) has one.

A [potion of cure disease](potions.md#potion-of-cure-disease) ends every disease at once, and so
does the spell [Remove disease](magic.md#spell-remove-disease).

A [priest](class-priest.md)'s prayer against [disease](prayer.md#prayer-disease) (from level 12)
ends every disease, the brewing ones too.

[Horn's grace](abilities.md#ability-horns-grace), the gift of those born in the year of the
Unicorn, shakes off one disease that shows.

Now and then a [forgotten altar](altars-forges-springs.md#forgotten-altars) answers a prayer by
cleansing you of every disease.

A temple's **cure** does the same for gold, for every disease. A temple's **mending** gives back
the [stat damage](#stat-damage) at once, and the points the undead drained (see
[Services](npcs.md#services) and [Drained stats](stats.md#drained-stats)).

The **[Heal](skills.md#skill-heal)** skill eases a disease that shows. You may try as often as you
like, but it never cures (see [Treating yourself](skill-uses.md#treating-yourself)).

## In the message line

A disease breaking out or worsening stops a [rest](resting.md), like any news. A disease leaving
you ("Your fever breaks.") shows in the message line but lets you rest on.

See also [Status effects](status-effects.md) and [Stats](stats.md).
