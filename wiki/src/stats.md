# Stats

You have six **stats**, each a number that says how strong, quick, hardy, clever, steadfast, or
striking you are. They are inspired by the abilities of Dungeons & Dragons, and three of them go
by other names here: Toughness is D&D's Constitution, Willpower its Wisdom, and Presence its
Charisma.

{{#include generated/stats.md}}

Your stats are set when you create your character at the start of a run: your
[race](races.md), your gender, and your [class](classes.md) each add to or take from the average
of 10 (see [Your character](character.md)). After that they grow by use: see
[Training](#training) below.

## Seeing your stats

Press `@` to open the **character screen**. It shows your name, your race, gender, and class,
your age, birthday, and [birth sign](horoscope.md), your stats in the order above (with their
short names; a stat that a status effect or a disease lowers is shown lowered, in red), your hit
points, mana (see [Mana](mana.md)), and Defence (see [Combat](combat.md)), your weight: your own, plus what you carry (see [Objects and your pack](objects.md)), and how much you can carry (see
[Burden](burden.md)). Looking takes no time; press `@` or `Esc` to close it.

Creatures have stats too, but you cannot see theirs in the game: they are on
[Creatures in numbers](creature-numbers.md).

## What they do

In a fight (see [Combat](combat.md)), under a load (see [Burden](burden.md)), at a door (see
[Doors](doors.md)), and in your mana (see [Mana](mana.md)):

- **Strength** helps you hit up close and adds to the damage of your blows, and draws a bow. It
  counts double toward how much you can carry, and the stronger you are, the sooner you kick a
  door in and dig through a wall.
- **Dexterity** helps you aim a shot or a throw and dodge blows: it adds to your Defence (less
  under a heavy load or heavy armour).
- **Toughness** gives you hit points, and makes them come back sooner. It counts toward how much
  you can carry, and fights off most [diseases](diseases.md).
- **Intelligence** helps you find hidden things when you search ([Searching](searching.md)), and
  adds to your [mana](mana.md).
- **Willpower** helps you notice hidden things in passing, adds to your [mana](mana.md) and makes
  it come back sooner, and fights off a few diseases (brain fever).
- **Presence** moves [shop](shops.md) prices in your favour, or against you if it is low, and adds
  to your mana.

### Modifiers

Most rules read a stat's **modifier** rather than its score, as in D&D: the score less 10,
halved and rounded down. The game never shows a modifier; here they are:

{{#include generated/stat-modifiers.md}}

What reads the modifier and what reads the score, exactly:

{{#include generated/stat-uses.md}}

### Potentials

Every stat also has a **potential**, which the game never shows: how far it can grow for good.
A corpse that raises a stat never raises it past its potential (magic items raise it only while
you wear them, potential or not); training slows down there but goes on (see
[Training](#training)). Yours are set at the start of a run: the potential your race
gives, rolled once, and never below your starting score (a score above it raises the potential
to the score):

{{#include generated/potentials.md}}

### What changes them

Stats do not change how far you see or how quietly you walk. Strong poison lowers your Strength
and Dexterity by one while it lasts, for everything they do (see
[Status effects](status-effects.md)), and hunger saps them for a while (Weak lowers your
Strength and Dexterity; see [Hunger](hunger.md)). Diseases take points off your stats (**stat
damage**), which come back a point a day once no disease shows (see [Diseases](diseases.md)).
Eating the corpse of some creatures may raise one for good, as far as it can grow (see
[Food and corpses](food.md)), and so may [training](#training) at a new level. The touch of some
undead lowers one for good ([Drained stats](#drained-stats)). Nothing else raises or lowers them
for good.

## Training

You never pick a stat to raise. Instead your stats grow with what you do: fighting up close,
landing criticals, and blocking with a shield train Strength, dodging, parrying, and shooting
Dexterity, being hurt and felling worthy foes Toughness, reading and searching Intelligence,
enduring, noticing, and casting Willpower, talking and trading Presence. Each such use may
earn the stat a **training mark** (marks never fade and have no limit), and every time you gain
an [experience level](experience.md), each stat in turn gets one chance to rise by a point; so
it does once a day, at 06:00, between levels too (see [The daily check](#the-daily-check)). The
more marks it holds, the likelier; a rise uses up all its marks, a stat that did not rise keeps
them for the next check. A stat rises by at most one point a level, and one a day. The game says
only "You feel stronger." (quicker, tougher, sharper, more resolute, more confident); the new
score shows on the character screen. A rise in Toughness brings its hit point at once.

What earns a mark, and how likely:

{{#include generated/stat-training.md}}

Asking someone about a topic takes no time, so it trains nothing; a long action counts only
when it lasts more than one turn and is not interrupted.

At a level, a stat rises with chance **marks / (marks + need)**, where need is 20 + 3 × the
stat's score. At or above the stat's potential the chance is divided by 4, and a rise there
lifts the potential with the score: a long career outgrows its nature slowly, never all at once.
Below the potential:

{{#include generated/stat-checks.md}}

### The daily check

Every game day at **06:00** (the hour a rest until dawn ends), each stat gets the same check
again, with a stiffer need: the chance is **marks / (marks + 3 × need)**, divided by 4 at or
above the potential as at a level. Nothing else changes: a rise uses up the marks and says the
same words, a failure keeps them, and the level's check still comes at every level. A rise at
06:00 stops a rest. The check comes whether you are awake or resting, once a day, at that hour
and no other; nothing decays and nothing is capped, so the only limit is the clock. The
**Hard training** talent makes the need 2 × instead of 3 ×. Below the potential:

{{#include generated/stat-daily-checks.md}}

With 8 to 15 marks a day in a stat you use, Strength 14 against a potential of 17, and a level
every two days, the daily check adds about 4 to 8 rises to the levels' over a career of 120
game days, most of them before the potential; past it a stat rises between levels about once
in 25 days.

## Drained stats

Some of the undead drink the life out of the living. When such a creature's touch hits you and
you live, you make a **check** of a stat (Willpower, unless the creature says otherwise): 2d12
plus the stat's modifier against the drain's difficulty, as a disease's check. If you hold, "A
deathly chill passes through you." and nothing is lost. If not, points come off one of your stats
**for good** ("You feel your strength drain away!"): your score is simply lower from then on,
for everything it does. A drain of Toughness lowers your most hit points, one of Intelligence,
Willpower, or Presence your most mana; it never kills you, and no drain takes a score below 3
(nor below what the stat damage on it needs). Resting never brings drained points back.

Your **potential** stays where it was, so a drained stat may grow back by
[training](#training) and corpses like any other. A temple's **mending** gives every drained
point back at once, with any stat damage (see [Services](npcs.md#services)). Talents help you
hold: **Unbroken soul** (+4 to the check), **Iron will** (+2 to every Willpower check), and the
paladin's **Divine grace** (+2 to the check); see [Talents](talents.md).

The creatures that drain:

{{#include generated/drain-attacks.md}}
