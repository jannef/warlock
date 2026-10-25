# Talents

A **talent** is a knack your character picks up as they grow: a little more of something the
rules already measure, chosen by you. Talents come in **trees** (Vigour, Might, Finesse, and so
on), some have several **ranks**, and most ask something of you first.

## When you pick

A new character picks **two talents at creation**, on the Talents step after the class; the
**bard**, the lucky class, picks three. The list shows the talents the character you have made so
far may take (its scores, skills, class abilities, and birth sign count), and marking one may
open another that needs it (Healthy, then Fast healer). The **Random** row lets chance pick
them, and a character started without picks (from the command line) gets those same random
picks. The [character](character.md) screen counts your talents.

{{#include generated/talents-creation.md}}

Every third **level** brings a talent: levels 3, 6, 9, and so on, twenty by level 60 (see
[Experience](experience.md)). The pick is made **the moment the level comes**: nothing is banked
for later, and until you have chosen, you can do nothing else ("You must choose a talent
first."). A kill that brings two levels at once brings both picks, lowest level first. There is
no changing your mind afterwards: a talent, once learned, is yours for good.

Some [classes](classes.md#levels) also give a **bonus talent** at a few levels (the everyday
folk at 10, 20, 30, 45, and 55), from the trees the class names: "You must choose a talent of
Weaponcraft or Delver." When a level brings both kinds, a talent from the class's trees fills
the bonus pick first. The commoner's bonus talents come from any tree.

Should there be nothing you can take (a run with no talents, or every talent you qualify for
already learned), the pick is simply lost, so a level never waits for the impossible.

The **level-up screen** opens by itself when a level comes. It walks the level's choices in
order: the [skill rolls](skills.md) first, then a bonus talent, then the level's talent, listing
only the talents you may take now, by tree. A talent with a choice (Weapon focus, Apprentice)
asks for it next. The screen can't be closed until everything is chosen. Two levels at once are
one sequence, the lower level's choices first.

The **Talents page** (page 7 of the information pages) lists your talents, then every other
talent you could still pick, by tree, with what it needs; the parts you already meet stand out
from those you don't, so you can plan a build there.

**Class abilities** are talents too, of their own tree: your class gives them as its levels
come (see [Classes](classes.md#levels)), you never pick them, and one your class gives that you
already learned is not given again. Some are talents anyone may pick (the aristocrat's
Riposte, the commoner's Second wind).

## Ranks, choices, prerequisites

- A talent with **ranks** (up to three) is picked once per rank, each pick a level's talent:
  "Hardy (2)". Each rank may ask more than the last.
- A talent with a **choice** (a weapon skill, a fighting style, a skill, an element) is picked
  for that choice, and the same talent may be picked again for another: Weapon focus with long
  blades and Weapon focus with bows are two talents. A weapon skill chosen is one you attack with
  (never shield blocking); a fighting style is chosen as a style; a skill may be any skill,
  trained or not (Apprentice, whose ranks count for each skill on its own).
- **Prerequisites** must all hold when you pick, and are never checked again: a Strength lowered
  by disease later does not take a talent away. They may be another talent (at a rank), a stat
  score (the score itself, as the [character screen](character.md) shows it; never what items or
  a sickness do to it), a [skill](skills.md) at a value, a [weapon skill](weapon-skills.md) or
  style at a rank (or any of a kind: "a missile weapon skill 3" is bows, crossbows, or
  slings), a character level, your race, your class, your eyes (your race's
  [vision](vision.md)), your birth sign's animal or element or a lucky day (see
  [Horoscope](horoscope.md)), or your [alignment](alignment.md) band; a few ask for either of two
  ("Intelligence 12 or Appraise 20"). The game names what you lack: "You cannot take Dodge: you
  need Dexterity 13."

## Gifts of the stars

The first tree, **Stars**, holds what your birth sign gives you: the gifts (and an animal's flaw)
of your animal, your element, and a lucky day (see [Horoscope](horoscope.md)). They are yours
from birth, without a pick, and nobody can pick them. A flaw is never something another talent
asks for.

## What a talent does

In the game a talent is described in words; the tables below state the rule exactly. Most
talents simply change a rule for good. An **active** talent (Second wind) is something you do:
it appears in the [abilities menu](abilities.md) under Talents, and after use it leaves a
status (Winded) during which it cannot be used again; Blood magic is active too, and has no
waiting time: it costs hit points instead (see [Mana](mana.md#spending-it)). A few talents that
need the class resources (Battle trance, Inner calm, Patient blade) or the lantern's hood
(Hooded light) are still to come. Apprentice's
skill points are rolled once, as the rank is taken. Treasure hunter changes loot as it is made: a
level you have already been to keeps what lies there, and the levels themselves never change.
Where a
talent scales ("one hit point per four levels"), what it gave earlier grows as you level: a
talent taken at level 3 still matters at level 50.

{{#include generated/talents.md}}

## Coverage

Every talent names the parts of the game it touches, from a fixed list, and the game checks at
start-up that no part of the game on that list goes without talents: at least two talents each
(and, once classes have their level tables, at least one class ability). Parts marked pending
are waiting for theirs.

This is a promise about the future too: **every new part of the game arrives with its
talents**, so no system is ever left with nothing that makes it better at it. That holds for
new resources as well: mana came with talents for a deeper pool, a quicker refill, and cheaper
casting, and the same goes for each class's own pool later (a barbarian's
rage, a ranger's focus). Abilities that cost such a resource have no waiting time on top: the
resource coming back with time is the limit.

{{#include generated/talents-by-system.md}}
