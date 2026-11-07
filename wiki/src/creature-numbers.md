# Creatures in numbers

The game never shows you a creature's numbers: you see what it does, and
[look mode](controls.md#look-mode) tells you how hurt it looks. This page gives them exactly, as
the rules use them, generated from the game's data. How creatures behave is on the
[Creatures](creatures.md) page.

## Stats and skills

Each creature's level, the [experience](experience.md) a kill of it is worth while you are at that level (less
when you are higher; see [Experience](experience.md)), and its six [stats](stats.md), each with
its [modifier](stats.md#modifiers) in brackets. A creature's [potentials](stats.md#potentials) are how far each score could grow; creatures
do not train or grow yet. Skill values are on the scale of the [Skills](skills.md) page, weapon
skill ranks on that of [Weapon skills](weapon-skills.md).

{{#include generated/creature-stats.md}}

## Hit points, Defence, and resistances

A creature's [hit points](hit-points.md#hit-points) are its [Toughness](stats.md#stat-toughness) plus what its kind adds, as yours are your Toughness
plus what your race and class add. The [Defence](combat.md#defence) here is a creature's with nothing worn or
wielded: 10, plus its [Dexterity](stats.md#stat-dexterity) [modifier](stats.md#modifiers), its natural armour, and the [Dodge](combat.md#defence) its skills give;
an incorporeal creature adds its **Wall** bonus while it stands inside a wall
([Creatures](creature-bodies.md#incorporeal-creatures)).
A creature placed with a weapon, armour, or a shield adds what those give, by the same rules as
yours (see [Combat](combat.md#defence)). The [Protection](combat.md#protection) is its natural armour's: what it wears
adds to it. [Resistances](damage-types.md#armour-and-resistance) take their share off each kind of damage after Protection (see
[Damage types](damage-types.md)).

{{#include generated/creature-defences.md}}

## Attacks

What each creature does with nothing in its hands: its natural attack (a bite, a hoof) or its
fists, with its Strength modifier and its ranks counted, and how long a blow takes it. A
creature placed with a weapon fights with that instead, by the same rules as you
([Combat](combat.md#hitting)): a [kobold](creatures.md#creature-kobold) may carry a spear, and an archer shoots the bow
it carries ([Ranged combat](ranged.md)). Creatures that never fight belong to factions with no
enemies (see [Creatures](creatures.md#factions)). What a hit may cause is a chance for each,
rolled when the hit lands and the target lives; [a disease](diseases.md) is then resisted with a check (see
[Diseases](diseases.md)).

{{#include generated/creature-attacks.md}}

### By role

Two creatures of one level are not the same fight: each kind has a role, and the role sets its
numbers around its level's L. A natural attack hits at a quarter of L (each part of these
rounded down); a **quick** one (or an assassin) adds an eighth of L, at most +20, and its blow
[threatens a critical](combat.md#criticals) from 19; a **brute** takes an eighth off, and its
critical comes only on a natural 24 but does three times the damage; a **shambler** or a
**caster** takes the eighth off too. A creature placed with a weapon hits at a quarter of L and
fights with its weapon's numbers. Its [hit points](hit-points.md#hit-points) are a standard's
(about `12 + 2.9 L`) times the role's factor, within 35 % either way: 0.5 for the quick, the
small, and the casters, 0.6 for the spirits, 1.5 for the shamblers, and 2.0 for the brutes. Its
[speed](turns.md#speed) is its kind's temperament, 800 to 1250 for most, 950 to 1050 for the
rank and file of a people (orcs, hobgoblins, drow, duergar); a [dread](dungeons.md#dread) one
moves at 600 or 700, slow and awful, or at 1400 and more.

## Surprise

The creatures with a craft for [surprise](status-effects.md#status-surprised). Their
**[sneak attack](skills.md#skill-sneak-attack)** adds the dice to a hit that finds you unaware or
surprised, and makes it harder to keep your wits (the difficulty column: 13 plus this, against your
[Dexterity](stats.md#stat-dexterity) and your own [initiative](skills.md#skill-initiative)). Their
**initiative** helps them keep their wits when you catch them unawares, and the time it saves if
they do not. Their **[spot](skills.md#skill-spot)** counts against you when you
[slip past](skill-uses.md#slipping-past) them in the dark. A creature that **lurks** waits where it
stands for you to come beside it, until it is hurt (see [Creatures](creatures.md#what-they-do)).
Every other creature has none of this: its initiative check is its Dexterity alone, and it
strikes the unwary with no extra dice.

{{#include generated/creature-surprise.md}}

## Pace, senses, and size

A creature's **[speed](turns.md#speed)** says how much time its actions take: 1000 is yours, and twice that does
everything in half the time (see [Turns and movement](turns.md#creatures-act-at-their-own-pace)).
It sees as far as its **[sight](sight.md)**, by its [vision](vision.md), under the same [light](light.md#what-light-is) as you. Its
**[hearing](hearing.md)** is the faintest sound it notices (lower hears more, its [listen skill](skills.md#skill-listen) counted), and
its **[footsteps](hearing.md#what-makes-noise)** how loud its steps are on normal ground, more or less on other terrain (see
[Hearing](hearing.md)). Its **nose** is how far it smells a foe as someone's
[companion](companions.md#companions), through the dark and a closed door; **tameable** says whether
it may be one at all. It is [burdened](burden.md#burden-1) past a third of its **[carrying limit](burden.md#how-much-you-can-carry)**, as you are (see
[Burden](burden.md)).

{{#include generated/creature-senses.md}}
