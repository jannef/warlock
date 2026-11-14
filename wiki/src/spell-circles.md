# Spell circles

A **spell circle** is a thin ring that turns slowly around anyone you see who carries a buff or a
hex. A buff or hex here is a [status](status-effects.md) laid by one of the spells in the table
below, or the same status from anywhere else: a [potion of quickness](potions.md#potion-of-quickness)
hastes you just as [Haste](magic.md#spell-haste) does, and shows the same circle.

A circle tells you no more than the [status bar](the-screen.md#the-screen) and
[look mode](controls-prompts.md#look-mode) do. It shows which kinds of magic someone carries, never how
strong they are or how long they will last.

## Reading a circle

- **The figure inside the rings** counts the effects: one is a triangle, two a square, three a
  triangle again, four a square, five a pentagon, six a hexagon. A seventh changes nothing.
- **There is one thin ring per effect**, up to three: one ring for one effect, two for two, three
  for three or more.
- **Buffs are whole and turn clockwise**, one turn a minute. **Hexes are broken**, with dashed
  rings and sides, and turn the other way, one turn in 45 seconds. Someone with both has the
  buffs' circle inside and the hexes' circle around it.
- **The colour** is the [school](schools.md) of the first effect, in the order the status bar lists
  them. The rings glow faintly in their colour, and so does the glyph of whoever wears the circle
  (in the buffs' colour if they have both).
- **Glints and arcs**: now and then a tiny glint flashes on the wearer's glyph (two beats, every 5.3
  and 7.1 seconds, each showing for about a tenth of that). A soft brighter arc runs round each
  ring every 7 seconds.
- **A circle keeps to what you see.** Its rings show only over floor you can see right now. They
  never cover walls, darkness, or anything else drawn on the map (a creature, an item, a torch on
  the wall), and fade out softly where they reach one. Where two circles crowd together, the
  later one's rings pass under the earlier one's (yours come first).
- **A circle fades in** over half a second when the first buff or hex begins, and fades out just as
  long when the last one ends. When an effect begins or ends while others go on, the old figure
  and rings fade into the new ones over 0.4 seconds. A hexes' circle moves outward when a buffs'
  circle appears inside it, and back in once that one is gone.
- With **Reduced effects** or **Slides, camera** off, none of this moves, nor do the glints or the
  arcs: a circle appears, changes, and goes at once. Reduced effects also removes the glow
  ([Settings](settings.md)).

Sleep is the one hex without a circle. A creature [asleep](status-list.md#status-asleep), from
[Slumber](magic.md#spell-slumber) or in a den, shows none and sheds no light.

Every buff shows on a creature, whatever gave it. Look mode calls a hasted creature "quickened", a
bark-skinned one "bark-skinned", and one that drank a [potion of might](potions.md#potion-of-might)
"fortified" (see [Status effects](status-effects.md)).

The drawn circles are only for show: the **Spell circles** switch in [Settings](settings.md) hides
them, and Reduced effects only stops them moving. The light they shed is real, though (below).

{{#include generated/spell-circles.md}}

## The circle's light

A status with a circle sheds a faint, steady [light](light.md) in its school's colour. It does so
whatever gave the status (a potion of quickness's Hasted too) and whoever has it, you or a
creature. These are the spell circle lights of the [lights table](light.md#light-sources): each
reaches one cell and has a strong [ultraviolet](vision.md#the-four-types) share.

The light's strength never changes with the status's [severity](status-effects.md#how-they-work)
or the time left, and it goes out the turn the status ends. Someone with several statuses that
have circles sheds only one such light: that of the first status in status order. The Light
spell's own light is brighter and wins over it.

- **Normal eyes** (a [brightness](vision.md#the-four-types) of 40) see the glowing one's own cell,
  and so the glowing one, at any distance in line of sight, even in the dark (40 to 52 there, plus
  the cell's other light). They do not see the four cells beside it, which are half as bright (20
  to 26). The diagonal cells get no light.
- **Low-light eyes** (12) see the four cells beside it too.
- **Eyes that see UV** see it plainly (130 to 142 on its cell), and the four cells beside it as
  well (65 to 71), where the UV shows as a violet tint.
- **A buff lights you up.** Creatures see by the same rules, so a hasted, blessed, or bark-skinned
  sneak is seen by any normal-eyed creature that has them in line of sight, just as with Inner
  light. A hex marks its target the same way: a webbed or drowsy creature glows for as long as
  the hex lasts.
- **[Light-blind](vision.md#the-four-types) eyes** add this light to the cell's other light. A
  drow in daylight just under their glare (250) can be dazzled by their own
  [Blessing](magic.md#spell-blessing).
- A disguised [mimic](mimics.md) sheds no light: its disguise would glow.

See also: [Buffs and hexes](buffs.md), [Settings](settings.md), and [Potions](potions.md).
