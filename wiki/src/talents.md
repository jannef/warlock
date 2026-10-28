# Talents

A **talent** is a knack your character picks up as they grow: a little more of something the
rules already measure, chosen by you. Talents come in **trees** (Vigour, Might, Finesse, and so
on), some have several **ranks**, and most ask something of you first.

## When you pick

A new character picks **two talents at creation**, on the Talents step after the [class](classes.md); the
**[bard](classes.md#bard)**, the lucky [class](classes.md), picks one more, and so does a **[human](races.md#race-human)** (their racial [Versatile](talents.md#talent-versatile), below):
a [human](races.md#race-human) [bard](classes.md#bard) picks four. The list shows the talents the character you have made so
far may take (its scores, [skills](skills.md), [class abilities](talents.md#class-abilities), and [birth sign](horoscope.md#what-a-sign-does) count), and marking one may
open another that needs it ([Healthy](talents.md#talent-healthy), then [Fast healer](talents.md#talent-fast-healer)). The **Random** row lets chance pick
them, and a character started without picks (from the command line) gets those same random
picks. The [character](character.md) screen counts your talents.

{{#include generated/talents-creation.md}}

Every third **[level](experience.md#levels)** brings a talent: levels 3, 6, 9, and so on, twenty by level 60 (see
[Experience](experience.md)). The pick is made **the moment the level comes**: nothing is banked
for later, and until you have chosen, you can do nothing else ("You must choose a talent
first."). A [human](races.md#race-human)'s [Versatile](talents.md#talent-versatile) brings a second talent at level 30. A kill that brings two [levels](experience.md#levels)
at once brings both picks, lowest level first. There is
no changing your mind afterwards: a talent, once learned, is yours for good.

Some [classes](classes.md#levels) also give a **bonus talent** at a few levels (the everyday
folk at 10, 20, 30, 45, and 55), from the trees the [class](classes.md) names: "You must choose a talent of
Weaponcraft or Delver." When a level brings both kinds, a talent from the [class](classes.md)'s trees fills
the bonus pick first. The [commoner](classes.md#commoner)'s bonus talents come from any tree.

Should there be nothing you can take (a run with no talents, or every talent you qualify for
already learned), the pick is simply lost, so a level never waits for the impossible.

The **[level-up screen](experience.md#what-a-level-gives)** opens by itself when a level comes. It walks the level's choices in
order: the [skill rolls](skills.md) first (each [skill](skills.md) with the dice a roll on it adds), then a bonus talent, then the level's talent, listing
only the talents you may take now, by tree. A talent with a choice ([Weapon focus](talents.md#talent-weapon-focus), [Apprentice](talents.md#talent-apprentice))
asks for it next. The screen can't be closed until everything is chosen. Two levels at once are
one sequence, the lower level's choices first.

The **Talents page** (`G`, or page 7 of the [information pages](controls.md#info-pages)) lists your talents, then every other
talent you could still pick, by tree, with what it needs; the parts you already meet stand out
from those you don't, so you can plan a build there.

**Class abilities** are talents too, of their own tree: your [class](classes.md) gives them as its levels
come (see [Classes](classes.md#levels)), you never pick them, and one your class gives that you
already learned is not given again. Some are talents anyone may pick (the [aristocrat](classes.md#aristocrat)'s
[Riposte](talents.md#talent-riposte), the [commoner](classes.md#commoner)'s [Second wind](talents.md#talent-second-wind)).

## Ranks, choices, prerequisites

- A talent with **ranks** (up to three) is picked once per rank, each pick a level's talent:
  "Hardy (2)". Each rank may ask more than the last.
- A talent with a **choice** (a [weapon skill](weapon-skills.md), a [fighting style](fighting-styles.md), a [skill](skills.md), an element) is picked
  for that choice, and the same talent may be picked again for another: [Weapon focus](talents.md#talent-weapon-focus) with [long
  blades](weapon-skills.md#weapon-skill-long-blades) and Weapon focus with [bows](weapon-skills.md#weapon-skill-bows) are two talents. A [weapon skill](weapon-skills.md) chosen is one you attack with
  (never [shield blocking](weapon-skills.md#weapon-skill-shield-block)); a [fighting style](fighting-styles.md) is chosen as a style; a [skill](skills.md) may be any skill,
  trained or not ([Apprentice](talents.md#talent-apprentice), whose ranks count for each skill on its own).
- **Prerequisites** must all hold when you pick, and are never checked again: a [Strength](stats.md#stat-strength) lowered
  by [disease](diseases.md#how-a-disease-goes) later does not take a talent away. They may be another talent (at a rank), a [stat](stats.md)
  score (the score itself, as the [character screen](character.md) shows it; never what items or
  a sickness do to it), a [skill](skills.md) at a value, a [weapon skill](weapon-skills.md) or
  style at a rank (or any of a kind: "a missile weapon skill 3" is [bows](weapon-skills.md#weapon-skill-bows), [crossbows](weapon-skills.md#weapon-skill-crossbows), or
  [slings](weapon-skills.md#weapon-skill-slings)), a character level, your [race](races.md), your [class](classes.md), your eyes (your race's
  [vision](vision.md)), your [birth sign](horoscope.md#what-a-sign-does)'s animal or element or a lucky day (see
  [Horoscope](horoscope.md)), or your [alignment](alignment.md) band; a few ask for either of two
  ("Intelligence 12 or Appraise 20"). The game names what you lack: "You cannot take Dodge: you
  need [Dexterity](stats.md#stat-dexterity) 13."

## Gifts of the stars

The first tree, **Stars**, holds what your [birth sign](horoscope.md#what-a-sign-does) gives you: the gifts (and an animal's flaw)
of your animal, your element, and a lucky day (see [Horoscope](horoscope.md)). They are yours
from birth, without a pick, and nobody can pick them. A flaw is never something another talent
asks for.

## Heritage: only at character creation

The **Heritage** tree holds what your family gave you. Its talents are offered **only at
character creation** (the Talents step, or a random pick there), never at a level: the Talents
page shows them with "to be a new character", which no later level meets. What they give comes
at once, on top of your [starting kit](starting-kit.md).

- **Family wealth**: three times the gold coins you start with more (your class's purse, and any
  a start of the world adds), at least 60 and at most 150 more, and two or three loose
  [gems](gems.md), as found at item level 2. Coins weigh 10 g each, so the weakest of the small
  peoples (halflings, gnomes, goblins, kobolds, ratfolk) may set out Burdened.
- **Father's sword** (needs Family wealth): your people's heirloom weapon, not always a sword,
  but always a weapon. It is [magic](magic-items.md) with 2 to 4 affixes rolled at item level 6
  (two make it magic, more make it rare and give it a name), and you know what it does from the
  start. It is readied in its own slot (the main hand, the missile weapon slot for a sling or a
  bow) if your kit left that free; otherwise it waits in your pack beside the kit's own weapon.

{{#include generated/talents-heirlooms.md}}

## Racial talents

The **Racial** tree holds what each people is good at. Every talent in it asks for its [race](races.md) (and
some for a score, a level, or a [weapon skill](weapon-skills.md) besides), so only an [elf](races.md#race-elf) may take [Elven accuracy](talents.md#talent-elven-accuracy) and
only a [dwarf](races.md#race-dwarf) [Stone sense](talents.md#talent-stone-sense); they are picked like any other talent. [Humans](races.md#race-human), who have no gift of
their own, are instead given **[Versatile](talents.md#talent-versatile)** from the start: one more talent at creation and one
more at level 30. Like the gifts of the stars it is never on a pick list, and the
[character](character.md) screen's Talents page marks it as your [race](races.md)'s.

## What a talent does

In the game a talent is described in words; the tables below state the rule exactly. Most
talents simply change a rule for good. An **active** talent ([Second wind](talents.md#talent-second-wind)) is something you do:
it appears in the [abilities menu](abilities.md) under Talents, and after use it leaves a
status ([Winded](status-effects.md#status-winded)) during which it cannot be used again; [Blood magic](talents.md#talent-blood-magic) is active too, and has no
waiting time: it costs [hit points](combat.md#hit-points) instead (see [Mana](mana.md#spending-it)). Four talents
work on a [class](classes.md)'s [resources](resources.md#talents-on-resources) and can be taken only by that
class ([Battle trance](talents.md#talent-battle-trance), [Inner calm](talents.md#talent-inner-calm), [Steady hand](talents.md#talent-steady-hand), [Patient blade](talents.md#talent-patient-blade)); one that needs the [lantern's hood](lantern.md#the-hood)
(Hooded light) is still to come. [Apprentice](talents.md#talent-apprentice)'s
[skill](skills.md) points are rolled once, as the rank is taken. [Treasure hunter](talents.md#talent-treasure-hunter) changes loot as it is made: a
level you have already been to keeps what lies there, and the levels themselves never change.
Where a
talent scales ("one hit point per four levels"), what it gave earlier grows as you level: a
talent taken at level 3 still matters at level 50.

{{#include generated/talents.md}}

## Coverage

Every talent names the parts of the game it touches, from a fixed list, and the game checks at
start-up that no part of the game on that list goes without talents: at least two talents each
(and, once [classes](classes.md) have their level tables, at least one [class ability](talents.md#class-abilities)). Parts marked pending
are waiting for theirs.

This is a promise about the future too: **every new part of the game arrives with its
talents**, so no system is ever left with nothing that makes it better at it. That holds for
new [resources](resources.md) as well: [mana](mana.md#how-much) came with talents for a deeper pool, a quicker refill, and cheaper
casting, [spells](magic.md#the-spells) with the Spellcraft tree ([Quick study](talents.md#talent-quick-study), [Overreach](talents.md#talent-overreach), [School focus](talents.md#talent-school-focus)) and talents for
[wands](magic.md#wands) and [rods](magic.md#rods) ([Wand mastery](talents.md#talent-wand-mastery), [Channeler](talents.md#talent-channeler)), and the same goes for each [class](classes.md)'s own pool later (a [barbarian](classes.md#barbarian)'s
[rage](resources.md#rage), a [ranger](classes.md#ranger)'s [focus](resources.md#resource-focus)). [Abilities](abilities.md) that cost such a [resource](resources.md) have no waiting time on top: the
[resource](resources.md) coming back with time is the limit.

{{#include generated/talents-by-system.md}}
