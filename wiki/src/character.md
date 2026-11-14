# Your character

Every new run starts with **character creation**. Choose **New run** on the main menu and you
decide who you are, one step at a time, before you set out into [the Ascmearc](ascmearc.md).

1. **Race**: your people. Your race shapes your [stats](stats.md), gives you your
   [eyes](vision.md#the-four-types), and sets how old you can be (see [Races](races.md)).
2. **Gender**: a small push to one [stat](stats.md) (below).
3. **Class**: what you did before the adventure. It shapes your [stats](stats.md) again and
   trains you in a handful of [skills](skills.md) (see [Classes](classes.md)).
4. **Talents**: pick two [talents](talents.md) from those your character may take. A bard or a
   human picks three, a human bard four. A letter or `Enter` marks one, **Random** lets chance
   pick, and **Done** goes on.
5. **Name**: type your character's name, up to 20 characters (below).
6. **Summary**: everything you chose, and what follows from it. It shows your name, race,
   gender, class, and talents; your age, birthday, and [birth sign](horoscope.md#what-a-sign-does);
   the [stats](stats.md) you start with, and beside them the [skills](skills.md) you are trained
   in; your [starting kit](starting-kit.md) item by item; the [alignment](alignment.md) you start
   in; your [vision](vision.md#the-four-types); and where you start. **Start the run** begins it.
   **Name**, **Race**, **Gender**, **Class**, and **Talents** take you back to that step.

Each step lists its options with a short description. Races and classes also show how they change
your stats. Move with the arrow keys (or `j` and `k`), press `Enter` to choose and go on, and
`Esc` to go back one step. The steps remember what you chose when you go back and forward again.

Every step also has **Random**, which leaves the choice to chance. Come back and press it again
for another pick. On the Talents step, Random always gives the same picks for the same character.

## Name

On the name step every key types its letter (`j` and `k` too), as long as the game's font can draw
it. `Backspace` erases and `Enter` accepts. `Tab` puts in one of your people's names, picked at
random by race and gender; press it again for another.

A name cannot be empty. If you accept an empty field, you get the random name it shows, dimmed.
These keys are fixed (see [Text fields](controls.md#text-fields)).

Your name appears on the character screen, the summary, the death screen, and the list of saved
characters (see [Saving](saving.md)). It changes nothing else.

## Gender

{{#include generated/genders.md}}

For **other**, fate chooses the [stat](stats.md). Character creation does not tell you which, and
the summary shows only your final scores. Once the run has begun, the character screen names it.

## Age, birthday, and sign

You do not choose these. Your age is picked at random from your race's range, and your birthday
from the whole year. The year you were born gives you your birth sign: an animal and an element
from the [horoscope](horoscope.md), such as the Fire Manticore. Your sign changes your
[stats](stats.md) a little too.

## How your stats add up

Each of your six [stats](stats.md) starts at 10, the average. Then the changes are added in
order: your race's, your gender's, your class's, and your sign's animal's and element's. Nothing
goes below 1.

Some changes are **dice**, rolled when you start:

- `1d3` is one three-sided die: 1, 2, or 3.
- `2d2+1` is two two-sided dice plus 1: 3, 4, or 5.
- `1d4-1` is one four-sided die minus 1: 0 to 3.
- `−1d2` takes one two-sided die away: −1 or −2.
- A plain number, such as `+2` or `−1`, is always that.

Classes roll their key stats. A [warrior](classes.md#warrior)'s
[Strength](stats.md#stat-strength) is `1d3+1`, a [wizard](classes.md#wizard)'s
[Intelligence](stats.md#stat-intelligence) `2d2+1`. [Races](races.md) are mostly plain numbers,
with a few dice. The [horoscope](horoscope.md) rolls none. The summary shows only the final
scores. Change a choice and the dice are rolled again.

The choices are not balanced, and they are not meant to be. Some combinations are poor on
purpose: a [halfling](races.md#race-halfling) [commoner](classes.md#commoner) or a
[gnome](races.md#race-gnome) [berserker](classes.md#berserker) has a hard start. That is part of
the fun. There is one consolation. Your race and class also set how much
[experience](experience.md#levels) each level needs. Long-lived [races](races.md) and patient
callings level slowly; short-lived races and everyday folk level quickly.

## Seeing who you are

The character screen (`@`) shows:

- your [race](races.md), gender, and [class](classes.md), your age, birthday, and
  [sign](horoscope.md);
- your [stats](stats.md), with your attacks beside them: what you hit with, your bonus to hit,
  your [damage](combat.md#damage), and a [missile](ranged.md#getting-ready)'s range (see
  [Combat](combat.md#hitting));
- your [alignment](alignment.md), your [level](experience.md#levels), and your experience (see
  [Experience](experience.md));
- your [hit points](hit-points.md#hit-points): your [Toughness](stats.md#stat-toughness) plus what
  your race and class give;
- your [mana](mana.md), [Defence](combat.md#defence), and [Protection](combat.md#protection);
- each [resource](resources.md) you hold;
- what ails you (see [Status effects](status-effects.md)) and the diseases you know you have (see
  [Diseases](diseases.md));
- what you weigh and carry, against what you can carry (see [Burden](burden.md)).

`A`, there or anywhere in play, shows your [skills](skills.md). `W` shows your
[ranks](weapon-skills.md#raising-a-weapon-skill) in each [weapon skill](weapon-skills.md), which your race
and class start you with, and in each [fighting style](fighting-styles.md), which your class
starts you with.

Your race, gender, class, and sign never change once a run has started. You do get a year older on
every birthday.

See also: [Classes](classes.md), [Stats](stats.md), and [What's in the game](features.md).
