# Magic

A **spell** is cast from the [abilities menu](abilities.md#the-abilities-menu) (`a`), under its own heading *Spells*, and paid in
[mana](mana.md). You know a spell for good once your class gives it or once you have learned it
from a scroll (below). Every spell is measured by the same rules below: its [spell level](#spell-levels), its
[shape](#shapes), and how it finds its target; your [caster level](#caster-level) is what you can learn.

## What your class gives

Each casting class gives only a few spells, with its [class table](classes.md#levels): a full
caster a [cantrip](#spell-levels) at level 1 and about four in all, a bard, a paladin, or a ranger a cantrip or a
boon, and the rest none. Every other spell is learned from scrolls, so a caster who finds few
scrolls stays a modest one. A spell a class gives never comes above your [caster level](#caster-level) at the level
it comes.

{{#include generated/class-spells.md}}

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

Every spell has a level, 0 to 70. It is the spell's base [mana](mana.md#spending-it) cost, and the [caster level](#caster-level) at which you
learn it from a scroll for certain. A spell of level 0 or 1 is a *cantrip*: it costs 1 mana, and
anyone with mana learns it for certain. What a spell does is its own roll and ladders (the list
above), whatever its level.

## Caster level

Your caster level is how far your learning reaches: a scroll of a spell at or below it teaches
you for certain; above it, the study is a check (see [Learning](#learning-spells-from-scrolls)). It never changes what a
spell does or costs: your [stats](stats.md) grow a spell (see [How a spell grows](#how-a-spell-grows)), and your
[mana](mana.md#how-much) is what holds you back.

It is your [level](experience.md#levels) times your class's **casting** share, rounded down, plus what [talents](talents.md) and
[magic items](magic-items.md) (*of the Adept*) add; anyone with any [mana](mana.md#how-much) has at least caster level 1. Everyone
learns a little: a level-30 [warrior](classes.md#warrior) is a caster of level 7. A [wizard](classes.md#wizard)'s *[Arcane
training](talents.md#talent-arcane-training)* adds a caster level for every 5 [character levels](experience.md#levels), so a level-30 [wizard](classes.md#wizard) learns spells
of level 36 for certain; *[Attuned](talents.md#talent-attuned)* counts the caster level your magic items give half again.

{{#include generated/spell-casting.md}}

## What a spell costs

A spell's mana is its [level](#spell-levels), at least 1, times its [shape](#shapes)'s share (below), times its
[resolution](#how-a-spell-finds-its-target)'s (a touch is cheaper), times what your [armour](objects.md#class-body-armour) adds, rounded to the
nearest point once at the end and never below 1. Your [caster level](#caster-level) never counts: a spell you know
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
- **A rider**: *[Numbing cold](talents.md#talent-numbing-cold)* leaves everyone your cold spells harm a degree [Sluggish](status-effects.md#status-sluggish) besides;
  nothing on a miss.
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
sold, or given away. You need some [mana](mana.md#how-much) to learn at all.

A study takes time, by the spell's level: 10 minutes and 2 more a level, at a normal pace (Fireball,
level 11: 32 minutes):

{{#include generated/study-times.md}}

The world goes on around you. A creature coming into view, a blow, or anything else that would
stop a rest stops the study: the time is lost, but you keep the scroll. Once you begin, you
cannot stop of your own accord.

A spell of a level at most your [caster level](#caster-level) is learned for certain. Above it, every level too
high (the **overreach**) makes it harder: a roll of 2d12 plus your bonus must reach the difficulty,
12 + 1 per level of overreach (*[Overreach](talents.md#talent-overreach)* counts a third fewer, rounded down), a point more for
every 20 levels of the spells you already know, added up. Your bonus is your [Intelligence](stats.md#stat-intelligence) [modifier](stats.md#modifiers), a tenth of your
*[Spellcraft](skills.md#skill-spellcraft)* and of your *[Decipher script](skills.md#skill-decipher-script)* (each at most +20), your class's affinity with the
spell's school, and *[Quick study](talents.md#talent-quick-study)*'s +2. A natural 24 always succeeds and a natural 2 always fails.
The game tells you the chance in words: *certain*, *easy*, *a struggle*, or *far beyond you*.

{{#include generated/study-chances.md}}

{{#include generated/study-affinities.md}}

If you fail, the scroll crumbles to dust and a **mishap** befalls you, worse the farther the
spell was beyond you: *slight* below 6 levels of overreach, *bad* from 6, *terrible* from
15. A few lucky mishaps can come with any of them. A failure costs only that scroll:
the spell may come your way again.

{{#include generated/mishaps.md}}

Studying trains *[Spellcraft](skills.md#skill-spellcraft)*, *[Decipher script](skills.md#skill-decipher-script)*, and your [Intelligence](stats.md#stat-intelligence), whatever comes of it.
In a [scriptorium](special-rooms.md#room-scriptorium) ([Special rooms](special-rooms.md)) [Decipher script](skills.md#skill-decipher-script)'s mark is twice as likely,
so certain.

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
its **charges** stay unknown until you [identify](magic-items.md#unidentified-items) that wand (the [scroll of identify](scrolls.md#scroll-of-identify), or carrying
it long enough with *[Lore](talents.md#talent-lore)*). Wands lie in the dungeon and in general stores.

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
others shrug it off. Your caster level plays no part: no skill makes a rod better or cheaper,
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
hastes you as [Haste](#spell-haste) does) has a thin circle of runes turning slowly about them.
It says no more than the [status bar](controls.md#the-screen) and [look mode](controls.md#look-mode)
do: which buffs and hexes, never how strong or for how long.

- **Each spell has a rune**, one character, drawn in the colour of its [school](schools.md).
- **The runes sit at the corners of a figure**, repeated so that there are always at least three:
  one effect is a triangle of its rune, two a square of both in turn (`A B A B`), three a
  triangle `A B C`, four a square, five a pentagon, six a hexagon. A seventh has no rune.
- **One thin ring per effect**, three at most: one ring is one effect, two rings two, three rings
  three or more (count the different runes then).
- **Buffs are whole and turn clockwise**, a turn a minute. **Hexes are broken** (dashed rings and
  sides) **and turn the other way**, a turn in 45 seconds. Someone with both has the buffs' circle
  inside and the hexes' around it.
- The runes go in the order the status bar lists the statuses, so the same buffs always make the
  same figure. Each rune faces the middle of its circle (its foot toward the centre, so the one
  at the bottom stands on its head), has a soft dark patch under it and a faint glow of its
  colour, and slowly swells and fades a little, each on its own beat (every 3.6 seconds).
- **A circle keeps to what you see**: its runes and rings show only over floor you can see now,
  never over walls, darkness, or anything else drawn on the map (a creature, an item, a torch on
  the wall); they fade out softly where they reach one. Where two circles crowd, the later one's
  runes and rings give way to the earlier's (yours first).
- **A circle fades in** over half a second when the first buff or hex begins, and out as long
  when the last ends. When one begins or ends while others go on, the figure changes over 0.4
  seconds: the runes glide round the ring to their new corners, and the old figure and rings fade
  into the new. A hexes' circle moves outward when a buff's circle comes inside it, and back in
  once that one is gone. With Reduced effects or Slides, camera off, none of this moves (nor do
  the runes swell): a circle appears, changes, and goes at once. Reduced effects also takes the
  runes' glow away ([Settings](settings.md)).

Every buff shows on a creature, whatever gave it: look mode names a hasted creature "quickened",
a bark-skinned one "bark-skinned", one that drank a [potion of might](potions.md#potion-of-might) "fortified" (see
[Status effects](status-effects.md)). The drawn circles are only for show (the **Spell circles**
switch in [Settings](settings.md) hides them; Reduced effects only stills them), but the light
they shed is not.

{{#include generated/spell-runes.md}}

### The circle's light

A runed status sheds a faint, steady [light](light.md) of its rune's school, whatever gave the
status (the potion of quickness's Hasted too) and whoever has it, you or a creature: the
`circle_<school>` lights of the [lights table](light.md#light-sources), each reaching one cell,
with a strong [ultraviolet](vision.md#the-four-types) share. Its strength never changes with the status's
[severity](status-effects.md#how-they-work) or the time left, and it goes the turn the status ends. Someone with several runed
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
