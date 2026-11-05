# Magic

A **spell** is cast from the [abilities menu](abilities.md#the-abilities-menu) (`a`), under its own heading *Spells*, and paid in
[mana](mana.md). You know a spell for good once your class gives it or once you have learned it
from a scroll (below). Every spell is measured by the same rules below: its [spell level](#spell-levels), its
[shape](#shapes), and how it finds its target; your [learning](#learning) is what you can learn.

## What your class gives

Each casting class gives only a few spells, with its [class table](classes.md#levels): a full
caster a [cantrip](#spell-levels) at level 1 and from three to a dozen in all (the [wizard](classes.md#wizard) and the [witch](classes.md#witch) also start
with a short [cone](#shapes) that needs no hit roll, [Burning hands](#spell-burning-hands) and [Grave breath](#spell-grave-breath): good for a
fight with a few weak foes, and a few casts empty the pool), a bard, a paladin, or a ranger a few
boons and hexes, and the rest none. A class's spells are only where it meets them first: any
spell can be learned from a scroll, and some come only from scrolls ([Acid arrow](#spell-acid-arrow),
[Grave spear](#spell-grave-spear), [Slumber](#spell-slumber), [Glamour](#spell-glamour), and others), so a caster who finds few
scrolls stays a modest one. A spell your class gives is yours at that level, whatever your [Spellcraft](skills.md#skill-spellcraft).

{{#include generated/class-spells.md}}

Three spells are aimed at a [corpse](food.md#corpses) you see, and any age will do: without one
they are refused, for no time and no mana. [Charnel burst](#spell-charnel-burst) goes off where
the corpse lies, flying nowhere, and what it kills leaves corpses for the next. [Grave
feast](#spell-grave-feast) reaches only a corpse beside you or under your feet, and gives back
hit points and mana, never past your maximum. [Rouse the dead](allies.md#the-risen) stands it up.

## The spells

Every spell has a **[level](#spell-levels)**, 0 to 70: what it costs in [mana](mana.md#spending-it), and how learned a caster must
be to take it from a scroll for certain. Every spell has a **base**, what it does for anyone who
casts it, and may have **ladders**: for its damage (or healing), its reach, and how long its
effect lasts. A ladder names one [stat](stats.md) and lists scores with what the spell does at each:
*[Fireball](#spell-fireball): 2d6 fire; Intelligence 13: 3d6, 16: 4d6, 19: 5d6, 25: 5d6+2*. When your score
reaches a number, you cast that rung. **The last rung is as far as your stats take it.** Your
[talents](talents.md) and [magic items](magic-items.md) may add to that in their own ways, each said in its own row: a rung
up, a die more, bigger dice, a bonus, a die rolled again (see [How a spell grows](#how-a-spell-grows)). A
[scroll](scrolls.md#spell-scrolls) read and a [wand](#wands) used cast at a rung of their own.

The list below gives each spell's level, its base (what it does and how far it reaches), its
ladders ("Grows with": a stat alone is the damage or healing ladder; "range", "length",
"radius", and "degrees" name the others), and its [mana](mana.md#how-much) without armour or talents:

{{#include generated/spell-list.md}}

Every spell belongs to one of seven [schools of magic](schools.md), and each school is a skill
you train by casting its spells: your rank in it rolls them stronger, cheaper, or quicker.

The abilities menu shows a spell's roll as you would cast it now, with its damage type, as the
[character screen](character.md) shows a weapon's (`4d6 fire, 26 mana, aimed`), and what it costs
you now, with your [talents](talents.md) and armour counted; the line below it says how far it
reaches now and which dice you roll again. **[Identify](#spell-identify)** asks which of your things to cast it on: the list of
what you carry unidentified opens, as for the [scroll of identify](scrolls.md#scroll-of-identify) (see
[Scrolls](scrolls.md#the-scroll-of-identify)).

Every [scroll](scrolls.md) is a spell: each one you can read casts a spell, and each teaches its spell if you
study it, the [scroll of identify](scrolls.md#scroll-of-identify) [Identify](#spell-identify) and the [scroll of light](scrolls.md#scroll-of-light) [Light](#spell-light).

## Spell levels

Every spell has a level, 0 to 70. It is the spell's base [mana](mana.md#spending-it) cost, and the [learning](#learning) at which you
learn it from a scroll for certain. A spell of level 0 or 1 is a *cantrip*: it costs 1 mana (but for the few that
name a price of their own: [Burning hands](#spell-burning-hands) and [Grave breath](#spell-grave-breath) cost 3), and
anyone trained in [Spellcraft](skills.md#skill-spellcraft) learns it for certain. What a spell does is its own roll and ladders (the list
above), whatever its level.

## Learning

Your **learning** is how far your study of magic reaches: a scroll of a spell at or below it
teaches you for certain; above it, the study is a check (see [Learning spells from
scrolls](#learning-spells-from-scrolls)). It never changes what a spell does or costs: your
[stats](stats.md) grow a spell (see [How a spell grows](#how-a-spell-grows)), and your
[mana](mana.md#how-much) is what holds you back.

It comes from the [Spellcraft](skills.md#skill-spellcraft) skill and nothing else: the skill's
value, with what [magic items](magic-items.md) (*of the Scholar*) and [talents](talents.md) add
to it, read on this curve, and at least 1. The first fifty points buy the cantrips and the first
spells, the soft cap of 100 reaches level 30, and 150 the last spell level:

{{#include generated/learning.md}}

**A character not trained in Spellcraft cannot learn a spell from a scroll at all**, cantrips
included: the study is refused, and takes no time. Trained means the skill is among your
[skills](skills.md); an item's or a talent's bonus to a skill you do not have does not train you.
You can still read a scroll to [cast it](scrolls.md#spell-scrolls), [by a check](scrolls.md#making-out-a-scroll) (your learning reads a scroll at or
below it with none), and your class's own spells still come at their levels.

Every class that casts starts trained, and [its dice are in the class table](skills.md):
[wizards](classes.md#wizard), [witches](classes.md#witch), [priests](classes.md#priest), and
[druids](classes.md#druid) 2d6+10; [bards](classes.md#bard), [paladins](classes.md#paladin), and
[rangers](classes.md#ranger) 2d4+4; an [aristocrat](classes.md#aristocrat) one time in three (35 %, 2d4+4)
and an [ascetic](classes.md#ascetic) one in five (20 %, 2d4). Other classes do not start with it. Two
[background stories](background.md) add to it, for classes that already have it, and level-up rolls raise it like any other skill.

## What a spell costs

A spell's mana is its [level](#spell-levels), at least 1, times its [shape](#shapes)'s share (below), times its
[resolution](#how-a-spell-finds-its-target)'s (a touch is cheaper), times what your [armour](objects.md#class-body-armour) adds, rounded to the
nearest point once at the end and never below 1. Your [learning](#learning) never counts: a spell you know
costs the same whoever you are, and your [mana](mana.md#how-much) is what holds you back. Then your talents, their
percents added up and rounded down, never below 1: *[School focus](talents.md#talent-school-focus)* takes 10 % off the spells of its
school or element, *[Archmage](talents.md#talent-archmage)* 50 %, and *[Empower](talents.md#talent-empower)* 10 %, and *[Brimming](talents.md#talent-brimming)* adds 10 %; and then *[Frugal caster](talents.md#talent-frugal-caster)* takes a fifth off,
and [magic items](magic-items.md) *of Thrift* their percent with it, never more than half together (see [Mana](mana.md)).

A few spells have a cost of their own instead, whatever their level: **[Identify](#spell-identify)** costs 60 [mana](mana.md#spending-it)
(before your armour and talents), so no caster can cast it in their first levels. A [wizard](classes.md#wizard) has
the mana for it from about level 7, a [priest](classes.md#priest) or a [druid](classes.md#druid) from about level 8. Its level (6) says only
how hard it is to learn.

Heavy [armour](objects.md#class-body-armour) and big [shields](objects.md#class-shields) hinder casting: while worn, each makes every spell dearer, and what
you wear adds up. A *Mage-wrought* piece hinders casting less by its own number (body armour, a
[helmet](objects.md#class-helmets), or a shield that hinders at all): Mage-wrought plate of 15 adds 25 % instead of 40 %, and
one eased to nothing (chain mail of 10 or more) adds none and no longer counts as armour for
your [caster style](schools.md#caster-styles). Its ease is its own: it never eases
another piece.

{{#include generated/spell-armour.md}}

## Shapes

A spell strikes one target (a ray, a bolt, a touch, or yourself) or an area (a cone, a line, or a
burst). Areas cost more [mana](mana.md#spending-it): the base cost is multiplied by the shape's.

{{#include generated/spell-shapes.md}}

- A **ray** or a **bolt** flies along the line toward the place you aim at and strikes the first
  creature in its way. A [hex](#spell-hex-of-weakness) is cast the same way at a hostile you see, and falls on it.
- A **line** runs through everyone on it, as far as its length, until a wall stops it.
- A **cone** fans out from you toward where you aim, a quarter turn wide, as far as its length.
- A **burst** flies toward where you aim, goes off on the first creature in its way (or there),
  and strikes everyone within its radius that it reaches, half as hard at its edge. It can catch
  you too.
- A **touch** reaches only a creature next to you.

How far a spell reaches is in its row, and so is how far your **[Willpower](stats.md#stat-willpower)** takes it. A
[hex](#spell-hex-of-weakness)'s or a [blessing](#spell-blessing)'s row says how long your **[Presence](stats.md#stat-presence)** makes it last (each degree lasts
as long again, up to the most its [status](status-effects.md#how-they-work) has). A burst costs by its base radius.

## How a spell finds its target

Each spell resolves one way. An **attack** rolls [to hit](combat.md#hitting) like a [missile](ranged.md#shooting-and-throwing) (2d12 plus your [Dexterity](stats.md#stat-dexterity)'s
modifier against the target's [Defence](combat.md#defence); a spell may name another stat to aim with, none
does yet; a spell scroll or a wand always aims with Dexterity), can miss, and can be a [critical hit](combat.md#criticals), and does full damage.
Like a blow, it threatens a [critical](combat.md#criticals) on a natural 22 or more; [magic items](magic-items.md) *of the Magus* add
their number to your own spells' roll to hit, and *Baleful* ones widen their threat by theirs
(a Baleful +2 threatens from 20), never below 2. Neither helps a spell scroll you read or a
wand you use.
Your rank in the spell's [school](schools.md) and your
[caster style](schools.md#caster-styles) add to its roll when you cast it yourself.
An **auto** spell always hits, for less. An **area** spell hits everyone in its shape with no
roll, for less again per target, and *[Evasion](talents.md#talent-evasion)* halves what it does. A **touch** spell rolls to hit
at arm's length and does the most, for less mana. Only attack and touch spells can be [critical
hits](combat.md#criticals); a [critical](combat.md#criticals) rolls the damage again. [Resistances](damage-types.md#armour-and-resistance) take their
share of any spell's damage, and [armour](objects.md#class-body-armour) none.

What each resolution does to a spell's cost:

{{#include generated/spell-resolutions.md}}

## Spell types

Every spell of a [damage type](damage-types.md) carries that type's rule, whether cast from [mana](mana.md#spending-it), a [scroll](#learning-spells-from-scrolls), or a [wand](#wands). It
is magic's alone: a [Flaming](magic-items.md#prefixes) sword's blow reads no rule. It spares no one: you and your [allies](allies.md) are struck like anyone it reaches.

{{#include generated/spell-types.md}}

**Fire catches.** One creature in four that a fire spell harms is left [burning](status-effects.md#burning), whatever the
spell: a [Fireball](#the-spells) on five kobolds leaves one or two alight. A creature that takes no harm from it
never burns. A fire burst melts the ice on the cells it reaches, and whoever stands on that ice
[falls in](temperature.md#water-freezes); it leaves a fire on the cell at its centre, which gives light for a minute. No rule reads that
fire, and it never spreads.

**Cold slows the blood.** Every creature a cold spell harms is [Sluggish](status-effects.md#status-sluggish) besides, and whatever burned on it goes
out; nothing on a miss. A cold burst freezes the [water](terrain.md#water) on the cells it reaches, into ice that holds a
laden person, and puts out the fires there.

**Lightning seeks the nearest path.** A ray, a bolt, a touch, or a line of electricity that harms a creature
strikes once more, one time in four, at the nearest creature beside it not yet struck (ties go to the one that
came into the world first), at the same roll, and no roll to hit nor [Evasion](talents.md#talent-evasion) saves anyone from it. That one may arc again, and
so on. It is not choosy: [Spark](#the-spells) at a rat beside you may jump back into you, or into your [ally](allies.md).

**Water carries it.** A creature harmed while standing in [water](terrain.md#water) is one of its whole body, every
cell of open water connected to it, diagonals too: everyone standing in that body not yet struck is struck at the same
roll, you too if you wade, one after another in the order they came into the world, and each of them may arc. Every
electricity spell does this, whatever its shape. [Ice](temperature.md#water-freezes) is not water: it carries nothing, and a frozen band parts a
pond in two. Each creature only ever takes a cast once. Aiming shows you none of this.

**Chain lightning** takes its first jumps for certain, three at first and up to six as your
[Willpower](stats.md#stat-willpower) grows, and any jump of it reaches two cells instead of one, so long as nothing
that stops sight stands between. Past those, a jump is one time in four as usual. **[Conductor](talents.md#talent-conductor)** makes
every arc of your electricity spells happen, and you resist electricity 25 %: it spares you nothing, it only bites less. A chain ends
when no creature not yet struck is within reach of the last, or the last took no harm.

**A ray, a bolt, or a touch** does to one cell what a burst does to all of its own: the cell of the creature it
strikes, else the cell you aimed at. So [Ray of frost](#the-spells) makes a bridge across a lake, a cell a cast.
A line does it to the last cell it reaches, a cone not at all.

**Force pushes.** A force spell that takes at least a third of a creature's [hit points](combat.md#hit-points), or lands a
[critical](combat.md#criticals), knocks it a cell away: from the centre of a burst, from you otherwise, and from you too for
the one at the burst's centre. A cell that is not free stops it, and so does [being held or webbed](status-effects.md#status-held); you do not
move when you are at the centre of your own burst, though you take the harm. A force burst with a blast
breaks the [shut doors](doors.md) it reaches, however sturdy, locked or trapped, the trap going with the door and
striking no one; [chests](chests.md) break under any blast below their hardness. It also shatters thin [ice](temperature.md#water-freezes), and whoever
stands on that ice falls in. A fire blast leaves doors standing.

**Poison lingers.** Every creature a poison spell harms is [Poisoned](status-effects.md#poisoned) a degree, besides what the spell's own
line says.

**Death takes the dying first.** A necrotic spell does half again as much, rounded down, to a creature below
half its hit points, before what it resists. The undead are immune to necrotic harm.

**Acid keeps eating.** Every creature an acid spell harms is [Corroded](status-effects.md#corroded) a degree: two more hit points in the
next twelve seconds. Acid does not eat armour.

## Spells with a rule of their own

Some [spells](#the-spells) do one thing no other does. None of them spares you or your [allies](allies.md).

- **Thunderclap** and **Benison** go off about you: they ask for no aim, and you are at their
  centre. Thunderclap strikes you with the full roll, as everyone at a burst's centre, and leaves
  you [Stunned](status-effects.md#status-stunned) with the rest; the whole level hears it. Benison's one roll heals you and every ally of yours within it who is hurt; it does nothing to anyone
  else, and it is not cast when no one there is hurt.
- **Leech** gives you back half the [hit points](combat.md#hit-points) it takes, rounded down, never past your maximum. The undead, immune to [necrotic](#spell-types) harm, give nothing.
- **Rebuke** reaches only the undead, yours and others': the living in it are not touched. It is
  fire, so it [burns](status-effects.md#burning) as fire does and leaves a fire at its centre.
- **Bramble** catches only those standing where things grow: grass, marsh, reeds, flowers, bushes,
  forest floor, fungus (the [terrain](terrain.md) table says which). On stone it does nothing.
- **Gale** drives everyone on its line a cell straight away from you, the nearest first: one with
  someone behind it stays put, and so does one [held or webbed](status-effects.md#status-held). It harms no one, and it puts out the fires on its line and tears away the [zones](#zones) of fire, foul cloud, and darkness on it; slick ground stays. A shut door stops it.
- **Blink** takes you to a free cell within 5, any of them as likely, through walls or not. It
  works while you are [held](status-effects.md#status-held), [webbed](status-effects.md#status-webbed), or [pinned](status-effects.md#status-pinned), and ends all three.
- **Sense the quick** lets you feel every living creature within 8 cells that you cannot see as a
  heartbeat, [heard](hearing.md) up to 2 cells from where it is, every turn while it lasts. The undead, constructs, and
  animated things have none.
- **Cloudburst**, cast under the open sky, brings an hour of [rain](weather.md) and at once puts out every fire and
  everyone [burning](status-effects.md#burning) under the open sky on the level.

## Zones

A **zone** is an area that lies on the floor for a while: [Balefire](#the-spells) burns, [Reek](#the-spells) poisons, [Mirk](#the-spells) darkens, and [Slick](#the-spells)
trips. All four come only from [scrolls](#learning-spells-from-scrolls) you find. A zone's cells stay as they were laid, and it never spreads. It
takes you and your [allies](allies.md) like anyone: the caster has no protection against their own, though a
kill it makes is theirs. A zone laid over another replaces it on the cells they share. It stays when you
leave the level, its time running while you are away. No zone can be laid on the [overworld](overworld.md).

| Kind | Who it takes | What it does |
|---|---|---|
| Fire | whoever starts a turn on it or steps onto it | the spell's [fire](#spell-types) harm, one number rolled as it is laid and the same for all its time, halved by [Evasion](talents.md#talent-evasion); then fire's one chance in four of [burning](status-effects.md#burning). Each cell sheds a fire's light, and the [ice](temperature.md#water-freezes) under it melts as it is laid. It lies only on walkable ground out of the water |
| Foul cloud | whoever starts a turn in it | [Poisoned](status-effects.md#poisoned) one degree more, no other harm; nothing to the [poison-immune](damage-types.md). It does not block sight |
| Slick ground | whoever steps onto it or off it | a [Dexterity](stats.md#stat-dexterity) check against 12: failed, [Staggered](status-effects.md#status-staggered), the next action half again as slow. The step is still taken. Nothing incorporeal slips. It lies only on walkable ground out of the water |
| Darkness | no one directly | no light reaches its cells, the sun's included, and a light inside sheds none, a burning creature's too. Eyes that need light see nothing inside; [darkvision](vision.md#the-four-types) sees in as far as it reaches. The player's eyes follow the same rule, and from inside it they see only the cell they stand on and what is lit beyond |

A [cold](#spell-types) spell puts out the fire on each cell it reaches, and the rest of the zone burns on. A zone shows as a still
tint of its cells, and look mode names it.

## How a spell grows

Each spell's row lists its ladders: a stat and the scores at which the spell changes. Your score
reaches the number, you cast that rung. The score is the one your character screen shows, with
what your [status effects](status-effects.md), [stat damage](status-effects.md), and [magic items](magic-items.md) do to it; a low score never shrinks a
spell below its base. Spell damage is rolled dice, as written in the spell's row; what grows it is
its ladder, and your talents and items.

Your [talents](talents.md) and [magic items](magic-items.md) may add to that, each in its own way, said in its own row:

- **A rung**: *[Empower](talents.md#talent-empower)* (a [wizard](classes.md#wizard)'s at 32) takes your spells a rung up their damage and
  healing ladders, *[Archmage](talents.md#talent-archmage)* (at 60) two; on the last rung they stay there.
- **Bigger dice**: *[Keen mind](talents.md#talent-keen-mind)*, taken for a school of magic or an element, makes those spells roll
  dice a size bigger for you: a d6 is a d7, a d8 a d9.
- **A die more**: *[Brimming](talents.md#talent-brimming)* rolls a die more on your damage and healing spells, and they cost a
  tenth more [mana](mana.md#spending-it).
- **A bonus**: *Potent* [magic items](magic-items.md#prefixes) add 1 to 3 to every spell's roll, harm or healing (a Potent +2 ring
  makes 4d6 into 4d6+2).
- **A die rolled again**: *[Sure hand](talents.md#talent-sure-hand)* rolls a spell die that shows a 1 once more, and that roll stands.
- **Reach and degrees**: *[School focus](talents.md#talent-school-focus)*, taken for a school or an element, makes those spells reach
  a cell farther (range or length, not a burst's radius) and cost 10 % less; [magic items](magic-items.md#suffixes) *of
  Farcasting* add their cells to every spell's reach, *of Lingering* their degrees to every
  blessing and hex.

They apply in that order: the rung, then the dice, then the roll (with its rerolls), then the
harm, then the riders. A [critical](combat.md#criticals) rolls the dice twice, as a blow's does. A **[spell scroll](scrolls.md#spell-scrolls)**
you read casts at the rung the scroll's row says (the [scroll of fireball](scrolls.md#scroll-of-fireball)'s is 1: 3d6), a rung higher
with *[Scroll adept](talents.md#talent-scroll-adept)*, whatever your stats; a **[wand](#wands)** at a rung of its own, a rung higher with
*[Wand mastery](talents.md#talent-wand-mastery)* and higher again by *of Wands*. Your other talents and items work on them as on
your own casts, and their reach and degrees are yours. Most of these talents are Spellcraft's,
the tree a [wizard](classes.md#wizard) takes bonus talents from beside Scholar; see [Talents](talents.md) for every talent that
touches spells.

Two kinds of damage only magic does, **[force](damage-types.md#damage-force)** and **[necrotic](damage-types.md#damage-necrotic)**, join the
[damage types](damage-types.md).

Spells you cast are yours: what they kill earns you its [experience](experience.md), and a spell of your own that
catches you can kill you.

## Learning spells from scrolls

Every spell comes on a scroll of its own (the [scroll of magic missile](scrolls.md#scroll-of-magic-missile) teaches *[Magic missile](#spell-magic-missile)*);
the [scroll of fireball](scrolls.md#scroll-of-fireball) teaches *[Fireball](#spell-fireball)*, the [scroll of lightning](scrolls.md#scroll-of-lightning) *[Lightning bolt](#spell-lightning-bolt)*, the [scroll of
frost](scrolls.md#scroll-of-frost) *[Frost](#spell-frost)*. **[Reading](scrolls.md)** a scroll casts its spell once at the scroll's own rung and costs no
[mana](mana.md#spending-it) ([Scrolls](scrolls.md#spell-scrolls)). **Studying** it (`L`, then the scroll) teaches you its
spell for good, and uses the scroll up whatever comes of it.

You can study a scroll once you know what it is (you have read one like it, or [identified](magic-items.md#unidentified-items) it)
and only if you do not know its spell yet; an unknown scroll is refused ("You do not know what
this says."), so [identify](magic-items.md#unidentified-items) your scrolls first. A scroll of a spell you know can still be read,
sold, or given away. You need some [mana](mana.md#how-much) to learn at all, and training in
[Spellcraft](skills.md#skill-spellcraft): without it the study is refused, and takes no time ("You have no training in
Spellcraft; you cannot make sense of how to learn a spell from a scroll.").

A study takes time, by the spell's level: 10 minutes and 2 more a level, at a normal pace (Fireball,
level 11: 32 minutes):

{{#include generated/study-times.md}}

The world goes on around you. A creature coming into view, a blow, or anything else that would
stop a rest stops the study: the time is lost, but you keep the scroll. Once you begin, you
cannot stop of your own accord.

A spell of a level at most your [learning](#learning) is learned for certain. Above it, every level too
high (the **overreach**) makes it harder: a roll of 2d12 plus your bonus must reach the difficulty,
12 + 1 per level of overreach (*[Overreach](talents.md#talent-overreach)* counts a third fewer, rounded down), a point more for
every 20 levels of the spells you already know, added up. Your bonus is your [Intelligence](stats.md#stat-intelligence) [modifier](stats.md#modifiers), a tenth of your
*[Spellcraft](skills.md#skill-spellcraft)* (at most +20), your class's affinity with the
spell's school, and *[Quick study](talents.md#talent-quick-study)*'s +2. A natural 24 always succeeds and a natural 2 always fails.
A few spells are harder to learn than their level says: their own difficulty is added to the
check, and a spell made harder this way is never learned for certain, however high your
learning (its overreach then counts as 0). [The spell list](#the-spells) says which, and by how much.
The game tells you the chance in words: *certain*, *easy*, *a struggle*, or *far beyond you*.

{{#include generated/study-chances.md}}

{{#include generated/study-affinities.md}}

If you fail, the scroll crumbles to dust and a **mishap** befalls you, worse the farther the
spell was beyond you: *slight* below 6 levels of overreach, *bad* from 6, *terrible* from
15. A few lucky mishaps can come with any of them. A failure costs only that scroll:
the spell may come your way again.

{{#include generated/mishaps.md}}

Studying trains *[Spellcraft](skills.md#skill-spellcraft)* and your [Intelligence](stats.md#stat-intelligence), whatever comes of it. *[Decipher script](skills.md#skill-decipher-script)* has no part in a
study: it is for [making out a scroll](scrolls.md#making-out-a-scroll) you only want to read.
Spending [mana](mana.md) on a spell also trains *[Spellcraft](skills.md#skill-spellcraft)*, one time in four.

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
description and the aiming line say what it does in your hands ("Holds Fireball: 4d6 fire,
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
flies, how far away a burst may land) is read as a [spell's](#how-a-spell-grows) is: the first number is
what anyone reaches, and each rung after it is the reach of a user whose Willpower is at least
the rung's score. Sleep and terror take only creatures of a level up to the rod's own (20); the
others shrug it off. Your level plays no part: no skill makes a rod better or cheaper,
and what it costs and how far it reaches are the same in anyone's hands of the same Willpower.

- **Digging** breaks every [wall](terrain.md#walls) on a straight line toward where you aim, as if dug through,
  until a wall too hard for it (granite), a [door](doors.md), or the level's edge stops it.
- **Teleport other** sends the first creature its bolt meets to a random free cell of the
  level. It counts as an attack: a [shopkeeper](shops.md) will not forgive it.
- **Sleep** puts the creatures in its burst into a true sleep (not the [Sleep](#spell-sleep) spell's
  drowsiness): they take no turns until a noise, a blow, or a cry wakes them.
- **Terror** makes the creatures in its burst flee from you a few turns.

Rods are found unknown, each kind with a look of its own for the whole game ("a pewter rod");
using one or identifying it tells you what it is, and its cost and reach then show when you
look at it and aim it. Rods lie in the dungeon and in general stores.

*[Channeler](talents.md#talent-channeler)* (Spellcraft) makes every rod cost a quarter less mana (rounded to the nearest, at
least 1); *[Frugal caster](talents.md#talent-frugal-caster)* then takes its fifth off what is left.

{{#include generated/rods.md}}

## Spell circles

Everyone you see who has a buff or a hex on them (a [status](status-effects.md) laid by one of
the spells below, or the same status from anywhere else: a [potion of quickness](potions.md#potion-of-quickness)
hastes you as [Haste](#spell-haste) does) has a thin circle turning slowly about them.
It says no more than the [status bar](controls.md#the-screen) and [look mode](controls.md#look-mode)
do: which kinds of magic, never how strong or for how long.

- **A figure inside the rings**: one effect is a triangle, two a square, three a triangle again,
  four a square, five a pentagon, six a hexagon. A seventh changes nothing.
- **One thin ring per effect**, three at most: one ring is one effect, two rings two, three rings
  three or more.
- **Buffs are whole and turn clockwise**, a turn a minute. **Hexes are broken** (dashed rings and
  sides) **and turn the other way**, a turn in 45 seconds. Someone with both has the buffs' circle
  inside and the hexes' around it.
- **The colour** is the [school](schools.md) of the first effect in the order the status bar
  lists them. The rings have a faint glow of their colour, and so does the glyph of whoever wears
  the circle (in its rings' colour, the buffs' if they have both). Now and then a tiny glint
  flashes on the wearer's glyph (two beats, every 5.3 and 7.1 seconds, each showing for about a
  tenth of it), and a soft brighter arc runs round each ring (every 7 seconds).
- **A circle keeps to what you see**: its rings show only over floor you can see now, never over
  walls, darkness, or anything else drawn on the map (a creature, an item, a torch on the wall);
  they fade out softly where they reach one. Where two circles crowd, the later one's rings pass
  under the earlier's (yours first).
- **A circle fades in** over half a second when the first buff or hex begins, and out as long
  when the last ends. When one begins or ends while others go on, the old figure and rings fade
  into the new over 0.4 seconds. A hexes' circle moves outward when a buff's circle comes inside
  it, and back in once that one is gone. With Reduced effects or Slides, camera off, none of this
  moves (nor do the glints or the arcs): a circle appears, changes, and goes at once. Reduced
  effects also takes the glow away ([Settings](settings.md)).

Sleep is the one hex without a circle: a creature [asleep](status-effects.md#status-asleep), from
[Slumber](#spell-slumber) or in a den, shows none and sheds no light.

Every buff shows on a creature, whatever gave it: look mode names a hasted creature "quickened",
a bark-skinned one "bark-skinned", one that drank a [potion of might](potions.md#potion-of-might) "fortified" (see
[Status effects](status-effects.md)). The drawn circles are only for show (the **Spell circles**
switch in [Settings](settings.md) hides them; Reduced effects only stills them), but the light
they shed is not.

{{#include generated/spell-circles.md}}

### The circle's light

A circled status sheds a faint, steady [light](light.md) of its school, whatever gave the
status (the potion of quickness's Hasted too) and whoever has it, you or a creature: the
`circle_<school>` lights of the [lights table](light.md#light-sources), each reaching one cell,
with a strong [ultraviolet](vision.md#the-four-types) share. Its strength never changes with the status's
[severity](status-effects.md#how-they-work) or the time left, and it goes the turn the status ends. Someone with several circled
statuses sheds one such light, the first in status order's. The Light spell's own light is
brighter and wins over it.

- **Normal eyes** (a [brightness](vision.md#the-four-types) of 40) see the circled one's own cell, and so the
  circled one, at any distance in line of sight, even in the dark (40 to 52 there, plus the
  cell's other light), but not the four cells beside it (half as bright: 20 to 26; the diagonal ones get none).
- **Low-light eyes** (12) see the four cells beside it too.
- **Eyes that see UV** see it plainly (130 to 142 on its cell) and the four cells beside it as well (65
  to 71), where the UV shows as a violet tint.
- **A buff lights your cell.** Creatures see by the same rule, so a hasted, blessed, or
  bark-skinned sneak is seen by any normal-eyed creature with them in line of sight, as Inner
  light is. A hex marks its target the same way: a webbed or sleeping creature glows for as long
  as it lasts.
- **[Light-blind](vision.md#the-four-types) eyes** add it to the cell's light: a drow in daylight just under their glare (250)
  can be dazzled by their own [Blessing](#spell-blessing).
- A disguised [mimic](mimics.md) sheds none: its disguise would glow.
