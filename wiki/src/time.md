# Time and the calendar

The game has a clock that every action that takes time advances. The clock is part of your run and
is saved with it. Nothing depends on real-world time.

## Days

A day has 24 hours, as usual. The [status bar](controls.md#the-screen) shows the time, the date,
and the phase of the [moon](sun.md#phases) like this: `14:32 · 12 Rainmoon, Year 1444 · ◑ First quarter`.

- **Dawn** is at 06:00 and **dusk** is at 18:00. The sun rises at dawn and sets at dusk; see
  [Sun, moon, and sky](sun.md).
- A new run starts at a time set by its scenario. The default is 08:00 on the 11th day of the
  eleventh month (Darkening) of year 1444, unless a scenario sets another date.

## The calendar

A year has 12 months of 30 days each: a 360-day year. There are no weeks. Months have names:

{{#include generated/calendar.md}}

Days are numbered 1 to 30 within a month, and years count up from 1.

## The moon's phase

The [moon](sun.md#phases) waxes and wanes once a month (see [Sun, moon, and sky](sun.md)). The [status bar](controls.md#the-screen) shows its
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

If the game world has more than one moon, the [status bar](controls.md#the-screen) shows each moon's name and picture
instead, for example `Moon ●  Ilse ☽`.

Under the open sky the [status bar](controls.md#the-screen) also shows the [weather](weather.md), after the moon. When the
window is narrow, the [status bar](controls.md#the-screen) shortens the moon's phase to its picture and the date to its day
and month before it drops them; see [Controls](controls.md#the-screen).

## What takes time

Almost everything you do that changes the world takes game time, and what you do decides how
much: every action has a [cost](turns.md#what-actions-cost), and a normal action, such as waiting, opening a [door](doors.md), or a step,
takes 6 seconds. Costlier actions take longer, and a step through [water](terrain.md#water) takes twice as long as a
normal one. Steps on the [overworld](overworld.md#travel) are journeys of minutes or hours. Creatures that are [quicker](turns.md#speed)
than you take less time for the same action, so they act more often. See
[Turns and movement](turns.md) for what each action costs, and [Resting](resting.md) for passing
a long time at once.

Actions that do not change the world, such as looking around, opening menus, and reading help, take
no time. Refused actions, such as walking into a wall, take no time either.
