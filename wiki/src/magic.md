# Magic

A **spell** is cast from the [abilities menu](abilities.md#the-abilities-menu) (`a`), under its own heading *Spells*, and paid in
[mana](mana.md). You know a spell for good once your class gives it or once you have learned it
from a scroll (below). Every spell is measured by the same rules below: its [spell level](#spell-levels), its
[shape](spellcasting.md#shapes), and how it finds its target; your [learning](#learning) is what you can learn.
Creatures that cast use the same spells by the same rules (see [Spells](creature-powers.md#spells)).

## What your class gives

Each casting class gives only a few spells, with its [class table](classes.md#levels): a full
caster a [cantrip](#spell-levels) at level 1 and from three to a dozen in all (the [wizard](classes.md#wizard) and the [witch](classes.md#witch) also start
with a short [cone](spellcasting.md#shapes) that needs no hit roll, [Burning hands](#spell-burning-hands) and [Grave breath](#spell-grave-breath): good for a
fight with a few weak foes, and a few casts empty the pool), a bard, a paladin, or a ranger a few
boons and hexes, and the rest none. A class's spells are only where it meets them first: any
spell can be learned from a scroll, and some come only from scrolls ([Acid arrow](#spell-acid-arrow),
[Grave spear](#spell-grave-spear), [Slumber](#spell-slumber), [Glamour](#spell-glamour), and others), so a caster who finds few
scrolls stays a modest one. A spell your class gives is yours at that level, whatever your [Spellcraft](skills.md#skill-spellcraft).

{{#include generated/class-spells.md}}

Three spells are aimed at a [corpse](food.md#corpses) you see, and any age will do: without one
they are refused, for no time and no mana. [Charnel burst](#spell-charnel-burst) goes off where
the corpse lies, flying nowhere, and what it kills leaves corpses for the next. [Grave
feast](#spell-grave-feast) reaches only a corpse beside you or under your feet, and gives back
hit points and mana, never past your maximum. [Rouse the dead](allies.md#the-risen) stands it up.

[Beguile](#spell-beguile) makes a hostile creature you see your [ally](allies.md#the-charmed) for
a few minutes, against its own band; it is itself again where it stands when the charm ends, or
the moment you leave its level.

## The spells

Every spell has a **[level](#spell-levels)**, 0 to 70: what it costs in [mana](mana.md#spending-it), and how learned a caster must
be to take it from a scroll for certain. Every spell has a **base**, what it does for anyone who
casts it, and may have **ladders**: for its damage (or healing), its reach, and how long its
effect lasts. A ladder names one [stat](stats.md) and lists scores with what the spell does at each:
*[Fireball](#spell-fireball): 4d6 fire; Intelligence 13: 6d6, 16: 8d6, 19: 10d6, 25: 10d6+2*. When your score
reaches a number, you cast that rung. **The last rung is as far as your stats take it.** Your
[talents](talents.md) and [magic items](magic-items.md) may add to that in their own ways, each said in its own row: a rung
up, a die more, bigger dice, a bonus, a die rolled again (see [How a spell grows](spellcasting.md#how-a-spell-grows)). A
[scroll](scrolls.md#spell-scrolls) read and a [wand](wands-and-rods.md#wands) used cast at a rung of their own.

The list below gives each spell's level, its base (what it does and how far it reaches), its
ladders ("Grows with": a stat alone is the damage or healing ladder; "range", "length",
"radius", and "degrees" name the others), and its [mana](mana.md#how-much) without armour or talents:

{{#include generated/spell-list.md}}

Every spell belongs to one of seven [schools of magic](schools.md), and each school is a skill
you train by casting its spells: your rank in it rolls them stronger, cheaper, or quicker.

The abilities menu shows a spell's roll as you would cast it now, with its damage type, as the
[character screen](character.md) shows a weapon's (`4d6 fire, 26 mana, aimed`), and what it costs
you now, with your [talents](talents.md) and armour counted; the line below it says how far it
reaches now and which dice you roll again. **[Identify](#spell-identify)** asks which of your things to cast it on: the list of
what you carry unidentified opens, as for the [scroll of identify](scrolls.md#scroll-of-identify) (see
[Scrolls](scrolls.md#the-scroll-of-identify)).

Every [scroll](scrolls.md) is a spell: each one you can read casts a spell, and each teaches its spell if you
study it, the [scroll of identify](scrolls.md#scroll-of-identify) [Identify](#spell-identify) and the [scroll of light](scrolls.md#scroll-of-light) [Light](#spell-light).

## Spell levels

Every spell has a level, 0 to 70. It is the spell's base [mana](mana.md#spending-it) cost, and the [learning](#learning) at which you
learn it from a scroll for certain. A spell of level 0 or 1 is a *cantrip*: it costs 1 mana (but for the few that
name a price of their own: [Burning hands](#spell-burning-hands) and [Grave breath](#spell-grave-breath) cost 3), and
anyone trained in [Spellcraft](skills.md#skill-spellcraft) learns it for certain. What a spell does is its own roll and ladders (the list
above), whatever its level.

## Learning

Your **learning** is how far your study of magic reaches: a scroll of a spell at or below it
teaches you for certain; above it, the study is a check (see [Learning spells from
scrolls](#learning-spells-from-scrolls)). It never changes what a spell does or costs: your
[stats](stats.md) grow a spell (see [How a spell grows](spellcasting.md#how-a-spell-grows)), and your
[mana](mana.md#how-much) is what holds you back.

It comes from the [Spellcraft](skills.md#skill-spellcraft) skill and nothing else: the skill's
value, with what [magic items](magic-items.md) (*of the Scholar*) and [talents](talents.md) add
to it, read on this curve, and at least 1. The first fifty points buy the cantrips and the first
spells, the soft cap of 100 reaches level 30, and 150 the last spell level:

{{#include generated/learning.md}}

**A character not trained in Spellcraft cannot learn a spell from a scroll at all**, cantrips
included: the study is refused, and takes no time. Trained means the skill is among your
[skills](skills.md); an item's or a talent's bonus to a skill you do not have does not train you.
You can still read a scroll to [cast it](scrolls.md#spell-scrolls), [by a check](scrolls.md#making-out-a-scroll) (your learning reads a scroll at or
below it with none), and your class's own spells still come at their levels.

Every class that casts starts trained, and [its dice are in the class table](skills.md):
[wizards](classes.md#wizard), [witches](classes.md#witch), [priests](classes.md#priest), and
[druids](classes.md#druid) 2d6+10; [bards](classes.md#bard), [paladins](classes.md#paladin), and
[rangers](classes.md#ranger) 2d4+4; an [aristocrat](classes.md#aristocrat) one time in three (35 %, 2d4+4)
and an [ascetic](classes.md#ascetic) one in five (20 %, 2d4). Other classes do not start with it. Two
[background stories](background.md) add to it, for classes that already have it, and level-up rolls raise it like any other skill.

## What a spell costs

A spell's mana is its [level](#spell-levels), at least 1, times its [shape](spellcasting.md#shapes)'s share (below), times its
[resolution](spellcasting.md#how-a-spell-finds-its-target)'s (a touch is cheaper), times what your [armour](objects.md#class-body-armour) adds, rounded to the
nearest point once at the end and never below 1. Your [learning](#learning) never counts: a spell you know
costs the same whoever you are, and your [mana](mana.md#how-much) is what holds you back. Then your talents, their
percents added up and rounded down, never below 1: *[School focus](talents.md#talent-school-focus)* takes 10 % off the spells of its
school or element, *[Archmage](talents.md#talent-archmage)* 50 %, and *[Empower](talents.md#talent-empower)* 10 %, and *[Brimming](talents.md#talent-brimming)* adds 10 %; and then *[Frugal caster](talents.md#talent-frugal-caster)* takes a fifth off,
and [magic items](magic-items.md) *of Thrift* their percent with it, never more than half together (see [Mana](mana.md)).

A few spells have a cost of their own instead, whatever their level: **[Identify](#spell-identify)** costs 60 [mana](mana.md#spending-it)
(before your armour and talents), so no caster can cast it in their first levels. A [wizard](classes.md#wizard) has
the mana for it from about level 7, a [priest](classes.md#priest) or a [druid](classes.md#druid) from about level 8. Its level (6) says only
how hard it is to learn.

Heavy [armour](objects.md#class-body-armour) and big [shields](objects.md#class-shields) hinder casting: while worn, each makes every spell dearer, and what
you wear adds up. A *Mage-wrought* piece hinders casting less by its own number (body armour, a
[helmet](objects.md#class-helmets), or a shield that hinders at all): Mage-wrought plate of 15 adds 25 % instead of 40 %, and
one eased to nothing (chain mail of 10 or more) adds none and no longer counts as armour for
your [caster style](schools.md#caster-styles). Its ease is its own: it never eases
another piece.

{{#include generated/spell-armour.md}}

## Learning spells from scrolls

Every spell comes on a scroll of its own (the [scroll of magic missile](scrolls.md#scroll-of-magic-missile) teaches *[Magic missile](#spell-magic-missile)*);
the [scroll of fireball](scrolls.md#scroll-of-fireball) teaches *[Fireball](#spell-fireball)*, the [scroll of lightning](scrolls.md#scroll-of-lightning) *[Lightning bolt](#spell-lightning-bolt)*, the [scroll of
frost](scrolls.md#scroll-of-frost) *[Frost](#spell-frost)*. **[Reading](scrolls.md)** a scroll casts its spell once at the scroll's own rung and costs no
[mana](mana.md#spending-it) ([Scrolls](scrolls.md#spell-scrolls)). **Studying** it (`L`, then the scroll) teaches you its
spell for good, and uses the scroll up whatever comes of it.

You can study a scroll once you know what it is (you have read one like it, or [identified](magic-items.md#unidentified-items) it)
and only if you do not know its spell yet; an unknown scroll is refused ("You do not know what
this says."), so [identify](magic-items.md#unidentified-items) your scrolls first. A scroll of a spell you know can still be read,
sold, or given away. You need some [mana](mana.md#how-much) to learn at all, and training in
[Spellcraft](skills.md#skill-spellcraft): without it the study is refused, and takes no time ("You have no training in
Spellcraft; you cannot make sense of how to learn a spell from a scroll.").

A study takes time, by the spell's level: 10 minutes and 2 more a level, at a normal pace (Fireball,
level 11: 32 minutes):

{{#include generated/study-times.md}}

The world goes on around you. A creature coming into view, a blow, or anything else that would
stop a rest stops the study: the time is lost, but you keep the scroll. Once you begin, you
cannot stop of your own accord.

A spell of a level at most your [learning](#learning) is learned for certain. Above it, every level too
high (the **overreach**) makes it harder: a roll of 2d12 plus your bonus must reach the difficulty,
12 + 1 per level of overreach (*[Overreach](talents.md#talent-overreach)* counts a third fewer, rounded down), a point more for
every 20 levels of the spells you already know, added up. Your bonus is your [Intelligence](stats.md#stat-intelligence) [modifier](stats.md#modifiers), a tenth of your
*[Spellcraft](skills.md#skill-spellcraft)* (at most +20), your class's affinity with the
spell's school, and *[Quick study](talents.md#talent-quick-study)*'s +2. A natural 24 always succeeds and a natural 2 always fails.
A few spells are harder to learn than their level says: their own difficulty is added to the
check, and a spell made harder this way is never learned for certain, however high your
learning (its overreach then counts as 0). [The spell list](#the-spells) says which, and by how much.
The game tells you the chance in words: *certain*, *easy*, *a struggle*, or *far beyond you*.

{{#include generated/study-chances.md}}

{{#include generated/study-affinities.md}}

If you fail, the scroll crumbles to dust and a **mishap** befalls you, worse the farther the
spell was beyond you: *slight* below 6 levels of overreach, *bad* from 6, *terrible* from
15. A few lucky mishaps can come with any of them. A failure costs only that scroll:
the spell may come your way again.

{{#include generated/mishaps.md}}

Studying trains *[Spellcraft](skills.md#skill-spellcraft)* and your [Intelligence](stats.md#stat-intelligence), whatever comes of it. *[Decipher script](skills.md#skill-decipher-script)* has no part in a
study: it is for [making out a scroll](scrolls.md#making-out-a-scroll) you only want to read.
Spending [mana](mana.md) on a spell also trains *[Spellcraft](skills.md#skill-spellcraft)*, one time in four.
