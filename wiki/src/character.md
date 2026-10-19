# Your character

Every new run starts with **character creation**: after you pick a scenario and a seed on the
New run screen, you choose who you are, one step at a time.

1. **Race**: your people, which shapes your stats, gives you your eyes, and sets how old you can
   be (see [Races](races.md)).
2. **Gender**: a small push to one stat (below).
3. **Class**: what you did before the adventure, which shapes your stats again and trains you
   in a handful of [skills](skills.md) (see [Classes](classes.md)).
4. **Summary**: your race, gender, and class, your age, birthday, and birth sign, the
   [stats](stats.md) you will start with and, beside them, the [skills](skills.md) you are
   trained in, your vision, and the scenario and seed. **Start the run** begins it; **Race**,
   **Gender**, and **Class** take you back to that step.

Each step lists its options with a short description, and for races and classes how they change
your stats. Move with the arrow keys (or `j` and `k`), press `Enter` to choose and go on, and `Esc`
to go back one step. When you go back and forward again, the steps remember what you chose.
Every step also has **Random**, which picks an option for you. Random choices come from the run's
seed, so the same seed rolls the same choices; pressing Random again rolls again.

## Gender

{{#include generated/genders.md}}

For **other**, the stat comes from the run's seed: the same seed always raises the same stat.
The gender step names it for the seed you chose, and so do the summary and the character screen.

## Age, birthday, and sign

You do not choose these. Your age is picked at random from your race's range, and your birthday
from the whole year. The year you were born gives you your birth sign: an animal and an element
from the [horoscope](horoscope.md), such as the Fire Manticore. Your sign changes your stats a
little too.

## How your stats add up

Each of your six [stats](stats.md) starts at 10, the average. Your race's change is added, then
your gender's, then your class's, then your sign's animal's and element's. Nothing goes below 1.

Some changes are **dice**, rolled in the background when you start, the way ADOM does it:

- `1d3` is one three-sided die: 1, 2, or 3.
- `2d2+1` is two two-sided dice plus 1: 3, 4, or 5.
- `1d4-1` is one four-sided die minus 1: 0 to 3.
- `−1d2` takes one two-sided die away: −1 or −2.
- A plain number, such as `+2` or `−1`, is always that.

Classes roll their key stats: a warrior's Strength is `1d3+1`, a wizard's Intelligence `2d2+1`.
Races are mostly plain numbers, with a little dice; the horoscope rolls none. The rolls come from
the run's seed and your choices, so the same seed and the same choices always make the same
character. The summary shows only the final scores, after the rolls; change a choice and the dice
are rolled again.

The choices are not balanced, and they are not meant to be. Some combinations are poor on
purpose: a halfling commoner, or a gnome barbarian, makes for a hard start. That is part of the
fun.

## Seeing who you are

The character screen (`@`) shows your race, gender, and class, your age, birthday, and sign,
your stats, your level and experience (see [Experience](experience.md)), and your hit points (your Toughness plus what your race and class give; see
[Combat](combat.md#hit-points)) and Defence, what ails you (see
[Status effects](status-effects.md)) and which diseases you know you have (see
[Diseases](diseases.md)), and what you weigh and carry against what you can (see
[Burden](burden.md)). `S` there, or anywhere in play, shows your [skills](skills.md). The game
does not change your race, gender, class, or sign once a run has started; you get a year older
on every birthday.
