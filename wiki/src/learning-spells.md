# Learning spells

You learn a spell from its grimoire, and deepen it by reading the grimoire again. Your class's own
spells come at their levels instead (see [What your class gives](magic.md#what-your-class-gives)).

## Learning a spell

You learn a spell from its **grimoire**, a book written for teaching it. Every spell with a scroll
has a grimoire too: the grimoire of fireball teaches *[Fireball](magic.md#spell-fireball)*, the grimoire of
identify *[Identify](magic.md#spell-identify)*, the grimoire of lightning
*[Lightning bolt](magic.md#spell-lightning-bolt)*. A [scroll](scrolls.md) teaches nothing: reading one
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

To learn at all you need some [mana](mana.md#how-much-mana-you-have), and training in
[Decipher script](skills.md#skill-decipher-script). Without the training, reading a grimoire is refused and
takes no time ("You have no training in Decipher script; you cannot make sense of how to learn a
spell from a grimoire.").

Reading a grimoire takes time, by the spell's level: 10 minutes plus 2 more per level, at a normal
pace. For Fireball, level 11, that is 32 minutes:

{{#include generated/study-times.md}}

The world goes on around you while you read. A creature coming into view, a blow, or anything
else that would stop a rest stops the reading. The time is lost. Once you begin, you cannot stop
of your own accord.

A spell whose level is at most your [learning](#your-learning) is learned for certain. Above that, every
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
counts as 0). [The spell list](magic.md#the-spells) says which, and by how much.

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

## Your learning

Your **learning** is how far your study of magic reaches. A grimoire of a spell at or below your
learning teaches you for certain. Above it, reading it is a check (see
[Learning a spell](#learning-a-spell)).

Learning never changes what a spell does or costs. Your [stats](stats.md) make a spell grow (see
[How a spell grows](spellcasting.md#how-a-spell-grows)), and your [mana](mana.md#how-much-mana-you-have) is what
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

- [wizards](class-wizard.md), [witches](class-witch.md), [priests](class-priest.md), and
  [druids](class-druid.md): 2d6+10;
- [bards](class-bard.md), [paladins](class-paladin.md), and [rangers](class-ranger.md): 2d4+4;
- an [aristocrat](class-aristocrat.md) one time in three (35 %, 2d4+4);
- an [ascetic](class-ascetic.md) one time in five (20 %, 2d4);
- a [merchant](class-merchant.md) one time in four (25 %, 2d4).

Other classes do not start with it. Some [background stories](background.md) add to it, and
level-up rolls raise it like any other skill. So does reading: see
[Learning a spell](#learning-a-spell).

## Spell ranks

Every spell you know has a **rank**, from 1 to 10. You know a spell at rank 1 when you learn it,
or when your class gives it to you. The [Magic page](schools.md#the-magic-page) shows each spell's
rank, and so does the grimoire's description.

**Reading a spell's grimoire again raises its rank.** Nothing else does: casting a spell a thousand
times leaves it where it was. Reading for rank 2 takes twice the
[learning time](#learning-a-spell), rank 3 three times, and so on to rank 10, and the
check is always made, however high your [learning](#your-learning): the same 2d12 check as learning the
spell, 2 harder for every rank you already hold (the overreach counts as 0 when the spell is within
your learning). The read list shows the rank a grimoire would bring you to and your chance
("rank 3 of 10: a struggle"), and asks before you begin.

{{#include generated/deepening.md}}

If you fail, you lose only the time. There is no mishap, and you may read the book again at once,
as often as you like. If you succeed, the spell is yours at the next rank: "The words settle
deeper: you know Frost at rank 3 of 10." A rank brings no experience. A
[miscast](miscasts.md#forgotten-spells) can cost you ranks, never below rank 1; you win them back
the same way.

**What a rank gives** is the spell's own. Each spell has a table of what each rank from 2 adds,
and the additions stack: at rank 5 you have everything ranks 2 to 5 added. A rank may add to the
roll (a bonus, a die more, bigger dice, dice rolled again), to the reach, to how long a blessing or
hex lasts, take a share off the mana or the casting time, or, for a spell that must hit, sharpen
the aim or widen the [critical](combat.md#criticals) range. A spell with no table casts the same
at every rank. Only your own casts gain: a [scroll](scrolls.md#spell-scrolls) you read or a
[wand](wands-and-rods.md#wands) you use casts at rank 1, and so do creatures.

{{#include generated/spell-ranks.md}}
