# Spell circles

Everyone you see who has a buff or a hex on them (a [status](status-effects.md) laid by one of
the spells below, or the same status from anywhere else: a [potion of quickness](potions.md#potion-of-quickness)
hastes you as [Haste](magic.md#spell-haste) does) has a thin circle turning slowly about them.
It says no more than the [status bar](the-screen.md#the-screen) and [look mode](controls.md#look-mode)
do: which kinds of magic, never how strong or for how long.

- **A figure inside the rings**: one effect is a triangle, two a square, three a triangle again,
  four a square, five a pentagon, six a hexagon. A seventh changes nothing.
- **One thin ring per effect**, three at most: one ring is one effect, two rings two, three rings
  three or more.
- **Buffs are whole and turn clockwise**, a turn a minute. **Hexes are broken** (dashed rings and
  sides) **and turn the other way**, a turn in 45 seconds. Someone with both has the buffs' circle
  inside and the hexes' around it.
- **The colour** is the [school](schools.md) of the first effect in the order the status bar
  lists them. The rings have a faint glow of their colour, and so does the glyph of whoever wears
  the circle (in its rings' colour, the buffs' if they have both). Now and then a tiny glint
  flashes on the wearer's glyph (two beats, every 5.3 and 7.1 seconds, each showing for about a
  tenth of it), and a soft brighter arc runs round each ring (every 7 seconds).
- **A circle keeps to what you see**: its rings show only over floor you can see now, never over
  walls, darkness, or anything else drawn on the map (a creature, an item, a torch on the wall);
  they fade out softly where they reach one. Where two circles crowd, the later one's rings pass
  under the earlier's (yours first).
- **A circle fades in** over half a second when the first buff or hex begins, and out as long
  when the last ends. When one begins or ends while others go on, the old figure and rings fade
  into the new over 0.4 seconds. A hexes' circle moves outward when a buff's circle comes inside
  it, and back in once that one is gone. With Reduced effects or Slides, camera off, none of this
  moves (nor do the glints or the arcs): a circle appears, changes, and goes at once. Reduced
  effects also takes the glow away ([Settings](settings.md)).

Sleep is the one hex without a circle: a creature [asleep](status-effects.md#status-asleep), from
[Slumber](magic.md#spell-slumber) or in a den, shows none and sheds no light.

Every buff shows on a creature, whatever gave it: look mode names a hasted creature "quickened",
a bark-skinned one "bark-skinned", one that drank a [potion of might](potions.md#potion-of-might) "fortified" (see
[Status effects](status-effects.md)). The drawn circles are only for show (the **Spell circles**
switch in [Settings](settings.md) hides them; Reduced effects only stills them), but the light
they shed is not.

{{#include generated/spell-circles.md}}

## The circle's light

A circled status sheds a faint, steady [light](light.md) of its school, whatever gave the
status (the potion of quickness's Hasted too) and whoever has it, you or a creature: the
`circle_<school>` lights of the [lights table](light.md#light-sources), each reaching one cell,
with a strong [ultraviolet](vision.md#the-four-types) share. Its strength never changes with the status's
[severity](status-effects.md#how-they-work) or the time left, and it goes the turn the status ends. Someone with several circled
statuses sheds one such light, the first in status order's. The Light spell's own light is
brighter and wins over it.

- **Normal eyes** (a [brightness](vision.md#the-four-types) of 40) see the circled one's own cell, and so the
  circled one, at any distance in line of sight, even in the dark (40 to 52 there, plus the
  cell's other light), but not the four cells beside it (half as bright: 20 to 26; the diagonal ones get none).
- **Low-light eyes** (12) see the four cells beside it too.
- **Eyes that see UV** see it plainly (130 to 142 on its cell) and the four cells beside it as well (65
  to 71), where the UV shows as a violet tint.
- **A buff lights your cell.** Creatures see by the same rule, so a hasted, blessed, or
  bark-skinned sneak is seen by any normal-eyed creature with them in line of sight, as Inner
  light is. A hex marks its target the same way: a webbed or drowsy creature glows for as long
  as it lasts.
- **[Light-blind](vision.md#the-four-types) eyes** add it to the cell's light: a drow in daylight just under their glare (250)
  can be dazzled by their own [Blessing](magic.md#spell-blessing).
- A disguised [mimic](mimics.md) sheds none: its disguise would glow.
