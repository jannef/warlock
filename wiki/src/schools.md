# Schools of magic

Every [spell](magic.md#the-spells) belongs to one of seven **schools**: evocation, necromancy,
abjuration, restoration, transmutation, enchantment, and divination. Each school is a **skill with
ten ranks**, trained the way a [weapon skill](weapon-skills.md) is.

Every spell you cast with your own [mana](mana.md#how-much) earns **marks** in its school. It earns
more when it lands, more against a strong foe, and fewer as your rank rises. When you have enough
marks you gain a rank: "You feel more skilled at evocation."

A spell **lands** when it does what it is for: harm to someone, a hex that takes hold, a heal of a
hit point or more, a buff on you, a cure, an item identified.

- A spell that lands earns the marks of an attempt and of a success.
- A spell that misses (a ray or a touch that
  [rolls to hit](spellcasting.md#how-a-spell-finds-its-target) and fails), or finds nobody, earns
  only an attempt's marks.

The marks are weighed by the strongest creature the spell struck at. When it struck at nobody (a
buff, a fireball into an empty corridor), they are weighed by your own level. An area spell counts
as one cast: it earns once, weighed by the strongest creature it caught, however many it caught.
The marks and their weighing are the same as for
[weapon skills](weapon-skills.md#marks-and-ranks), on a gentler curve of their own (the School
columns there).

Nothing limits how often you may train. A [Light](magic.md#spell-light) cast ten times trains
evocation ten times, for its mana and its time. A spell read from a [scroll](scrolls.md) or used
from a [wand](wands-and-rods.md#wands) trains nothing: the magic is the item's.

Each rank improves every spell of its school, in one of these ways:

- a die that shows 1 is rolled again;
- every die is a size bigger;
- one die more;
- a point more on every roll;
- a cell more of reach;
- a degree longer;
- a little less mana;
- a quicker cast;
- for the school's spells that roll to hit, a better aim and a wider
  [critical range](combat.md#criticals).

These work like the spellcraft [talents](talents.md) and add to them. A die rolled again by your
school and by *[Sure hand](talents.md#talent-sure-hand)* counts both. What a hex or a buff gains in
degrees is still capped by its [status](status-effects.md)'s maximum.

The game shows your ranks and a bar of your marks on the [Magic page](#the-magic-page). What each
rank gives is stated exactly here, not in the game.

## The Magic page

The **Magic page** is page `6` of the [info pages](the-screen.md#info-pages), between the
[weapon skills](weapon-skills.md#seeing-your-skills) and the [quest log](quests.md#the-quest-log).
Open any info page and press `6`, or `Tab` to it. It has no key of its own in play. It is one
column, which scrolls (↑↓) when the window is too small for it. From the top, it shows:

- your [mana](mana.md#how-much), as the status bar does;
- the seven schools, each with your rank and a faint bar of your marks toward the next. Schools you
  have trained stand out; the rest are dimmer;
- the [caster styles](#caster-styles), with a `•` on Battle caster while your armour hinders
  casting ("• the style you cast in now"). They are shown to anyone who knows a spell or has
  trained a caster style;
- every spell you know, by level then name, with the roll it makes now and its mana, as the
  [abilities menu](abilities.md) shows them ("Magic missile   4d4 force, 3 mana, aimed").

The page is for reading only: spells are cast from the abilities menu. Looking at it takes no time.
A character who knows no spells sees "You know no spells." under the schools.

## The schools

{{#include generated/school-skills.md}}

A rank in one school carries over to its related schools, as with weapon skills: a rank 8 evoker
is a rank 4 transmuter without ever having tried. Divination stands alone.

## What each rank gives

Each school grows in its own way:

- The harm schools (evocation, necromancy) grow their dice.
- The mending school (restoration) grows its heals and makes them quicker.
- The lasting schools (abjuration, transmutation, enchantment) make their effects last longer and
  reach farther.
- Divination makes its spells cheaper and quicker. It gives little until it has more spells than
  [Identify](magic.md#spell-identify); its table is already written for the detection spells to
  come.

Every school's spells that roll to hit aim with its *To hit* and *Critical range*, as a weapon's
blows aim with its skill's. The columns of the table mean:

- *Rolled again*: a die showing that number or less is rolled once more.
- *Dice size*: every die is that many sides bigger (a d6 becomes a d7).
- *Dice*: that many dice more.
- *Every roll*: a point more on every roll, harm or healing.
- *Reach*: that many cells farther (a range, a length).
- *Lasts*: that many degrees more of a buff's or a hex's status.
- *Mana* and *Time*: that percent off the spell's mana (never below 1) and off its casting time.

{{#include generated/school-benefits.md}}

What each rank brings, in a sentence:

- **Evocation**: 1 a surer aim; 2 nothing new; 3 a surer aim still, and a die that shows 1 is
  rolled again; 4 a wider critical range, a twentieth off the mana; 5 a surer aim, every die a
  size bigger (a d6 a d7); 6 a die more; 7 a surer aim, a wider critical range, a tenth off the
  mana; 8 a die showing 1 or 2 is rolled again; 9 a surer aim, a second die more; 10 the surest
  aim, the widest critical range, every die two sizes bigger, three twentieths off the mana. For
  example, a [Fireball](magic.md#spell-fireball) at the top rung (10d6, 35 on average) rolls 12d8,
  rerolling 1s and 2s, at rank 10: 63.0 on average.
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

Just as a weapon skill's ranks teach [weapon abilities](weapon-skills.md#what-a-rank-teaches), a
school's ranks teach **school abilities**. Each one works by itself, free and instant, when a spell
of its school that you cast with your own [mana](mana.md) **lands**. It works once for every
creature the spell harms, every creature a hex takes hold of, and every creature it heals, and for
a buff, on you. A [Fireball](magic.md#spell-fireball) that burns three kobolds sets it off three
times, once for each in turn.

A spell that misses, finds nobody, or is read from a [scroll](scrolls.md) or loosed from a wand
sets off none. An ability meant for the one the spell lands on is never laid on you: Thunderstruck
does nothing to the caster of [Light](magic.md#spell-light).

School abilities are listed in the [abilities menu](abilities.md) under their school, as working
by themselves. *[Early mastery](talents.md#talent-early-mastery)* teaches them a rank sooner, as it
does a weapon's. Restoration, transmutation, and divination teach none yet.

{{#include generated/school-abilities.md}}

A school ability and the [rule of a spell's type](spellcasting.md#spell-types) both apply: one comes
from your rank, the other from the spell's type.

## Talents

- *[Schooled](talents.md#talent-schooled)* (chosen for a school you hold at rank 2) casts that
  school's spells as if you were a rank more practised.
- *[Devotee](talents.md#talent-devotee)* gives every school half again the marks.
- The wizard's *[Magister](talents.md#talent-magister)* (level 45) raises every school you hold by
  a rank.

A rank a talent adds counts only for a school you have trained.

## Caster styles

**Where you cast from is your style.** A caster style is a [fighting style](fighting-styles.md) of
its own, alongside the school:

- A spell you aim at a creature **beside you**, a touch or anything else, is cast as a
  **[Toucher](#style-toucher)**.
- A spell you aim **farther** than that, at a creature or a cell, a ray or a burst alike, is cast
  as a **[Sniper](#style-sniper)**. A Sniper is at their best from five cells away.
- A spell cast in **armour that hinders casting** is a **[Battle caster](#style-battle-caster)**'s,
  wherever it is aimed.
- A spell on yourself, out of such armour, is cast in no style.

Every spell you cast with your own [mana](mana.md#how-much) trains the style it was cast in,
whether it lands or not, just as every blow trains its style. It earns the same marks its school
earns, weighed the same way. A spell read from a scroll or used from a wand trains no style.

- **[Sniper](#style-sniper)**: a spell aimed farther than beside you, out of such armour. That is
  a ray, a bolt, a burst, a cone, a line, a hex on a foe across the room, or a cell with nobody in
  it. Its spells that [roll to hit](spellcasting.md#how-a-spell-finds-its-target) hit more often,
  and better still against a target five cells away or more, where its
  [critical range](combat.md#criticals) is wider too. From rank 5 every spell cast this way reaches
  a cell farther, and from rank 9 two. From rank 4 its best shots pin the target where it stands
  (*[Pinning ray](#style-ability-pinning-ray)*).
- **[Toucher](#style-toucher)**: a spell aimed at a creature beside you, out of such armour. That is
  a touch, a heal laid on the one next to you, or a ray or a hex at them. Its spells that roll to
  hit hit more often, its critical range is wider, and its criticals roll the damage up to twice
  more. From rank 4 every spell cast this way does a point more, harm or healing, pressed home;
  from rank 7 two, and at rank 10 three. From rank 4 its criticals
  [stagger](status-effects.md#status-staggered)
  (*[Staggering touch](#style-ability-staggering-touch)*). At rank 10, every spell cast this way
  that rolls to hit and hits [weakens](status-effects.md#status-weakened) the foe
  (*[Withering touch](#style-ability-withering-touch)*).
- **[Battle caster](#style-battle-caster)**: any spell cast while you wear armour that makes spells
  cost more (mail, plate, a great helm, a large shield), a buff on yourself as much as a bolt at a
  foe. Its spells that roll to hit hit more often, and it shrinks your armour's extra mana cost for
  **every** spell you cast. From rank 4, a spell that lands wards you with a point of
  [Defence](combat.md#defence) until your next action
  (*[Spell guard](#style-ability-spell-guard)*). A *Mage-wrought* piece eased so far that it makes
  spells cost no more does not count as such armour: in it you cast as a Sniper or a Toucher.

The [wizard](classes.md#wizard) and the [druid](classes.md#druid) start in Sniper, the
[witch](classes.md#witch) and the [priest](classes.md#priest) in Toucher, and the
[paladin](classes.md#paladin) in Battle caster. Any caster may advance in any of them by casting
that way.

Sniper and Toucher default to each other at 4 ranks less, and Toucher and Battle caster at 3 (see
[Related styles](fighting-styles.md#related-styles)). The [Magic page](#the-magic-page) marks
Battle caster as the style you cast in while you wear such armour. Sniper and Toucher depend on each
cast's target, so it never marks them.

What each rank gives, exactly: *To hit far away* and *Critical range far away* count from five
cells. *Reach* is cells farther (a range, a length), on top of the school's. *Every roll* is a point
on every roll, on top of the school's. *Armour's extra mana cost* is the percent taken off what
your armour adds.

{{#include generated/caster-style-benefits.md}}

What a rank teaches, as with a weapon style ([What a rank teaches](fighting-styles.md#what-a-rank-teaches)):

{{#include generated/caster-style-abilities.md}}
