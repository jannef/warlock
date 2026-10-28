# Light

Light decides what you can [see](sight.md) in the dark places of the world. It is part of the rules, not just
decoration.

## What light is

Every cell has a light level with four channels: red, green, blue, and ultraviolet (UV). For each
channel, 255 means fully lit: a surface then shows its true color. More than 255 is overbright, and
the surface brightens a little further. Colored light is real: a red [crystal](terrain.md#terrain-crystal) lights its
surroundings red, and a surface lit only by red light shows only red.

A cell's light is the sum of:

- the level's **ambient** light (zero in most [dungeons](dungeons.md), so they are dark),
- **[sunlight](sun.md#the-suns-path), [moonlight](sun.md#moonlight), and [sky light](sun.md#sky-light)** under the open sky, dimmed by [clouds](weather.md#clouds-and-light) (see
  [Sun, moon, and sky](sun.md) and [Weather](weather.md)),
- every **light source** that reaches the cell, and
- a little **bounced** light.

A [special room](special-rooms.md) can change that: a [shadowed room](special-rooms.md#room-shadowed) drains every part of it, and
a [sanctuary](special-rooms.md#room-sanctuary) adds a faint warm glow of its own.

## Light sources

Light comes from [wall torches](terrain.md#terrain-sconce), [braziers](terrain.md#terrain-brazier), lamps, magical [crystals](terrain.md#terrain-crystal), glowing [fungus](terrain.md#terrain-fungus), [runes](terrain.md#terrain-uv-rune), [lava](terrain.md#terrain-lava), the
[altar](terrain.md#terrain-altar), [creatures](creatures.md) that carry a torch, and your own [lantern](lantern.md). Each source has a color
and a reach in cells:

{{#include generated/lights.md}}

Light is strongest at the source and fades smoothly to nothing at the edge of its reach. [Walls](terrain.md#walls),
closed [doors](doors.md), and anything else that blocks [sight](sight.md#line-of-sight) (such as [trees](terrain.md#terrain-tree) and [boulders](terrain.md#terrain-boulder)) stop light, so a
light does not shine through them, and a light source in a cell that blocks sight, such as one on
a closed door, gives no light.

Shadows come from the same rule: a pillar or a wall casts a shadow behind it, away from the light,
and cells there get none of that light.

## Bounce

Surfaces reflect a little light. Lit floor and walls glow faintly into the cells next to them, in a
tint that depends on the surface (a grassy meadow bounces green-yellow, [water](terrain.md#water) bounces blue). Bounce
never crosses a wall and reaches only three cells from the surface, and it is never more than about
an eighth of the direct light.

## Walls

A wall is lit by a source the way a floor cell at the same distance would be, but only where you can
see a face of it that the source lights. This keeps light from appearing to leak through walls.

## What is only for show

Some effects look like light but never change what you can see:

- **Flicker.** Torches and fires flicker and drift in color, and lamps burn more calmly. Magical
  light ([crystals](terrain.md#terrain-crystal), [runes](terrain.md#terrain-uv-rune), [fungus](terrain.md#terrain-fungus)) holds perfectly steady. This can be switched off in the [settings](settings.md).
- **Creature shadows.** [Creatures](creatures.md), you included, cast soft shadows. They are only drawn: a creature
  cannot hide in another creature's shadow, and no creature sees you any less for standing in one.
- **Glow and particles.** Sparks over a [brazier](terrain.md#terrain-brazier) or the glow of a [rune](terrain.md#terrain-uv-rune) are decoration.
- **Falling rain.** The drops are decoration; the clouds that dim the light are not (see
  [Weather](weather.md)).

Each of these but the creature shadows has its own switch in the [settings](settings.md), and
[reduced effects](settings.md) turns them all off. Creature shadows are always drawn.
