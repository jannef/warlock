# Hit points and death

INTRO

## Hit points

Your **hit points** are your [Toughness](stats.md#stat-toughness) plus what your [race](races.md) and your
[class](classes.md) give:

{{#include generated/hit-points.md}}

A [dwarf](races.md#race-dwarf) [berserker](classes.md#berserker) with Toughness 16 has 6 + 8 + 16 = 30 hit points; an [elf](races.md#race-elf) [wizard](classes.md#wizard) with Toughness
8 has 10. You start every run unhurt. The [status bar](the-screen.md#the-screen) shows your hit points (`HP 18/24`), and so
does the character screen.

Creatures have hit points too, but you never see their numbers in the game (they are on
[Creatures in numbers](creature-numbers.md#hit-points-defence-and-resistances)).
[Look mode](controls.md#look-mode) says how hurt a creature looks: unhurt, lightly wounded (more than two
thirds left), wounded (more than a third), badly wounded (more than a sixth), or nearly dead:

{{#include generated/health.md}}

### Getting better

Hit points come back with time, faster the tougher you are, twice as fast while you
[rest](resting.md#how-resting-works), and slower or not at all while you [bleed](status-effects.md#bleeding), are [poisoned](status-effects.md#status-poisoned), or are [diseased](status-effects.md#status-diseased): see
[Healing](healing.md). A [disease](diseases.md) that lowers your [Toughness](stats.md#stat-toughness) lowers your most hit points too (see
[Diseases](diseases.md)). Creatures heal the same way, while you are on their level. In a
[sanctuary](special-rooms.md#room-sanctuary) (a [special room](special-rooms.md)) wounds heal three times as fast, yours and
anyone's.

## Death

A creature at 0 [hit points](#hit-points) dies: "The giant rat dies." It is gone, and everything it carried,
what it wore and wielded too, falls where it stood, for you to pick up, with its corpse on top
(see [Food and corpses](food.md)). A wounded creature may also die later of its wounds: "The
giant rat bleeds to death."

When **you** reach 0 [hit points](#hit-points), you die: "You die..." The run is over, and a screen tells you
what killed you (if you saw it, or "Bled to death" when your wounds did, or that you starved; see
[Hunger](hunger.md)), where, when, and after how many turns. There is no going back:
your save is deleted at once (see [Saving](saving.md)). Press `Enter` or `Esc` to return to the
main menu and start a new run.

## Blood

Big hits and kills leave blood where they land. A blow that takes at least a third of its
target's [hit points](#hit-points), or any [critical](combat.md#criticals) ("hits you hard!"), leaves a little spatter; a kill leaves a
pool where the victim fell and a spray behind it, away from whoever struck the blow. How much
depends on how big the victim was and how it died: a blow far past what killed it, or a critical,
spills more; an arrow or a thrown weapon less, sprayed along its flight. Blood that sprays into
a wall right beside the victim stains that wall on the side facing it.

Blood adds up: one kill is easy to miss, but where many died the floor is soaked. Fresh blood is
red, and it dries darker and browner over a few hours; the cave beetle bleeds a greenish-black
ichor. Water, lava, and fire take no stains. You see blood where you see the floor (on a wall,
only from the side that was splattered), remember it as you last saw it, and [look mode](controls.md#look-mode) mentions
it: "on blood-stained stone floor".

Blood is only for show: it never changes a fight, and the Blood switch in the
[settings](settings.md) hides it.
