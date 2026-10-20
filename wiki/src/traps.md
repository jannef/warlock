# Traps

Some floors are trapped. A trap lies on a cell, **hidden**: it looks like the floor around it,
and nothing tells you it is there until it goes off. Whoever steps onto a trap sets it off, you
and the creatures alike.

## Finding traps

You learn where a trap is when you find it, searching or in passing (see
[Searching](searching.md)), or when it goes off:

- when **you** set it off;
- when you **see a creature** set it off. Creatures know no more about traps than you do, so they
  walk onto them; a trap that goes off where you cannot see it stays hidden. You may still hear
  it, or see the blast of a fireball where you see part of it.

From then on the trap is **known**: it shows as `^` in its own color, over anything lying on it,
and you remember it like the terrain, even out of sight. Look mode names it: "A fireball trap, on
stone floor.", or "A kobold (hostile, unhurt), on a pit on stone floor." Some places show you
their traps from the start.

Stepping onto a trap you know asks first, in a small window: `Really step onto the fireball
trap?` Choose **Step onto it** (`y`) to step on, or **Stay back** (`n` or `Esc`, chosen at first)
to stay where you are. Holding a movement key stops you short of the trap instead of asking.

## What traps do

{{#include generated/traps.md}}

- **Attacks**: an arrow or spears attack like a weapon, rolled against the Defence of whoever
  stepped on the trap (see [Combat](combat.md)): they can miss ("You dodge the arrow.", "The spear
  glances off your armour."), hit, or hit hard. An arrow flies like a shot (see
  [Ranged combat](ranged.md)): it attacks whoever stands in its way, and lands where it falls, to
  be picked up.
- **Blasts**: a fireball burns everyone around the trap and a lightning bolt strikes everyone
  along its line, you included if you are there, with no roll to hit. Walls stop both, but a
  fireball blows the softer walls it reaches apart (see [Digging and walls](digging.md)) and melts
  the ice it reaches, and whoever stands on that ice falls in. A frost trap bites everyone around
  it with cold and freezes the water it reaches at once, into ice that holds a laden person (see
  [Temperature and ice](temperature.md)). The lightning runs one way, along the longest clear
  line from the trap unless the place says otherwise.
- **A pit** costs you your next action as you climb out, as well as the fall.
- **An alarm** rings a bell that every creature within earshot hears; the ones who have enemies
  come to look (see [Hearing](hearing.md)).
- **A teleport trap** moves you to another free place on the same level, somewhere you may never
  have seen ("You are wrenched elsewhere.").
- **Charges**: some traps go off only once or twice; then they are spent and gone.
- **The same spells** come on [scrolls](scrolls.md): a scroll of fireball, lightning, or frost
  casts what the trap does, where you aim it.

A trap's blow wounds like any other: a big one spills blood and can leave you
[bleeding](status-effects.md#bleeding). Traps can kill, you as well as creatures. Creatures
killed by a trap drop what they carried, as from any death, but a trap's kill earns you no
[experience](experience.md) and counts for no [quest](quests.md), even if you lured the creature
there; if a trap kills you, the death screen names it ("Killed by a fireball trap"). Someone who
makes way for you onto a trap you stood on sets it off too.

No trap lies on water that can freeze, and a teleport never sets you down on ice.

## Where traps are

- **The Depths** hide a few traps on every level, never on or next to the stairs, and never where
  a creature walks its rounds. Some rooms hide their own: an offering before an altar may not be
  unguarded.
- **The Trap hall**, reached by the third staircase down in the showcase hub's stairs nook (and
  its own scenario), is the place to learn them: a row of known traps to try, one of each kind,
  and creatures going their rounds across hidden ones, so you can watch fireballs and lightning
  catch them. Its creatures leave you alone.
- **The showcase** (the hub, the vast cavern, the courtyard) and the overworld have no traps.
