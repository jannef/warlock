# Miscasts

Every spell you cast with your own [mana](mana.md) can go wrong. It never fails to go off: a
miscast comes *after* the spell has done everything it would have done. Then something else
happens as well, from a harmless shimmer to a blast that can kill you. Only a spell far beyond
your [Spellcraft](skills.md#skill-spellcraft) can kill you this way.

[Scrolls](scrolls.md) and [wands](wands-and-rods.md#wands) never miscast: their magic is the
item's, not yours. [Creatures](creature-tricks.md#spells) that cast miscast just as you do, and you
are told when you see it happen.

## The check

Each cast makes a [check](diseases.md#checks): 2d12 + your
[Intelligence](stats.md#stat-intelligence) [modifier](stats.md#modifiers) + what
[Spellcraft](skills.md#skill-spellcraft) adds, against a difficulty.

- The difficulty grows by 1 for each [spell level](magic.md#spell-levels).
- It drops by 1 for each [rank](magic.md#spell-ranks) you hold in the spell above the first. A
  spell at rank 10 is as safe to cast as one nine levels lower.
- [Armour](objects.md#class-body-armour) that hinders casting adds 1 for every 10 % it adds to a
  spell's [cost](magic.md#what-a-spell-costs), counted as the cost counts it.
- A roll of **1 and 1** always fails, whatever your bonus. Even a master miscasts now and then,
  one cast in 144.

Missing the check is a miscast. How far you missed it matters: see the table below.

The chance of a miscast, by spell level and your bonus (Intelligence modifier + Spellcraft's points),
out of armour, at rank 1:

{{#include generated/miscast-chances.md}}

A new [wizard](classes.md#wizard) has a bonus of about 3, so about one cast in fourteen of a level-1
spell goes wrong. A spell ten levels above that miscasts most of the time.

## The miscast table

A miscast rolls on one table, ordered from mild to deadly. The total is a 2d12 roll, plus five
times how far you missed the check, minus what wards you: Spellcraft's points and your
[stance](stances.md#stances). The higher the total, the worse it gets. How far you missed matters
far more than the dice: a near miss lands in the first rows, and a spell twenty levels beyond you
lands at the bottom of the table. Some near misses are boons, and some are plain chaos.

The results below 71 are **protected**. Their harm always leaves you at least 1 hit point, and none
of them takes anything from you for good. To roll 71 or more you must miss the check by 10 or more:
a spell about ten levels beyond what your Spellcraft and the spell's rank carry. Below that, the
worst a miscast does is leave you hurt, drained of mana, or hindered in a fight that can still kill
you.

A [Defensive stance](stances.md#stances) takes 5 off every total, even when you cast from safety.

"Someone near" is anyone within 6 cells of you in a clear line, friend or foe.

{{#include generated/miscast-table.md}}

A miscast that kills you is put down to your own miscast spell.

Every cast trains Spellcraft the same whether it miscasts or not.
