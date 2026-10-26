# Creatures in numbers

The game never shows you a creature's numbers: you see what it does, and
[look mode](controls.md#look-mode) tells you how hurt it looks. This page gives them exactly, as
the rules use them, generated from the game's data. How creatures behave is on the
[Creatures](creatures.md) page.

## Stats and skills

Each creature's level, the experience a kill of it is worth while you are at that level (less
when you are higher; see [Experience](experience.md)), and its six [stats](stats.md), each with
its modifier in brackets. A creature's potentials are how far each score could grow; creatures
do not train or grow yet. Skill values are on the scale of the [Skills](skills.md) page, weapon
skill ranks on that of [Weapon skills](weapon-skills.md).

{{#include generated/creature-stats.md}}

## Hit points, Defence, and resistances

A creature's hit points are its Toughness plus what its kind adds, as yours are your Toughness
plus what your race and class add. The Defence here is a creature's with nothing worn or
wielded: 10, plus its Dexterity modifier, its natural armour, and the Dodge its skills give;
an incorporeal creature adds its **Wall** bonus while it stands inside a wall
([Creatures](creatures.md#incorporeal-creatures)).
A creature placed with a weapon, armour, or a shield adds what those give, by the same rules as
yours (see [Combat](combat.md#defence)). The Protection is its natural armour's: what it wears
adds to it. Resistances take their share off each kind of damage after Protection (see
[Damage types](damage-types.md)).

{{#include generated/creature-defences.md}}

## Attacks

What each creature does with nothing in its hands: its natural attack (a bite, a hoof) or its
fists, with its Strength modifier and its ranks counted, and how long a blow takes it. A
creature placed with a weapon fights with that instead, by the same rules as you
([Combat](combat.md#hitting)): a kobold may carry a spear, and an archer shoots the bow
it carries ([Ranged combat](ranged.md)). Creatures that never fight belong to factions with no
enemies (see [Creatures](creatures.md#factions)). What a hit may cause is a chance for each,
rolled when the hit lands and the target lives; a disease is then resisted with a check (see
[Diseases](diseases.md)).

{{#include generated/creature-attacks.md}}

### Throws

Giants hurl boulders: a throw flies like any missile, along its line as far as its range,
striking the first one in its way (see [Ranged combat](ranged.md)). A creature that throws does
so whenever you are in range with nobody between you, until it is next to you; then it fights
up close. Its throws never run out, but each takes it the time below, and what it throws breaks
where it lands. A throw adds the thrower's Dexterity modifier to hit and its Strength modifier
to the damage, as yours do; a boulder of two kinds does whichever hurts its target more.

{{#include generated/creature-throws.md}}

## Pace, senses, and size

A creature's **speed** says how much time its actions take: 1000 is yours, and twice that does
everything in half the time (see [Turns and movement](turns.md#creatures-act-at-their-own-pace)).
It sees as far as its **sight**, by its [vision](vision.md), under the same light as you. Its
**hearing** is the faintest sound it notices (lower hears more, its listen skill counted), and
its **footsteps** how loud its steps are on normal ground, more or less on other terrain (see
[Hearing](hearing.md)). It is burdened past a third of its **carrying limit**, as you are (see
[Burden](burden.md)).

{{#include generated/creature-senses.md}}
