# Skills

Skills are what you have trained in: moving quietly, tumbling out of harm's way, finding your way
through the wild. Your [race](races.md) and your [class](classes.md) train you in some of them
when your character is made; the rest you are **untrained** in.

- An untrained skill does not exist for you. It is not shown anywhere, and it gives you nothing.
  That is not the same as a skill at 0: a skill you are trained in at 0 is shown, and it can
  grow.
- A trained skill has a **value**, usually between 5 and 30 when you start. 100 is a
  master. Past 100 a skill still grows, but slowly.
- `A` shows your **Skills** screen: every skill you are trained in, with its value, and beside it
  a small meter (`━━──`) of how practised it is: how big its next roll at a level-up will be
  (see [Getting better](#getting-better)). Use the arrow keys to read what each
  skill is about. It takes no time. `A` on the [character screen](character.md) (`@`) opens it
  too, and the summary at the end of character creation lists the skills you will start with.

What a skill does for you is never shown in numbers in the game: you see the value, and feel the
difference. This page gives the rules, with the exact numbers.

## The skills

Some skills work by themselves, all the time (**passive**); some will be used as **actions**
(searching, picking a lock, binding a wound); some are **read by rules** when something calls for
them (athletics when you kick a door). For now only the passive skills, Appraise, Athletics
(kicking doors in, see [Doors](doors.md)), Search (see [Searching](searching.md)), Disable
device (disarming traps, see [Traps](traps.md#disarming-traps)), Open lock (picking locks,
see [Doors](doors.md#picking-locks)), and Steal (see [Stealing](#stealing) below) do something:
the others are there to be trained, and get
their uses as the game grows. The skills used as actions are in the [abilities
menu](abilities.md) once you are trained in them, greyed out until their uses come; Disable
device's use is there as **Disarm a trap**, Open lock's as **Pick lock**, Steal's as **Steal**.

{{#include generated/skills.md}}

## What skills give

The passive skills help you more the higher they are:

- **Tumble** adds to your Defence in the **Dodge** kind (see [Combat](combat.md#defence)), the one
  kind whose bonuses add up.
- **Stealth** makes your footsteps and the doors you open and close quieter (see
  [Hearing](hearing.md)). Your attacks are as loud as ever.
- **Listen** lets you hear fainter sounds: your hearing threshold drops by the number shown (it
  never drops below 1).
- **Balance** takes some of the slowdown of a heavy load away (see [Burden](burden.md)): a
  burdened master is slowed by a tenth, not a quarter.
- **Survival** makes your journeys on the [overworld](overworld.md) quicker. It does nothing in a
  local area or a dungeon.
- **Appraise** gets you better prices in [shops](shops.md): you pay less and are offered more.
- **Search** helps you find secret doors and hidden traps when you search (`s`, `S`), and a
  master's search reaches a cell farther (see [Searching](searching.md)).
- **Spot** lets you notice hidden things in passing at all, and the higher it is, the more you
  notice.
- **Open lock** lets you pick locks at all (the Pick lock ability), and the higher it is, the
  more often a try works (see [Doors](doors.md#picking-locks)).
- **Steal** lets you steal at all, and the higher it is, the likelier your hand goes unnoticed
  (see [Stealing](#stealing)).

{{#include generated/skill-benefits.md}}

## Who trains what

Each race trains one or two skills; each class a handful, a few of them well (`2d6+10`) and the
rest a little (`2d4+4`). A skill your race and your class both train adds the two. Humans train
none of their own, but they put extra work into one of their class's skills, picked at random.
The dice are rolled when your character is made, after your [stats](character.md#how-your-stats-add-up),
so they never change a stat.

{{#include generated/skill-grants.md}}

Some creatures are trained in skills too: deer listen well, and the kobold shaman knows something
of spells. You never see a creature's skills in the game; they are on
[Creatures in numbers](creature-numbers.md#stats-and-skills).

## Getting better

Skills grow the way ADOM does it: **use charges a skill, and each new level cashes it in.**

**Training marks.** Each time you use a skill in a way that matters, it may earn a **training
mark**. The chance is shown in the table of skills above; skills you use all the time, such as
balance, earn marks rarely, so they do not race ahead.

- **Stealth**: a step that a creature nearby could have heard, if you had not stepped softly.
- **Listen**: hearing something new.
- **Tumble**: dodging an attack ("You dodge the kobold's spear.") or stepping past a trap you
  know of ("You deftly avoid the arrow trap.").
- **Balance**: doing anything but waiting while your load slows you down.
- **Survival**: a journey on the overworld.
- **Steal**: every attempt to steal, whatever comes of it.
- **Open lock**: a try at picking a lock, whether it works or not.

Only skills you are trained in earn marks. Marks raise nothing by themselves: they gather until
your next level. The Skills screen's meter shows how practised a skill is (the step its marks
have reached on the ladder below), never the marks.

**Skill rolls.** Every level gives you a number of **skill rolls** (see
[Experience](experience.md)). You spend each one on a trained skill of your choice, before you
do anything else; nothing stops you putting every roll into one skill. A roll throws the die the
skill's marks have reached on the ladder and adds it to the skill, and spends half of the skill's
marks (rounded up), so a second roll on the same skill that level is a step or two smaller
(50 marks → 25 → 12 → 6 → 3: `3d3`, `2d4`, `1d5`, `1d3`, `+1`). A skill with no marks still
gains 1. Past 100 a roll adds half its result, rounded up. The game says "Your stealth improves."

- A skill your **class is known for** (its `2d6+10` skills) throws the die one step up the
  ladder.
- Any **other skill your class trains**, or a skill **your race trains**, adds 1 to the roll.
  Both stack: a halfling thief's stealth is known for and racial, one step up and +1.

The level-up screen shows these numbers before you choose: beside each skill the dice a roll on
it adds (`1d3`, `+2`, `1d5+1`; `(1d3)/2` past 100), and for the selected skill how they came
about (`Rolls 1d5+1: practice (20 marks) gives 1d5, +1 as your class trains it.`), with a line
saying where your rolls come from (`4 skill rolls a level: class 3, race 1`). The roll itself
is thrown only when you choose.

{{#include generated/skill-ladder.md}}

How many rolls a level gives: your class's, plus your race's, plus your Intelligence's.

{{#include generated/skill-rolls.md}}

Only you train, for now; creatures keep the skills they have. Your skills and their marks are
saved with your run.

## Stealing

If you are trained in **Steal** (thieves are), **Steal** is in your
[abilities menu](abilities.md): choose it, then a direction, and you try to lift something from
whoever stands there. You can steal only from someone you see who is not hostile to you: a
peaceful villager, a shopkeeper, a creature that leaves you alone. It takes a turn whatever comes
of it.

Whether you go unnoticed is a contest, the same dice on both sides: 2d12 plus your Dexterity
modifier plus what your Steal skill gives (the benefits table above), against 2d12 plus your
victim's Willpower modifier. A tie is yours; a roll of 2 always fails and a roll of 24 always
works. Only the difference between the two sides matters, your **edge**:

{{#include generated/stealing.md}}

- **Unnoticed**, you take one stack of what they carry in their pack, all of it: a whole purse.
  You never see what someone carries in their pack, so you do not choose: your fingers find
  something. What they wear or hold in their hands stays where it is. With nothing in their
  pack you come away empty-handed ("You find nothing to take from the gardener."), and they
  notice nothing all the same.
- **Caught**, they turn on you and come after you, for good ("The kobold catches you stealing and
  turns on you!"). A shopkeeper who catches you is outraged as a robbed one is: the shop trades
  no more, and whatever you had not paid for is stolen (see [Shops](shops.md)).

Stealing from a shopkeeper takes from their purse, never from the shop floor: what lies in a shop
is for sale, and taking it out without paying is theft by the shop's own rules. A shopkeeper
whose purse you lifted unnoticed still trades with you, but cannot pay for what you sell.

You may try as often as you like; someone who caught you stays angry, and that is the price.
