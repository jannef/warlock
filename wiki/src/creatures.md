# Creatures

Creatures are the other glyphs that move about the world. They belong to factions, keep to their
routines, notice what they see and hear, and some of them fight. Fights are only announced:
nothing is hurt, and nothing dies. This page will grow as the game does.

## Factions

Every creature belongs to a **faction**, and so do you. Two factions are either **hostile** to
each other or **neutral**, both ways: kobolds hunt giant rats, and the rats fight back; wildlife
leaves everyone alone, and everyone leaves it alone. A faction is neutral to itself.

{{#include generated/factions.md}}

You can tell how a creature stands toward you from what it is: look at it (look mode) and a
hostile one is marked, as in "A giant rat (hostile), on stone floor."

## What they do

When nothing is about, each creature goes about its **routine**: it walks a fixed, looping patrol
route, one step at a time, or stays where it is.

A creature that **sees** something hostile to it (you, or a creature of a hostile faction) goes
for the nearest one:

- Creatures that fight **up close** come at it along the shortest way they can find, and attack
  once they stand next to it.
- Creatures that **shoot** or **cast spells** need to be close enough and to have a clear line:
  nothing solid, and nobody, in between. If they have one, they attack from where they are;
  otherwise they move to the nearest place that gives them one.
- When it loses sight of its target, it goes to where it last saw it and looks around.
- When it **hears** something it cannot see, it goes to have a look, if it has enemies at all
  (wildlife does not bother). See [Hearing](hearing.md).
- With nothing left to chase, it walks back to where it left its routine and carries on.

Creatures behave the same way every time in the same situation: there is no chance in it. They
see by the same rules you do (see [Sight and memory](sight.md)), each with its own eyes: a
creature without darkvision does not see you standing in the dark, and lighting your lantern
gets you noticed. Some creatures open closed doors in their way; others wait at them.

Other creatures block the way: a creature walks a little way round someone in its way, or waits
behind them.

## Attacks

An attack is announced, never resolved: "The kobold attacks you.", "The kobold archer shoots at
the giant rat.", "The kobold shaman casts a spell at you." You only hear of attacks you perceive:
ones where you see the attacker or the target, or are the target. If you cannot see who is
attacking you, "Something attacks you." Attacks are noisy, so you may also hear fighting, a
bowstring, or chanting out of sight.

Moving into a hostile creature attacks it ("You attack the giant rat."), which takes a turn.
Moving into a neutral one does not: "The deer is in the way."

## Pace and levels

- Creatures move at their **own pace**. Some are quicker than you and may act more than once
  while you take one turn; some are slower and now and then let a turn of yours pass without
  acting. A creature of your own pace acts once for each of your turns. The game never tells you a
  creature's pace: you learn it by watching it move.
- How often a creature acts follows game time, not your turns: while you wade through water or
  walk a long overworld step, creatures get to act for as long as that took you.
- Creatures only act on the level you are on. On other levels they stand still until you come
  back, and then carry on where they left off: they do not catch up on the time you were away.
- You only see a creature while you see its cell, so creatures in the dark are hidden unless they
  carry a light. You never remember where one was.
- Creatures make noise as they walk (bats fly silently), so you may hear one you cannot see, and
  they hear your steps and doors.
- Some creatures carry [objects](objects.md). You cannot see what they carry, and nothing makes
  them drop it yet.

## The creatures

{{#include generated/creatures.md}}
