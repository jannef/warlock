# Ranged combat

You can fight from a distance. Shoot with a **[missile weapon](ranged.md#getting-ready)** (a [bow](weapon-skills.md#weapon-skill-bows), a [crossbow](weapon-skills.md#weapon-skill-crossbows), a [sling](weapon-skills.md#weapon-skill-slings)), or
**[throw](ranged.md#shooting-and-throwing)** a throwing weapon (a dagger, a spear, a rock). [Hitting](combat.md#hitting), [criticals](combat.md#criticals), and [Defence](combat.md#defence) work as on
[Combat](combat.md). This page covers what is different.

Out of arrows or bolts? With a knife and your bow or crossbow [ready](#getting-ready), you can [fletch](skill-uses.md#fletching) more.

## Getting ready

Ready what you will shoot or throw on the [equipment](equipment.md) screen (`i`).

- The **Missile weapon slot** takes a missile weapon, any other weapon, or anything made for
  throwing, such as a stack of rocks.
- The **Missiles slot** takes the ammunition.

Each missile weapon shoots its own kind: bows shoot arrows, crossbows shoot bolts, and slings shoot
stones (see [Ammunition](ammunition.md)).

## Shooting and throwing

`t` uses whatever is in the Missile weapon slot and lets you aim with the cursor (see
[Controls](controls.md#aiming), "Aiming"). Press `Enter`, or `t` again, to loose it.

- A **missile weapon** shoots one piece of ammunition from the Missiles slot.
- A **throwing weapon** (a dagger, a spear, a rock) is thrown, one from the stack.
- **Any other weapon** is thrown whole. It leaves the slot and lands like anything thrown. It was
  not made for throwing, so it flies only 3 cells and you take a **-4 penalty to [hit](combat.md#hitting)**.
- With nothing ready: "You have nothing ready to throw or shoot." No time passes.

`T` opens a list of everything you carry that is made for throwing, so you can throw any of it
without readying it first. Your [potions](potions.md#flasks) are on it too: a potion thrown
shatters where it lands, and a flask works a spell there. A shot takes a [turn](time.md#what-takes-time), and so does a throw. A [crossbow](weapon-skills.md#weapon-skill-crossbows) takes longer, because it
must be reloaded after every shot. [Turns and movement](turns.md) gives the times.

{{#include generated/launchers.md}}

- **Hitting**: your **[Dexterity](stats.md#stat-dexterity)** helps you hit with every shot and throw.
- **Damage**: a shot does the missile weapon's own damage. A bow's draw weight is part of its dice
  (`1d8+2`), and your [Strength](stats.md#stat-strength) adds nothing. But a bow must be drawn: **each point of
  Strength you lack** below what it needs costs a point to hit and a point of damage. A crossbow
  and a sling need no Strength. A thrown weapon does its own damage plus your **[Strength](stats.md#stat-strength)**, just as a
  blow does.
- **Range**: you cannot shoot or throw farther than the range, though aiming can mark a target
  beyond it. A shot gets less sure near the end of its range, and also up close (see
  [Distance](ranged.md#distance)).
- **[Noise](hearing.md#what-makes-noise)**: a bowstring is heard some way off ("You hear a bowstring."). A throw is heard only as
  fighting, and more quietly (see [Hearing](hearing.md)).

## Distance

A shot from a missile weapon, yours or a creature's, [hits](combat.md#hitting) less often far from
the shooter and right beside them. Each creature in the way is judged by how far it stands from
the shooter when the missile reaches it, not by where the shot was aimed.

{{#include generated/shot-distance.md}}

The share of a weapon's range is measured the same way as the range itself, so each band starts in
a rough circle around the shooter. A target beside the shooter takes only the penalty for being
beside them, even with a weapon so short that the cell lies beyond a share of its range.

A target beside the shooter also keeps the [Parry](combat.md#defence) of the weapon it holds, and
may parry the missile. From anywhere else a shot ignores Parry, unless the target has learned
Measure (see [Fighting styles](fighting-styles.md#what-a-rank-teaches)). This applies to a
[Point-blank](weapon-skills.md#weapon-ability-point-blank) shot too: it is free, but the foe is
beside you.

None of this applies to thrown weapons or creatures' throws. A throw is equally sure at any
distance within its range.

## Where a missile goes

A missile flies in a straight line from you through the cell you aimed at, as far as its range. It
stops before anything solid: walls, closed [doors](doors.md), trees. A ghostly creature standing
inside the wall the missile reaches is attacked there ([Creatures](creature-bodies.md#incorporeal-creatures)). A missile that hits it is lost
in the stone. One that misses falls before the wall.

- **Anyone standing in the way** is attacked, nearest first. If the missile misses, it flies on
  and may hit someone behind. So watch who stands between you and your target.
- **Aimed at an empty cell**, the missile comes down on that cell.
- **Where it comes down**, it lies on the floor for you to pick up again: on the cell of whoever it
  hit, or where it stopped. An arrow or a bolt that hits may break (see
  [Ammunition](ammunition.md)). One that comes down where there is no floor is lost.
- **A [training dummy](weapon-skills.md#target-dummies)** stops whatever reaches it, hit or miss.
  One missile in four (25 %) is ruined, unless it never breaks; the rest drop on a free cell
  beside the dummy.

Each shot takes one piece of ammunition from your Missiles slot. When the last is gone, the slot is
empty ("You have no arrows left."). Each throw takes one of what you throw, out of your hand if you
were wielding it.

Creatures shoot the same way. The [kobold archer](creatures.md#creature-kobold-archer) carries a bow and a quiver of arrows, and its
arrows lie where they fell. Once it has shot its last arrow, it closes in and fights with its
fists. You see a missile fly wherever you see the cells it crosses. A burning one you see anywhere
in your [line of sight](sight.md#line-of-sight), even in the dark.

Giants throw too, and never run out. A [hill giant](creatures.md#creature-hill-giant) hurls boulders at you whenever you are in
range with nobody between you, and fights up close once it is next to you ("The hill giant hurls a
boulder." "The boulder strikes you."). A throw takes a giant longer than a blow, and the boulder
breaks where it lands. What each giant throws, how hard, and how far is under
[Creature numbers](creature-powers.md#throws).

## Flaming arrows

A flaming arrow is lit as it is loosed. It flies burning, lighting its way, and comes down as a
**burning arrow**. That lights the dark around it for a minute of game time, then burns away. A
burning arrow cannot be picked up.

Its [light](light.md#what-light-is) works like any other light: you see by it, and so do creatures. Shoot one into the
dark to see what waits there, or to show your friends who is watching from the dark. But anything
in its light can be seen, including you if you stand in it.

Eadric's Smithy in Cēaptūn ([Shops](shops.md)) sells slings, bows, [arrows](ammunition.md), and
bolts.
