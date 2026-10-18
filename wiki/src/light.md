# Light

Light decides what you can see in the dark places of the world. It is part of the rules, not just
decoration.

## What light is

Every cell has a light level with four channels: red, green, blue, and ultraviolet (UV). For each
channel, 255 means fully lit: a surface then shows its true color. More than 255 is overbright, and
the surface brightens a little further. Colored light is real: a red crystal lights its
surroundings red, and a surface lit only by red light shows only red.

A cell's light is the sum of:

- the level's **ambient** light (zero in most dungeons, so they are dark),
- **sunlight, moonlight, and sky light** under the open sky, dimmed by clouds (see
  [Sun, moon, and sky](sun.md) and [Weather](weather.md)),
- every **light source** that reaches the cell, and
- a little **bounced** light.

## Light sources

Light comes from wall torches, braziers, lamps, magical crystals, glowing fungus, runes, lava, the
altar, creatures that carry a torch, and your own [lantern](lantern.md). Each source has a color
and a reach in cells:

{{#include generated/lights.md}}

Light is strongest at the source and fades smoothly to nothing at the edge of its reach. Walls,
closed doors, and anything else that blocks sight (such as trees and boulders) stop light, so a
light does not shine through them, and a light source in a cell that blocks sight, such as one on
a closed door, gives no light.

Shadows come from the same rule: a pillar or a wall casts a shadow behind it, away from the light,
and cells there get none of that light.

## Bounce

Surfaces reflect a little light. Lit floor and walls glow faintly into the cells next to them, in a
tint that depends on the surface (a grassy meadow bounces green-yellow, water bounces blue). Bounce
never crosses a wall and reaches only three cells from the surface, and it is never more than about
an eighth of the direct light.

## Walls

A wall is lit by a source the way a floor cell at the same distance would be, but only where you can
see a face of it that the source lights. This keeps light from appearing to leak through walls.

## What is only for show

Some effects look like light but never change what you can see:

- **Flicker.** Torches and fires flicker and drift in color, and lamps burn more calmly. Magical
  light (crystals, runes, fungus) holds perfectly steady. This can be switched off in the settings.
- **Creature shadows.** Creatures, you included, cast soft shadows. They are only drawn: a creature
  cannot hide in another creature's shadow, and no creature sees you any less for standing in one.
- **Glow and particles.** Sparks over a brazier or the glow of a rune are decoration.
- **Falling rain.** The drops are decoration; the clouds that dim the light are not (see
  [Weather](weather.md)).

Each of these but the creature shadows has its own switch in the [settings](settings.md), and
reduced effects turns them all off. Creature shadows are always drawn.
