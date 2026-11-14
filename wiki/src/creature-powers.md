# Creature powers

Besides their blows (on [Creatures in numbers](creature-numbers.md#attacks)), many creatures have a
power of their own. They throw boulders, summon help, stare you into stone, split in two, breathe
fire, or burst. Each section explains one power and gives a table of the creatures that have it, with
its exact numbers. Thieves, blinkers, the unseen, casters, and auras are on
[Creature tricks](creature-tricks.md).

## Throws

Giants hurl boulders. A throw flies like any missile, along its line as far as its range, and
strikes the first one in its way (see [Ranged combat](ranged.md)).

A creature that throws does so whenever you are in range with nobody between you, until it is
next to you. Then it fights up close. Its throws never run out, but each takes it the time below,
and what it throws breaks where it lands.

A throw adds the thrower's [Dexterity](stats.md#stat-dexterity) modifier [to hit](combat.md#hitting)
and its [Strength](stats.md#stat-strength) modifier to the [damage](combat.md#damage), as yours do.
A boulder that does two kinds of damage does whichever hurts its target more.

**Bombers** throw [flasks](potions.md#flasks) instead: a kobold alchemist its flasks of oil, a
goblin firebomber flasks of fire, a cult chymist stinking flasks. A bomber's flask has no roll
to [hit](combat.md#hitting): it breaks where a flask you throw would, on the first one in its
way, and its spell's splash is the attack (the table names the spell). Whoever else stands in the
splash catches it too, the bomber's own kind and the bomber itself. If you see where it breaks,
you learn what the flask was, as when you throw one. Its flasks never run out either, but it
carries one or two, which fall where it dies.

{{#include generated/creature-throws.md}}

## Summoning

Summoners call up creatures of their own side. A
[goblin wizard](creatures.md#creature-goblin-wizard) calls its kin and their
[wargs](creatures.md#creature-warg), a [necromancer](creatures.md#creature-necromancer) the lesser
dead, and a [dark wizard](creatures.md#creature-dark-wizard) devils.

A summoner that sees a foe within its reach (you, or anyone its side fights) summons before it
does anything else. The creatures appear on free cells around it that it can see, nearest first.
How many come is rolled each time. They are real creatures of the summoner's side: they fight,
carry gear, drop it where they die, and give their full experience when you kill them. Killing
the summoner does not send them away.

Each summoning costs the summoner [mana](mana.md#how-much-mana-you-have), and then it must wait a while before
it can summon again. A few summoners, the breeders, pay no mana. A summoner never has more of its
summoned alive at once than the last column says. Its mana comes back as anyone's does (see
[Mana](mana.md)), so one that has spent itself summons again only much later. A summoner that
cannot cast ([silenced](status-effects.md#status-silenced)) cannot summon.

A breeder calls up its own brood while it fights, as the
[wormlord](creatures.md#creature-wormlord) calls its grave worms. Kill it first, or the fight gets
worse the longer it lasts.

You cannot summon this way.

{{#include generated/creature-summons.md}}

## Gazes

Some creatures harm with a look. A [vampire](creatures.md#creature-vampire)'s stare holds you
[transfixed](status-effects.md#status-transfixed). A [sea hag](creatures.md#creature-sea-hag)'s
leaves you [frightened](status-effects.md#status-frightened) and
[drains](stats.md#drained-stats) your Toughness. A [spectre](creatures.md#creature-spectre)'s
drains your will.

A gazer that sees you within its reach looks you in the eye before anything else except
summoning. A gaze takes as long as a spell.

The eyes must meet both ways. The gazer needs a [line of sight](sight.md#line-of-sight) to you,
and you must see it. If it stands in a cell too dark for your eyes, or behind a wall or a closed
[door](doors.md), it cannot catch your eye. You only feel its cold stare, and its look is wasted.

When the eyes meet, the gaze catches your eye at the chance in the table. It then does what its
row says, each effect by its own chance. A drain is still resisted by its check. Hit or miss, the
gazer must wait before it looks again.

A creature with no [vitals](combat.md#criticals) has no eyes to meet and is never gazed at.
Gazers look at your allies as they look at you.

### Forced gazes

A gaze marked **forced** in the table makes you **avert your eyes** by reflex while you see its
gazer. You have no choice in it and nothing to switch. With your eyes averted you fight
half-blind: −4 to [hit](combat.md#hitting), your [Dodge](combat.md#defence) halved, and no
[free blows](stances.md#free-blows). The gaze still finds a glance you cannot help, at the chance
in that column. Darkness, a wall, or a door between you ends the averting, as it ends the gaze.

The [basilisk](creatures.md#creature-basilisk)'s is the one forced gaze, and the deadliest. A
glance it catches gives you a degree of [Stiffening](status-effects.md#status-stiffening) unless
you make a [Toughness](stats.md#stat-toughness) check against 15. Each degree is told (your limbs
stiffen, your skin greys, you can barely move), and the fourth turns you to stone, which is death.

A degree eases in two minutes. A basilisk is slow, so one glimpsed and walked away from is only a
fright. The scroll of restoration, a temple's cure, or Still mind's Stillness ends the Stiffening,
and the Heal skill eases it by a degree each try.

The [cockatrice](creatures.md#creature-cockatrice)'s peck gives the same Stiffening by touch: one
peck in four, against a Toughness check of 13. The [gorgon](creatures.md#creature-gorgon)'s breath
gives it to everyone it harms (see [Breath](#breath)).

A creature that is stoned leaves no corpse. It leaves a **statue** holding what it carried (see
[Doors](doors.md) for kicking one apart, [Digging](digging.md) for digging through it).

{{#include generated/creature-gazes.md}}

## Splitting and dying into

**Splitting.** The [charnel heap](creatures.md#creature-charnel-heap) and the
[black pudding](creatures.md#creature-black-pudding) split under a blade's
[slashing](damage-types.md#damage-slashing) and [piercing](damage-types.md#damage-piercing). Such
a blow cuts the creature in two if it leaves it alive with at least twice the hit points in the
third column. The creature keeps the greater half of what it has left. A second one, with the
other half and nothing it carried, stands on the nearest free cell beside it. The new one is of
its side and hunts what it hunted. With no free cell beside it, the creature does not split.

Blunt blows, fire, acid, and every other kind of damage leave it whole. Below twice that floor of
hit points, it dies like anything else.

A half you kill gives a **quarter** of the creature's experience (an ally's kill a quarter of the
ally's share), and no [quest](quests.md) counts it.

**Dying into.** A creature that **dies into** something lets it out of its corpse as it dies. As
many come as the table says, each drawn at random, on free cells it could see within that many
cells. They are of the dead one's side. Each gives a **tenth** of its experience when killed, and
no quest counts them either. Look mode shows such a creature as *writhing*. A
[hydra](creatures.md#creature-hydra)'s heads crawl out of it this way and go on biting.

{{#include generated/creature-splits.md}}

## Breath

Dragons breathe their element. The [fire drake](creatures.md#creature-fire-drake), for one,
breathes a long fan of [fire](damage-types.md#damage-fire).

A dragon that sees you within its cone's length, with nothing solid and nobody between you,
breathes whenever its breath is ready, before anything else. The [cone](spellcasting.md#shapes)
fans out a quarter turn wide from the dragon toward you, like
[Burning hands](magic.md#spell-burning-hands) but far longer.

Everyone in the cone is struck by one roll of its damage, with no roll [to hit](combat.md#hitting).
[Evasion](talents.md#talent-evasion) halves it, and
[resistance](damage-types.md#armour-and-resistance) takes its share. The dragon's own kind is
immune to its element and takes nothing.

The element does to those it harms what any [spell of it](spellcasting.md#spell-types) does: fire
sets one in four [burning](status-effects.md#status-burning). A breath may carry more, as a blow's
added effects do. The [gorgon](creatures.md#creature-gorgon)'s grey vapour leaves those it harms
[stiffening](status-effects.md#status-stiffening) unless they make the Toughness check (the *Then*
column).

After breathing, the dragon cannot breathe again for the time below. Meanwhile it fights up close
with its claws and bite. Each is struck in turn and rolled on its own, one row each in the
[attacks table](creature-numbers.md#attacks). Unseen, a dragon's breath is heard as a roar.

{{#include generated/creature-breaths.md}}

## Bursts

Some creatures go off in a burst of their element.

**Vortices.** The motes are vortices. A [vortex](creatures.md#creature-ember-mote) has no blow at
all. Once it is beside its target it bursts instead, and is gone. Killed before it gets there, it
goes out quietly. So shoot or throw at one as it comes, or close a door on it (a mote opens none).

**Death throes.** Some creatures burst where they die, however they die:

- every elemental except the earth line;
- the [burning skull](creatures.md#creature-burning-skull);
- the [magma crawler](creatures.md#creature-magma-crawler);
- the [animated lantern](creatures.md#creature-animated-lantern) (its oil);
- the [balor](creatures.md#creature-balor).

So an elemental is best killed from a cell or more away. The earth elementals do not burst.
Instead they hurl stones at what is not beside them, as the giants do (see [Throws](#throws)), and
pound what is.

**How a burst strikes.** A burst is centred on the creature's own cell. It reaches everyone within
its radius and in its line, except creatures of its own kind. One roll of its damage strikes each
of them in full, with no roll [to hit](combat.md#hitting) and no halving at the edge.
[Evasion](talents.md#talent-evasion) halves it, and
[resistance](damage-types.md#armour-and-resistance) takes its share. It strikes your
[allies](allies.md) and other creatures as readily as you.

A burst does to the floor what an area [spell](spellcasting.md#what-lies-in-its-way) of its
element does, so the scrolls under a dying fire elemental may burn. Its element does to the cells
what such a spell's burst does: fire is left burning, water frozen. The spell types' added effects
do not come with it, but the creature's own do, by the chance below, on those it harmed. A water
elemental's wave puts out the [burning](status-effects.md#status-burning) of everyone it reaches
and the fires there. An air elemental's gust blows what lies loose on its cells a cell farther
out.

The dice of a death throe are half of what a damage spell of the creature's level rolls:
(2 + level ÷ 8)d6, rounded down. Unseen, a burst is heard as an explosion.

{{#include generated/creature-bursts.md}}
