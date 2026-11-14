# Creatures in numbers

The game never shows you a creature's numbers. You see what it does, and
[look mode](controls-prompts.md#look-mode) tells you how hurt it looks. This page gives the numbers
exactly, as the rules use them, taken from the game's data. How creatures behave is on the
[Creatures](creatures.md) page. Their special powers (throwing, summoning, breath, spells, and the
rest) are on [Creature powers](creature-powers.md).

The tables are split by family, a page each:

- [The dead and spirits](creature-numbers-dead.md): the undead (zombies, ghouls, wights, vampires,
  liches, skeletons) and the spirits.
- [Fiends](creature-numbers-fiends.md): demons, devils, yugoloths, and the gates a miscast opens.
- [The fey](creature-numbers-fey.md): sprites, pixies, redcaps, dryads, and the rest of the courts.
- [Elementals](creature-numbers-elementals.md): the elementals of air, earth, fire, ice, lightning,
  and water, and their motes.
- [Animated things and mimics](creature-numbers-animated.md): animated objects, constructs, and
  mimics.
- [Giants, trolls, hags, and dragons](creature-numbers-giants.md): giants and ogres, trolls, hags,
  and the dragons.
- [Kobolds, goblins, and orcs](creature-numbers-goblins.md): kobolds, goblins, hobgoblins, and
  orcs.
- [Drow, duergar, and deep gnomes](creature-numbers-deep-folk.md): the peoples of the deep: drow,
  duergar, and deep gnomes.
- [Beasts and vermin](creature-numbers-beasts.md): animals, giant vermin, and the things of the
  marsh.
- [People and other creatures](creature-numbers-others.md): townsfolk, outlaws, cultists, and every
  creature of no family above.

This page says what each table's columns mean.

## Stats and skills

The first table gives each creature's level and the [experience](experience.md) a kill is worth
while you are at that level. You get less when your level is higher (see
[Experience](experience.md)). It also gives its rarity and its skills.

The second gives the six [stats](stats.md), each with its [modifier](stats.md#modifiers) in
brackets. Where a score could grow further, its [potential](stats.md#potentials) follows it, but
creatures do not train or grow yet. Skill values are on the scale of the [Skills](skills.md) page, and weapon
skill ranks on the scale of [Weapon skills](weapon-skills.md).

## Hit points, Defence, and resistances

A creature's [hit points](hit-points.md#hit-points) are its [Toughness](stats.md#stat-toughness)
plus what its kind adds, just as yours are your Toughness plus what your race and class add.

The [Defence](combat.md#defence) shown is the creature's with nothing worn or wielded. It is 10,
plus its [Dexterity](stats.md#stat-dexterity) [modifier](stats.md#modifiers), its natural armour,
and the [Dodge](combat.md#defence) its skills give. An incorporeal creature adds its **Wall** bonus
while it stands inside a wall ([How creatures fight](creature-bodies.md#incorporeal-creatures)). A
creature placed with a weapon, armour, or a shield adds what those give, by the same rules as
yours (see [Combat](combat.md#defence)).

The [Protection](combat.md#protection) shown is its natural armour's, and what it wears adds to
it. [Resistances](damage-types.md#armour-and-resistance) take their share off each kind of damage
after Protection (see [Damage types](damage-types.md)).

## Attacks

This table shows what each creature does with nothing in its hands: its natural attack (a bite, a
hoof) or its fists. The numbers count its Strength modifier and its ranks, and give how long a
blow takes it.

A creature placed with a weapon fights with that instead, by the same rules as you
([Combat](combat.md#hitting)). A [kobold](creatures.md#creature-kobold) may carry a spear, and an
archer shoots the bow it carries ([Ranged combat](ranged.md)). Creatures that never fight belong
to factions with no enemies (see [Creatures](creatures.md#factions)).

What a hit may cause is a chance for each effect, rolled when the hit lands and the target lives.
[A disease](diseases.md) is then resisted with a check (see [Diseases](diseases.md)).

### By role

Two creatures of the same level are not the same fight. Each kind has a role, and the role sets
its numbers around its level, L. Each part below is rounded down.

**To hit.** A natural attack hits at a quarter of L.

- A **quick** creature (or an assassin) adds an eighth of L, at most +20, and its blow
  [threatens a critical](combat.md#criticals) from 19.
- A **brute** takes an eighth off. Its critical comes only on a natural 24, but does three times
  the damage.
- A **shambler** or a **caster** takes the eighth off too.
- A creature placed with a weapon hits at a quarter of L and fights with its weapon's numbers.

**Hit points.** Its [hit points](hit-points.md#hit-points) are a standard amount (about
`12 + 2.9 L`) times the role's factor, within 35 % either way. The factors are 0.5 for the quick,
the small, and the casters, 0.6 for the spirits, 1.5 for the shamblers, and 2.0 for the brutes.
The first levels are softer: a creature of level 1 has 40 % of that, level 2 55 %, level 3 70 %,
level 4 80 %, and level 5 90 % (its Toughness and what its kind adds cut alike).

**What its blows carry.** The poisons, diseases, and holds of a creature of level 1 to 5 are rare
and mild: at most a 1 % chance at level 1, 3 % at levels 2 and 3, 4 % at level 4, and 5 % at
level 5, one degree of a [status effect](status-effects.md) at most, and no poison at level 1.
A creature whose blow is its own (a spider's or a centipede's venom, the blood a leech draws, a
[mimic](mimics.md)'s grip) may have up to 12 % from level 2.

**Speed.** Its [speed](turns.md#speed) comes from its kind's temperament: 800 to 1250 for most,
and 950 to 1050 for the rank and file of a people (orcs, hobgoblins, drow, duergar). A
[dread](dungeons.md#dread) creature moves at 600 or 700, slow and awful, or at 1400 and more.

## Surprise

These are the creatures with a craft for [surprise](status-list.md#status-surprised).

- Their **[sneak attack](skills.md#skill-sneak-attack)** adds its dice to a hit that finds you
  unaware or surprised. It also makes it harder for you to keep your wits: the difficulty column
  is 13 plus this, against your [Dexterity](stats.md#stat-dexterity) and your own
  [initiative](skills.md#skill-initiative).
- Their **initiative** helps them keep their wits when you catch them unawares, and saves them
  time if they do not.
- Their **[spot](skills.md#skill-spot)** counts against you when you
  [slip past](skill-uses.md#slipping-past) them in the dark.
- A creature that **lurks** waits where it stands for you to come beside it, until it is hurt (see
  [How creatures behave](creature-behaviour.md#what-they-do)).

Every other creature has none of this. Its initiative check is its Dexterity alone, and it strikes
the unwary with no extra dice.

{{#include generated/creature-surprise.md}}

## Pace, senses, and size

- **[Speed](turns.md#speed)** says how much time a creature's actions take. 1000 is yours, and
  twice that does everything in half the time (see
  [Turns and movement](turns.md#creatures-act-at-their-own-pace)). In brackets is how long a
  normal action takes it.
- **[Sight](sight.md)** is how far it sees, with which [vision](vision.md), under the same
  [light](light.md#what-light-is) as you.
- **[Hearing](hearing.md)** is the faintest sound it notices. Lower hears more, and its
  [listen skill](skills.md#skill-listen) is counted in.
- **[Footsteps](hearing.md#what-makes-noise)** is how loud its steps are on normal ground. Other
  terrain makes them louder or softer (see [Hearing](hearing.md)).
- The second table gives its weight, its size, and its
  **[carrying limit](burden.md#how-much-you-can-carry)**. It is
  [burdened](burden.md#burden-levels) past a third of that, as you are (see [Burden](burden.md)).
- **Tameable** says whether it may be a [companion](companions.md#companions) at all, and its
  **nose** how far it smells a foe when it is one, through the dark and a closed door.

See also: [Creatures](creatures.md), [Stats](stats.md), and [Combat](combat.md).
