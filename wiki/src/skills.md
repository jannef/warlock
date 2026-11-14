# Skills

Skills are what you have trained in: moving quietly, tumbling out of harm's way, finding your way
through the wild. Your [race](races.md) and your [class](classes.md) train you in some of them when your character is made.
You are **untrained** in the rest. Some skills also let you do something new, such as stealing or
treating a wound; those uses are explained on [Using your skills](skill-uses.md).

- An untrained skill does not exist for you. It is not shown anywhere, and it gives you nothing.
  That is not the same as a skill at 0: a skill you are trained in at 0 is shown, and it can grow.
- A trained skill has a **value**, usually between 5 and 30 when you start. 100 is a master. Past
  100 a skill still grows, but slowly.
- `A` shows your **[Skills](the-screen.md#skills)** screen. It lists every skill you are trained in, with its value and a
  small meter (`━━──`) of how practised it is, which is how big its next roll will
  be (see [Improving a skill](#improving-a-skill)). Use the arrow keys to read what each skill is about. Looking takes no
  time. `A` on the [character screen](character.md) (`@`) opens it too, and the summary at the end of character
  creation lists the skills you will start with.

The game never shows in numbers what a skill does for you. You see the value and feel the
difference. This page gives the rules, with the exact numbers.

## The skills

Skills come in three sorts:

- **Passive** skills work by themselves, all the time.
- Some are used as **actions**: [searching](searching.md#how-to-search), [picking a lock](doors.md#picking-locks), [binding a wound](skill-uses.md#treating-yourself).
- Some are **checked by rules** when something calls for them, such as [athletics](skills.md#skill-athletics) when you [kick a door](doors.md#kicking-a-door-open).

Skills used as actions appear in the [abilities menu](abilities.md) once you are trained in them. Their
names there:

- [Disable device](skills.md#skill-disable-device): **[Disarm a trap](abilities.md#ability-disarm)**
- [Open lock](skills.md#skill-open-lock): **[Pick lock](abilities.md#ability-pick-lock)**
- [Steal](skills.md#skill-steal): **[Steal](abilities.md#ability-steal)**
- [Bluff](skills.md#skill-bluff): **[Feint](abilities.md#ability-feint)**
- [Heal](skills.md#skill-heal): **[Heal](abilities.md#ability-heal)**
- [Decipher script](skills.md#skill-decipher-script): **[Decipher](abilities.md#ability-decipher)** (see [Deciphering a scroll unread](scrolls.md#deciphering-a-scroll-unread))
- [Survival](skills.md#skill-survival), though it is a passive skill: **[Forage](abilities.md#ability-forage)**
- [Jewelcraft](skills.md#skill-jewelcraft): **[Jewelcraft…](abilities.md#ability-jewelcraft)**, its [profession](skill-uses.md#professions)'s recipes (Cut and Set, see [Gem cutting](gem-cutting.md); rings and amulets, see [Making jewellery](jewellery.md))
- [Metalcraft](skills.md#skill-metalcraft): **[Metalcraft…](abilities.md#ability-metalcraft)**, its profession's recipes (Temper among them)
- [Woodcraft](skills.md#skill-woodcraft): **[Woodcraft…](abilities.md#ability-woodcraft)**, its profession's recipes (arrows, bolts, sling stones, and Whittle among them), and **[Wedge a door](abilities.md#ability-wedge)** for those
  trained in it (see [Wedging a door](doors.md#wedging-a-door))

{{#include generated/skills.md}}

Benefits marked **trained only** are given only to those trained in the skill. If you are
untrained you cannot try, and a magic item that raises the skill (see [Magic items](magic-items.md)) does not
change that. Everything else a skill gives, a magic item gives even to the untrained, as if the skill
were trained at the item's bonus.

## A skill at work

A passive skill helps more the higher it is, and a skill used as an action works more often.
What each one gives, skill by skill, with its exact numbers from 0 to 200, is on
[What skills give](skill-benefits.md). Here is one of them at work.

> **Example: a thief disarms an arrow trap.** A thief with Dexterity 14 and
> [Disable device](skills.md#skill-disable-device) 25 tries to
> [disarm](traps.md#disarming-traps) an [arrow trap](traps.md#trap-arrow) of difficulty 14 (it
> is harder on deeper dungeon floors).
>
> - **What she adds:** +2 for Dexterity 14 ([modifiers](stats.md#modifiers)), and +2 from
>   Disable device: its points grow evenly from +0 at 0 to +8 at 100, so 25 gives
>   8 × 25 ÷ 100 = 2. In all, **+4**.
> - **What the dice must reach:** 14 − 4 = **10**: 108 of the 144 rolls, **75%**.
> - **Setting it off:** falling short by 5 or more (a total of 9 or less, so dice of 6 or less)
>   sets the trap off at her: 15 rolls in 144, **10.4%**.
> - At Disable device 100 she adds +10 and the dice need 4: only 3 rolls in 144 fail (2.1%), and
>   none sets the trap off.

## Which skills you start with

Each [race](races.md) trains one or two skills. Each [class](classes.md) trains a handful: a few of them well (`2d6+10`), the
rest a little (`2d4+4`). A skill both your race and your class train adds the two. [Humans](races.md#race-human) train no skill of
their own, but they put extra work into one of their class's skills, picked at random. Every
[commoner](class-commoner.md) learned one trade growing up: one of the [professions](crafting.md), picked at
random, which they always start with. The dice are
rolled when your character is made, after your [stats](character.md#how-your-stats-add-up), so they never change a [stat](stats.md).

Each class also has a few skills its members **may** start with. Each has the chance in the last
column, then its dice, rolled once when your character is made. So two characters of the same
class can begin with different skills. The game shows only the skills you have, never the chances.

Your race's skill and the human's extra class skill add to one of these as to any other. But a
level-up roll on one of them is an ordinary roll: the class's +1 is only for the skills in its Trains
column, and the same goes for the commoner's trade. A race may have such a skill too: a [drow](races.md#race-drow) may start with [Sneak attack](skills.md#skill-sneak-attack), whatever their class
(rolled after the class's own).

{{#include generated/skill-grants.md}}

A skill you never started with can still be learnt for gold, from a teacher
([Training](npcs.md#services)): [Alberich](npcs.md#alberich) in Fyrdelf teaches
[Mining](skills.md#skill-mining), and [Eadric the smith](npcs.md#eadric-the-smith) in Cēaptūn
[Metalcraft](skills.md#skill-metalcraft).

Some [creatures](creatures.md) are trained in skills too: deer listen well, and the [kobold shaman](creatures.md#creature-kobold-shaman) knows something of
[spells](magic.md#the-spells). You never see a creature's skills in the game. They are on
[Creatures in numbers](creature-numbers.md#stats-and-skills).

## Improving a skill

In short: **using a skill builds up practice, and each new level turns that practice into points.**

**Training marks.** Each time you use a skill in a way that matters, it may earn a **training
mark**. The chance is in the table of skills above. Skills you use all the time, such as
[balance](skills.md#skill-balance), earn marks rarely, so they do not race ahead.

- **[Stealth](skills.md#skill-stealth)**: a step that a [creature](creatures.md) nearby could have heard if you had not stepped softly.
- **[Listen](skills.md#skill-listen)**: [hearing](hearing.md) something new, and every turn spent [listening at a door](hearing.md#listening-at-a-door).
- **[Tumble](skills.md#skill-tumble)**: dodging an attack ("You dodge the kobold's spear."), or stepping past a [trap](traps.md#finding-traps) you know
  of ("You deftly avoid the arrow trap."). A blow that misses you for no reason your [Defence](combat.md#defence)
  explains (it would have missed anyone) counts as a dodge one time in four.
- **Balance**: doing anything but waiting while your load slows you down (5 %), a step onto rough
  ground that slows you (10 %: bush, reeds, rubble, ice; not water), and an action taken while
  [staggered](status-list.md#status-staggered) (20 %).
- **[Survival](skills.md#skill-survival)**: a journey on the [overworld](overworld.md#travel).
- **[Herbalism](skills.md#skill-herbalism)**: a forage that runs its time, one time in four, and every
  [gather](herbs.md#gathering) that runs its time.
- **[Mining](skills.md#skill-mining)**: every wall you dig through.
- **[Swim](skills.md#skill-swim)**: a step wading through [water](terrain.md#water) or [marsh](terrain.md#terrain-marsh) now and then (10 %), every [plunge through the ice](temperature.md#falling-in), and a
  quarter of the steps in [deep water](terrain.md#deep-water), made or not.
- **[Steal](skills.md#skill-steal)**: every attempt to steal, whatever comes of it.
- **[Bluff](skills.md#skill-bluff)**: half of your feints, and every try at [talking your way past](encounters.md#talking-past) a party, whatever comes of
  them.
- **[Open lock](skills.md#skill-open-lock)**: a try at picking a lock, whether it works or not.
- **[Heal](skills.md#skill-heal)**: a try at treating yourself, whether it works or not.

Only skills you are trained in earn marks. They gather until a roll on the skill spends them. The
Skills screen's meter shows how practised a skill is (the step its marks have reached on the ladder
below), never the marks.

**Rolls of their own.** When a skill's marks reach 100, it gets a roll at once, by itself, without
waiting for your next level. It is the same roll as a level-up's (below), and it is told the same
way: "Your stealth improves." It spends half the marks, so the skill drops back below 100 and must
earn its next one. It comes on top of your level's rolls, never instead of one, and it does not
wait for a level-up you have not finished.

**Skill rolls.** Every level gives you a number of **skill rolls** (see [Experience](experience.md)). You spend each
one on a trained skill of your choice, before you do anything else. Nothing stops you putting every
roll into one skill.

A roll throws the die the skill's marks have reached on the ladder, adds it to the skill, and
spends half of the skill's marks (rounded up). So a second roll on the same skill that level is a
step or two smaller (50 marks → 25 → 12 → 6 → 3: `3d3`, `2d4`, `1d5`, `1d3`, `+1`). A skill with no
marks still gains 1. Past 100, a roll adds half its result, rounded up. The game says "Your stealth
improves."

- A skill your **[class](classes.md) is known for** (its `2d6+10` skills) throws the die one step up the ladder. So does a **[trade](talents.md#crafts-a-craftspersons-trades)** of yours; a skill that is both still goes up one step, not two.
- Any **other skill your class trains**, or a skill **your [race](races.md) trains**, adds 1 to the roll.

Both stack: a [halfling](races.md#race-halfling) [thief](class-thief.md)'s stealth is both a class specialty and racial, so it rolls one step up and +1.

The level-up screen shows these numbers before you choose. Beside each skill it shows the dice a
roll on it adds (`1d3`, `+2`, `1d5+1`; `(1d3)/2` past 100). For the selected skill it explains how
they came about (`Rolls 1d5+1: practice (20 marks) gives 1d5, +1 as your class trains it.`), with a
line saying where your rolls come from (`4 skill rolls a level: class 3, race 1`). The roll itself
is thrown only when you choose.

{{#include generated/skill-ladder.md}}

How many rolls a level gives: your class's, plus your race's, plus your [Intelligence](stats.md#stat-intelligence)'s.

{{#include generated/skill-rolls.md}}

Only you train, for now. Creatures keep the skills they have. Your skills and their marks are saved
with your run.
