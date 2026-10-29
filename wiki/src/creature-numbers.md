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

A creature's [hit points](combat.md#hit-points) are its [Toughness](stats.md#stat-toughness) plus what its kind adds, as yours are your Toughness
plus what your race and class add. The [Defence](combat.md#defence) here is a creature's with nothing worn or
wielded: 10, plus its [Dexterity](stats.md#stat-dexterity) [modifier](stats.md#modifiers), its natural armour, and the [Dodge](combat.md#defence) its skills give;
an incorporeal creature adds its **Wall** bonus while it stands inside a wall
([Creatures](creatures.md#incorporeal-creatures)).
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

### Throws

Giants hurl boulders: a throw flies like any missile, along its line as far as its range,
striking the first one in its way (see [Ranged combat](ranged.md)). A creature that throws does
so whenever you are in range with nobody between you, until it is next to you; then it fights
up close. Its throws never run out, but each takes it the time below, and what it throws breaks
where it lands. A throw adds the thrower's [Dexterity](stats.md#stat-dexterity) modifier [to hit](combat.md#hitting) and its [Strength](stats.md#stat-strength) modifier
to the [damage](combat.md#damage), as yours do; a boulder of two kinds does whichever hurts its target more.

{{#include generated/creature-throws.md}}

### Summoning

Summoners call forth creatures of their side: a [goblin wizard](creatures.md#creature-goblin-wizard) its kin and their [wargs](creatures.md#creature-warg), a
[necromancer](creatures.md#creature-necromancer) the lesser dead, a [dark wizard](creatures.md#creature-dark-wizard) devils. A summoner that sees a foe within its reach
(you, or anyone its side fights) summons before it does anything else, and the creatures appear
on free cells around it that it can see, nearest first. How many come is rolled each time; they
are real creatures of the summoner's side that fight, carry gear, drop it where they die, and give
their full experience when you kill them. Killing the summoner does not send them away.

Each summoning costs the summoner [mana](mana.md#how-much) and a while before it can summon again; it never has more
of its summoned alive at once than the last column says. Its mana comes back as anyone's does
(see [Mana](mana.md)), so a summoner that has spent itself summons again only much later. A
summoner that cannot cast ([silenced](status-effects.md#status-silenced)) cannot summon. You cannot summon.

{{#include generated/creature-summons.md}}

## Surprise

The creatures with a craft for [surprise](status-effects.md#status-surprised). Their
**[sneak attack](skills.md#skill-sneak-attack)** adds the dice to a hit that finds you unaware or
surprised, and makes it harder to keep your wits (the difficulty column: 13 plus this, against your
[Dexterity](stats.md#stat-dexterity) and your own [initiative](skills.md#skill-initiative)). Their
**initiative** helps them keep their wits when you catch them unawares, and the time it saves if
they do not. Their **[spot](skills.md#skill-spot)** counts against you when you
[slip past](skills.md#slipping-past) them in the dark. A creature that **lurks** waits where it
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
[Hearing](hearing.md)). It is [burdened](burden.md#burden-1) past a third of its **[carrying limit](burden.md#how-much-you-can-carry)**, as you are (see
[Burden](burden.md)).

{{#include generated/creature-senses.md}}
