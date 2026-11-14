# Creature tricks

Some creatures have more than claws and breath. They steal from your pack, blink from place to place, go unseen, give themselves away by signs, cast spells, or carry an aura. Their blows and their attacking powers are on [Creatures in numbers](creature-numbers.md) and [Creature powers](creature-powers.md).

## Thieves

Some creatures pick your pack as they fight you. The [pixie](creatures.md#creature-pixie), the
[imp](creatures.md#creature-imp), the [succubus](creatures.md#creature-succubus), and the
[incubus](creatures.md#creature-incubus) take only small things: coins, gems, potions, rings, and
scrolls. The [nymph](creatures.md#creature-nymph) takes anything.

Each time one of their blows [hits](combat.md#hitting) you up close and leaves you alive, the
chance below decides whether it also takes one whole stack of what you carry. The stack is picked
at random among those it may take. A thief never takes what you wear or wield, and never robs
anyone but you. After a theft it steals nothing more until the time below has passed. A thief that
casts spells strikes you instead of casting when it is beside you with its theft ready, to rob
you.

A thief does not run. It fights on with your things in its pack, and they fall where it dies, as
anything a creature carries does. The message log names what it took, so you know what to look
for. If you cannot see it, "Something" snatches it.

{{#include generated/creature-thieves.md}}

## Blinkers

The [blink dog](creatures.md#creature-blink-dog) steps through nothing. When its way to you is
blocked (a doorway held, a corridor full of its fellows, a door it cannot open), it is suddenly
beside you, on a free cell next to you within its reach that it can see. Then it must run on its
paws until the time below has passed.

It never blinks away. It blinks only to where it sees, so a closed door between you is still a
door.

{{#include generated/creature-blinks.md}}

## The unseen

Some creatures you do not see, even though you see where they stand.

**Shadowy** creatures are hidden in dim light. On a cell whose light is under twice what your
[eyes](vision.md) need to see at all, a shadowy creature is not there to you, however good your
darkvision, until it stands beside you. You feel a cold draught when one you do not see comes
within two cells.

**Invisible** creatures are not there to you in any light, at any distance. The invisible stalker
is always invisible. The rest are invisible while they are
[Invisible](status-list.md#status-invisible). They give themselves that status at the start of
their turn whenever the time below has passed since the last time, at no cost, and then go on
with their turn. While invisible a creature sheds no light of its own.

What shows them after all:

- [Seeing](status-list.md#status-seeing), from the spell
  [See invisible](magic.md#spell-see-invisible) or the potion of seeing;
- [Blood scent](talents.md#talent-blood-scent), on one it scents;
- your companion's [nose](companions.md#companions);
- a blow of yours that hits, for that turn.

You hear them as you hear anyone: their steps, and the fighting where an unseen blow came from.
Their blows are "something's". Walking into one strikes at it at −4 to hit. One that hunts by
scent comes to you wherever you are on its level.

{{#include generated/creature-unseen.md}}

## Warning signs

Some creatures leave signs around where they are found:

- a [basilisk](creatures.md#creature-basilisk) stands among statues of what it has stoned;
- spiders live among old webs, which catch no one;
- acid-hided things leave the floor eaten into pits;
- torn cloth lies where a light-fingered thing has been.

The signs are laid within four cells of each creature a floor is peopled with or a vault holds,
from one to a few for each kind. A statue is placed only where open ground is all about it. The
statues are of nobodies of the floor's own kind, hold nothing, and can be dug and kicked as any
statue.

A basilisk's statues may bear the names of your own past characters who were
[turned to stone](status-list.md#status-stiffening): "a statue of Brakka the dwarf". Every
character who dies goes into the graveyard, a file kept beside the settings that no new version
of the game invalidates. Its petrified characters are named on statues, the most recent first,
and never twice on one floor. The rest are buried in
[Līctūn](ascmearc.md#līctūn), Cēaptūn's churchyard.

The more dreadful creatures are named once, the first time in a life you see one, with the line
below. Each life remembers which it has seen.

{{#include generated/creature-signs.md}}

## Spells

Casters cast real [spells](magic.md), the very ones you may learn. They are rolled at the
caster's own [stats](stats.md) as yours are at yours, with the same
[reach and dice](spellcasting.md#how-a-spell-grows), the same [areas](spellcasting.md#shapes),
[spell types](spell-types.md), and [resistances](damage-types.md#armour-and-resistance).
A spell that rolls to hit adds a quarter of the caster's level to the roll, as its blows do.

Each cast costs the [mana](mana.md#how-much-mana-you-have) below from the caster's pool, which comes back as
slowly as anyone's. So a caster casts a handful of spells and then fights with whatever it holds.
One that cannot cast ([silenced](status-list.md#status-silenced)) fights the same way.

A caster casts the first spell on its list that it has the mana for and that would do something
from where it stands:

- a buff it does not have yet;
- a heal by touch on itself, or on a friend beside it, below half its hit points;
- a hex within reach that you are not under (and are not immune to);
- harm within reach, with nothing solid and nobody between you for a ray, a bolt, a cone, or a
  line.

It never casts an area spell that would catch a friend of its own. Failing all that, it comes
closer, to where its longest spell would reach.
[Spell reflection](fighting-styles.md#style-ability-spell-reflection) turns its rays and bolts
back at it, as it turns anyone's. Casters up to level 10 know only the weakest spells and are very
rare. Unseen, a caster is heard chanting.

{{#include generated/creature-spells.md}}

## Auras and cries

**Auras.** Some creatures carry an aura that touches every foe within its reach on the creature's
turn. It then does not touch again until its time below has passed. There are two kinds:

- a seen aura reaches anyone in its line of sight, with no light needed (a chill comes in the
  dark);
- a heard aura reaches anyone within the reach who would hear a voice as loud as the one given,
  through doors and round corners, by the rules of [hearing](hearing.md).

Each foe it reaches makes the check in the "kept off by" column: 2d12 plus the stat's modifier
against the number, as any [check](stats.md). On a failure it suffers the status's degrees, caused
by the creature. A [resistance](damage-types.md#armour-and-resistance) to the type named takes its
share of them, and an [immunity](status-effects.md) keeps it off. The first time an aura touches
you anew, you are told its cause ("A chill comes off the wraith."). The reach is short, so keep
your distance and shoot.

**Always on.** What a creature **always has** comes back every turn it takes, and is never shown
when you look at it. A fire salamander is always
[cloaked in flame](status-list.md#status-fire-cloaked), and whoever strikes it burns.

**Cries.** A **cry** is a sound a creature makes now and then where it stands, on a turn it does
not fight. Each turn the chance is 6 in the number of seconds given. It is heard by the rules of
[hearing](hearing.md) like any sound, so you hear the hound before you see it.

{{#include generated/creature-auras.md}}

See also: [Creature powers](creature-powers.md), [Creatures](creatures.md), and
[Creatures in numbers](creature-numbers.md).
