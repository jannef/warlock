# Damage types

Every blow, shot, trap, and spell does damage of a certain **kind**. Weapons do the three physical
kinds: [slashing](damage-types.md#damage-slashing), [piercing](damage-types.md#damage-piercing), and [bludgeoning](damage-types.md#damage-bludgeoning). The elements are [fire](damage-types.md#damage-fire), [cold](damage-types.md#damage-cold), [electricity](damage-types.md#damage-electricity), [acid](damage-types.md#damage-acid), and [poison](damage-types.md#damage-poison).
Two more kinds come only from [magic](magic.md): [force](damage-types.md#damage-force) and [necrotic](damage-types.md#damage-necrotic).

The kind decides what armour does against the damage, what resists it, and whether it makes you
[bleed](status-effects.md#bleeding), and whether it spills [blood](hit-points.md#blood) on the floor or leaves [char](hit-points.md#char). It also decides how the message reads: "The kobold stabs you.", "You cleave the giant
rat.", "The fireball burns the kobold."

{{#include generated/damage-types.md}}

## What weapons do

Each weapon does one kind of damage, or two. A sword can cut or stab. Against each target it uses
**whichever is better**: first the kind the target resists least, then the kind its armour stops
least.

A flaming arrow stabs and burns. Its [fire](damage-types.md#damage-fire) is **added damage**: it is rolled after the arrow's own
damage, and armour never soaks it. Magic weapons can add any element except force in the same way
(see [Magic items](magic-items.md) and the table below).

{{#include generated/damage-weapons.md}}

Creatures' bites stab, their claws cut, and their hooves and slams strike. A shield bash, a kick,
and bare hands strike too.

## Armour and resistance

**Protection** (see [Combat](combat.md#protection)) soaks physical damage only. A [fireball](magic.md#spell-fireball), a
[lightning bolt](magic.md#spell-lightning-bolt), the [cold](damage-types.md#damage-cold) of a [plunge through the ice](temperature.md#falling-in), and [poison](damage-types.md#damage-poison) all go straight through
mail. A piece of armour may do better or worse against one physical kind. Chain mail turns a blade
well and a point badly.

**Resistance** takes a share off damage of one kind, in percent, after armour has done its part.
A fur cloak resists [cold](damage-types.md#damage-cold), a leather jerkin resists [acid](damage-types.md#damage-acid) a little, and magic *of* an element resists
that element (see the table below).

Everything adds up: your race, what you wear, magic, [talents](talents.md), and [potions](potions.md). There is no cap on
the total, but **at most 75 % counts**. With 200 % fire resistance you take a quarter of any
fire. Lose 100 % of that again and you are still at 75 %, since 100 % remains. Below zero you take
more, but at worst double: −100 % is the floor of what counts. The same cap holds for creatures.

Nothing you can wear or learn makes you immune. Immunity belongs to some creatures alone: a
creature immune to a kind takes nothing of it, whatever it wears or suffers.

The share comes off each kind on its own: a flaming arrow's point and its fire are reduced
separately. The result is rounded to the nearest point, so a 1-point hit still hurts at 50 % but
does nothing at 75 %.

{{#include generated/damage-armour.md}}

## Magic of the elements

Each element has a magic [prefix](affixes.md#prefixes) and a magic [suffix](affixes.md#suffixes). The prefix adds a die of the element to every hit of
a weapon, a missile weapon, or [ammunition](ammunition.md). The suffix resists the element.

[Force](damage-types.md#damage-force) has no prefix. It is the spells' own kind ([Magic missile](magic.md#spell-magic-missile), [Arcane bolt](magic.md#spell-arcane-bolt)), and items can only
resist it. The physical kinds have neither prefix nor suffix: [Protection](combat.md#protection) is their resistance.

{{#include generated/damage-elements.md}}

Two [talents](talents.md) work on damage types:

- **[Elemental ward](talents.md#talent-elemental-ward)** is taken for one element: [fire](damage-types.md#damage-fire), [cold](damage-types.md#damage-cold), [electricity](damage-types.md#damage-electricity), [acid](damage-types.md#damage-acid), [poison](damage-types.md#damage-poison),
  [force](damage-types.md#damage-force), or [necrotic](damage-types.md#damage-necrotic). It resists that element 25 % per rank, up to two ranks. It counts like gear,
  under the same 75 % cap. A ward against poison also shrinks each dose of poison.
- **[Crusher](talents.md#talent-crusher)** lets your striking blows and kicks ignore 2 points of the target's [Protection](combat.md#protection). That
  covers a mace, a club, a staff, bare hands, and a [shield bash](abilities.md#ability-shield-bash). Shots and throws are not
  blows.

Three [birth-sign gifts](horoscope.md) resist too, 25 % each, counted like gear under the same cap.
The Wyvern's **[Flame-hardened](horoscope.md#talent-flame-hardened)** and the Fire element's **[Kindled](horoscope.md#talent-kindled)** resist [fire](damage-types.md#damage-fire), so a Fire Wyvern
has 50 %. The Kraken's **[Of the deep](horoscope.md#talent-of-the-deep)** resists [cold](damage-types.md#damage-cold).

The [character screen](character.md) (`@`) tells you what you resist in words only: "Resistant to
cold", "Vulnerable to fire", "Immune to poison". Item descriptions show the numbers for what you
hold: "Resists cold 20%", "+15% cold resistance", "Protection 3 (+1 against slashing, −1 against
piercing)", "Adds 1d4 fire damage".

Creatures have resistances and weaknesses of their own, and some are immune to a kind. The game
never gives their numbers. You learn them by playing. A blow that is shrugged off says so: "The
fireball does the salamander no harm." A blow stopped by armour alone glances off instead. You may
also simply know: [look mode](controls-prompts.md#look-mode) describes a creature's resistances in words when your
[Knowledge](skills.md#skill-knowledge) stands above its level.

## What else the kind decides

- **[Bleeding](status-effects.md#bleeding)** comes only from cuts and stabs. A big hit or a critical with a club, a fall into a
  [pit](traps.md#trap-pit), or a [fireball](magic.md#spell-fireball) opens no wound (see [Status effects](status-effects.md)).
- **[Poison](status-effects.md#status-poisoned)** is damage of its own kind. A creature immune to [poison](damage-types.md#damage-poison) is never [poisoned](status-effects.md#status-poisoned), and resisting
  poison shrinks each dose, whether from a rat's bite or bad food.
- **[Fire](damage-types.md#damage-fire)** melts ice where it bursts, and **[cold](damage-types.md#damage-cold)** freezes water, as a [fireball](magic.md#spell-fireball) and a [frost](magic.md#spell-frost)
  burst do (see [Temperature and ice](temperature.md)). The plunge through broken ice is cold.
- A **trap's** arrows and spears stab, its fireball burns, its frost chills, its lightning
  shocks, and its pit strikes (see [Traps](traps.md)). Scrolls cast the same spells (see
  [Scrolls](scrolls.md)).

[Starving](hunger.md#how-hungry-you-are), a [disease](diseases.md) eating at you, and the slow loss of blood from a wound have no kind.
Nothing resists them.
