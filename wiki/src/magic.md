# Magic

You cast a **spell** from the [abilities menu](abilities.md#the-abilities-menu) (`a`), where spells
have their own heading, *Spells*. Spells are paid for in [mana](mana.md). You know a spell for good
once your class gives it to you or once you have learned it from a grimoire (see [Learning spells](learning-spells.md)), unless a
[miscast](miscasts.md#forgotten-spells) makes you forget it.

Every spell follows the same rules: its [spell level](#spell-levels), its
[shape](spellcasting.md#shapes), and how it finds its target. Your [learning](learning-spells.md#your-learning)
decides which spells you can learn. Creatures that cast use the same spells by the same rules (see
[Spells](creature-tricks.md#spells)).

More about magic:

- [How spells work](spellcasting.md): shapes, targeting, spell types, zones, and how a spell grows
  with your stats.
- [Learning spells](learning-spells.md): grimoires, your learning, and a spell's ranks.
- [Schools and caster styles](schools.md): the seven schools of magic, the Magic page, and the
  styles you cast in.
- [Miscasts](miscasts.md): what can go wrong when you cast, and the miscast table.
- [Wands and rods](wands-and-rods.md): magic anyone can use.
- [Spell circles](spell-circles.md): the rings that show buffs and hexes on the map.

## What your class gives

Each casting class gives only a few spells, at the levels in its [class table](classes.md#levels):

- A full caster gets a [cantrip](#spell-levels) at level 1, and from three to a dozen spells in all.
  The [wizard](class-wizard.md) and the [witch](class-witch.md) also start with a short
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
ladders, and its [mana](mana.md#how-much-mana-you-have) cost without armour or talents. Under "Grows with", a
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
for good from [grimoires](learning-spells.md#learning-a-spell): the grimoire of identify teaches
[Identify](#spell-identify), and the grimoire of light teaches [Light](#spell-light).

## Spell levels

Every spell has a level, from 0 to 70. It is the spell's base [mana](mana.md#spending-it) cost
(at least 4: see [What a spell costs](#what-a-spell-costs)), and the [learning](learning-spells.md#your-learning) at which
you learn it from a grimoire for certain.

A spell of level 0 or 1 is a *cantrip*. It is priced as a level-4 spell, like every spell below
level 4: [Ray of frost](#spell-ray-of-frost) costs 4 mana, [Burning hands](#spell-burning-hands)
6 (a cone), [Mend](#spell-mend) 3 (a touch). Anyone trained in
[Decipher script](skills.md#skill-decipher-script) learns a cantrip for certain.

What a spell does comes from its own roll and ladders (the list above) and its
[rank](learning-spells.md#spell-ranks), whatever its level.

## What a spell costs

A spell's mana cost is worked out in steps:

1. Start with its [level](#spell-levels), at least 4: a spell of level 0 to 4 counts as level 4,
   so a cantrip is never a trifle. A starting [wizard](class-wizard.md)'s pool buys four to
   eight casts of [Ray of frost](#spell-ray-of-frost).
2. Multiply by its [shape](spellcasting.md#shapes)'s share and by its
   [resolution](spellcasting.md#how-a-spell-finds-its-target)'s share (a touch is cheaper).
3. Multiply by what your [armour](objects.md#class-body-armour) adds.
4. Round to the nearest point, once, and never below 1.
5. Apply your talents and the spell's [rank](learning-spells.md#spell-ranks), their percents added together,
   rounding down and never below 1:
   *[School focus](talents-mind.md#talent-school-focus)* takes 10 % off the spells of its school or
   element, *[Archmage](class-wizard.md#talent-archmage)* 50 %, and *[Empower](class-wizard.md#talent-empower)*
   10 %, while *[Brimming](talents-mind.md#talent-brimming)* adds 10 %; a rank takes off what the
   spell's own table says.
6. Last, *[Frugal caster](talents-mind.md#talent-frugal-caster)* takes a fifth off, and
   [magic items](magic-items.md) *of Thrift* their percent with it, never more than half together
   (see [Mana](mana.md)).

Your [learning](learning-spells.md#your-learning) never counts. A spell you know costs the same whoever you are, and your
[mana](mana.md#how-much-mana-you-have) is what holds you back.

A few spells have a cost of their own instead, whatever their level. **[Identify](#spell-identify)**
costs 60 [mana](mana.md#spending-it) (before your armour and talents), so no caster can cast it in
their first levels. A [wizard](class-wizard.md) has the mana for it from about level 7, and a
[priest](class-priest.md) or a [druid](class-druid.md) from about level 8. Its level (6) only
says how hard it is to learn.

Heavy [armour](objects.md#class-body-armour) and big [shields](objects.md#class-shields) hinder
casting. While worn, each makes every spell cost more, and what you wear adds up.

A *Mage-wrought* piece (body armour, a [helmet](objects.md#class-helmets), or a shield that hinders
at all) hinders casting less, by its own number. Mage-wrought plate of 15 adds 25 % instead of 40 %.
A piece eased to nothing (chain mail of 10 or more) adds nothing, and no longer counts as armour for
your [caster style](schools.md#caster-styles). The ease applies only to that piece, never to
another.

{{#include generated/spell-armour.md}}

See also: [Miscasts](miscasts.md), [Wands and rods](wands-and-rods.md), and
[Spell circles](spell-circles.md).
