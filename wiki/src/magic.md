# Magic

Spells are coming. This page holds what is already decided about them: their tiers and what a
character's **caster level** is. Nothing on this page can be cast yet; the casters' spells of
today (see [Classes](classes.md#levels)) work as before, paid in [mana](mana.md).

## Caster level

Your caster level is how strong a caster you are. It is your level times your class's **casting**
share, rounded down, plus what talents and magic items may one day add; anyone with any mana has
at least caster level 1. Everyone casts a little: a level-30 warrior is a caster of level 7, a
level-30 wizard of level 30.

{{#include generated/spell-casting.md}}

## Tiers

Every spell has a **tier**, from 0 (a *cantrip*) to 9. Each tier has a **comfortable caster
level**: at or above it, learning a spell of the tier is certain and casting it costs its base
mana cost. A spell above your caster level's tier will cost more mana to cast, half again for
each tier too high. Each tier also has a **damage ceiling**: the most a single-target spell of
that tier can ever do in one cast at its highest roll. No spell is made that can pass it, and
nothing you find or learn lifts a spell over it; casting at a higher caster level brings a spell
toward its ceiling, never past.

{{#include generated/spell-tiers.md}}

## Shapes

A spell strikes one target (a ray, a bolt, a touch, or yourself) or an area (a cone, a line, or a
burst). Areas cost more mana: the base cost is multiplied by the shape's.

{{#include generated/spell-shapes.md}}

## How a spell finds its target

Each spell resolves one way. An **attack** rolls to hit like a missile and does full damage. An
**auto** spell always hits, for less. An **area** spell hits everyone in its shape with no roll,
for less again per target, and a dodge halves it. A **touch** spell rolls to hit at arm's length
and does the most, for less mana. The most a spell may do is its tier's ceiling times its
resolution's share:

{{#include generated/spell-resolutions.md}}

Spell damage is always rolled dice, with a small fixed bonus at most a quarter of the ceiling,
and rarely any. Two kinds of damage only magic does, **force** and **necrotic**, join the
[damage types](damage-types.md).
