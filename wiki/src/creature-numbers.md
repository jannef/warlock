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

Each summoning costs the summoner [mana](mana.md#how-much) (a few, the breeders, pay none) and a while before it can summon again; it never has more
of its summoned alive at once than the last column says. A breeder calls forth its own brood
while it fights, the [wormlord](creatures.md#creature-wormlord) its grave worms: kill it first,
or the fight gets worse the longer it lasts. Its mana comes back as anyone's does
(see [Mana](mana.md)), so a summoner that has spent itself summons again only much later. A
summoner that cannot cast ([silenced](status-effects.md#status-silenced)) cannot summon. You cannot summon.

{{#include generated/creature-summons.md}}

### Gazes

Some creatures harm with a look: a [vampire](creatures.md#creature-vampire)'s stare holds you
[transfixed](status-effects.md#status-transfixed), a [sea hag](creatures.md#creature-sea-hag)'s
sends you [frightened](status-effects.md#status-frightened) and [drains](stats.md#drained-stats)
your Toughness, a [spectre](creatures.md#creature-spectre)'s drains your will. A gazer that sees
you within its reach looks you in the eye before it does anything else but summon, and takes as
long as a spell. The eyes must meet both ways: it needs a
[line of sight](sight.md#line-of-sight) to you, and you must see it. A gazer in a cell too dark
for your eyes, behind a wall or a closed [door](doors.md), cannot catch your eye: you only feel
its cold stare, and it has wasted its look. Met, the gaze catches your eye at the chance below,
and then does what its row says, each by its own chance (a drain is still resisted by its
check). Either way it must wait before it looks again. A creature with no
[vitals](combat.md#criticals) has no eyes to meet, and is never gazed at; the gazers look at
your allies as they look at you.

A gaze marked **forced** in the table makes you **avert your eyes** by reflex while you see its
gazer: no choice, and nothing to switch. Averted, you fight half-blind: −4 to
[hit](combat.md#hitting), your [Dodge](combat.md#defence) halved, and no
[free blows](combat.md#free-blows). Its gaze still finds a glance you cannot help, at the chance
in that column. Darkness, a wall, or a door between you ends the averting as it ends the gaze.

The [basilisk](creatures.md#creature-basilisk)'s is the one forced gaze, and the deadliest: a glance
it catches gives a degree of [Stiffening](status-effects.md#status-stiffening) unless you make a
[Toughness](stats.md#stat-toughness) check against 15. Each degree is told (your limbs stiffen, your
skin greys, you can barely move), and the fourth turns you to stone: death. A degree eases in two
minutes, so a basilisk glimpsed and walked away from (it is slow) is a fright; the scroll of
restoration, a temple's cure, or Still mind's Stillness ends it, and the Heal skill eases it a degree
a try. The [cockatrice](creatures.md#creature-cockatrice)'s peck gives the same Stiffening by touch
(a Toughness check against 13, one peck in four), and the [gorgon](creatures.md#creature-gorgon)'s
breath to everyone it harms (see Breath below). A creature it stones leaves no corpse but a **statue** holding what it carried (see
[Doors](doors.md) for kicking one apart, [Digging](digging.md) for digging through it).

{{#include generated/creature-gazes.md}}
### Splitting and dying into

A blow of a kind a creature **splits** under (the [charnel heap](creatures.md#creature-charnel-heap)'s and the [black pudding](creatures.md#creature-black-pudding)'s: a blade's
[slashing](damage-types.md#damage-slashing) and [piercing](damage-types.md#damage-piercing)) that leaves it alive
with at least twice the hit points in the third column cuts it in two: it keeps the greater
half of what it has left, and a second one of it, with the other half and nothing it carried,
stands on the nearest free cell beside it, of its side and hunting what it hunted. With no free
cell beside it, it does not split. Blunt blows, fire, acid, and every other kind leave it whole,
and below twice that floor it dies like anything. A half you kill gives a **quarter** of the
creature's experience (an ally's kill a quarter of the ally's share), and no
[quest](quests.md) counts it.

A creature that **dies into** something lets it out of its corpse as it dies: so many of what
the table says, each drawn at random, on free cells it could see within that many cells, of the
dead one's side. Each gives a **tenth** of its experience when killed, and no quest counts them
either. Look mode tells such a creature by its *writhing*. A
[hydra](creatures.md#creature-hydra)'s heads crawl out of it so, and go on biting.

{{#include generated/creature-splits.md}}

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
[burning](status-effects.md#status-burning). A breath may carry more, as a blow's riders do:
the [gorgon](creatures.md#creature-gorgon)'s grey vapour leaves those it harms
[stiffening](status-effects.md#status-stiffening) unless they make the Toughness check (the
*Then* column). Then the dragon cannot
breathe again for the time below, and fights up close with its claws and bite, each struck in
turn and rolled on its own (the table above, one row each). Unseen, a dragon's breath is heard
as a roar.

{{#include generated/creature-breaths.md}}

### Bursts

Some creatures go off in a burst of their element. The motes are **vortices**: a
[vortex](creatures.md#creature-ember-mote) has no blow at all, and once it is beside its target it
bursts instead, and is gone. Killed before it gets there, it goes out quietly, so shoot or throw
at one as it comes, or close a door on it (a mote opens none). The elementals but the earth line,
the [burning skull](creatures.md#creature-burning-skull), the
[magma crawler](creatures.md#creature-magma-crawler), the
[animated lantern](creatures.md#creature-animated-lantern) (its oil), and the [balor](creatures.md#creature-balor)
have **death throes**: they burst where they die, however they die, so an elemental is best killed
from a cell or more away. The earth elementals do not burst; they hurl stones at what is not
beside them, as the giants do (the throws above), and pound what is.

A burst is centred on the creature's own cell and reaches everyone within its radius in its
line, but creatures of its own kind: one roll of its damage strikes each of them in full, with no
roll [to hit](combat.md#hitting) and no halving at the edge;
[Evasion](talents.md#talent-evasion) halves it and
[resistance](damage-types.md#armour-and-resistance) takes its share. It strikes your
[allies](allies.md) and other creatures as readily as you. It does to the floor what an area
[spell](magic.md#what-lies-in-its-way) of its element does, so the scrolls under a dying fire elemental may burn,
and its element does to the cells what such a spell's burst does (fire left burning, water
frozen). The spell types' riders do not come with it; its own do, by the chance below, on those it
harmed. A water elemental's wave puts out the [burning](status-effects.md#status-burning) of
everyone it reaches and the fires there; an air elemental's gust blows what lies loose on its
cells a cell farther out. The dice of a death throe are half of what a damage spell of the
creature's level rolls, (2 + level ÷ 8)d6, rounded down. Unseen, a burst is heard as an
explosion.

{{#include generated/creature-bursts.md}}
### Thieves

Some creatures pick your pack as they fight you: the [pixie](creatures.md#creature-pixie), the
[imp](creatures.md#creature-imp), the [succubus](creatures.md#creature-succubus) and the
[incubus](creatures.md#creature-incubus) take only what is small (coins, gems, potions, rings, and
scrolls), the [nymph](creatures.md#creature-nymph) anything. Each time one of their blows
[hits](combat.md#hitting) you up close and leaves you alive, the chance below decides whether it
also takes one whole stack of what you carry, picked at random among those it may take. Beside
you, with its theft ready, a thief that casts spells strikes instead, to rob you. It never
takes what you wear or wield, and it never robs anyone but you. Then it steals nothing more until
the time below has passed. It does not run: it fights on, with your things in its pack, and they
fall where it dies, as anything a creature carries does. The message log names what it took, so
you know what to look for. Unseen, "Something" snatches it.

{{#include generated/creature-thieves.md}}

### Blinkers

The [blink dog](creatures.md#creature-blink-dog) steps through nothing. When its way to you is
blocked (a doorway held, a corridor full of its fellows, a door it cannot open) it is suddenly
beside you, on a free cell next to you within its reach that it can see; then it runs on its
paws until the time below has passed. It never blinks away. It blinks only to where it sees, so
a closed door between you is still a door.

{{#include generated/creature-blinks.md}}

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

### Warning signs

Some creatures leave signs about them where they are found: a floor's
[basilisk](creatures.md#creature-basilisk) stands among statues of what it stoned (nobodies of
the floor's own kind, holding nothing, dug and kicked as any statue), spiders among old webs
(which catch no one), and acid-hided things on floor eaten into pits, and torn cloth lies where
a light-fingered thing has been. They are laid within four cells of each creature a floor is
peopled with or a vault holds (a statue only where open ground is all about it), from one to a
few of each kind. A basilisk's statues may bear the names of your own past characters who were
[turned to stone](status-effects.md#status-stiffening), "a statue of Brakka the dwarf": every character who dies goes into the
graveyard, a file kept beside the settings that no new version of the game invalidates, and its
petrified are named on statues, the most recent first, never twice on a floor. The more dreadful creatures are named once, the first time in a life you see
one, with the line below; the life remembers which.

{{#include generated/creature-signs.md}}

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

### Auras and cries

Some creatures carry something about them. An **aura** touches every foe within its reach on
the creature's turn, then not again until its time below has passed: one in its line of sight
(no light needed: a chill comes in the dark), or, a heard aura, anyone within the reach who hears
a voice as loud as the one given, through doors and round corners by the rules of
[hearing](hearing.md). Each foe it reaches makes the check in the "kept off by" column (2d12 plus
the stat's modifier against the number, as any [check](stats.md)), or suffers the status's
degrees, caused by the creature; a [resistance](damage-types.md#armour-and-resistance) to the
type named takes its share of them, and an [immunity](status-effects.md) keeps it off. The first
touch you suffer anew names its cause ("A chill comes off the wraith."). Distance is the
answer: the reach is short, so shoot it.

What a creature **always has** comes back every turn it takes, and is never shown when you look
at it: a fire salamander is always [cloaked in flame](status-effects.md#status-fire-cloaked), and
whoever strikes it burns. A **cry** is a sound it makes now and then where it stands, on a turn it
does not fight (a chance of 6 in the seconds given each turn), heard by the rules of
[hearing](hearing.md) as any sound is: the hound heard before it is seen.

{{#include generated/creature-auras.md}}

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
