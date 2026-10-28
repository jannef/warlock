# Ranged combat

You can fight from a distance: shoot with a **[missile weapon](ranged.md#getting-ready)** (a [bow](weapon-skills.md#weapon-skill-bows), a [crossbow](weapon-skills.md#weapon-skill-crossbows), a [sling](weapon-skills.md#weapon-skill-slings)) or
**[throw](ranged.md#shooting-and-throwing)** a throwing weapon (a dagger, a spear, a rock). [Hitting](combat.md#hitting), [criticals](combat.md#criticals), and [Defence](combat.md#defence) work as in
[Combat](combat.md); here is what is different.

## Getting ready

Ready what you will shoot or throw on the [equipment](equipment.md) screen (`i`). The **Missile
weapon slot** takes a missile weapon, or any weapon, or anything made for throwing (a stack of
rocks); the **Missiles slot** takes the ammunition. Each missile weapon shoots its own kind: bows
shoot arrows, crossbows bolts, slings stones (see [Ammunition](ammunition.md)).

## Shooting and throwing

`t` uses whatever is in the Missile weapon slot, and aims it with the cursor (see
[Controls](controls.md#aiming), "Aiming"); press `Enter` (or `t` again) to loose it:

- a **missile weapon** shoots a piece of its ammunition from the Missiles slot;
- a **throwing weapon** (a dagger, a spear, a rock) is thrown, one of a stack;
- **any other weapon** is thrown whole: it leaves the slot and lands like anything thrown. It was
  not made for it, so it flies only 3 cells and you take a **-4 penalty to [hit](combat.md#hitting)**;
- with nothing there: "You have nothing ready to throw or shoot." No time passes.

`T` opens a list of what you carry that is made for throwing, to throw any of it without readying
it first. A shot takes a [turn](time.md#what-takes-time) (a [crossbow](weapon-skills.md#weapon-skill-crossbows) longer: it must be reloaded after every shot), and so
does a throw; see [Turns and movement](turns.md) for the times.

{{#include generated/launchers.md}}

- **Hitting**: your **[Dexterity](stats.md#stat-dexterity)** helps you hit, as it does with every shot and throw.
- **Damage**: a shot does the missile weapon's own damage. A bow carries its draw weight in its
  dice (`1d8+2`), and your [Strength](stats.md#stat-strength) adds nothing, but a bow must be drawn: **each point of
  Strength you lack** of what it needs costs a point to hit and a point of damage. A crossbow and
  a sling need no Strength. A thrown weapon does its own damage plus your **[Strength](stats.md#stat-strength)**, as a blow
  does.
- **Range**: you cannot shoot or throw farther than the range. Aiming marks a target beyond it.
- **[The noise](hearing.md#what-makes-noise)**: a bowstring is heard some way off ("You hear a bowstring."); a throw only as
  fighting, more quietly (see [Hearing](hearing.md)).

## Where a missile goes

A missile flies along a straight line from you through the cell you aimed at, as far as its range,
and stops before anything solid: walls, closed [doors](doors.md), trees. A ghostly creature standing inside
the wall it reaches is attacked there ([Creatures](creatures.md#incorporeal-creatures)): a missile
that hits it is lost in the stone, one that misses falls before the wall.

- **Whoever stands in its way** is attacked, the nearest first. If it misses, the missile flies
  on, and may hit someone behind. So mind who stands between you and your target.
- **Aimed at an empty cell**, it comes down on that cell.
- **Where it comes down** it lies, on the floor, for you to pick up again: on the cell of whom it
  hit, or where it stopped. An arrow or a bolt that hits may break (see
  [Ammunition](ammunition.md)); one that comes down where there is no floor is lost.

Each shot takes one piece of ammunition from your Missiles slot; when the last is gone, the slot
is empty ("You have no arrows left."). Each throw takes one of what you throw, out of your hand if
you were wielding it.

Creatures shoot the same way: the [kobold archer](creatures.md#creature-kobold-archer) carries a bow and a quiver of arrows, and its
arrows lie where they fell. Once it has shot its last arrow it closes in and fights with its
fists. You see a missile fly where you see the cells it flies over; a burning one you see
wherever it flies in your [line of sight](sight.md#line-of-sight), even in the dark.

Giants throw too, with nothing to run out of: a [hill giant](creatures.md#creature-hill-giant) hurls boulders at you whenever you
are in range with nobody between you, and fights up close once it is next to you ("The hill
giant hurls a boulder." "The boulder strikes you."). A throw takes a giant longer than a blow,
and the boulder breaks where it lands. What each giant throws, how hard, and how far is under
[Creature numbers](creature-numbers.md#throws).

## Flaming arrows

A flaming arrow is lit as it is loosed. It flies burning, lighting its way, and comes down as a
**burning arrow**, which lights the dark around it for a minute of game time, then has burned
away. Its [light](light.md#what-light-is) is light like any other: you see by it, and so do the creatures. Shoot one into
the dark to see what waits there, or to show the dark's watchers to your friends; but anything
lit by it is seen, and you too, if you stand in its light. A burning arrow cannot be picked up.

Ælfric's General Store and Eadric's Forge in Cēaptūn ([Shops](shops.md)) sell slings, bows,
and [arrows](ammunition.md).
