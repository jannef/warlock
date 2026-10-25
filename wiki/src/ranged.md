# Ranged combat

You can fight from a distance: shoot with a **missile weapon** (a bow, a crossbow, a sling) or
**throw** a throwing weapon (a dagger, a spear, a rock). Hitting, criticals, and Defence work as in
[Combat](combat.md); here is what is different.

## Getting ready

Ready what you will shoot or throw on the [equipment](equipment.md) screen (`i`). The **Missile
weapon slot** takes a missile weapon, or any weapon, or anything made for throwing (a stack of
rocks); the **Missiles slot** takes the ammunition. Each missile weapon shoots its own kind: bows
shoot arrows, crossbows bolts, slings stones (see [Ammunition](ammunition.md)).

## Shooting and throwing

`t` uses whatever is in the Missile weapon slot, and aims it with the cursor (see
[Controls](controls.md), "Aiming"); press `Enter` (or `t` again) to loose it:

- a **missile weapon** shoots a piece of its ammunition from the Missiles slot;
- a **throwing weapon** (a dagger, a spear, a rock) is thrown, one of a stack;
- **any other weapon** is thrown whole: it leaves the slot and lands like anything thrown. It was
  not made for it, so it flies only 3 cells and you take a **-4 penalty to hit**;
- with nothing there: "You have nothing ready to throw or shoot." No time passes.

`T` opens a list of what you carry that is made for throwing, to throw any of it without readying
it first. A shot takes a turn (a crossbow longer: it must be reloaded after every shot), and so
does a throw; see [Turns and movement](turns.md) for the times.

{{#include generated/launchers.md}}

- **Hitting**: your **Dexterity** helps you hit, as it does with every shot and throw.
- **Damage**: a shot does the missile weapon's own damage. A bow carries its draw weight in its
  dice (`1d8+2`), and your Strength adds nothing, but a bow must be drawn: **each point of
  Strength you lack** of what it needs costs a point to hit and a point of damage. A crossbow and
  a sling need no Strength. A thrown weapon does its own damage plus your **Strength**, as a blow
  does.
- **Range**: you cannot shoot or throw farther than the range. Aiming marks a target beyond it.
- **The noise**: a bowstring is heard some way off ("You hear a bowstring."); a throw only as
  fighting, more quietly (see [Hearing](hearing.md)).

## Where a missile goes

A missile flies along a straight line from you through the cell you aimed at, as far as its range,
and stops before anything solid: walls, closed doors, trees.

- **Whoever stands in its way** is attacked, the nearest first. If it misses, the missile flies
  on, and may hit someone behind. So mind who stands between you and your target.
- **Aimed at an empty cell**, it comes down on that cell.
- **Where it comes down** it lies, on the floor, for you to pick up again: on the cell of whom it
  hit, or where it stopped. An arrow or a bolt that hits may break (see
  [Ammunition](ammunition.md)); one that comes down where there is no floor is lost.

Each shot takes one piece of ammunition from your Missiles slot; when the last is gone, the slot
is empty ("You have no arrows left."). Each throw takes one of what you throw, out of your hand if
you were wielding it.

Creatures shoot the same way: the kobold archer carries a bow and a quiver of arrows, and its
arrows lie where they fell. Once it has shot its last arrow it closes in and fights with its
fists. You see a missile fly where you see the cells it flies over; a burning one you see
wherever it flies in your line of sight, even in the dark.

## Flaming arrows

A flaming arrow is lit as it is loosed. It flies burning, lighting its way, and comes down as a
**burning arrow**, which lights the dark around it for a minute of game time, then has burned
away. Its light is light like any other: you see by it, and so do the creatures. Shoot one into
the dark to see what waits there, or to show the dark's watchers to your friends; but anything
lit by it is seen, and you too, if you stand in its light. A burning arrow cannot be picked up.

The showcase's storeroom has bows, a crossbow, a sling, flaming arrows, and rocks; the patrol
gallery beyond the corridor is dark in the middle. In the arena, a bow and arrows lie beside
you in your dark corner.
