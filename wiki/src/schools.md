# Schools of magic

Every [spell](magic.md#the-spells) belongs to one of seven **schools**: evocation, necromancy,
abjuration, restoration, transmutation, enchantment, divination. Each school is a **skill of
ten ranks**, trained as a [weapon skill](weapon-skills.md) is: every spell you cast with your own
[mana](mana.md#how-much) earns **marks** in its school, more when it lands, more against a strong
foe, fewer as your rank rises. When you have enough marks you gain a rank: "You feel more skilled
at evocation."

A spell **lands** when it does what it is for: a harm on someone, a hex that takes, a heal of a
hit point or more, a buff on you, a cure, an item identified. One that lands earns an attempt's
marks and a success's; one that misses (a ray or a touch that [rolls to
hit](magic.md#how-a-spell-finds-its-target) and fails) or finds nobody earns an attempt's. The
marks are weighed by the strongest creature the spell struck at, or by your own level when it
struck at nobody (a buff, a fireball into an empty corridor). An area spell is one cast: it
earns once, weighed by the strongest creature it caught, however many it caught. The marks and
their weighing are the [weapon skills'](weapon-skills.md#marks-and-ranks), on the missile curve. Nothing
limits how often you may train: a [Light](magic.md#spell-light) cast ten times trains evocation
ten times, for its mana and its time.

A spell read from a [scroll](scrolls.md) or used from a [wand](magic.md#wands) trains nothing:
the magic is the item's.

Each rank is a leg up on every spell of its school, said in one sentence a row of the school's
table below: a die that shows 1 is rolled again; every die a size bigger; a die more; a point on
every roll; a cell more of reach; a degree longer; a little less mana; a quicker cast; and for the
school's spells that roll to hit, a better aim and a wider [critical
range](combat.md#criticals). These work as the spellcraft [talents](talents.md) do
and add to them: a die rolled again by your school and by *[Sure
hand](talents.md#talent-sure-hand)* counts both, and what a hex or a buff gains in degrees is
still held by its [status](status-effects.md)'s most. The game shows your ranks and a bar of your
marks on the [Magic page](#the-magic-page); what a rank gives is here, exactly, and not in the
game.

## The Magic page

The **Magic page** is page `6` of the [info pages](controls.md#info-pages), between the
[weapon skills](weapon-skills.md#seeing-your-skills) and the [quest log](quests.md#the-quest-log):
open any info page and press `6`, or `Tab` to it. It has no key of its own in play. It is one
column that scrolls (↑↓) when the window is too small for it, and shows, from the top: your
[mana](mana.md#how-much), as the status bar does; the seven schools, each with your rank and a
faint bar of your marks toward the next (schools you have trained stand out, the rest are
dimmer); the [caster styles](#caster-styles), a `•` on Battle caster while your armour hinders
casting ("• the style you cast in now"), shown to anyone who knows a spell or has trained a caster
style; and every spell you know, by level then name, with the roll it makes now and its mana, as
the [abilities menu](abilities.md) says them ("Magic missile   2d4 force, 3 mana, aimed"). It is
read only: spells are cast from the abilities menu. It takes no time. A character who knows no
spells reads "You know no spells." under the schools.

## The schools

{{#include generated/school-skills.md}}

A rank in one school carries over to its kin, as weapon skills do: a rank 8 evoker is a rank 4
transmuter untried. Divination stands alone.

## What each rank gives

The harm schools (evocation, necromancy) grow their dice; the mending school (restoration) grows
its heals and quickens them; the lasting schools (abjuration, transmutation, enchantment)
lengthen and reach; divination cheapens and quickens, thin until it has more spells than
[Identify](magic.md#spell-identify), and its table is written for the detections to come. Every
school's spells that roll to hit aim with its *To hit* and *Critical range*, as a weapon's blows
aim with its skill's. *Rolled again* is a die showing that or less rolled once more; *Dice size*
every die that many sides bigger (a d6 a d7); *Dice* that many dice more; *Every roll* a point
on every roll, harm or healing; *Reach* cells farther (a range, a length); *Lasts* degrees more
of a buff's or a hex's status; *Mana* and *Time* percent off the spell's mana (never below 1)
and its casting time.

{{#include generated/school-benefits.md}}

What each rank brings, in a sentence:

- **Evocation**: 1 a surer aim; 2 a surer aim still; 3 a die that shows 1 is rolled again; 4 a
  wider critical range, a twentieth off the mana; 5 every die a size bigger (a d6 a d7); 6 a die
  more; 7 a tenth off the mana; 8 a die showing 1 or 2 is rolled again; 9 a second die more; 10
  every die two sizes bigger. A [Fireball](magic.md#spell-fireball) at the top rung (5d6, 17.5
  on average) rolls 7d8 rerolling 1s and 2s at rank 10: 36.8.
- **Necromancy**: 1 a surer aim; 2 a point more on every roll; 3 1s rolled again, hexes a degree
  longer; 4 a wider critical range, a twentieth off the mana; 5 two points more; 6 hexes two
  degrees longer; 7 a tenth off; 8 a die more, 1s and 2s rolled again, three points more; 9
  hexes three degrees longer; 10 four points more.
- **Restoration**: 1 a surer touch; 2 a point more healed; 3 1s rolled again, a tenth quicker;
  4 every die a size bigger, a twentieth off the mana; 5 a fifth quicker; 6 two points more; 7 a
  die more, a quarter quicker; 8 1s and 2s rolled again; 9 every die two sizes bigger, a third
  quicker; 10 three points more.
- **Abjuration**: 1 a surer aim; 2 its buffs a degree longer, a twentieth off the mana; 3 a
  twentieth quicker; 4 a tenth off; 5 two degrees longer, a tenth quicker; 6 three twentieths
  off; 7 three twentieths quicker; 8 three degrees longer, a fifth off; 9 a fifth quicker; 10 a
  quarter off the mana.
- **Transmutation**: 1 a surer aim; 2 its buffs and hexes a degree longer; 3 a surer aim still;
  4 a cell farther, a twentieth quicker and a twentieth off; 5 two degrees longer; 6 a tenth
  quicker; 7 a tenth off; 8 three degrees longer, two cells farther; 9 three twentieths quicker;
  10 a fifth quicker, three twentieths off.
- **Enchantment**: 1 a surer aim; 2 its hexes a degree longer; 3 a cell farther; 4 a twentieth
  off the mana; 5 two degrees longer; 6 two cells farther; 7 a tenth off; 8 three degrees
  longer; 9 three cells farther; 10 three twentieths off.
- **Divination**: 1 nothing yet but marks; 2 a twentieth off the mana; 3 a tenth off, a tenth
  quicker; 4 a cell farther (for a detection with a range); 5 three twentieths off, a fifth
  quicker; 6 a fifth off; 7 three tenths quicker; 8 a quarter off, two cells farther; 9 three
  tenths off, two fifths quicker; 10 two fifths off, half the time.

## School abilities

As a weapon skill's ranks teach [weapon abilities](weapon-skills.md#what-a-rank-teaches), a school's
teach **school abilities**. Each fires by itself, free and instant, when a spell of the school
that you cast with your own [mana](mana.md) **lands**: once for every creature it harms, every
creature a hex takes on, every creature it heals, and, for a buff, on you. A
[Fireball](magic.md#spell-fireball) that burns three kobolds fires three times, one for each in
turn. A spell that misses, finds nobody, or is read from a [scroll](scrolls.md) or loosed from a
wand fires none, and an ability meant for the one the spell lands on is never laid on you
(Thunderstruck does nothing to the caster of [Light](magic.md#spell-light)). They are listed in
the [abilities menu](abilities.md) under their school, as firing by themselves, and *[Early
mastery](talents.md#talent-early-mastery)* teaches them a rank sooner, as it does a weapon's.
Restoration, transmutation, and divination teach none yet.

{{#include generated/school-abilities.md}}

A school ability and a talent that lays a status on what a spell harms (*[Numbing
cold](talents.md#talent-numbing-cold)*) both work: one is the talent's, the other the rank's.

## Talents

*[Schooled](talents.md#talent-schooled)* (chosen for a school you hold at rank 2) casts that
school's spells as one a rank more practised; *[Devotee](talents.md#talent-devotee)* gives every
school half again the marks; the wizard's *[Magister](talents.md#talent-magister)* (level 45)
raises every school you hold a rank. A rank a talent adds counts only for a school you have
trained.

## Caster styles

**Where you cast from is your style**, a [fighting style](fighting-styles.md) of its own beside
the school. A spell you aim at a creature **beside you**, a touch or anything else, is cast as a
**[Toucher](schools.md#style-toucher)**. A spell you aim **farther** than that, at a creature or
a cell, a ray or a burst alike, is cast as a **[Sniper](schools.md#style-sniper)**; from five
cells away a Sniper is at their best. A spell cast in **armour that hinders casting** is a
**[Battle caster](schools.md#style-battle-caster)**'s, wherever it is aimed. A spell on yourself
out of such armour is cast in no style. Every spell you cast with your own
[mana](mana.md#how-much) trains the style it was cast in, lands or not, as every blow trains its
style: the same marks its school earns, weighed the same way. A spell read from a scroll or used
from a wand trains no style.

- **[Sniper](schools.md#style-sniper)**: a spell aimed farther than beside you, out of such
  armour: a ray, a bolt, a burst, a cone, a line, a hex on a foe across the room, or a cell with
  nobody in it. Its spells that [roll to hit](magic.md#how-a-spell-finds-its-target) hit more
  often, and better still against a target five cells away or more, where its [critical
  range](combat.md#criticals) is wider too. From rank 5 every spell cast so reaches a cell
  farther, from rank 9 two. From rank 4 its best shots pin the target where it stands
  (*[Pinning ray](schools.md#style-ability-pinning-ray)*).
- **[Toucher](schools.md#style-toucher)**: a spell aimed at a creature beside you, out of such
  armour: a touch, a heal laid on the one next to you, or a ray or a hex at them. Its spells
  that roll to hit hit more often, its critical range is wider, and its criticals roll the
  damage up to twice more. From rank 4 every spell cast so does a point more, harm or healing,
  pressed home; from rank 7 two, at rank 10 three. From rank 4 its criticals
  [stagger](status-effects.md#status-staggered) (*[Staggering
  touch](schools.md#style-ability-staggering-touch)*); at rank 10 every spell cast so that
  rolls to hit and hits [weakens](status-effects.md#status-weakened) the foe (*[Withering
  touch](schools.md#style-ability-withering-touch)*).
- **[Battle caster](schools.md#style-battle-caster)**: any spell cast while you wear armour that
  makes spells dearer (mail, plate, a great helm, a large shield), a buff on yourself as much as
  a bolt at a foe. Its spells that roll to hit hit more often, and it makes your armour's extra
  mana cost smaller for **every** spell you cast. From rank 4 a spell that lands wards you: a
  point of [Defence](combat.md#defence) until your next action (*[Spell
  guard](schools.md#style-ability-spell-guard)*). A *Mage-wrought* piece eased so far that it
  makes spells no dearer is not such armour: in it you cast as a Sniper or a Toucher.

The [wizard](classes.md#wizard) and the [druid](classes.md#druid) start in Sniper, the
[witch](classes.md#witch) and the [priest](classes.md#priest) in Toucher, the
[paladin](classes.md#paladin) in Battle caster; any caster may advance in any of them by casting
from there. Sniper and Toucher default to each other at 4 ranks less, Toucher and Battle caster
at 3 ([Related styles](fighting-styles.md#related-styles)). The [Magic page](#the-magic-page) marks Battle caster as the
style you cast in while you wear such armour; Sniper and Toucher are decided by each cast's
target, so it never marks them.

What each rank gives, exactly: *To hit far away* and *Critical range far away* count from five
cells, *Reach* is cells farther (a range, a length) after the school's, *Every roll* a point on
every roll after the school's, and *Armour's extra mana cost* percent off what your armour adds.

{{#include generated/caster-style-benefits.md}}

What a rank teaches, as a weapon style's does ([What a rank
teaches](fighting-styles.md#what-a-rank-teaches)):

{{#include generated/caster-style-abilities.md}}
