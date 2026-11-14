# Weather

Under the open sky (outdoor areas and the [overworld](overworld.md)) there is weather: **clear**,
**overcast**, **rain**, or **heavy rain**. It is the same everywhere in the land at a time. In
[dungeons](dungeons.md) you cannot tell what the weather is outside.

The [status bar](the-screen.md#the-screen) shows the weather while you are under the sky, for
example `14:32 · 12 Rainmoon, Year 1444 · ◑ First quarter · Rain`.

## How the weather changes

The weather changes every three hours or so, not always on the hour. It tends to last: a rainy
spell often goes on for half a day. It mostly changes to a neighbouring kind: from clear to
overcast, from overcast to rain, from rain to heavy rain, and back. When it changes, the
[light](light.md#what-light-is) dims or brightens over about an hour as the clouds come and go.

How likely each weather is depends on the [month](time.md#the-calendar). Rainmoon is the wettest month and Highsun the
driest:

{{#include generated/weather.md}}

Over many days each weather comes up as often as its chance says. Nothing you do changes it: the
weather is the same however you spend the time, waiting, resting, or travelling.

## Clouds and light

Clouds take light away from the sky. They block most of the direct light of the
[sun](sun.md#the-suns-path) and the [moon](sun.md#moonlight), so [shadows](sun.md#shadows) grow
faint. They spread some of that light over the whole sky, so the light is flat and grey. Overcast
days are duller, and rainy days darker still:

{{#include generated/weather-effects.md}}

By day you still see well enough in any weather. At night clouds hide the
[moon](sun.md#moonlight): under an overcast sky even a full moon is too faint for [normal
eyes](vision.md#the-four-types). So [outdoor areas](overworld.md#local-areas) are dark unless you
bring a [lantern](lantern.md). See [Sun, moon, and sky](sun.md).

## Clouds and warmth

The weather also changes the temperature: clear nights are colder, overcast ones milder, and rain
cools the air (see [Temperature and ice](temperature.md)).

## Rain and sight

On the [overworld](overworld.md), rain shortens how far you [see](sight.md) by the cells in the last
column above, but never below the 2 cells you see on the darkest night. Overcast skies take no
cells away. At night, though, any cloud cover dims the moon, and with it how far you see by
moonlight: on an overcast night a full [moon](sun.md#moonlight) shows you less than on a clear one.
On foot, in outdoor areas, the weather only changes the light.

## Messages and resting

When the weather changes while you are under the sky, the [message line](the-screen.md#messages) says so:

| From | To | Message |
|---|---|---|
| Clear | Overcast | "Clouds gather." |
| Clear or overcast | Rain | "It starts to rain." |
| Clear or overcast | Heavy rain | "It starts to pour." |
| Rain | Heavy rain | "The rain grows heavy." |
| Heavy rain | Rain | "The rain eases." |
| Rain or heavy rain | Overcast | "The rain stops." |
| Rain or heavy rain | Clear | "The rain stops and the sky clears." |
| Overcast | Clear | "The sky clears." |

Arriving under the sky says nothing; the status bar shows the weather.

Rain starting, or growing heavier, wakes you: it stops a [rest](resting.md). Other news (clouds
gathering or parting, the rain easing or stopping) is written to the message line but does not
stop a rest.

## What you see

While it rains, faint drops fall on the cells you see under the sky, splash on open ground, and
ring on still [water](terrain.md#water). They never fall under a roof. Heavy rain falls thicker.
Drops are lit like everything else, so you hardly see them at night except in the light of your
[lantern](lantern.md). The Weather switch in the [settings](settings.md), and [reduced
effects](settings.md), turn the falling rain off. The darker light stays, since it is part of what
you see.

See also: [Sun and moon](sun.md), [Temperature and ice](temperature.md), and
[The overworld](overworld.md).
