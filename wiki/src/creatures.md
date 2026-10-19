# Creatures

Creatures are the other glyphs that move about the world. They belong to factions, keep to their
routines, notice what they see and hear, and some of them fight, to the death (see
[Combat](combat.md)). This page will grow as the game does.

## Factions

Every creature belongs to a **faction**, and so do you. Two factions are either **hostile** to
each other or **neutral**, both ways: kobolds hunt giant rats, and the rats fight back; wildlife
leaves everyone alone, and everyone leaves it alone. A faction is neutral to itself.

{{#include generated/factions.md}}

You can tell how a creature stands toward you from what it is: look at it
([look mode](controls.md#look-mode)) and a hostile one is marked, as in "A giant rat (hostile,
unhurt), on stone floor."

## What they do

When nothing is about, each creature goes about its **routine**: it walks a fixed, looping patrol
route, one step at a time, or stays where it is.

A creature that **sees** something hostile to it (you, or a creature of a hostile faction) goes
for the nearest one:

- Creatures that fight **up close** come at it along the shortest way they can find, and attack
  once they stand next to it.
- Creatures that **shoot** or **cast spells** need to be close enough and to have a clear line:
  nothing solid, and nobody, in between. If they have one, they attack from where they are;
  otherwise they move to the nearest place that gives them one. An archer spends an arrow a
  shot, and with none left (or no bow) it comes to fight up close ([Ranged combat](ranged.md)).
- When it loses sight of its target, it goes to where it last saw it (or next to that spot, if
  someone it cannot see stands there). If it finds no one there, or cannot get there, it gives up.
- When it **hears** something it cannot see, it goes to have a look, if it has enemies at all
  (wildlife does not bother). See [Hearing](hearing.md).
- With nothing left to chase, it walks back to where it left its routine and carries on.

Creatures behave the same way every time in the same situation: there is no chance in it. They
see by the same rules you do (see [Sight and memory](sight.md)), each with its own eyes and under
the same light, clouds included: a creature without darkvision does not see you standing in the
dark, and lighting your [lantern](lantern.md) gets you noticed. Some creatures open closed doors
in their way, which takes them a turn and makes noise; the others cannot get past a closed door.
A locked door stops even those, unless they carry its key (see [Doors](doors.md)).

Other creatures block the way: a creature walks a little way round someone in its way, or waits
behind them.

## Attacks

Blows and shots are rolled, hit or miss, hurt, and kill, by the rules of [Combat](combat.md):
"The kobold hits you.", "The giant rat bites the kobold hard!", "You block the kobold's spear
with your shield.", "The kobold archer shoots the giant rat.", "The giant rat dies." Spells are
only announced: "The kobold shaman casts a spell at you." You only hear of attacks you perceive:
ones where you see the attacker or the target, or are the target. An end you cannot see is
"something": "Something hits you.", "The kobold archer misses something." Attacks are noisy, so
you may also hear fighting, a bowstring, or chanting out of sight.

A creature that dies is gone, and what it carried lies where it fell. Creatures do not flee or
fight any differently when they are hurt, but [look mode](controls.md#look-mode) shows how hurt
they look, and whether they bleed. Big hits and criticals make creatures bleed as they make you
bleed, and a badly wounded one may bleed to death after the fight; a giant rat's bite may poison
whatever it bites (see [Status effects](status-effects.md)), or give it filth fever (see
[Diseases](diseases.md)).

When you see both ends of an attack, you also see it happen: a creature fighting up close (you
too) lunges at its target, an arrow flies from the archer to its target, and a spell lands with a
faint violet sparkle (with the matching [settings](settings.md) on).

Moving into a hostile creature attacks it ("You hit the giant rat.", "You miss something." if
you cannot see it), which takes a turn. Moving into any other creature swaps places with it, for
your step's time ("You swap places with the deer."); it walks back to where it stood afterwards
and carries on. A few will not make way ("The sentry doesn't budge."), which takes no time; see
[People and talk](npcs.md#getting-past).

The showcase's people (the gardener, the bard, the hermit) are peaceful folk by nature, and talk
to you (see [People and talk](npcs.md)). Some creatures are hostile by nature but placed among
the **peaceful folk** by the place they live in: the kobold in the showcase hub and the kobolds and rats of the Vast Caverns walk their
rounds and never attack you. The showcase is safe. Fighting happens in the arena and in the Depths.

**Shopkeepers** are **merchants**, who trade with everyone and fight no one, until you rob or
attack one: then it joins the outraged shopkeepers and comes after you (see [Shops](shops.md)).

The **Arena** scenario (see [Starting a run](introduction.md#starting-a-run)) is the place to watch
this: a kobold war band and a pack of giant rats fight to the death there, and turn on you when
they see you. Light your lantern there and the war band's archer will find you.

## Pace and levels

- Creatures move at their **own pace**. Some are quicker than you and may act more than once
  while you take one turn; some are slower and now and then let a turn of yours pass without
  acting. A creature of your own pace acts once for each normal action of yours. The game never
  tells you a creature's pace: you learn it by watching it move.
- Their actions have [costs](turns.md#what-actions-cost) like yours: a creature's step through
  water takes it twice as long as a step on open ground, and a creature type's attacks may cost
  it more or less than a normal action.
- How often a creature acts follows game time, not your turns: while you wade through water or
  [rest](resting.md), creatures get to act for as long as that took you.
- Creatures only act on the level you are on. On other levels they stand still until you come
  back, and then carry on where they left off: they do not catch up on the time you were away.
- You only see a creature while you see its cell, so creatures in the dark are hidden unless they
  carry a light. You never remember where one was.
- Creatures make noise as they walk (bats fly silently), so you may hear one you cannot see, and
  they hear your steps, doors, and attacks.
- Some creatures carry [objects](objects.md), and wear or wield some of them: the kobolds of the
  arena carry spears, their archer a bow and arrows, their shaman an amulet. You see what a
  creature wears and wields ([look mode](controls.md#look-mode) says it), never what else it
  carries. They fight with it ([Equipment](equipment.md)), and drop all of it when they die.

## The creatures

{{#include generated/creatures.md}}
