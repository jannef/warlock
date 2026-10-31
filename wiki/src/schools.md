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
range](combat.md#criticals). These work as the [Spellcraft talents](magic.md#spellcraft-talents)
do and add to them: a die rolled again by your school and by *[Sure
hand](talents.md#talent-sure-hand)* counts both, and what a hex or a buff gains in degrees is
still held by its [status](status-effects.md)'s most. The game shows your ranks and a bar of your
marks (the `schools` list); what a rank gives is here, exactly, and not in the game.

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

## Talents

*[Schooled](talents.md#talent-schooled)* (chosen for a school you hold at rank 2) casts that
school's spells as one a rank more practised; *[Devotee](talents.md#talent-devotee)* gives every
school half again the marks; the wizard's *[Magister](talents.md#talent-magister)* (level 45)
raises every school you hold a rank. A rank a talent adds counts only for a school you have
trained.

## Caster styles

Where and how you cast is a [fighting style](fighting-styles.md#caster-styles) of its own
(Sniper, Toucher, Battle caster), trained beside the school by the same cast and added to its
aim.
