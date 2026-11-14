# Hit points and death

Hit points measure how much harm you can take. They come back slowly with time. When they run out
you die, and death in Warlock is for good. This page also covers the blood that fights leave
behind. How blows land and how much they do is on [Combat](combat.md). Every way to get your
health back is on [Healing](healing.md).

## Hit points

Your **hit points** are your [Toughness](stats.md#stat-toughness) plus what your [race](races.md) and your
[class](classes.md) give:

{{#include generated/hit-points.md}}

A [dwarf](races.md#race-dwarf) [berserker](class-berserker.md) with Toughness 16 has 6 + 8 + 16 = 30 hit points. An [elf](races.md#race-elf) [wizard](class-wizard.md) with Toughness
8 has 10. You start every run unhurt. The [status bar](the-screen.md#the-screen) shows your hit points (`HP 18/24`), and so
does the character screen.

Creatures have hit points too, but the game never shows their numbers. They are listed on
[Creatures in numbers](creature-numbers.md#hit-points-defence-and-resistances).
[Look mode](controls-prompts.md#look-mode) tells you how hurt a creature looks:

- unhurt
- lightly wounded: more than two thirds left
- wounded: more than a third left
- badly wounded: more than a sixth left
- nearly dead

{{#include generated/health.md}}

### Getting hit points back

Hit points come back with time. They come back faster the tougher you are, and twice as fast while
you [rest](resting.md#how-resting-works). They come back slower, or not at all, while you [bleed](status-effects.md#bleeding), are [poisoned](status-effects.md#status-poisoned), or are
[diseased](status-effects.md#status-diseased). The details are on [Healing](healing.md).

A [disease](diseases.md) that lowers your [Toughness](stats.md#stat-toughness) lowers your maximum hit points too (see
[Diseases](diseases.md)). Creatures heal the same way while you are on their level. In a
[sanctuary](special-rooms.md#room-sanctuary) (a [special room](special-rooms.md)) wounds heal three times as fast, yours and
everyone else's.

## Death

A creature at 0 [hit points](#hit-points) dies: "The giant rat dies." Everything it carried, including what it
wore and wielded, falls where it stood for you to pick up, with its corpse on top (see
[Food and corpses](food.md)). A wounded creature may also die later of its wounds: "The giant rat
bleeds to death."

When **you** reach 0 [hit points](#hit-points), you die: "You die..." The run is over. A screen tells you what
killed you, where, when, and after how many turns. It names the killer if you saw it, says "Bled to
death" if your wounds did it, or says you starved (see [Hunger](hunger.md)).

There is no going back. Your save is deleted at once (see [Saving](saving.md)). Press `Enter` or
`Esc` to return to the main menu and start a new run.

## Blood

Big hits and kills leave blood where they land:

- A blow that takes at least a third of its target's [hit points](#hit-points), or any [critical](combat.md#criticals) ("hits you
  hard!"), leaves a little spatter.
- A kill leaves a pool where the victim fell, and a spray behind it, away from whoever struck the
  blow.

Only some [kinds of damage](damage-types.md) spill blood. Blades, blunt blows, [acid](damage-types.md#damage-acid), and
[force](damage-types.md#damage-force) do. [Fire](damage-types.md#damage-fire) sears the wound shut, and [cold](damage-types.md#damage-cold), [electricity](damage-types.md#damage-electricity),
[poison](damage-types.md#damage-poison), and [necrotic](damage-types.md#damage-necrotic) harm leave no wound to bleed: a creature killed by a fireball leaves no
blood, only [char](#char). A sword that cuts or stabs spills blood either way. The table on
[Damage types](damage-types.md) lists every kind.

How much blood depends on how big the victim was and how it died. A blow far beyond what was needed
to kill, or a critical, spills more. An arrow or a thrown weapon spills less, sprayed along its
flight. Blood that sprays into a wall right beside the victim stains that wall on the side facing
it.

Blood adds up. One kill is easy to miss, but where many died the floor is soaked. Fresh blood is
red and dries darker and browner over a few hours. The cave beetle bleeds a greenish-black ichor.
Water, lava, and fire take no stains.

You see blood wherever you see the floor. On a wall you see it only from the splattered side. You
remember blood as you last saw it, and [look mode](controls-prompts.md#look-mode) mentions it: "on blood-stained stone floor".

Blood is only for show. It never changes a fight, and the Blood switch in the
[settings](settings.md) hides it.

## Char

[Fire](damage-types.md#damage-fire) leaves scorch marks where it burns. The more a spot burns, the darker it gets: first a little
brown soot, then blackened, then charred through.

- A [fireball](magic.md#spell-fireball) or a fire flask chars every floor cell its blast reaches: 20, half that at the
  blast's edge.
- A cone of fire, such as [Burning hands](magic.md#spell-burning-hands) or a dragon's breath, chars every cell it covers: 20.
- A [fire zone](spell-rules.md#zones) chars the cells it burns on: 24 as it is laid.
- A creature burned by fire chars the cell it stands on: 3, plus a third of the blood the blow
  would have spilled.
- Walls beside a burning cell take 75 % of that cell's char on the side facing it (half that
  across a corner).

Each share is 60 to 140 % of the number above, at random. A cell holds at most 100: under 30 it
is scorched, under 65 blackened, then charred. Water, lava, and fire take no char.

You see and remember char the way you see and remember blood, and look mode names it first:
"A charred blood-stained stone floor." Blood on a charred floor is drawn over the char. Char is
only for show, never changes a fight, and the Blood switch hides it too.
