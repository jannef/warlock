# Magic

A **spell** is cast from the abilities menu (`a`), under its own heading *Spells*, and paid in
[mana](mana.md). You know a spell for good once your class gives it or once you have learned it
from a scroll (below). Every spell is measured by the same rules below: its tier, your caster
level, its shape, and how it finds its target.

## What your class gives

Each casting class gives only a few spells, with its [class table](classes.md#levels): a full
caster a cantrip at level 1 and about four in all, a bard, a paladin, or a ranger a cantrip or a
boon, and the rest none. Every other spell is learned from scrolls, so a caster who finds few
scrolls stays a modest one. No spell a class gives costs more when it comes than its tier's base
(it never comes above your caster level's tier).

{{#include generated/class-spells.md}}

## The spells

What each spell does, how far it reaches (and how far your Willpower can take it), and its mana at
its tier's comfortable caster level, without armour:

{{#include generated/spell-list.md}}

The abilities menu shows a spell's tier and what it costs you now, with your caster level,
talents, and armour counted.

## Caster level

Your caster level is how strong a caster you are. It is your level times your class's **casting**
share, rounded down, plus what talents and magic items add; anyone with any mana has at least
caster level 1. Everyone casts a little: a level-30 warrior is a caster of level 7. A wizard's
*Arcane training* adds a caster level for every 5 character levels, so a level-30 wizard casts at
level 36.

## Spellcraft talents

The Spellcraft tree is for casters (the wizard takes its bonus talents from it, beside Scholar):
*Quick study* (learning, below), *Overreach* (a spell above your caster level's tier costs a
third less over its price), and *School focus*, taken for a school of magic or an element (a
spell's tags, in the list above), once per choice: those spells cost 10 % less and reach a cell
farther, never past their limit and never harder. See [Talents](talents.md) for every talent
that touches spells.

{{#include generated/spell-casting.md}}

## Tiers

Every spell has a **tier**, from 0 (a *cantrip*) to 9. Each tier has a **comfortable caster
level**: at or above it, learning a spell of the tier is certain and casting it costs its base
mana cost. A spell above your caster level's tier costs more mana to cast, half again for each
tier too high. Each tier also has a **damage ceiling**: the most a single-target spell of that
tier can ever do in one cast at its highest roll.

{{#include generated/spell-tiers.md}}

## What a spell costs

A spell's mana is its tier's base cost, times the overreach (half again per tier above your
caster level's; with *Overreach*, a third less of that: 33.5 % a tier), times its shape's share
(below), times its resolution's (a touch is cheaper), times what your armour adds, rounded to the
nearest point once at the end and never below 1. Then your talents, their percents added up and
rounded down, never below 1: *School focus* takes 10 % off the spells of its school or element,
*Archmage* 50 %, and *Empower* 10 %; and then *Frugal caster* takes a fifth off (see
[Mana](mana.md)).

Heavy armour and big shields hinder casting: while worn, each makes every spell dearer, and what
you wear adds up.

{{#include generated/spell-armour.md}}

## Shapes

A spell strikes one target (a ray, a bolt, a touch, or yourself) or an area (a cone, a line, or a
burst). Areas cost more mana: the base cost is multiplied by the shape's.

{{#include generated/spell-shapes.md}}

- A **ray** or a **bolt** flies along the line toward the place you aim at and strikes the first
  creature in its way. A hex is cast the same way at a hostile you see, and falls on it.
- A **line** runs through everyone on it, as far as its length, until a wall stops it.
- A **cone** fans out from you toward where you aim, a quarter turn wide, as far as its length.
- A **burst** flies toward where you aim, goes off on the first creature in its way (or there),
  and strikes everyone within its radius that it reaches, half as hard at its edge. It can catch
  you too.
- A **touch** reaches only a creature next to you.

Your **Willpower** lets a spell reach farther: a cell more of range or length, and a cell more of
a burst's radius, for every 2 points of its modifier, but never past the spell's own limit (the
"to" in the list). *School focus* adds a cell of range or length (not of a burst's radius) to
the spells of its school or element, again never past the spell's limit. A burst costs by its
own radius, however far your Willpower spreads it. Your
**Presence** makes a blessing or a hex last longer: a degree more of it for every 2 points of its
modifier, up to the most the status has (each degree lasts as long again).

## How a spell finds its target

Each spell resolves one way. An **attack** rolls to hit like a missile (2d12 plus your Dexterity's
modifier against the target's Defence), can miss, and can be a critical hit, and does full damage.
An **auto** spell always hits, for less. An **area** spell hits everyone in its shape with no
roll, for less again per target, and *Evasion* halves what it does. A **touch** spell rolls to hit
at arm's length and does the most, for less mana. Only attack and touch spells can be critical
hits; a critical rolls the damage again, still held to the spell's most. Resistances take their
share of any spell's damage, and armour none.

The most a spell may do to one target is its tier's ceiling times its resolution's share:

{{#include generated/spell-resolutions.md}}

## Damage and its cap

Spell damage is always rolled dice, with a small fixed bonus at most a quarter of the ceiling,
and rarely any. A spell's dice may grow with your caster level (a die more every few levels
above its tier's comfortable level), up to its own most. Talents add to the roll: *Scroll adept*
adds 1 to every die, *Empower* rolls a die more, *Archmage* adds 2 to every die; they bring a
spell to its most sooner, and nothing lifts it past: whatever the dice, the talents, or a
critical, one cast never does more to one target than the "never more than" in the list. The
same talents strengthen a spell scroll you read, held to the most of the spell it teaches (or to
the scroll's own most, if it rolls more of itself: the scroll of lightning's 4d6). Two kinds of damage only magic does, **force**
and **necrotic**, join the [damage types](damage-types.md).

Spells you cast are yours: what they kill earns you its experience, and a spell of your own that
catches you can kill you.

## Learning spells from scrolls

Every spell comes on a scroll of its own (the scroll of magic missile teaches *Magic missile*);
the scroll of fireball teaches *Fireball*, the scroll of lightning *Lightning bolt*, the scroll of
frost *Frost*. **Reading** a scroll casts its spell once at the scroll's caster level and costs no
mana ([Scrolls](scrolls.md#spell-scrolls)). **Studying** it (`L`, then the scroll) teaches you its
spell for good, and uses the scroll up whatever comes of it.

You can study a scroll once you know what it is (you have read one like it, or identified it)
and only if you do not know its spell yet; an unknown scroll is refused ("You do not know what
this says."), so identify your scrolls first. A scroll of a spell you know can still be read,
sold, or given away. You need some mana to learn at all.

A study takes time, by the spell's tier, at a normal pace:

{{#include generated/study-times.md}}

The world goes on around you. A creature coming into view, a blow, or anything else that would
stop a rest stops the study: the time is lost, but you keep the scroll. Once you begin, you
cannot stop of your own accord.

A spell at or below your caster level's tier (see Tiers above) is learned for certain. Above it,
every tier too high (the **overreach**) makes it harder: a roll of 2d12 plus your bonus must reach
the difficulty, 12 + 4 per tier of overreach, a point more for every 10 tiers of the spells you
already know (a cantrip counting 1). Your bonus is your Intelligence modifier, a tenth of your
*Spellcraft* and of your *Decipher script* (each at most +20), your class's affinity with the
spell's school, and *Quick study*'s +2. A natural 24 always succeeds and a natural 2 always fails.
The game tells you the chance in words: *certain*, *easy*, *a struggle*, or *far beyond you*.

{{#include generated/study-chances.md}}

{{#include generated/study-affinities.md}}

If you fail, the scroll crumbles to dust and a **mishap** befalls you, worse the farther the
spell was beyond you: *slight* at one tier of overreach, *bad* at two or three, *terrible* at
four or more. A few lucky mishaps can come with any of them. A failure costs only that scroll:
the spell may come your way again.

{{#include generated/mishaps.md}}

Studying trains *Spellcraft*, *Decipher script*, and your Intelligence, whatever comes of it.

## Wands

A wand holds one spell and a number of charges. Ready it in the **missile weapon slot** (it takes
a launcher's place, and a launcher takes its place) and use it as you would shoot: `t`, aim, and
it casts its spell. Each cast uses a charge instead of your mana, and the spell is cast at the
**wand's own caster level**, never yours, so a warrior with no magic fires a wand as well as a
wizard does. The spell is otherwise its own: a ray rolls to hit as any spell attack does (with
your Dexterity), magic missile never misses, an area catches whoever is in it, and its reach
grows with your Willpower as your own spells' does. Its damage still stops at the spell's cap.
A cast that cannot go off (aimed at yourself, out of reach) uses no charge and takes no time;
one that goes off takes a cast's time.

An empty wand does nothing ("Nothing happens.") and takes no time; it stays a wand, to sell or
to keep. Wands are **never recharged**.

Wands are found unknown, each kind with a look of its own for the whole game ("an oak wand").
Using one tells you what kind it is, and from then on every wand of that kind shows by its name;
its **charges** stay unknown until you identify that wand (the scroll of identify, or carrying
it long enough as a *Connoisseur*). Wands lie in the dungeon and in general stores.

*Wand mastery* (Weaponcraft) makes every wand you use cast at a caster level 2 higher.

{{#include generated/wands.md}}

The charges a wand is made with go by its spell's tier:

{{#include generated/wand-charges.md}}
