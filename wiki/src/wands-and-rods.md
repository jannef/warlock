# Wands and rods

INTRO

## Wands

A wand holds one spell and a number of charges. Ready it in the **[missile weapon slot](equipment.md#what-fits-where)** (it takes
a [launcher](ranged.md#getting-ready)'s place, and a launcher takes its place) and use it as you would shoot: `t`, aim, and
it casts its spell. Each cast uses a charge instead of your [mana](mana.md#how-much), and the spell is cast at the
**wand's own rung** of its ladder: a wand casts its spell at a rung of its own, rolled when it was
made and shown once you know the wand (the table's "Strength" gives the rolls at the lowest and
the highest rung a wand of the kind may have; each rung between is as likely). Your stats never
change it, so a warrior with no magic fires a wand as well as a wizard does. The spell is otherwise its own: a ray [rolls to hit](combat.md#hitting) as any spell attack does (with
your [Dexterity](stats.md#stat-dexterity)), magic missile never misses, an area catches whoever is in it, and how far it
reaches is in the spell's row, with your [Willpower](stats.md#stat-willpower), as for your own casts.
A cast that cannot go off (aimed at yourself, out of reach) uses no charge and takes no time;
one that goes off takes a cast's time.

An empty wand does nothing ("Nothing happens.") and takes no time; it stays a wand, to sell or
to keep. Wands are **never recharged**.

Wands are found unknown, each kind with a look of its own for the whole game ("an oak wand").
Using one tells you what kind it is, and from then on every wand of that kind shows by its name;
its **charges** stay unknown until you [identify](magic-items.md#unidentified-items) that wand (the [scroll of identify](scrolls.md#scroll-of-identify), or a [merchant](classes.md#merchant)'s
[Merchant's eye](talents.md#talent-merchants-eye)). Wands lie in the dungeon and in general stores.

*[Wand mastery](talents.md#talent-wand-mastery)* (Weaponcraft) takes every wand you use a rung up, and [magic items](magic-items.md#suffixes)
*of Wands* their rungs with it, never past the ladder's last rung. Once you know a wand, its
description and the aiming line say what it does in your hands ("Holds Fireball: 8d6 fire,
radius 2"), your talents and items counted.

{{#include generated/wands.md}}

Each wand is made with charges of its own (the table above).

## Rods

A rod holds an **effect no spell has**: [digging](digging.md#digging), sleep, terror, sending a creature away. Like a
wand it goes in the **missile weapon slot** and is used as you would shoot (`t`, aim), but it has
no charges: each use costs **your own [mana](mana.md#spending-it)**, the rod's cost, and it never runs out while you
have the [mana](mana.md#how-much) to feed it. Without enough mana it does nothing and takes no time ("You have not
the mana to use it."), and nor can you use one while a state that stops casting holds you
([raging](status-effects.md#status-raging)). A use takes a cast's time.

A rod's row says how far it reaches and how far your [Willpower](stats.md#stat-willpower) takes it, and the
strongest creature its sleep or terror takes. Its reach (how far a line digs, how far a bolt
flies, how far away a burst may land) is read as a [spell's](spellcasting.md#how-a-spell-grows) is: the first number is
what anyone reaches, and each rung after it is the reach of a user whose Willpower is at least
the rung's score. Sleep and terror take only creatures of a level up to the rod's own (20); the
others shrug it off. Your level plays no part: no skill makes a rod better or cheaper,
and what it costs and how far it reaches are the same in anyone's hands of the same Willpower.

- **Digging** breaks every [wall](terrain.md#walls) on a straight line toward where you aim, as if dug through,
  until a wall too hard for it (granite), a [door](doors.md), or the level's edge stops it.
- **Teleport other** sends the first creature its bolt meets to a random free cell of the
  level. It counts as an attack: a [shopkeeper](shops.md) will not forgive it.
- **Sleep** puts the creatures in its burst into a true sleep (not the [Sleep](magic.md#spell-sleep) spell's
  drowsiness): they take no turns until a noise, a blow, or a cry wakes them.
- **Terror** makes the creatures in its burst flee from you a few turns.

Rods are found unknown, each kind with a look of its own for the whole game ("a pewter rod");
using one or identifying it tells you what it is, and its cost and reach then show when you
look at it and aim it. Rods lie in the dungeon and in general stores.

*[Channeler](talents.md#talent-channeler)* (Spellcraft) makes every rod cost a quarter less mana (rounded to the nearest, at
least 1); *[Frugal caster](talents.md#talent-frugal-caster)* then takes its fifth off what is left.

{{#include generated/rods.md}}
