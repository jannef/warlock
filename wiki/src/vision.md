# Vision types

What you see depends on your eyes. Each vision type has three properties:

- how much light a cell needs before you can see it (a **threshold**; full light is 100%),
- a **darkvision radius**: cells this close are visible however dark they are, and
- whether you **see UV** light.

The brightness of a cell is the weighted brightness of its red, green, and blue light (green counts
most, blue least). For eyes that see UV, UV light adds to it. For other eyes UV does not exist at
all: they do not see it, and it does not light anything for them.

{{#include generated/vision.md}}

## The four types

- **Normal** eyes need real light. Where there is none, you see nothing but yourself. A full moon
  high in the sky is just enough to make out open ground.
- **Low-light** eyes need only about a third as much light as normal eyes, so they can see by the
  faint glow of fungus where normal eyes cannot, and by a half moon.
- **Darkvision** needs the same light as normal eyes, but also sees everything within its radius,
  lit or not. Cells seen only that way are drawn as a faint monochrome glow that fades out toward
  the edge of the radius. The darker and farther a cell is, the less color it keeps.
- **UV sight** needs the same light as normal eyes, and adds UV light: a UV-only source, such as a
  faint rune, is invisible to everyone else but lights the way for you. UV light is drawn as a
  violet tint, and only if you see UV.

Your vision type comes from your race: the table on [Races](races.md) lists each race's eyes.
[Talents](talents.md) of the Shadow tree change them: **Night eyes** (normal eyes) halves the light
a cell needs, **Dark adaptation** (darkvision) reaches 2 cells farther, **Owl eyes** (low-light)
gives darkvision of 2 cells, and **Rune sight** sees UV light. Your eyes keep their type's name; the
talents only change what it does. **Blood scent** (darkvision) sees a bleeding creature in your
line of sight within 9 cells however dark it is, and **Moon-eyed** (low-light) sees a cell farther
across the [overworld](overworld.md) at night. A [developer key](controls.md#developer-keys)
(`F3`) cycles through the types for trying them out.

Creatures have eyes of these types too, and see by the same rules (see
[Creatures](creatures.md)); the game does not tell you which eyes a creature has.
