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

### Breath

Dragons breathe their element: the [fire drake](creatures.md#creature-fire-drake) a long fan of
[fire](damage-types.md#damage-fire). A dragon that sees you within its cone's length, with
nothing solid and nobody between you, breathes whenever its breath is ready, before it does
anything else; the [cone](magic.md#shapes) fans out a quarter turn wide from the dragon toward
you, as [Burning hands](magic.md#spell-burning-hands) does, but far longer. Everyone in it is
struck by one roll of its damage, with no roll [to hit](combat.md#hitting):
[Evasion](talents.md#talent-evasion) halves it and
[resistance](damage-types.md#armour-and-resistance) takes its share, so the dragon's own kind,
immune to its element, takes nothing. Its element does to those it harms what any
[spell of it](magic.md#spell-types) does: fire sets one in four
[burning](status-effects.md#status-burning). Then the dragon cannot
breathe again for the time below, and fights up close with its claws and bite, each struck in
turn and rolled on its own (the table above, one row each). Unseen, a dragon's breath is heard
as a roar.

{{#include generated/creature-breaths.md}}

### The unseen

Some creatures you do not see though you see where they stand. A **shadowy** one, on a cell
whose light is under twice what your [eyes](vision.md) need to see at all, is not there to you
however good your darkvision, until it stands beside you; you feel a cold draught as one you do
not see comes within two cells. An **invisible** one is not there to you in any light and at any
distance: the invisible stalker always, and the rest while they are
[Invisible](status-effects.md#status-invisible), which they give themselves at the start of their
turn whenever the time below has passed since the last, at no cost, and then go on with their
turn. While invisible a creature sheds no light of its own. What shows them after all:
[Seeing](status-effects.md#status-seeing) (the spell [See invisible](magic.md#spell-see-invisible)
and the potion of seeing), [Blood scent](talents.md#talent-blood-scent) on one it scents, your
companion's [nose](allies.md#companions), and a blow of yours that hits, for that turn. You hear them as
you hear anyone: their steps, and the fighting where an unseen blow came from. Their blows are
something's; walking into one strikes at it at −4 to hit. One that hunts by scent comes to you
wherever you are on its level.

{{#include generated/creature-unseen.md}}

### Spells

Casters cast real [spells](magic.md), the very ones you may learn, rolled at their own
[stats](stats.md) as yours are at yours: the same [reach and dice](magic.md#how-a-spell-grows),
the same [areas](magic.md#shapes), [spell types](magic.md#spell-types), and
[resistances](damage-types.md#armour-and-resistance). A spell that rolls to hit adds a quarter of
the caster's level to the roll, as its blows do. Each cast costs the [mana](mana.md#how-much)
below from its pool, which comes back as slowly as anyone's, so a caster casts a handful of
spells and then fights with whatever it holds; one that cannot cast
([silenced](status-effects.md#status-silenced)) fights the same way.

A caster casts the first spell of its list that it has the mana for and that would do something
from where it stands: a buff it does not have yet, a heal by touch on itself or a friend beside
it below half its hit points, a hex you are not under (and are not immune to) within reach, or
harm within reach, with nothing solid and nobody between you for a ray, a bolt, a cone, or a
line. It never casts an area that would catch a friend of its own. Failing all that, it comes
closer, to where its longest spell would reach. [Spell reflection](fighting-styles.md#style-ability-spell-reflection)
turns its rays and bolts back at it, as it turns anyone's. Casters up to level 10 know only the
weakest spells and are very rare. Unseen, a caster is heard chanting.

{{#include generated/creature-spells.md}}

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
[Hearing](hearing.md)). Its **nose** is how far it smells a foe as someone's
[companion](allies.md#companions), through the dark and a closed door; **tameable** says whether
it may be one at all. It is [burdened](burden.md#burden-1) past a third of its **[carrying limit](burden.md#how-much-you-can-carry)**, as you are (see
[Burden](burden.md)).

{{#include generated/creature-senses.md}}
