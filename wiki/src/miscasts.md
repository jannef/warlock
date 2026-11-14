# Miscasts

Every spell you cast with your own [mana](mana.md) can go wrong. A miscast never stops the spell:
it comes *after* the spell has done everything it would have done. Then something else happens as
well, from a harmless shimmer to a blast that can kill you.

**Can a miscast kill you?** Only when you cast a spell far beyond your
[Spellcraft](skills.md#skill-spellcraft): about ten [spell levels](magic.md#spell-levels) more
than your skill and the spell's [rank](learning-spells.md#spell-ranks) carry. Below that, every miscast is
[protected](#the-protected-half): it can hurt you, but it always leaves you alive.

**What never miscasts.** [Scrolls](scrolls.md) and [wands](wands-and-rods.md#wands): their magic is
the item's, not yours. That holds for a scroll you wrote yourself with the wizard's
[Scribe](abilities.md#ability-scribe) too, and writing it makes no check either. You pay
[three times the mana](scrolls.md#writing-scrolls) for it, and in return the spell is safe to read
later, even in the middle of a fight.

**Creatures** that [cast](creature-tricks.md#spells) miscast just as you do, by the same rules and
the same table. When you see it happen, you are told: "Sparks bite the kobold shaman's fingers."

## The check

Each cast makes a [check](diseases.md#checks) once the spell has gone off: 2d12 + your **bonus**
against a difficulty. Your bonus is your [Intelligence](stats.md#stat-intelligence)
[modifier](stats.md#modifiers) plus what [Spellcraft](skills.md#skill-spellcraft) adds (the second
table below). Missing the check is a miscast.

The difficulty is 8, and then:

- **+1 for each [spell level](magic.md#spell-levels).**
- **−1 for each [rank](learning-spells.md#spell-ranks)** you hold in the spell above the first. A spell at
  rank 10 is as safe to cast as one nine levels lower. A creature's spells count as rank 1.
- **+1 for every 10 %** that the [armour](objects.md#class-body-armour) you wear adds to a spell's
  [cost](magic.md#what-a-spell-costs), counted as the cost counts it.

A roll of **1 and 1** always fails, whatever your bonus. Even a master miscasts now and then, one
cast in 144.

The chance of a miscast, by spell level and your bonus, out of armour, at rank 1. The second table
gives what Spellcraft adds to your bonus, and what it takes off a miscast's total (see
[The miscast table](#the-miscast-table)):

{{#include generated/miscast-chances.md}}

A new [wizard](class-wizard.md) has a bonus of about 3, so about one cast in fourteen of a level-1
spell goes wrong. A spell ten levels above that miscasts most of the time.

Spellcraft is how you move the odds. A wizard who puts every [skill
roll](skills.md#improving-a-skill) on Spellcraft has a bonus of about 20 at level 10 and 28 at level
20. At level 10 only a roll of 1 and 1 goes wrong on any spell up to level 15, and at level 20 on
any spell up to level 23. A spell twenty levels above them still goes wrong four casts in five,
and one in five at [rank](learning-spells.md#spell-ranks) 10. A wizard who puts no roll on it stays at
about 5 to 8 all career.

Classes that cast on the side start with little Spellcraft. A [ranger](class-ranger.md) of
level 6 casting Barkskin miscasts about one cast in four, and a new [bard](class-bard.md)
casting Mend about one in seven. That is intended. Their miscasts stay in the protected rows (see
below), and a [skill roll](skills.md#improving-a-skill) or two on Spellcraft at a level-up moves the
odds a long way. So does [deepening the spell](learning-spells.md#spell-ranks): every rank above the first
counts as one point of bonus.

Every cast trains Spellcraft the same, whether it miscasts or not.

## The miscast table

A miscast rolls on one table, ordered from mild to deadly. The higher the **total**, the worse it
gets:

- **2d12**,
- **+ 5 for each point** you missed the check by,
- **− what Spellcraft takes off** (the second table above: 10 at 100, 20 at 200),
- **± your [stance](stances.md#stances)**: Defensive takes 5 off, Aggressive adds 5, and a
  berserker's Frenzy adds 10.

The total is never below 1. How far you missed matters far more than the dice: a near miss lands in
the first rows, and a spell twenty levels beyond you lands at the bottom of the table. Some near
misses are even boons: your mana back, the spell a second time for free, a blessing, or haste.

"Someone near" in the table is anyone within 6 cells of you in a clear line, friend or foe.

{{#include generated/miscast-table.md}}

### The protected half

The results below 71 are **protected**:

- Their harm always leaves you at least 1 hit point. At 1 hit point it does nothing.
- A stat they take is [stat damage](diseases.md#stat-damage), which mends a point a day.
- They never drain a stat for good, make you forget a spell, turn the spell on you, or open a
  [gate](#gates).
- They can cost you a [rank](#forgotten-spells) or two in the spell, never below rank 1, and a
  [stat training mark](stats.md#raising-your-stats).

To reach 71 you must miss the check by 10 or more. So the worst a near miss does is leave you hurt,
out of mana, or hindered in a fight that can still kill you.

### From 71 on

Past the protected half, a miscast can kill you outright, and what it takes may be gone for good:

- **Force damage**, from 4d6 up to 20d10 at 129 and over.
- **Drained stats.** A [stat](stats.md) can lose points [for good](stats.md#drained-stats) unless
  you make a Willpower check. The worst rows drain two or three times over, each with its own
  check.
- **Forgotten spells** (see [below](#forgotten-spells)).
- **The spell turned on you.** A damage spell bursts around you, a hex falls on you, a heal mends
  the nearest foe you see, and a blessing takes hold of that foe. A summoning brings its creatures
  **berserk**: they attack you, your foes, and one another alike, whatever their kind (see
  [Factions](creatures.md#factions)). A wolf called this way is no longer a harmless beast of the
  wild. A spell that cannot turn on you this way hits you with force instead.
- **[Gates](#gates)** of the fiends, and at worst a stair into a [Pit](pits.md).

A miscast that kills you is put down to your own miscast spell.

### What you are told

Each result has its own line, the one in the table. When a result takes something, the next line
says what: the [stat](stats.md) in words ("You feel duller."), a [drain](stats.md#drained-stats)
you resisted as a chill that passes, the spell whose rank fell ("You know Fireball less well than
you did."), or the spells forgotten, by name.

## Forgotten spells

Some miscasts make you forget spells: the spell you cast, sometimes one more drawn at random, and at
worst every spell you know. A spell your [class](magic.md#what-your-class-gives) gave you is
forgotten just the same. A forgotten spell is gone from the [Magic page](schools.md#the-magic-page)
and the [abilities menu](abilities.md#the-abilities-menu), and you cannot cast it.

The only way back is its [grimoire](learning-spells.md#learning-a-spell), exactly as you
learned it the first time: the same reading, the same check, and the same experience. It comes
back at rank 1, whatever rank you had.

Lost **ranks** are milder, and can happen in the protected rows: the spell stays known, one or two
ranks lower, never below rank 1. Read its grimoire again to
[win the ranks back](learning-spells.md#spell-ranks).

## Gates

Nine of the worst results open a **gate** beside you: a rift, a hellmouth, a gate of gloom, an
abyssal, iron, black, or dire gate, an arch-gate, or the lords' gate. The deeper the result, the
stronger the fiends behind it: a rift lets [dretches](creatures.md#creature-dretch) and
[quasits](creatures.md#creature-quasit) through, the lords' gate a
[balor](creatures.md#creature-balor) or a [pit fiend](creatures.md#creature-pit-fiend).

**What a gate does.** A gate is a [rooted](creature-behaviour.md#what-they-do) creature that never strikes a
blow. Instead it [lets fiends through](creature-powers.md#summoning): one or two at a time, every
minute or more, as long as it sees you or one of your [allies](allies.md) within 8 cells. It stops
while four of its fiends are about (three for the arch-gate and the dire gate, two for the lords'
gate), and starts again when one dies.

**Whose side it is on.** The gates are a [faction](creatures.md#factions) of their own, at peace
with every monster and an enemy to you and your side alone. No monster attacks a gate. You, your
allies, and your [companion](companions.md) can, whether you set it on the gate or not. A blast,
such as a thunderclap, hurts a gate caught in it like anything else, even one a monster cast. Gates
shrug off fire and poison, and take half from cold.

**The loosed.** The fiends a gate lets through belong to no one. They attack you, your foes, the
townsfolk, and every other fiend alike, but never each other or a gate. A gate a creature's miscast
opens is no friend to that creature either: its fiends come for the caster too.

**How it ends.** A gate never closes on its own. Kill it and no more come, but the fiends already
through stay. The gates' numbers are in [Creatures in numbers](creature-numbers.md), and the fiends
each lets through in [Creature powers](creature-powers.md#summoning).

The gates a Pit's [warden](pits.md#the-warden) opens are a different matter: they stand on the
warden's side, and so does what comes through them.

## Pits

Four results go further: the floor splits and **a stair down into a [Pit](pits.md)** opens beside
you, with a rift, an abyssal gate, a dire gate, or the lords' gate at its head. Every one is a Pit
of its own. On the overworld or on a Pit's own floor no stair opens, and only the gate comes.
