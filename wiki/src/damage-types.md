# Damage types

Every blow, shot, trap, and spell does damage of a **kind**: the three physical kinds a weapon
does (slashing, piercing, bludgeoning) and the elements (fire, cold, electricity, acid, poison).
The kind decides what armour does against it, what resists it, whether it bleeds, and how the
message reads: "The kobold stabs you.", "You cleave the giant rat.", "The fireball burns the
kobold."

{{#include generated/damage-types.md}}

## What weapons do

Each weapon does one kind, or two: a sword cuts or stabs, and against each target it does
**whichever is better**, the kind the target resists least, then the one its armour stands least
against. A flaming arrow stabs and burns: its fire is **added damage**, rolled after the arrow's
own and never soaked by armour. Magic weapons can add fire or cold the same way (see
[Magic items](magic-items.md)).

{{#include generated/damage-weapons.md}}

Creatures' bites stab, their claws cut, and their hooves and slams strike; a shield bash, a kick,
and bare hands strike too.

## Armour and resistance

**Protection** (see [Combat](combat.md#protection)) soaks physical damage only: a fireball, a
lightning bolt, the cold of a plunge through the ice, and poison go straight through mail. A piece
may stand better or worse against one physical kind: chain mail turns a blade well and a point
badly.

**Resistance** takes a share off damage of a kind, in percent, after the armour has done its
part: a fur cloak against cold, a leather jerkin a little against acid, and magic that is *of the
Salamander* or *of Warmth*. Everything you wear adds up, capped at 75 %; nothing you can wear
makes you immune. The share comes off each kind on its own (a flaming arrow's point and its fire
separately), rounded to the nearest point, so a 1-point hit at 50 % still hurts and at 75 % does
not.

{{#include generated/damage-armour.md}}

Two [talents](talents.md) work on damage types. **Elemental ward**, taken for one element
(fire, cold, electricity, acid, or poison), resists it 25 % a rank, two ranks at most; it counts
as gear does, under the same cap, and a ward against poison shrinks each dose of poison too.
**Crusher** lets your blows and kicks that can strike (a mace, a club, a staff, bare hands, a
shield bash) ignore 2 points of the target's Protection; shots and throws are not blows.

The [character screen](character.md) (`@`) says what you resist in words only: "Resistant to
cold", "Vulnerable to fire", "Immune to poison". Item descriptions show the numbers of what you
hold ("Resists cold 20%", "+15% cold resistance", "Protection 3 (+1 against slashing, −1 against
piercing)", "Adds 1d4 fire damage").

Creatures have resistances and weaknesses of their own, and some are immune to a kind; the game
never lists them. You learn them by play: a blow that is shrugged off says so, "The fireball does
the salamander no harm.", and a blow that armour alone stopped glances off instead.

## What else the kind decides

- **Bleeding** comes only of cuts and stabs: a big hit or a critical with a club, a fall into a
  pit, or a fireball opens no wound (see [Status effects](status-effects.md)).
- **Poison** is damage of its kind: a creature immune to poison is never poisoned, and a
  resistance to it shrinks each dose it is given (a rat's bite, bad food).
- **Fire** melts the ice it bursts over and **cold** freezes the water, as a fireball and a frost
  burst do (see [Temperature and ice](temperature.md)); the plunge through broken ice is cold.
- A **trap's** arrows and spears stab, its fireball burns, its frost chills, its lightning
  shocks, and its pit strikes (see [Traps](traps.md)); scrolls cast the same spells (see
  [Scrolls](scrolls.md)).

Starving, a disease eating at you, and the slow loss of blood from a wound are of no kind:
nothing resists them.
