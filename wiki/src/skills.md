# Skills

Skills are what you have trained in: moving quietly, tumbling out of harm's way, finding your way
through the wild. Your [race](races.md) and your [class](classes.md) train you in some of them
when your character is made; the rest you are **untrained** in.

- An untrained skill does not exist for you. It is not shown anywhere, and it gives you nothing.
  That is not the same as a skill at 0: a skill you are trained in at 0 is shown, and it can
  grow.
- A trained skill has a **value**, usually between 5 and 30 when you start. 100 is a
  master. Past 100 a skill still grows, but slowly.
- `S` shows your **Skills** screen: every skill you are trained in, with its value, and beside it
  a small meter (`━━──`) of how near it is to its next point. Use the arrow keys to read what each
  skill is about. It takes no time. `S` on the [character screen](character.md) (`@`) opens it
  too, and the summary at the end of character creation lists the skills you will start with.

What a skill does for you is never shown in numbers in the game: you see the value, and feel the
difference. This page gives the rules.

## The skills

Some skills work by themselves, all the time (**passive**); some will be used as **actions**
(searching, picking a lock, binding a wound); some are **read by rules** when something calls for
them (athletics when you kick a door). For now only the passive skills, Appraise, Athletics
(kicking doors in, see [Doors](doors.md)), Search (see [Searching](searching.md)), and Disable
device (disarming traps, see [Traps](traps.md#disarming-traps)) do something: the others are
there to be trained, and get their uses as the game grows. The skills used as actions are in the
[abilities menu](abilities.md) once you are trained in them, greyed out until their uses come;
Disable device's use is there as **Disarm a trap**.

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
- **Search** helps you find secret doors and hidden traps when you search (`s`, `z`), and a
  master's search reaches a cell farther (see [Searching](searching.md)).
- **Spot** lets you notice hidden things in passing at all, and the higher it is, the more you
  notice.

{{#include generated/skill-benefits.md}}

## Who trains what

Each race trains one or two skills; each class a handful, a few of them well (`2d6+10`) and the
rest a little (`2d4+4`). A skill your race and your class both train adds the two. Humans train
none of their own, but they put extra work into one of their class's skills, picked at random.
The dice are rolled when your character is made, after your [stats](character.md#how-your-stats-add-up),
so they never change a stat.

{{#include generated/skill-grants.md}}

Some creatures are trained in skills too: deer listen well, and the kobold shaman knows something
of spells. You never see a creature's skills.

## Getting better

Skills grow by use, the way ADOM does it: each time you use a skill in a way that matters, it may
earn a **training mark**. The chance is shown in the table of skills above; skills you use all the
time, such as balance, earn marks rarely, so they do not race ahead.

- **Stealth**: a step that a creature nearby could have heard, if you had not stepped softly.
- **Listen**: hearing something new.
- **Tumble**: dodging an attack ("You dodge the kobold's spear.").
- **Balance**: doing anything but waiting while your load slows you down.
- **Survival**: a journey on the overworld.

Only skills you are trained in earn marks. When a skill has enough marks it gains a point, and the
game tells you: "Your stealth improves."

**This is provisional.** How skills increase is not settled yet; for now marks alone do it. A
point needs a few marks at first, a few more the higher the skill is, and past 100 three times as
many: reaching 120 takes about as many marks as the whole way to 100.

{{#include generated/skill-increase.md}}

Only you train, for now; creatures keep the skills they have. Your skills and their marks are
saved with your run.
