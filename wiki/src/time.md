# Time and the calendar

The game has a clock that every action that takes time advances. The clock is part of your run and
is saved with it. Nothing depends on real-world time.

## Days

A day has 24 hours, as usual. The status bar shows the time, the date, and the phase of the moon
like this: `14:32 · 12 Rainmoon, Year 1 · ◑ First quarter`.

- **Dawn** is at 06:00 and **dusk** is at 18:00. The sun rises at dawn and sets at dusk; see
  [Sun, moon, and sky](sun.md).
- A new run starts at a time set by its scenario. The default is 08:00 on the first day of the
  first month of year 1.

## The calendar

A year has 12 months of 30 days each: a 360-day year. There are no weeks. Months have names:

{{#include generated/calendar.md}}

Days are numbered 1 to 30 within a month, and years count up from 1.

## The moon's phase

The moon waxes and wanes once a month (see [Sun, moon, and sky](sun.md)). The status bar shows its
phase as a small picture of the moon and a name. The picture shows the lit part of the moon, which
is on the right while the moon waxes and on the left while it wanes:

| Picture | Phase | In a month |
|:---:|---|---|
| `○` | New moon | around the 1st |
| `☽` | Waxing crescent | |
| `◑` | First quarter, and waxing gibbous after it | around the 8th |
| `●` | Full moon | around the 16th |
| `◐` | Waning gibbous, and last quarter after it | around the 23rd |
| `☾` | Waning crescent | |

If the game world has more than one moon, the status bar shows each moon's name and picture
instead, for example `Moon ●  Ilse ☽`.

When the window is too narrow for everything, the status bar leaves out the least important parts
first: the seed, your position, your lantern, your vision, the light at your feet, and the turn
count; then it shortens the moon's phase to its picture, and only then drops it. The time and the
date stay longest, after the name of the level.

## What takes time

Almost everything you do that changes the world takes a turn, and a turn takes game time: seconds
in a dungeon or outdoor area, minutes on the overworld. See [Turns and movement](turns.md) for the
exact lengths, and [Resting](resting.md) for passing a long time at once.

Actions that do not change the world, such as looking around, opening menus, and reading help, take
no time. Refused actions, such as walking into a wall, take no time either.
