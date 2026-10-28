# Damage types

Every blow, shot, trap, and spell does damage of a **kind**: the three physical kinds a weapon
does ([slashing](damage-types.md#damage-slashing), [piercing](damage-types.md#damage-piercing), [bludgeoning](damage-types.md#damage-bludgeoning)) and the elements ([fire](damage-types.md#damage-fire), [cold](damage-types.md#damage-cold), [electricity](damage-types.md#damage-electricity), [acid](damage-types.md#damage-acid), [poison](damage-types.md#damage-poison),
and the two kinds only [magic](magic.md) does, [force](damage-types.md#damage-force) and [necrotic](damage-types.md#damage-necrotic)).
The kind decides what armour does against it, what resists it, whether it [bleeds](status-effects.md#bleeding), and how the
message reads: "The kobold stabs you.", "You cleave the giant rat.", "The fireball burns the
kobold."

{{#include generated/damage-types.md}}

## What weapons do

Each weapon does one kind, or two: a sword cuts or stabs, and against each target it does
**whichever is better**, the kind the target resists least, then the one its armour stands least
against. A flaming arrow stabs and burns: its [fire](damage-types.md#damage-fire) is **added damage**, rolled after the arrow's
own and never soaked by armour. Magic weapons can add any element but force the same way (see
[Magic items](magic-items.md) and the table below).

{{#include generated/damage-weapons.md}}

Creatures' bites stab, their claws cut, and their hooves and slams strike; a shield bash, a kick,
and bare hands strike too.

## Armour and resistance

**Protection** (see [Combat](combat.md#protection)) soaks physical damage only: a [fireball](magic.md#spell-fireball), a
[lightning bolt](magic.md#spell-lightning-bolt), the [cold](damage-types.md#damage-cold) of a [plunge through the ice](temperature.md#falling-in), and [poison](damage-types.md#damage-poison) go straight through mail. A piece
may stand better or worse against one physical kind: chain mail turns a blade well and a point
badly.

**Resistance** takes a share off damage of a kind, in percent, after the armour has done its
part: a fur cloak against [cold](damage-types.md#damage-cold), a leather jerkin a little against [acid](damage-types.md#damage-acid), and magic *of* the element (the table below). Everything you wear adds up, capped at 75 %; nothing you can wear
makes you immune. The share comes off each kind on its own (a flaming arrow's point and its fire
separately), rounded to the nearest point, so a 1-point hit at 50 % still hurts and at 75 % does
not.

{{#include generated/damage-armour.md}}

## Magic of the elements

Each element has a magic [prefix](magic-items.md#prefixes) that adds a die of it to a weapon's, a missile weapon's, or
[ammunition's](ammunition.md) every hit, and a [suffix](magic-items.md#suffixes) that resists it. [Force](damage-types.md#damage-force) has no prefix: it is the spells'
kind ([Magic missile](magic.md#spell-magic-missile), [Arcane bolt](magic.md#spell-arcane-bolt)), and only resisted. The physical kinds have neither:
[Protection](combat.md#protection) is their resistance.

{{#include generated/damage-elements.md}}

Two [talents](talents.md) work on damage types. **[Elemental ward](talents.md#talent-elemental-ward)**, taken for one element
([fire](damage-types.md#damage-fire), [cold](damage-types.md#damage-cold), [electricity](damage-types.md#damage-electricity), [acid](damage-types.md#damage-acid), [poison](damage-types.md#damage-poison), [force](damage-types.md#damage-force), or [necrotic](damage-types.md#damage-necrotic)), resists it 25 % a rank, two ranks at most; it counts
as gear does, under the same cap, and a ward against poison shrinks each dose of poison too.
**[Crusher](talents.md#talent-crusher)** lets your blows and kicks that can strike (a mace, a club, a staff, bare hands, a
[shield bash](abilities.md#ability-shield-bash)) ignore 2 points of the target's [Protection](combat.md#protection); shots and throws are not blows.

Three [birth-sign gifts](horoscope.md) resist too, 25 % each, counted as gear under the same cap:
the Wyvern's **[Flame-hardened](talents.md#talent-flame-hardened)** and the Fire element's **[Kindled](talents.md#talent-kindled)** resist [fire](damage-types.md#damage-fire) (a Fire Wyvern has
both, 50 %), and the Kraken's **[Of the deep](talents.md#talent-of-the-deep)** resists [cold](damage-types.md#damage-cold).

The [character screen](character.md) (`@`) says what you resist in words only: "Resistant to
cold", "Vulnerable to fire", "Immune to poison". Item descriptions show the numbers of what you
hold ("Resists cold 20%", "+15% cold resistance", "Protection 3 (+1 against slashing, −1 against
piercing)", "Adds 1d4 fire damage").

Creatures have resistances and weaknesses of their own, and some are immune to a kind; the game
never lists them. You learn them by play: a blow that is shrugged off says so, "The fireball does
the salamander no harm.", and a blow that armour alone stopped glances off instead.

## What else the kind decides

- **[Bleeding](status-effects.md#bleeding)** comes only of cuts and stabs: a big hit or a critical with a club, a fall into a
  [pit](traps.md#trap-pit), or a [fireball](magic.md#spell-fireball) opens no wound (see [Status effects](status-effects.md)).
- **[Poison](status-effects.md#status-poisoned)** is damage of its kind: a creature immune to [poison](damage-types.md#damage-poison) is never [poisoned](status-effects.md#status-poisoned), and a
  resistance to it shrinks each dose it is given (a rat's bite, bad food).
- **[Fire](damage-types.md#damage-fire)** melts the ice it bursts over and **[cold](damage-types.md#damage-cold)** freezes the water, as a [fireball](magic.md#spell-fireball) and a [frost](magic.md#spell-frost)
  burst do (see [Temperature and ice](temperature.md)); the plunge through broken ice is cold.
- A **trap's** arrows and spears stab, its fireball burns, its frost chills, its lightning
  shocks, and its pit strikes (see [Traps](traps.md)); scrolls cast the same spells (see
  [Scrolls](scrolls.md)).

[Starving](hunger.md#how-hungry-you-are), a [disease](diseases.md) eating at you, and the slow loss of blood from a wound are of no kind:
nothing resists them.
