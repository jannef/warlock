# Wands and rods

Wands and rods let anyone use magic, caster or not. Both are readied in the
[missile weapon slot](equipment.md#what-fits-where) and used the way you shoot: `t`, then aim. A
wand holds one spell and a number of charges, and runs dry. A rod holds an effect no spell has,
never runs out, and is paid for with your own [mana](mana.md#spending-it).

## Wands

A wand holds one spell and a number of charges. Ready it in the
**[missile weapon slot](equipment.md#what-fits-where)**, where it takes a
[launcher](ranged.md#getting-ready)'s place (and a launcher takes its place). Use it as you would
shoot: press `t`, aim, and it casts its spell. Each cast uses up a charge instead of your
[mana](mana.md#how-much-mana-you-have).

A wand casts its spell at a **rung of its own** on the spell's ladder (see
[How a spell grows](spellcasting.md#how-a-spell-grows)). The rung is rolled when the wand is made,
and you see it once you know the wand. In the table below, "Strength" gives the rolls at the
lowest and the highest rung a wand of that kind may have; every rung in between is as likely.
Your stats never change it, so a warrior with no magic fires a wand as well as a wizard does.

Otherwise the spell works as it always does:

- A ray [rolls to hit](combat.md#hitting) like any spell attack, with your
  [Dexterity](stats.md#stat-dexterity).
- Magic missile never misses.
- An area catches whoever is in it.
- How far it reaches is in the spell's row and depends on your
  [Willpower](stats.md#stat-willpower), just as for your own casts.

A cast that cannot go off (aimed at yourself, or out of reach) uses no charge and takes no time. One
that goes off takes a cast's time.

An empty wand does nothing ("Nothing happens.") and takes no time. It stays a wand, to sell or to
keep. Wands are **never recharged**.

Wands are found unknown. Each kind has its own look for the whole game ("an oak wand"). Using one
tells you its kind, and from then on every wand of that kind shows by name. Its **charges** stay
unknown until you [identify](magic-items.md#unidentified-items) that wand, with the
[scroll of identify](scrolls.md#scroll-of-identify) or a [merchant](class-merchant.md)'s
[Merchant's eye](class-merchant.md#talent-merchants-eye). Wands lie in the dungeon and are sold in general
stores.

The *[Wand mastery](talents.md#talent-wand-mastery)* talent (Weaponcraft) moves every wand you use
one rung up, and [magic items](affixes.md#suffixes) *of Wands* add their rungs too, never past
the ladder's last rung. Once you know a wand, its description and the aiming line say what it does
in your hands, with your talents and items counted ("Holds Fireball: 8d6 fire, radius 2").

{{#include generated/wands.md}}

Each wand is made with its own number of charges (in the table above).

## Rods

A rod holds an **effect no spell has**: [digging](digging.md#digging), sleep, terror, or sending a
creature away. Like a wand, it goes in the **missile weapon slot** and is used the way you shoot
(`t`, then aim). It has no charges. Each use costs **your own [mana](mana.md#spending-it)** (the
rod's cost), so it never runs out while you have the [mana](mana.md#how-much-mana-you-have) to feed it.

Without enough mana a rod does nothing and takes no time ("You have not the mana to use it."). You
cannot use one while something stops you casting, such as
[raging](status-list.md#status-raging). A use takes a cast's time.

A rod's row in the table says how far it reaches, how much further your
[Willpower](stats.md#stat-willpower) takes it, and the strongest creature its sleep or terror
affects. Its reach (how far a line digs, how far a bolt flies, how far away a burst may land) reads
like a [spell's](spellcasting.md#how-a-spell-grows): the first number is what anyone reaches, and
each rung after it is the reach of a user whose Willpower is at least that rung's score.

Sleep and terror only affect creatures of a level up to the rod's own (20); the others shrug it off.
Your own level plays no part. No skill makes a rod better or cheaper: in the hands of anyone with
the same Willpower, it costs the same and reaches as far.

- **Digging** breaks every [wall](terrain.md#walls) on a straight line toward where you aim, as if
  dug through. It stops at a wall too hard for it (granite), a [door](doors.md), or the level's edge.
- **Teleport other** sends the first creature its bolt meets to a random free cell of the level. It
  counts as an attack: a [shopkeeper](shops.md) will not forgive it.
- **Sleep** puts the creatures in its burst into a true sleep (not the drowsiness of the
  [Sleep](magic.md#spell-sleep) spell). They take no turns until a noise, a blow, or a cry wakes them.
- **Terror** makes the creatures in its burst flee from you for a few turns.

Rods are found unknown. Each kind has its own look for the whole game ("a pewter rod"). Using one,
or identifying it, tells you what it is; from then on its cost and reach show when you look at it
and aim it. Rods lie in the dungeon and are sold in general stores.

*[Channeler](talents.md#talent-channeler)* (Spellcraft) makes every rod cost a quarter less mana
(rounded to the nearest, at least 1). *[Frugal caster](talents.md#talent-frugal-caster)* then
takes a fifth off what is left.

{{#include generated/rods.md}}

See also: [Magic items](magic-items.md), [Ranged combat](ranged.md), and [Scrolls](scrolls.md).
