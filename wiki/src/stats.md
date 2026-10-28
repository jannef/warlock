# Stats

You have six **stats**, each a number that says how strong, quick, hardy, clever, steadfast, or
striking you are. They are inspired by the abilities of Dungeons & Dragons, and three of them go
by other names here: [Toughness](stats.md#stat-toughness) is D&D's Constitution, [Willpower](stats.md#stat-willpower) its Wisdom, and [Presence](stats.md#stat-presence) its
Charisma.

{{#include generated/stats.md}}

Your stats are set when you create your character at the start of a run: your
[race](races.md), your gender, and your [class](classes.md) each add to or take from the average
of 10 (see [Your character](character.md)). After that they grow by use: see
[Training](#training) below.

## Seeing your stats

Press `@` to open the **character screen**. It shows your name, your [race](races.md), gender, and [class](classes.md),
your age, birthday, and [birth sign](horoscope.md), your stats in the order above (with their
short names; a stat that a [status effect](status-effects.md#how-they-work) or a [disease](diseases.md#how-a-disease-goes) lowers is shown lowered, in red), your [hit
points](combat.md#hit-points), [mana](mana.md#how-much) (see [Mana](mana.md)), and [Defence](combat.md#defence) (see [Combat](combat.md)), your weight: your own, plus what you carry (see [Objects and your pack](objects.md)), and how much you can carry (see
[Burden](burden.md)). Looking takes no time; press `@` or `Esc` to close it.

[Creatures](creatures.md) have stats too, but you cannot see theirs in the game: they are on
[Creatures in numbers](creature-numbers.md).

## What they do

In a fight (see [Combat](combat.md)), under a load (see [Burden](burden.md)), at a door (see
[Doors](doors.md)), and in your [mana](mana.md#how-much) (see [Mana](mana.md)):

- **[Strength](stats.md#stat-strength)** helps you [hit](combat.md#hitting) up close and adds to the [damage](combat.md#damage) of your blows, and draws a bow. It
  counts double toward how much you can [carry](burden.md#how-much-you-can-carry), and the stronger you are, the sooner you kick a
  [door](doors.md#kicking) in and [dig](digging.md#digging) through a wall.
- **[Dexterity](stats.md#stat-dexterity)** helps you aim a shot or a throw and dodge blows: it adds to your [Defence](combat.md#defence) (less
  under a heavy load or heavy armour).
- **[Toughness](stats.md#stat-toughness)** gives you [hit points](combat.md#hit-points), and makes them come back sooner. It counts toward how much
  you can [carry](burden.md#how-much-you-can-carry), and fights off most [diseases](diseases.md).
- **[Intelligence](stats.md#stat-intelligence)** helps you find hidden things when you search ([Searching](searching.md)), and
  adds to your [mana](mana.md).
- **[Willpower](stats.md#stat-willpower)** helps you [notice hidden things in passing](searching.md#noticing-in-passing), adds to your [mana](mana.md) and makes
  it come back sooner, and fights off a few [diseases](diseases.md#how-a-disease-goes) (brain fever).
- **[Presence](stats.md#stat-presence)** moves [shop](shops.md) prices in your favour, or against you if it is low, and adds
  to your [mana](mana.md#how-much).

### Modifiers

Most rules read a stat's **modifier** rather than its score, as in D&D: the score less 10,
halved and rounded down. The game never shows a modifier; here they are:

{{#include generated/stat-modifiers.md}}

What reads the modifier and what reads the score, exactly:

{{#include generated/stat-uses.md}}

### Potentials

Every stat also has a **potential**, which the game never shows: how far it can grow for good.
A [corpse](food.md#corpses) that raises a stat never raises it past its potential ([magic items](magic-items.md) raise it only while
you wear them, potential or not); training slows down there but goes on (see
[Training](#training)). Yours are set at the start of a run: the potential your [race](races.md)
gives, rolled once, and never below your starting score (a score above it raises the potential
to the score):

{{#include generated/potentials.md}}

Besides training past it, only an **elixir** raises a potential: it raises its stat by 1 and
the potential by 1 or 2 more (see [Potions](potions.md#elixirs)).

### What changes them

Stats do not change how far you see or how quietly you walk. Strong [poison](status-effects.md#poisoned) lowers your [Strength](stats.md#stat-strength)
and [Dexterity](stats.md#stat-dexterity) by one while it lasts, for everything they do (see
[Status effects](status-effects.md)), and [hunger](hunger.md#how-hungry-you-are) saps them for a while ([Weak](hunger.md#how-hungry-you-are) lowers your
Strength and Dexterity; see [Hunger](hunger.md)). [Diseases](diseases.md#how-a-disease-goes) take points off your stats (**stat
damage**), which come back a point a day once no disease shows (see [Diseases](diseases.md)).
Eating the [corpse](food.md#corpses) of some [creatures](creatures.md) may raise one for good, as far as it can grow (see
[Food and corpses](food.md)), and so may [training](#training) at a new level. The touch of some
undead lowers one for good ([Drained stats](#drained-stats)). Nothing else raises or lowers them
for good.

## Training

You never pick a stat to raise. Instead your stats grow with what you do: fighting up close,
landing [criticals](combat.md#criticals), and blocking with a shield train [Strength](stats.md#stat-strength), dodging, parrying, and shooting
[Dexterity](stats.md#stat-dexterity), being hurt and felling worthy foes [Toughness](stats.md#stat-toughness), reading and [searching](searching.md#searching-1) [Intelligence](stats.md#stat-intelligence),
enduring, noticing, and casting [Willpower](stats.md#stat-willpower), talking and trading [Presence](stats.md#stat-presence). Each such use may
earn the stat a **training mark** (marks never fade and have no limit), and every time you gain
an [experience level](experience.md), each stat in turn gets one chance to rise by a point; so
it does once a day, at 06:00, between levels too (see [The daily check](#the-daily-check)). The
more marks it holds, the likelier; a rise uses up all its marks, a stat that did not rise keeps
them for the next check. A stat rises by at most one point a level, and one a day. The game says
only "You feel stronger." (quicker, tougher, sharper, more resolute, more confident); the new
score shows on the character screen. A rise in [Toughness](stats.md#stat-toughness) brings its [hit point](combat.md#hit-points) at once.

What earns a mark, and how likely:

{{#include generated/stat-training.md}}

Asking someone about a topic takes no time, so it trains nothing; a long action counts only
when it lasts more than one turn and is not interrupted.

At a level, a stat rises with chance **marks / (marks + need)**, where need is 20 + 3 × the
stat's score. At or above the stat's [potential](stats.md#potentials) the chance is divided by 4, and a rise there
lifts the potential with the score: a long career outgrows its nature slowly, never all at once.
Below the [potential](stats.md#potentials):

{{#include generated/stat-checks.md}}

### The daily check

Every game day at **06:00** (the hour a [rest](resting.md#how-resting-works) until dawn ends), each stat gets the same check
again, with a stiffer need: the chance is **marks / (marks + 3 × need)**, divided by 4 at or
above the [potential](stats.md#potentials) as at a level. Nothing else changes: a rise uses up the marks and says the
same words, a failure keeps them, and the level's check still comes at every level. A rise at
06:00 stops a [rest](resting.md#how-resting-works). The check comes whether you are awake or [resting](resting.md#how-resting-works), once a day, at that hour
and no other; nothing decays and nothing is capped, so the only limit is the clock. The
**[Hard training](talents.md#talent-hard-training)** [talent](talents.md) makes the need 2 × instead of 3 ×. Below the [potential](stats.md#potentials):

{{#include generated/stat-daily-checks.md}}

With 8 to 15 marks a day in a stat you use, [Strength](stats.md#stat-strength) 14 against a [potential](stats.md#potentials) of 17, and a level
every two days, the daily check adds about 4 to 8 rises to the levels' over a career of 120
game days, most of them before the potential; past it a stat rises between levels about once
in 25 days.

## Drained stats

Some of the undead drink the life out of the living. When such a [creature](creatures.md)'s touch hits you and
you live, you make a **check** of a stat ([Willpower](stats.md#stat-willpower), unless the creature says otherwise): 2d12
plus the stat's [modifier](stats.md#modifiers) against the drain's difficulty, as a [disease](diseases.md#how-a-disease-goes)'s check. If you hold, "A
deathly chill passes through you." and nothing is lost. If not, points come off one of your stats
**for good** ("You feel your strength drain away!"): your score is simply lower from then on,
for everything it does. A drain of [Toughness](stats.md#stat-toughness) lowers your most [hit points](combat.md#hit-points), one of [Intelligence](stats.md#stat-intelligence),
[Willpower](stats.md#stat-willpower), or [Presence](stats.md#stat-presence) your most [mana](mana.md#how-much); it never kills you, and no drain takes a score below 3
(nor below what the [stat damage](diseases.md#stat-damage) on it needs). [Resting](resting.md#how-resting-works) never brings drained points back.

Your **[potential](stats.md#potentials)** stays where it was, so a drained stat may grow back by
[training](#training) and corpses like any other. A temple's **mending** gives every drained
point back at once, with any [stat damage](diseases.md#stat-damage) (see [Services](npcs.md#services)). [Talents](talents.md) help you
hold: **[Unbroken soul](talents.md#talent-unbroken-soul)** (+4 to the check), **[Iron will](talents.md#talent-iron-will)** (+2 to every [Willpower](stats.md#stat-willpower) check), and the
[paladin](classes.md#paladin)'s **[Divine grace](talents.md#talent-divine-grace)** (+2 to the check); see [Talents](talents.md).

The [creatures](creatures.md) that drain:

{{#include generated/drain-attacks.md}}
