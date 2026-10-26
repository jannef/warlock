# Magic

A **spell** is cast from the abilities menu (`a`), under its own heading *Spells*, and paid in
[mana](mana.md). Today the spells are the casters' (the priest's, the druid's, the wizard's, and
the witch's, given by their [class tables](classes.md#levels)); learning spells from scrolls is
coming. Every spell is measured by the same rules below: its tier, your caster level, its shape,
and how it finds its target.

## The spells

What each spell does, how far it reaches (and how far your Willpower can take it), and its mana at
its tier's comfortable caster level, without armour:

{{#include generated/spell-list.md}}

The abilities menu shows a spell's tier and what it costs you now, with your caster level,
talents, and armour counted.

## Caster level

Your caster level is how strong a caster you are. It is your level times your class's **casting**
share, rounded down, plus what talents and magic items may one day add; anyone with any mana has
at least caster level 1. Everyone casts a little: a level-30 warrior is a caster of level 7, a
level-30 wizard of level 30.

{{#include generated/spell-casting.md}}

## Tiers

Every spell has a **tier**, from 0 (a *cantrip*) to 9. Each tier has a **comfortable caster
level**: at or above it, learning a spell of the tier is certain and casting it costs its base
mana cost. A spell above your caster level's tier costs more mana to cast, half again for each
tier too high. Each tier also has a **damage ceiling**: the most a single-target spell of that
tier can ever do in one cast at its highest roll.

{{#include generated/spell-tiers.md}}

## What a spell costs

A spell's mana is its tier's base cost, times the overreach (half again per tier above your
caster level's), times its shape's share (below), times its resolution's (a touch is cheaper),
times what your armour adds, rounded to the nearest point once at the end and never below 1. Then
your talents: *Archmage* halves it, and *Frugal caster* takes a fifth off (see [Mana](mana.md)).

Heavy armour and big shields hinder casting: while worn, each makes every spell dearer, and what
you wear adds up.

{{#include generated/spell-armour.md}}

## Shapes

A spell strikes one target (a ray, a bolt, a touch, or yourself) or an area (a cone, a line, or a
burst). Areas cost more mana: the base cost is multiplied by the shape's.

{{#include generated/spell-shapes.md}}

- A **ray** or a **bolt** flies along the line toward the place you aim at and strikes the first
  creature in its way. A hex is cast the same way at a hostile you see, and falls on it.
- A **line** runs through everyone on it, as far as its length, until a wall stops it.
- A **cone** fans out from you toward where you aim, a quarter turn wide, as far as its length.
- A **burst** flies toward where you aim, goes off on the first creature in its way (or there),
  and strikes everyone within its radius that it reaches, half as hard at its edge. It can catch
  you too.
- A **touch** reaches only a creature next to you.

Your **Willpower** lets a spell reach farther: a cell more of range or length, and a cell more of
a burst's radius, for every 2 points of its modifier, but never past the spell's own limit (the
"to" in the list). A burst costs by its own radius, however far your Willpower spreads it. Your
**Presence** makes a blessing or a hex last longer: a degree more of it for every 2 points of its
modifier, up to the most the status has (each degree lasts as long again).

## How a spell finds its target

Each spell resolves one way. An **attack** rolls to hit like a missile (2d12 plus your Dexterity's
modifier against the target's Defence), can miss, and can be a critical hit, and does full damage.
An **auto** spell always hits, for less. An **area** spell hits everyone in its shape with no
roll, for less again per target, and *Evasion* halves what it does. A **touch** spell rolls to hit
at arm's length and does the most, for less mana. Only attack and touch spells can be critical
hits; a critical rolls the damage again, still held to the spell's most. Resistances take their
share of any spell's damage, and armour none.

The most a spell may do to one target is its tier's ceiling times its resolution's share:

{{#include generated/spell-resolutions.md}}

## Damage and its cap

Spell damage is always rolled dice, with a small fixed bonus at most a quarter of the ceiling,
and rarely any. A spell's dice may grow with your caster level (a die more every few levels
above its tier's comfortable level), up to its own most. Talents add to the roll: *Scroll adept*
adds 1 to every die, *Empower* rolls a die more, *Archmage* adds 2 to every die. Nothing lifts a
spell past its cap: whatever the dice, the talents, or a critical, one cast never does more to
one target than the "never more than" in the list. Two kinds of damage only magic does, **force**
and **necrotic**, join the [damage types](damage-types.md).

Spells you cast are yours: what they kill earns you its experience, and a spell of your own that
catches you can kill you.
