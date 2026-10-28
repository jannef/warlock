# Vision types

What you see depends on your eyes. Each vision type has three properties:

- how much [light](light.md#what-light-is) a cell needs before you can see it (a **threshold**; full light is 100%),
- a **darkvision radius**: cells this close are visible however dark they are, and
- whether you **see UV** [light](light.md#what-light-is).

The brightness of a cell is the weighted brightness of its red, green, and blue light (green counts
most, blue least). For eyes that see UV, [UV](light.md#what-light-is) light adds to it. For other eyes UV does not exist at
all: they do not see it, and it does not light anything for them.

{{#include generated/vision.md}}

## The four types

- **Normal** eyes need real light. Where there is none, you see nothing but yourself. A full [moon](sun.md#moonlight)
  high in the sky is just enough to make out open ground.
- **Low-light** eyes need only about a third as much light as normal eyes, so they can see by the
  faint glow of [fungus](terrain.md#terrain-fungus) where normal eyes cannot, and by a [half moon](sun.md#moonlight).
- **Darkvision** needs the same light as normal eyes, but also sees everything within its radius,
  lit or not. Cells seen only that way are drawn as a faint monochrome glow that fades out toward
  the edge of the radius. The darker and farther a cell is, the less color it keeps.
- **UV sight** needs the same light as normal eyes, and adds UV light: a UV-only source, such as a
  faint [rune](terrain.md#terrain-uv-rune), is invisible to everyone else but lights the way for you. UV light is drawn as a
  violet tint, and only if you see UV.

Your vision type comes from your race: the table on [Races](races.md) lists each race's eyes.
[Talents](talents.md) of the Shadow tree change them: **[Night eyes](talents.md#talent-night-eyes)** (normal eyes) halves the light
a cell needs, **[Dark adaptation](talents.md#talent-dark-adaptation)** (darkvision) reaches 2 cells farther, **[Owl eyes](talents.md#talent-owl-eyes)** (low-light)
gives darkvision of 2 cells, and **[Rune sight](talents.md#talent-rune-sight)** sees UV light. Your eyes keep their type's name; the
talents only change what it does. **[Blood scent](talents.md#talent-blood-scent)** (darkvision) sees a [bleeding](status-effects.md#bleeding) [creature](creatures.md) in your
line of sight within 9 cells however dark it is, and **[Moon-eyed](talents.md#talent-moon-eyed)** (low-light) sees a cell farther
across the [overworld](overworld.md#sight-by-daylight-and-moonlight) at night.

[Drow](races.md#race-drow) eyes are also **light-blind**: in strong light (a perceived brightness of 250 or more on your
own cell: the [sun](sun.md#the-suns-path) well up under a clear sky, a bright fire close by) you are dazzled and see
nothing farther than ten cells, whatever your vision type (see
[Races](races.md#the-evil-races)).

A [developer key](controls.md#developer-keys)
(`F3`) cycles through the types for trying them out.

Creatures have eyes of these types too, and see by the same rules (see
[Creatures](creatures.md)); the game does not tell you which eyes a creature has.
