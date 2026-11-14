# Experience and levelling up

Killing monsters earns **experience points** (XP), and enough of them make you a higher **level**.
Your level (`Lv`) is in the [status bar](the-screen.md#the-screen). The
[character screen](character.md) (`@`) shows it with your XP and the XP at which the next level
comes, like `XP  85 / 128`. Dying shows the level you had reached.

## Earning experience

You get XP for:

- the kills you make yourself: with your weapon up close, with a shot, or with something you throw
  (see [Combat](combat.md) and [Ranged combat](ranged.md));
- a [creature](creatures.md) that [bleeds](status-effects.md#bleeding) to death from a wound you
  gave it (see [Status effects](status-effects.md));
- [quests](quests.md) you complete.

A creature that dies to a [trap](traps.md#finding-traps) or to another creature, or to a wound
another creature gave it, gives you nothing. Neither does anything else, such as a kill you merely
watched.

Every kind of [creature](creatures.md) has a **level** and an **XP value**. A creature is worth
about 5 XP per level plus 10; the level curve is built on that. The vermin are worth less, and some
creatures are not meant to be fought. A few are set to pay more or less than that, and the table
then shows what a kill pays and how, `45 (30 × 150 %)`:

{{#include generated/experience-creatures.md}}

## Experience for something new

Meeting something new teaches you a little.

- The first time you kill a kind of creature yourself, it is worth its XP once more, however far
  below you it is ("You learn something of kobolds.").
- The first time you arrive on a level, you gain a little for finding your way there ("You gain 10
  experience for finding your way here.").
- The first time you learn a spell by reading its [grimoire](magic.md#learning-a-spell), you gain 5 XP for each level of the spell
  (at least 5): 55 for a level 11 spell, 350 for a level 70 one. Spells a class gives you cost
  nothing and pay nothing; a failed reading pays nothing.

Only the first time counts. A second [kobold](creatures.md#creature-kobold), or a level you have
been on before, is worth nothing more.

{{#include generated/experience-discovery.md}}

## Fewer points for weak monsters

The further a monster is below you in level, the less its death is worth. A monster of your own
level gives its whole value.

- **Below you**, it gives its XP times one minus the difference in levels over a number that grows
  with your level (the table below). Once the monster is that many levels below you, it is
  **grey** and gives nothing. Any kill that is not grey gives at least 1 XP.
- **Above you**, each level it has over you adds 5% to its value, up to four levels (+20%).

{{#include generated/experience-zd.md}}

For a level 7 character (grey five levels down), a monster worth 100 XP at its own level gives:

{{#include generated/experience-scaling.md}}

So a [kobold shaman](creatures.md#creature-kobold-shaman) is worth less every level you gain:

{{#include generated/experience-example.md}}

## Levels

Every [class](classes.md) stops at level 60, its last, except the [bard](classes.md#bard), who stops
at 30 (the table below says so for each). The road there is long. A bard at 30 keeps every point of
XP earned after it. One who becomes a [hunter](prestige-classes.md#prestige-classes) gains at once
every level that XP buys, from level 31 on. The curve itself runs on to level 100, for levels a
class does not give.

Each level costs a number of kills of [creatures](creatures.md) of your own level: 26 for level 2,
about 110 a level by level 10, and more for every level after that, up to 243 for level 60 and 323
for level 100. The first levels come faster:

| Level | Share of the full price |
|---:|---:|
| 2 | 25 % (6 kills) |
| 3 | 30 % |
| 4 | 35 % |
| 5 | 40 % |
| 6 | 50 % |
| 7 | 60 % |
| 8 | 70 % |
| 9 | 80 % |
| 10 | 90 % |
| 11 | 95 % |
| 12 on | 100 % |

That is about 1,700 such kills to reach level 20, and 9,700 to reach level 60. Real counts run
higher, because weaker monsters are worth less and grey ones nothing. At your last level your XP
still counts, but buys nothing more. Gaining a level says "You feel more experienced." and "Welcome
to level N." One kill can bring several levels at once.

Your **[race](races.md) and [class](classes.md)** change how much XP every level needs. Long-lived
races and the patient callings need more; short-lived races and everyday folk need less. The two
multiply, so an [elf](races.md#race-elf) [wizard](classes.md#wizard) needs 175 % of the XP below,
and a [half-orc](races.md#race-half-orc) [commoner](classes.md#commoner) 72 %.
[Character creation](character.md) says "Levels come slowly." or "Levels come quickly." where it
matters:

{{#include generated/experience-factors.md}}

The XP needed for each level, for a character of 100 % (a [human](races.md#race-human)
[warrior](classes.md#warrior), say) and for the quickest and the slowest race and class. The table
also shows what each level grows (see [Levelling up](#levelling-up)) and the classes
that stop there:

{{#include generated/experience-levels.md}}

## Levelling up

**Hit points.** Each level up to 50 raises your maximum
**[hit points](hit-points.md#hit-points)**, and the hit points you have now, by half your
[class](classes.md)'s hit points plus your [Toughness](stats.md#stat-toughness)'s share, at least 1.
From level 51 on, levels add no hit points, so hit point pools do not run away.

**Stats.** Each level gives every [stat](stats.md) a chance to rise, by what you have done with it
since (see [Stats](stats.md#raising-your-stats)).

**Mana.** Up to level 60, each level raises your **[mana](mana.md#how-much-mana-you-have)** by half your class's
mana plus what the better of your [Intelligence](stats.md#stat-intelligence) and
[Willpower](stats.md#stat-willpower) adds, if that comes to anything (see [Mana](mana.md)).

**Skill rolls.** Every level gives **[skill rolls](skills.md#improving-a-skill)**: two for most classes,
three for a [thief](classes.md#thief) or a [ranger](classes.md#ranger), four for a
[bard](classes.md#bard), one more for a [human](races.md#race-human), and more for a high
[Intelligence](stats.md#stat-intelligence). You spend them on your trained [skills](skills.md)
before you do anything else. How much each roll adds depends on how much you have used the skill
since its last roll (see [Skills](skills.md#improving-a-skill)).

See also: [Talents](talents.md), [Skills](skills.md), and
[Prestige classes](prestige-classes.md).
