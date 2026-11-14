# Spell types

Every spell of a [damage type](damage-types.md) carries that type's rule, whether it is cast from
[mana](mana.md#spending-it), a [scroll](scrolls.md#spell-scrolls), or a
[wand](wands-and-rods.md#wands). The rules belong to magic alone: a
[Flaming](affixes.md#prefixes) sword's blow follows none of them. They spare no one: you and
your [allies](allies.md) are struck like anyone else the spell reaches.

{{#include generated/spell-types.md}}

**Fire catches.** One creature in four that a fire spell harms is left
[burning](status-effects.md#burning), whatever the spell. A [Fireball](magic.md#the-spells) on five
kobolds leaves one or two alight. A creature that takes no harm from it never burns.

A fire burst melts the ice on the cells it reaches, and whoever stands on that ice
[falls in](temperature.md#water-freezes). It leaves a fire on the cell at its centre, which gives
light for a minute. That fire does nothing else, and it never spreads.

**Cold slows the blood.** Every creature a cold spell harms is also
[Sluggish](status-list.md#status-sluggish), and anything burning on it goes out. A miss does
nothing. A cold burst freezes the [water](terrain.md#water) on the cells it reaches into ice that
holds a laden person, and puts out the fires there.

**Lightning seeks the nearest path.** When a ray, a bolt, a touch, or a line of electricity harms a
creature, one time in four it strikes once more. It hits the nearest creature beside that one that
has not been struck yet (ties go to the one that came into the world first), at the same roll.
Neither a roll to hit nor [Evasion](talents.md#talent-evasion) saves anyone from it. That creature
may arc on again, and so on. It is not choosy: [Spark](magic.md#the-spells) at a rat beside you may
jump back into you, or into your [ally](allies.md).

**Water carries it.** A creature harmed while standing in [water](terrain.md#water) is part of that
whole body of water: every cell of open water connected to it, diagonals too. Everyone standing in
that body who has not been struck yet is struck at the same roll, you too if you are wading. They
are struck one after another, in the order they came into the world, and each of them may arc.
Every electricity spell does this, whatever its shape.

[Ice](temperature.md#water-freezes) is not water. It carries nothing, and a frozen band splits a
pond in two. Each creature only ever takes one hit from a cast. Aiming shows you none of this.

**Chain lightning** takes its first jumps for certain: three at first, and up to six as your
[Willpower](stats.md#stat-willpower) grows. Any of its jumps reaches two cells instead of one, as
long as nothing that blocks sight stands between. After those, a jump happens one time in four as
usual. A chain ends when no creature that has not been struck is within reach of the last one, or
when the last one took no harm.

**[Conductor](talents.md#talent-conductor)** makes every arc of your electricity spells happen, and
you resist electricity 25 %. It spares you nothing; it only makes your own lightning bite less.

**A ray, a bolt, or a touch** does to one cell what a burst does to all of its cells: to the cell
of the creature it strikes, or else the cell you aimed at. So [Ray of frost](magic.md#the-spells)
builds a bridge across a lake, one cell per cast. A line does it to the last cell it reaches; a
cone does not do it at all.

**Force pushes.** A force spell that takes at least a third of a creature's
[hit points](hit-points.md#hit-points), or lands a [critical](combat.md#criticals), knocks it a cell
away. It is pushed away from the centre of a burst, or otherwise away from you (and away from you
too if it is the one at the burst's centre). A cell that is not free stops the push, and so does
[being held or webbed](status-list.md#status-held). You do not move when you are at the centre of
your own burst, though you take the harm.

A force burst with a blast breaks the [shut doors](doors.md) it reaches, however sturdy, locked or
trapped. A trap on the door goes with it and strikes no one. [Chests](chests.md) break under any
blast below their hardness. A force blast also shatters thin [ice](temperature.md#water-freezes),
and whoever stands on that ice falls in. A fire blast leaves doors standing.

**Poison lingers.** Every creature a poison spell harms is [Poisoned](status-effects.md#poisoned)
one degree, on top of whatever else the spell does.

**Death takes the dying first.** A necrotic spell does half again as much, rounded down, to a
creature below half its hit points, before what it resists. The undead are immune to necrotic harm.

**Acid keeps eating.** Every creature an acid spell harms is
[Corroded](status-effects.md#corroded) one degree: two more hit points over the next twelve seconds.
Acid that harms you also eats at what you wear: the first piece (your body armour first) is
[damaged](materials.md#item-damage) unless its material withstands acid by its chance, and a piece already
damaged is eaten away. Acid eats no creature's gear.

## What lies in its way

An area spell (a burst, a cone, or a line; see [shapes](spellcasting.md#shapes)) with a
[damage type](damage-types.md) may destroy [items lying on the floor](objects.md#piles) it reaches,
whether it is cast from [mana](mana.md#spending-it), a
[scroll](scrolls.md#spell-scrolls), or a [wand](wands-and-rods.md#wands). So may a
dragon's breath. A [Fireball](magic.md#the-spells) can burn the scrolls it lands on, and a blast of
[Frost](magic.md#the-spells) can shatter potions.

Each item in each stack there is lost at the chance given by the first row below that fits it, for
that spell type. A row fits by the item's [item class](objects.md#item-classes) or the
[material](materials.md#materials) of its main part. So a stack may lose some of its items or all
of them. A stack that no row fits is never touched.

Some things are never touched:

- what you carry or wear;
- what lies in a [chest](chests.md);
- what is [protected from item damage](materials.md#protected-items).

[Traps](traps.md) ruin nothing. A [corpse](food.md) that burns away never rises. You are told what
you see go: "The scroll burns up.", "3 potions freeze and shatter."

{{#include generated/spell-ruin.md}}
