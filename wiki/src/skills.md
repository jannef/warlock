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
(quicker in everything, and kicking doors in, see [Doors](doors.md)), Search (see [Searching](searching.md)), Disable
device (disarming traps, see [Traps](traps.md#disarming-traps)), Open lock (picking locks,
see [Doors](doors.md#picking-locks)), Steal (see [Stealing](#stealing) below), Heal (see
[Treating yourself](#treating-yourself) below), and Survival (foraging, see
[Foraging](#foraging) below) do something: the others are there to be trained, and get their uses
as the game grows. The skills used as actions are in the [abilities
menu](abilities.md) once you are trained in them, greyed out until their uses come; Disable
device's use is there as **Disarm a trap**, Open lock's as **Pick lock**, Steal's as **Steal**,
Heal's as **Heal**, and Survival's, though it is a passive skill, as **Forage**.

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
- **Athletics** makes you quicker in everything you do, a twentieth at 100 and a tenth at 200
  (it adds to the quickness of talents such as Fleet-footed and Quick; see
  [Burden](burden.md) for the load, which slows you on top of it). It also helps you kick doors
  in (see [Doors](doors.md)). Besides kicks, it is trained now and then by acting under a load
  that slows you and by journeys on the overworld.
- **Survival** makes your journeys on the [overworld](overworld.md) quicker (it does nothing for
  your steps in a local area or a dungeon), and lets you forage in the wild, finding food more
  often the higher it is (see [Foraging](#foraging)).
- **Appraise** gets you better prices in [shops](shops.md): you pay less and are offered more.
- **Search** helps you find secret doors and hidden traps when you search (`s`, `S`), and a
  master's search reaches a cell farther (see [Searching](searching.md)).
- **Spot** lets you notice hidden things in passing at all, and the higher it is, the more you
  notice.
- **Open lock** lets you pick locks at all (the Pick lock ability), and the higher it is, the
  more often a try works (see [Doors](doors.md#picking-locks)).
- **Steal** lets you steal at all, and the higher it is, the likelier your hand goes unnoticed
  (see [Stealing](#stealing)).
- **Heal** lets you bind your bleeding and tend your poisons and diseases at all, and the higher
  it is, the more often a try works (see [Treating yourself](#treating-yourself)).

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
- **Heal**: a try at treating yourself, whether it works or not.

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

## Treating yourself

If you are trained in **Heal** (priests, witches, druids, and paladins are), **Heal** is in your
[abilities menu](abilities.md), under Skills. It treats you, never anyone else (for now): your
**bleeding**, your **poison**, and a **disease** that shows (one you have caught but that has not
shown yet is unknown to you, so you cannot treat it). With nothing to treat it is greyed out,
"nothing to treat"; with more than one thing, it asks which ("Sickness" is a disease you cannot
name yet). Every try takes about a quarter of a minute, whatever comes of it, and makes no noise.

A try is a check (see [Diseases](diseases.md#checks)): 2d12 plus your Intelligence modifier plus
what your Heal skill gives (the benefits table above) against the difficulty below; a roll of 2
always fails and a roll of 24 always works.

- **Bleeding**: made, you bind the wound and the bleeding stops ("You bind the wound."); failed,
  "The blood still runs.", and you may try again at once, as often as you like. The worse the
  bleeding, the harder it is: 6 plus 2 for each degree.
- **Poison**: made, you draw out some of it: it is a degree weaker ("You draw out some of the
  poison."). You get **one try** at a dose, whatever comes of it: afterwards "You can do no more
  for the poison." until it wears off; poisoned again later, you may try again. 8 plus 1 for each
  degree.
- **A disease**: made, you ease it ("You ease the filth fever somewhat."): it counts as one of
  the checks in a row that shake it off, though never the last one, so treating never cures it
  by itself. A disease that never goes away by itself, or one a single won check from going,
  instead takes a full interval from your try before it next gets worse. Again **one try** a
  bout, whatever comes of it ("You can do no more for the filth fever."); caught again later,
  you may try again. Its difficulty is the disease's own.

{{#include generated/heal-targets.md}}

## Foraging

If you are trained in **Survival** (barbarians, druids, rangers, and lizardfolk are),
**Forage** is in your [abilities menu](abilities.md): choose it, and you spend a while searching
the land around you for something to eat. You can forage only in the wild: on the
[overworld](overworld.md), on open land, in woods, on the hills, or in the marshes, or in the
local area of such a cell (an area forages as its cell on the overworld does). Not on a road, not
in water or on ice, not in a town or a dungeon: there the menu greys Forage out ("not usable
here").

A forage takes ten minutes. Something coming into view, or anything else that would wake you
from a rest, stops it, and a forage cut short finds nothing. One that runs its time finds food
with the land's chance below, plus what your Survival gives (the benefits table above: up to 30
more); then one of the land's finds is picked by the shares shown, and you gather a few of it
("You gather 2 handfuls of berries."), fresh, into your pack. Otherwise "You find nothing worth
eating." Either way your Survival may earn a training mark.

{{#include generated/foraging.md}}

What you find is ordinary food (see [Hunger](hunger.md)): berries and mushrooms spoil within days,
hazelnuts keep for months, and an angelica root cures diseases. Forage as often as you like, in
the same spot too: the time is the price.
