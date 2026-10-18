# Time and the calendar

The game has a clock that every action that takes time advances. The clock is part of your run and
is saved with it. Nothing depends on real-world time.

## Days

A day has 24 hours, as usual. The status bar shows the time and date like this:
`14:32 · 12 Rainmoon`.

- **Dawn** is at 06:00 and **dusk** is at 18:00. The sun rises at dawn and sets at dusk; see
  [Sun, sky, and night](sun.md).
- A new run starts at a time set by its scenario. The default is 08:00 on the first day of the
  first month of year 1.

## The calendar

A year has 12 months of 30 days each: a 360-day year. There are no weeks. Months have names:

{{#include generated/calendar.md}}

Days are numbered 1 to 30 within a month, and years count up from 1.

## What takes time

Almost everything you do that changes the world takes a turn, and a turn takes game time: seconds
in a dungeon or outdoor area, minutes on the overworld. See [Turns and movement](turns.md) for the
exact lengths, and [Resting](resting.md) for passing a long time at once.

Actions that do not change the world, such as looking around, opening menus, and reading help, take
no time. Refused actions, such as walking into a wall, take no time either.
