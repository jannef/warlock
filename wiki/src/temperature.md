# Temperature and ice

The air has a temperature wherever you are, and the [status bar](controls.md#the-screen) always
shows it, as `4 °C` (or in °F: see [Settings](settings.md)). Below freezing it is tinted a cool
blue, and from 30 °C a warm orange; the number says it all. The cold does nothing to you yet; it
freezes water.

## Underground

Dungeons are cool and steady: 10 °C unless a place is colder or warmer, and the same day and
night, whatever the weather outside. Deeper levels may be a little colder, and some places are
below freezing.

## Under the open sky

Outdoors and on the [overworld](overworld.md) the temperature follows the month, the time of day,
and the [weather](weather.md). Each month has a mean temperature (in the middle of the month; the
days between go from one month's to the next). Over a day it swings around that mean: coldest
around 04:00, before dawn, warmest around 16:00. A clear sky lets the night grow cold and the
afternoon warm; clouds keep the night milder and the day cooler, and rain cools the air a little.
Like the weather, the temperature follows from the run's seed and the time, so it is the same
however you spend the time.

In the middle of each month:

{{#include generated/temperature.md}}

What the weather does to the day's swing (the difference between the coldest night and the
warmest afternoon) and how much it cools the air:

{{#include generated/temperature-weather.md}}

## Water freezes

Below freezing, still water grows ice, faster the colder it is: quickly at first, then ever more
slowly, since the ice keeps the cold from the water beneath it. Running water stays open in a
mild frost and freezes only in a hard one, and more slowly. Above freezing ice melts, 7 mm a day
for every degree of warmth. Nothing waits for you to look: the ice freezes and thaws while you
rest, travel, or are elsewhere, and is as it should be when you come back. A pond you first come
upon in deep winter is frozen already.

{{#include generated/freezing.md}}

From open water, at a steady temperature, still water freezes over in:

{{#include generated/ice-growth.md}}

In Darkening a pond under the open sky grows thin ice on clear nights, which the afternoon sun
thins again; in Deepwinter and Frostmere it freezes thick.

## Walking on ice

Ice looks like what it holds. Look mode names it:

- **Thin ice**: dark water shows through, with pale hairline cracks. A rat can cross it, you
  cannot.
- **Ice**: a grey-blue sheet. It holds a person; with a heavy pack, maybe not.
- **Thick ice**: pale and frosted all over. It holds anything that walks.

Ice too thin to see is no more than the water beneath: you wade through it, and it breaks.

What ice holds grows with the square of its thickness, as real lake ice does: 5 cm holds a person
with a pack, 15 cm three quarters of a tonne. What counts is your weight: your own (70 kg) and
everything you carry, worn and wielded things included (see [Burden](burden.md)). Up to what the
ice holds it holds you; beyond that it may give way, more likely the more you weigh, until at
twice what it holds it surely does. When it held only by luck, it creaks under your weight: a
warning. A heavily laden dwarf falls in where a lightly packed elf crosses.

{{#include generated/ice.md}}

A step onto ice is a careful one and takes a little longer than on dry ground; it is no louder.

## Falling in

When the ice gives way: "The ice cracks beneath you and you plunge into the freezing water!" The
cold costs you a few hit points, but never your last one (it cannot kill you yet). You are in the
water where the ice was: a hole of open water, which freezes over again in time (a few hours in a
hard frost).

- **Climbing out** onto the ice around the hole takes a few turns: the step onto the ice and a
  climb on top of it (see [Turns and movement](turns.md)). Ice that cannot hold you breaks ahead
  of you as you wade on, without another plunge; wading out onto the shore is an ordinary step.
- **What lay on the ice** sinks to the bottom of the hole. The water is shallow: wade in and pick
  it up.
- **Creatures** fall through too, by the same rules, and climb out the same way.

There is no going back once the ice cracks, but nothing is lost for good: only time, and a few
hit points.

## Heat and frost

Bursts of heat and frost change the ice at once (see [Traps](traps.md); spells will too): a
fireball melts the ice it reaches, and whoever stood on it falls in; a frost trap freezes the
water it reaches at once, into ice that holds a laden person (10 cm). A blast without heat (none
yet) will shatter thin ice. Someone who makes way for you onto ice you stood on may break through
it, if it is heavier than you.
