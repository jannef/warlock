# Sun, sky, and night

## Which places have a sky

Levels under the open sky are lit by the sun and the sky: outdoor areas and the
[overworld](overworld.md). Dungeons never are. Terrain with a roof (for example the floor under a
gazebo) never receives sun or sky light even on an outdoor level.

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

## Night

At night there is neither sun nor sky light, and **moonlight is darkness for now**: there is no
moon yet. Outdoor areas are completely dark at night unless you bring a light of your own. Light
your [lantern](lantern.md). A few things, such as a torch-carrying creature or a glowing altar,
still shine.

The overworld is a special case: it is drawn by a faint cold light at night, so you can tell the
cells you see from the ones you only remember. That light does not help you see; see
[The overworld](overworld.md) for how far you see.
