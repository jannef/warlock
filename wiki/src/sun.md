# Sun, moon, and sky

## Which places have a sky

Levels under the open sky are lit by the sun and the sky by day and by the moon at night: outdoor
areas and the [overworld](overworld.md). Dungeons never are. Terrain with a roof (for example the
floor under a gazebo) never receives sun, moon, or sky light even on an outdoor level.

## The sun's path

The sun follows the same path every day. It rises at dawn (06:00) due east, which is the right side
of the map. It climbs to its highest point at noon, due south (the bottom of the map), where it
stands 60 degrees above the horizon, and sets at dusk (18:00) in the west (the left of the map).
Days are always 12 hours of sun, since there are no seasons yet.

Its light changes with its height:

- Just after sunrise it is dim and deep orange. It turns orange, then warm gold, then warm white as
  it climbs, and from 40 degrees up it is a bright neutral white.
- The evening runs the same way in reverse.

## Sky light

By day every cell under the sky also gets a dim, bluish **sky light**, whether it is in the sun or
in shadow. It is what keeps the shade from being black. Sky light is dimmer and more violet while
the sun is low.

Around dawn and dusk there is a short twilight glow, from about 23 minutes before dawn until dawn
and from dusk until about 23 minutes after.

## Shadows

Terrain stands a number of cells tall (see the height column in the [terrain](terrain.md) table):
trees 2, pines 3, standing stones 3, bushes and reeds 1, and walls 1. Anything tall casts a shadow
away from the sun, and cells in a shadow get no direct sunlight, only sky light. The shadow is long
when the sun is low (at dawn and dusk) and short at noon, when it falls just a cell or so from a
tree. Shadows turn as the sun moves.

Creatures cast soft shadows too, but those are only for show and do not change what you can see.

## Night and the moon

At night there is neither sun nor sky light. What light there is comes from the **moon**.

### Phases

The moon waxes and wanes once a month: its cycle from new moon to new moon is 30 days, the length
of a month, so its phases fall on the same days every month.

- **New moon** on the 1st: the moon is dark and gives no light at all.
- It **waxes** through the first half of the month. Around the 8th it is half lit (the first
  quarter).
- **Full moon** on the night from the 15th to the 16th: the whole moon is lit.
- It **wanes** through the second half, half lit again around the 23rd (the last quarter).

The status bar shows the phase; see [Time and the calendar](time.md#the-moons-phase).

{{#include generated/moons.md}}

### The moon's path

The moon crosses the sky along the same path as the sun, east to south to west, but it lags behind
the sun by its phase:

- A new moon rises and sets with the sun, so it is never in the night sky.
- A first-quarter moon rises around noon, stands high at dusk, and sets around midnight.
- A full moon rises at dusk, stands high in the south at midnight, and sets at dawn.
- A last-quarter moon rises around midnight and stands high at dawn.

So the moon rises about 50 minutes later every day.

### Moonlight

Moonlight is faint and slightly purple. It is brightest when the moon is full and high in the sky,
less when only part of it is lit, faint just after it rises and just before it sets, and none at
all by day: the sun drowns it out, and it only grows in as the evening twilight fades. Besides its
direct light, the moon gives a very faint glow from the whole sky, on every cell under the sky.

Tall things cast **moon shadows** away from the moon, just as they cast shadows from the sun: long
while the moon is low, short when it is high. In a moon shadow there is only the faint glow of the
sky.

What you can see by moonlight depends on your [eyes](vision.md):

- **Normal eyes** can just make out open ground, dimly, under a full or nearly full moon high in
  the sky, but nothing in its shadows. Under a half moon, a low moon, or a new moon they see
  nothing without a light.
- **Low-light eyes** see by a half moon too, and under a full moon some of the shade as well.
- **Darkvision** sees what normal eyes see, and everything within its radius as always.

When the moon is new or down, outdoor areas are dark unless you bring a light of your own. Light
your [lantern](lantern.md). A few things, such as a torch-carrying creature or a glowing altar,
shine on any night.

The overworld is a special case: it is drawn by a faint cold light at night, so you can tell the
cells you see from the ones you only remember, and moonlight comes on top of it. That light does
not decide what you see; the moon does widen your sight there, though. See
[The overworld](overworld.md).
