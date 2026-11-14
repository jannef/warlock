# Time and the calendar

The game has its own clock. Every action that takes time moves it forward. The clock belongs to
your run and is saved with it; nothing depends on real-world time.

## Days

A day has 24 hours, as usual. The [status bar](the-screen.md#the-screen) shows the time, as `14:32`.
The [pause menu](the-screen.md#menus) (`Esc`) tells the date and the phase of the
[moon](sun.md#phases) like this: *The 12th of Rainmoon, 1444, under a waxing gibbous moon, ...*

- **Dawn** is at 06:00 and **dusk** is at 18:00. The sun rises at dawn and sets at dusk; see
  [Sun, moon, and sky](sun.md).
- A new run starts at a time set by its scenario. The default is 08:00 on the 11th day of the
  eleventh month (Darkening) of year 1444, unless a scenario sets another date.

## The calendar

A year has 12 months of 30 days each: a 360-day year. There are no weeks. Months have names:

{{#include generated/calendar.md}}

Days are numbered 1 to 30 within a month, and years count up from 1.

## The moon's phase

The [moon](sun.md#phases) waxes and wanes once a month (see [Sun, moon, and sky](sun.md)). The
[pause menu](the-screen.md#menus) names its phase in words, *under a full moon*:

| The pause menu says | In a month |
|---|---|
| a new moon | around the 1st |
| a waxing crescent moon | |
| a waxing half moon | around the 8th |
| a waxing gibbous moon | |
| a full moon | around the 16th |
| a waning gibbous moon | |
| a waning half moon | around the 23rd |
| a waning crescent moon | |

If the game world has more than one moon, the pause menu names each moon and its phase instead,
for example *under Moon full and Ilse waxing crescent*.

## What takes time

Almost everything you do that changes the world takes game time. Every action has a
[cost](turns.md#what-actions-cost). A normal action, such as waiting, opening a [door](doors.md),
or taking a step, takes 6 seconds. Costlier actions take longer, and a step through
[water](terrain.md#water) takes twice as long as a normal one. Steps on the
[overworld](overworld.md#travel) are journeys of minutes or hours. Creatures that are
[quicker](turns.md#speed) than you take less time for the same action, so they act more often. See
[Turns and movement](turns.md) for what each action costs, and [Resting](resting.md) for passing
a long time at once.

Actions that do not change the world, such as looking around, opening menus, and reading help, take
no time. Refused actions, such as walking into a wall, take no time either.

See also: [Sun and moon](sun.md) and [Weather](weather.md).
