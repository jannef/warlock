# Magic

You cast a **spell** from the [abilities menu](abilities.md#the-abilities-menu) (`a`), where spells
have their own heading, *Spells*. Spells are paid for in [mana](mana.md). You know a spell for good
once your class gives it to you or once you have learned it from a grimoire (below).

Every spell follows the same rules: its [spell level](#spell-levels), its
[shape](spellcasting.md#shapes), and how it finds its target. Your [learning](#learning) decides
which spells you can learn. Creatures that cast use the same spells by the same rules (see
[Spells](creature-tricks.md#spells)).

More about magic:

- [How spells work](spellcasting.md): shapes, targeting, spell types, zones, and how a spell grows
  with your stats.
- [Schools and caster styles](schools.md): the seven schools of magic, the Magic page, and the
  styles you cast in.
- [Wands and rods](wands-and-rods.md): magic anyone can use.
- [Spell circles](spell-circles.md): the rings that show buffs and hexes on the map.

## What your class gives

Each casting class gives only a few spells, at the levels in its [class table](classes.md#levels):

- A full caster gets a [cantrip](#spell-levels) at level 1, and from three to a dozen spells in all.
  The [wizard](classes.md#wizard) and the [witch](classes.md#witch) also start with a short
  [cone](spellcasting.md#shapes) that needs no hit roll: [Burning hands](#spell-burning-hands) and
  [Grave breath](#spell-grave-breath). These are good against a few weak foes, but a few casts
  empty the pool.
- A bard, a paladin, or a ranger gets a few boons and hexes.
- The other classes get none.

A class's spells are only where it starts. Any spell can be learned from a grimoire, and some come
only from grimoires ([Acid arrow](#spell-acid-arrow), [Grave spear](#spell-grave-spear),
[Slumber](#spell-slumber), [Glamour](#spell-glamour), and others). A caster who finds few
grimoires stays a modest one. A spell your class gives is yours at that level, whatever your
[Decipher script](skills.md#skill-decipher-script).

{{#include generated/class-spells.md}}

Three spells are aimed at a [corpse](food.md#corpses) you see, of any age. Without one they are
refused, costing no time and no mana.

- [Charnel burst](#spell-charnel-burst) goes off where the corpse lies, without flying there. What
  it kills leaves corpses for the next one.
- [Grave feast](#spell-grave-feast) reaches only a corpse beside you or under your feet. It gives
  back hit points and mana, never past your maximum.
- [Rouse the dead](allies.md#the-risen) stands the corpse up.

[Beguile](#spell-beguile) makes a hostile creature you see your [ally](allies.md#the-charmed) for a
few minutes, against its own band. When the charm ends, or the moment you leave its level, it is
itself again, wherever it stands.

## The spells

Every spell has a **[level](#spell-levels)**, from 0 to 70. The level sets its cost in
[mana](mana.md#spending-it), and how learned a caster must be to learn it from a grimoire for
certain.

Every spell has a **base**: what it does for anyone who casts it. It may also have **ladders** for
its damage (or healing), its reach, and how long its effect lasts. A ladder names one
[stat](stats.md) and lists scores, with what the spell does at each. For example,
*[Fireball](#spell-fireball): 4d6 fire; Intelligence 13: 6d6, 16: 8d6, 19: 10d6, 25: 10d6+2*. When
your score reaches a number, you cast at that rung.

**The last rung is as far as your stats take a spell.** Your [talents](talents.md) and
[magic items](magic-items.md) may add to that in their own ways, each listed in its own row: a rung
up, a die more, bigger dice, a bonus, a die rolled again (see
[How a spell grows](spellcasting.md#how-a-spell-grows)). A [scroll](scrolls.md#spell-scrolls) you
read and a [wand](wands-and-rods.md#wands) you use cast at a rung of their own.

The list below gives each spell's level, its base (what it does and how far it reaches), its
ladders, and its [mana](mana.md#how-much) cost without armour or talents. Under "Grows with", a
stat alone is the damage or healing ladder; "range", "length", "radius", and "degrees" name the
others.

{{#include generated/spell-list.md}}

Every spell belongs to one of seven [schools of magic](schools.md): the kind of magic it is.

The abilities menu shows a spell's roll as you would cast it now, with its damage type, the way the
[character screen](character.md) shows a weapon's (`4d6 fire, 26 mana, aimed`). It also shows what
the spell costs you now, with your [talents](talents.md) and armour counted. The line below says how
far it reaches now and which dice you roll again.

**[Identify](#spell-identify)** asks which of your things to cast it on. The list of what you carry
unidentified opens, as for the [scroll of identify](scrolls.md#scroll-of-identify) (see
[Scrolls](scrolls.md#the-scroll-of-identify)). When it tells you something new, the thing's description opens.

Every [scroll](scrolls.md) holds a spell, and reading it casts that spell once. Spells are learned
for good from [grimoires](#learning-spells-from-grimoires): the grimoire of identify teaches
[Identify](#spell-identify), and the grimoire of light teaches [Light](#spell-light).

## Spell levels

Every spell has a level, from 0 to 70. It is the spell's base [mana](mana.md#spending-it) cost
(at least 4: see [What a spell costs](#what-a-spell-costs)), and the [learning](#learning) at which
you learn it from a grimoire for certain.

A spell of level 0 or 1 is a *cantrip*. It is priced as a level-4 spell, like every spell below
level 4: [Ray of frost](#spell-ray-of-frost) costs 4 mana, [Burning hands](#spell-burning-hands)
6 (a cone), [Mend](#spell-mend) 3 (a touch). Anyone trained in
[Decipher script](skills.md#skill-decipher-script) learns a cantrip for certain.

What a spell does comes from its own roll and ladders (the list above) and its
[rank](#spell-ranks), whatever its level.

## Learning

Your **learning** is how far your study of magic reaches. A grimoire of a spell at or below your
learning teaches you for certain. Above it, reading it is a check (see
[Learning spells from grimoires](#learning-spells-from-grimoires)).

Learning never changes what a spell does or costs. Your [stats](stats.md) make a spell grow (see
[How a spell grows](spellcasting.md#how-a-spell-grows)), and your [mana](mana.md#how-much) is what
holds you back.

Learning comes from the [Decipher script](skills.md#skill-decipher-script) skill and nothing else. Take the
skill's value, add what [magic items](magic-items.md) (*of the Scholar*) and [talents](talents.md)
add to it, and read it on this curve; it is at least 1. The first fifty points buy the cantrips and
the first spells. The soft cap of 100 reaches level 30, and 150 reaches the last spell level:

{{#include generated/learning.md}}

**A character not trained in Decipher script cannot learn a spell from a grimoire at all**, not even a
cantrip. Reading one is refused and takes no time. "Trained" means the skill is among your
[skills](skills.md); an item's or a talent's bonus to a skill you do not have does not train you.

Without the skill you can still read a scroll to [cast it](scrolls.md#spell-scrolls), which needs
[a check](scrolls.md#making-out-a-scroll) (with learning, you read a scroll at or below it with no
check). Your class's own spells still come at their levels.

Every class that casts starts trained in Decipher script, with the dice given in
[the class table](skills.md):

- [wizards](classes.md#wizard), [witches](classes.md#witch), [priests](classes.md#priest), and
  [druids](classes.md#druid): 2d6+10;
- [bards](classes.md#bard), [paladins](classes.md#paladin), and [rangers](classes.md#ranger): 2d4+4;
- an [aristocrat](classes.md#aristocrat) one time in three (35 %, 2d4+4);
- an [ascetic](classes.md#ascetic) one time in five (20 %, 2d4);
- a [merchant](classes.md#merchant) one time in four (25 %, 2d4).

Other classes do not start with it. Some [background stories](background.md) add to it, and
level-up rolls raise it like any other skill. So does reading: see
[Learning spells from grimoires](#learning-spells-from-grimoires).

## What a spell costs

A spell's mana cost is worked out in steps:

1. Start with its [level](#spell-levels), at least 4: a spell of level 0 to 4 counts as level 4,
   so a cantrip is never a trifle. A starting [wizard](classes.md#wizard)'s pool buys four to
   eight casts of [Ray of frost](#spell-ray-of-frost).
2. Multiply by its [shape](spellcasting.md#shapes)'s share and by its
   [resolution](spellcasting.md#how-a-spell-finds-its-target)'s share (a touch is cheaper).
3. Multiply by what your [armour](objects.md#class-body-armour) adds.
4. Round to the nearest point, once, and never below 1.
5. Apply your talents and the spell's [rank](#spell-ranks), their percents added together,
   rounding down and never below 1:
   *[School focus](talents.md#talent-school-focus)* takes 10 % off the spells of its school or
   element, *[Archmage](talents.md#talent-archmage)* 50 %, and *[Empower](talents.md#talent-empower)*
   10 %, while *[Brimming](talents.md#talent-brimming)* adds 10 %; a rank takes off what the
   spell's own table says.
6. Last, *[Frugal caster](talents.md#talent-frugal-caster)* takes a fifth off, and
   [magic items](magic-items.md) *of Thrift* their percent with it, never more than half together
   (see [Mana](mana.md)).

Your [learning](#learning) never counts. A spell you know costs the same whoever you are, and your
[mana](mana.md#how-much) is what holds you back.

A few spells have a cost of their own instead, whatever their level. **[Identify](#spell-identify)**
costs 60 [mana](mana.md#spending-it) (before your armour and talents), so no caster can cast it in
their first levels. A [wizard](classes.md#wizard) has the mana for it from about level 7, and a
[priest](classes.md#priest) or a [druid](classes.md#druid) from about level 8. Its level (6) only
says how hard it is to learn.

Heavy [armour](objects.md#class-body-armour) and big [shields](objects.md#class-shields) hinder
casting. While worn, each makes every spell cost more, and what you wear adds up.

A *Mage-wrought* piece (body armour, a [helmet](objects.md#class-helmets), or a shield that hinders
at all) hinders casting less, by its own number. Mage-wrought plate of 15 adds 25 % instead of 40 %.
A piece eased to nothing (chain mail of 10 or more) adds nothing, and no longer counts as armour for
your [caster style](schools.md#caster-styles). The ease applies only to that piece, never to
another.

{{#include generated/spell-armour.md}}

## Learning spells from grimoires

You learn a spell from its **grimoire**, a book written for teaching it. Every spell with a scroll
has a grimoire too: the grimoire of fireball teaches *[Fireball](#spell-fireball)*, the grimoire of
identify *[Identify](#spell-identify)*, the grimoire of lightning
*[Lightning bolt](#spell-lightning-bolt)*. A [scroll](scrolls.md) teaches nothing: reading one
casts its spell once.

Grimoires are rare. Wherever a scroll of a spell turns up by chance (in loot, a chest, a shop, a
creature's pack, or your [starting kit](starting-kit.md)), it is its grimoire instead one time in
ten. A scroll placed by hand stays a scroll. A grimoire weighs 800 g and is worth five times its
scroll.

A grimoire is written in plain words, so you know which spell it teaches as soon as you see it.

**Reading** a grimoire (`r`, then the grimoire) is how you learn from it. The read list shows your
chance beside each grimoire you could learn from, and asks before you begin. You keep the grimoire
whatever comes of it: read it again after a failure, or keep it to deepen the spell (see
[Spell ranks](#spell-ranks)), or sell it. Reading the grimoire of a spell you know at rank 10
only tells you so, and takes no time.

To learn at all you need some [mana](mana.md#how-much), and training in
[Decipher script](skills.md#skill-decipher-script). Without the training, reading a grimoire is refused and
takes no time ("You have no training in Decipher script; you cannot make sense of how to learn a
spell from a grimoire.").

Reading a grimoire takes time, by the spell's level: 10 minutes plus 2 more per level, at a normal
pace. For Fireball, level 11, that is 32 minutes:

{{#include generated/study-times.md}}

The world goes on around you while you read. A creature coming into view, a blow, or anything
else that would stop a rest stops the reading. The time is lost. Once you begin, you cannot stop
of your own accord.

A spell whose level is at most your [learning](#learning) is learned for certain. Above that, every
level too high (the **overreach**) makes it harder:

- You roll 2d12 plus your bonus, and must reach the difficulty.
- The difficulty is 12, plus 1 per level of overreach
  (*[Overreach](talents.md#talent-overreach)* counts a third fewer levels, rounded down), plus a
  point for every 20 levels of the spells you already know, added together.
- Your bonus is your [Intelligence](stats.md#stat-intelligence) [modifier](stats.md#modifiers), a
  tenth of your *[Decipher script](skills.md#skill-decipher-script)* (at most +20), your class's affinity with
  the spell's school, and *[Quick study](talents.md#talent-quick-study)*'s +2.
- A natural 24 always succeeds, and a natural 2 always fails.

A few spells are harder to learn than their level says. Their own difficulty is added to the check,
and such a spell is never learned for certain, however high your learning (its overreach then
counts as 0). [The spell list](#the-spells) says which, and by how much.

The game tells you the chance in words: *certain*, *easy*, *a struggle*, or *far beyond you*.

{{#include generated/study-chances.md}}

{{#include generated/study-affinities.md}}

If you fail, a **mishap** befalls you. The farther the spell was beyond you, the worse it is:
*slight* below 6 levels of overreach, *bad* from 6, and *terrible* from 15. A few lucky mishaps can
come with any of them. A failure costs only the time and the mishap: you keep the grimoire, and may
read it again.

{{#include generated/mishaps.md}}

All reading trains *[Decipher script](skills.md#skill-decipher-script)*. Reading a grimoire to its end
trains it and your [Intelligence](stats.md#stat-intelligence), whatever comes of it. Reading a
scroll trains it one time in two, and [making out a scroll](scrolls.md#making-out-a-scroll)
always does.

Casting trains *[Spellcraft](skills.md#skill-spellcraft)*: spending [mana](mana.md) on a spell
trains it one time in four. Spellcraft does nothing else for now.

## Spell ranks

Every spell you know has a **rank**, from 1 to 10. You know a spell at rank 1 when you learn it,
or when your class gives it to you. The [Magic page](schools.md#the-magic-page) shows each spell's
rank, and so does the grimoire's description.

**Reading a spell's grimoire again raises its rank.** Nothing else does: casting a spell a thousand
times leaves it where it was. Reading for rank 2 takes twice the
[learning time](#learning-spells-from-grimoires), rank 3 three times, and so on to rank 10, and the
check is always made, however high your [learning](#learning): the same 2d12 check as learning the
spell, 2 harder for every rank you already hold (the overreach counts as 0 when the spell is within
your learning). The read list shows the rank a grimoire would bring you to and your chance
("rank 3 of 10: a struggle"), and asks before you begin.

{{#include generated/deepening.md}}

If you fail, you lose only the time. There is no mishap, and you may read the book again at once,
as often as you like. If you succeed, the spell is yours at the next rank: "The words settle
deeper: you know Frost at rank 3 of 10." A rank brings no experience.

**What a rank gives** is the spell's own. Each spell has a table of what each rank from 2 adds,
and the additions stack: at rank 5 you have everything ranks 2 to 5 added. A rank may add to the
roll (a bonus, a die more, bigger dice, dice rolled again), to the reach, to how long a blessing or
hex lasts, take a share off the mana or the casting time, or, for a spell that must hit, sharpen
the aim or widen the [critical](combat.md#criticals) range. A spell with no table casts the same
at every rank. Only your own casts gain: a [scroll](scrolls.md#spell-scrolls) you read or a
[wand](wands-and-rods.md#wands) you use casts at rank 1, and so do creatures.

{{#include generated/spell-ranks.md}}
