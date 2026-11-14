# Schools and caster styles

Every [spell](magic.md#the-spells) belongs to one of seven **schools of magic**: evocation,
necromancy, abjuration, restoration, transmutation, enchantment, and divination. A school is the
kind of magic a spell is. You do not train a school, and it has no ranks.

A spell's school matters for:

- *[Keen mind](talents.md#talent-keen-mind)* and *[School focus](talents.md#talent-school-focus)*,
  which you take for one school (or one element) and which help only its spells;
- your class's affinity with each school, which makes its spells easier or harder to
  [learn](learning-spells.md#your-learning);
- the colour of a [spell circle](spell-circles.md).

How well you cast depends on your stats, your [talents](talents.md), your items, and the
[caster style](#caster-styles) a spell is cast in.

## The schools

{{#include generated/schools.md}}

## The Magic page

The **Magic page** is page `6` of the [info pages](the-screen.md#info-pages), between the
[weapon skills](weapon-skills.md#seeing-your-skills) and the [quest log](quests.md#the-quest-log).
Open any info page and press `6`, or `Tab` to it. It has no key of its own in play. It is one
column, which scrolls (↑↓) when the window is too small for it. From the top, it shows:

- your [mana](mana.md#how-much-mana-you-have), as the status bar does;
- the [caster styles](#caster-styles), each with your rank and a faint bar of your marks toward the
  next, with a `•` on Battle caster while your armour hinders casting ("• the style you cast in
  now"). They are shown to anyone who knows a spell or has trained a caster style;
- every spell you know, by level then name, with its [rank](learning-spells.md#spell-ranks) and the roll it
  makes now and its mana, as the [abilities menu](abilities.md) shows them ("Magic missile
  rank 1   4d4 force, 4 mana, aimed").

The page is for reading only: spells are cast from the abilities menu. Looking at it takes no time.
A character who knows no spells sees "You know no spells."

## Caster styles

**The spell you cast decides your style.** A caster style is a [fighting style](fighting-styles.md)
of its own:

- A spell that [rolls to hit](spellcasting.md#how-a-spell-finds-its-target) and is a **touch** is
  cast as a **[Toucher](#style-toucher)**.
- A spell that rolls to hit and is anything else (a ray, a bolt) is cast as a
  **[Sniper](#style-sniper)**, wherever you aim it, even at the creature beside you.
- A spell cast in **armour that hinders casting** is a **[Battle caster](#style-battle-caster)**'s,
  whatever the spell.
- Every other spell, out of such armour, is cast in no style: a burst, a cone, a line, a bolt that
  never misses, a heal (a touch heal too), a buff, or a hex. It trains no style, and no style helps
  it.

Every spell you cast with your own [mana](mana.md#how-much-mana-you-have) trains the style it was cast in,
whether it lands or not, just as every blow trains its style. It earns **marks** toward the next
rank: more when it lands, more against a strong foe, and fewer as your rank rises. When you have
enough marks you gain a rank.

A spell **lands** when it does what it is for: harm to someone, a hex that takes hold, a heal of a
hit point or more, a buff on you, a cure, an item identified.

- A spell that lands earns the marks of an attempt and of a success.
- A spell that misses (a ray or a touch that
  [rolls to hit](spellcasting.md#how-a-spell-finds-its-target) and fails), or finds nobody, earns
  only an attempt's marks.

The marks are weighed by the strongest creature the spell struck at. When it struck at nobody (a
buff, a bolt into an empty corridor), they are weighed by your own level. The marks and their
weighing are the same as for [weapon skills](weapon-skills.md#raising-a-weapon-skill), on a slower curve
of their own (the Caster columns there). Nothing limits how often you may train. A spell read from
a [scroll](scrolls.md) or used from a [wand](wands-and-rods.md#wands) trains no style: the magic
is the item's.

- **[Sniper](#style-sniper)**: a ray or a bolt that rolls to hit, out of such armour. It hits more
  often, and better still against a target five cells away or more, where its
  [critical range](combat.md#criticals) is wider too. From rank 5 these spells reach a cell
  farther, and from rank 9 two.
- **[Toucher](#style-toucher)**: a touch that rolls to hit, out of such armour. It hits more often,
  its critical range is wider, and its criticals roll the damage up to twice more. From rank 4 it
  does a point more harm, pressed home; from rank 7 two, and at rank 10 three.
- **[Battle caster](#style-battle-caster)**: any spell cast while you wear armour that makes spells
  cost more (mail, plate, a great helm, a large shield), a buff on yourself as much as a bolt at a
  foe. Its spells that roll to hit hit more often, and it shrinks your armour's extra mana cost for
  **every** spell you cast. From rank 4, a spell that lands wards you with a point of
  [Defence](combat.md#defence) until your next action
  (*[Spell guard](#style-ability-spell-guard)*). A *Mage-wrought* piece eased so far that it makes
  spells cost no more does not count as such armour: in it you cast as a Sniper or a Toucher.

A touch reaches only the creature beside you. Nothing makes it reach farther: not
[School focus](talents.md#talent-school-focus), not a style, not an item.

The [wizard](class-wizard.md) and the [druid](class-druid.md) start in Sniper, the
[witch](class-witch.md) and the [priest](class-priest.md) in Toucher, and the
[paladin](class-paladin.md) in Battle caster. Any caster may advance in any of them by casting
that way.

The [Magic page](#the-magic-page) marks Battle caster as the style you cast in while you wear
such armour. Sniper and Toucher depend on the spell, so it never marks them.

What each rank gives, exactly: *To hit far away* and *Critical range far away* count from five
cells. *Reach* is cells farther (a range). *Every roll* is a point on the roll. *Armour's extra mana cost* is the percent taken off what your armour
adds.

{{#include generated/caster-style-benefits.md}}

What a rank teaches, as with a weapon style ([What a rank teaches](fighting-styles.md#what-a-rank-teaches)).
Only Battle caster teaches an ability:

{{#include generated/caster-style-abilities.md}}

See also: [Magic](magic.md), [Learning spells](learning-spells.md), and [Miscasts](miscasts.md).
