# Traps

Some floors are trapped, and some doors. A trap lies on a cell, **hidden**: it looks like the
floor around it, or the door it is on, and nothing tells you it is there until it goes off.
Whoever steps onto a trap sets it off, you and the [creatures](creatures.md) alike; a door's trap goes off when
someone opens or kicks the [door](doors.md) (see [Trapped doors](#trapped-doors)).

## Finding traps

You learn where a trap is when you find it, [searching](searching.md#searching-1) or [in passing](searching.md#noticing-in-passing), which your **[Detect traps](skills.md#skill-detect-traps)**
skill helps (see [Searching](searching.md) and [Skills](skills.md)), or when it goes off:

- when **you** set it off;
- when you **see a creature** set it off. Creatures know no more about traps than you do, so they
  walk onto them; a trap that goes off where you cannot see it stays hidden. You may still hear
  it, or see the blast of a fireball where you see part of it.

From then on the trap is **known**: it shows as `^` in its own color, over anything lying on it,
and you remember it like the terrain, even out of sight. [Look mode](controls.md#look-mode) names it: "A fireball trap, on
stone floor.", or "A kobold (hostile, unhurt), on a pit on stone floor." Some places show you
their traps from the start.

Stepping onto a trap you know asks first, in a small window: `Really step onto the fireball
trap?` Choose **Step onto it** (`y`) to step on, or **Stay back** (`n` or `Esc`, chosen at first)
to stay where you are. Holding a movement key stops you short of the trap instead of asking.

## Stepping past a known trap

When you step onto a trap you know of, on purpose, you have a good chance to **avoid** it: you
step lightly over the pit, or deftly past the pressure plate before the arrow flies ("You deftly
avoid the arrow trap.", "You step lightly over the pit."). The trap stays where it is, still
armed, and the step takes its usual time. Otherwise it goes off as usual.

The chance depends on your **[dodge](combat.md#defence)**: your [Dexterity](stats.md#stat-dexterity) modifier as your armour and [burden](burden.md#burden-1) leave it
(heavy armour and a heavy load cap it, as in [combat](combat.md)), plus every **Dodge** bonus you
have (the [Tumble](skills.md#skill-tumble) skill, [talents](talents.md), [magic](magic-items.md), your [stance](combat.md#stances), and your state). It is 55% at a dodge of 0,
10% more for each point above that and 10% less for each point below, never less than **15%**
and never more than **90%**: it is always a gamble.

{{#include generated/trap-avoid.md}}

A trap you have not found gets no such chance: you do not know to step lightly. Nor does a
step you did not choose, such as being turned back at a [shop's](shops.md) door, and creatures never try.
Avoiding a trap counts as [dodging](combat.md#defence) for training [Tumble](skills.md#skill-tumble) and [Dexterity](stats.md#stat-dexterity). A door's trap is not stepped
on; it goes off when the door opens (see [Trapped doors](#trapped-doors)). The **[Light step](talents.md#talent-light-step)**
talent's chance comes on top: if you fail to avoid a trap, it may still save you, and it works
on hidden traps too (see [Talents](talents.md)).

## What traps do

{{#include generated/traps.md}}

- **Attacks**: an [arrow](traps.md#trap-arrow) or [spears](traps.md#trap-spear) attack like a weapon, rolled against the [Defence](combat.md#defence) of whoever
  stepped on the trap (see [Combat](combat.md)): they can miss ("You dodge the arrow.", "The spear
  glances off your armour."), hit, or hit hard. An arrow flies like a shot (see
  [Ranged combat](ranged.md)): it attacks whoever stands in its way, and lands where it falls, to
  be picked up.
- **Blasts**: a [fireball](traps.md#trap-fireball) burns everyone around the trap and a [lightning bolt](traps.md#trap-lightning-bolt) strikes everyone
  along its line, you included if you are there, with no roll [to hit](combat.md#hitting). [Walls](terrain.md#walls) stop both, but a
  fireball blows the softer [walls](terrain.md#walls) it reaches apart (see [Digging and walls](digging.md)) and melts
  the [ice](temperature.md#walking-on-ice) it reaches, and whoever stands on that ice [falls in](temperature.md#falling-in). A [frost trap](traps.md#trap-frost) bites everyone around
  it with [cold](damage-types.md#damage-cold) and [freezes](temperature.md#water-freezes) the water it reaches at once, into ice that holds a laden person (see
  [Temperature and ice](temperature.md)). The [lightning](traps.md#trap-lightning-bolt) runs one way, along the longest clear
  line from the trap unless the place says otherwise.
- **[A pit](traps.md#trap-pit)** costs you your next action as you climb out, as well as the fall.
- **[An alarm](traps.md#trap-alarm)** rings a bell that every creature within [earshot](hearing.md#what-makes-noise) hears; the ones who have [enemies](creatures.md#factions)
  come to look (see [Hearing](hearing.md)).
- **[A teleport trap](traps.md#trap-teleport)** moves you to another free place on the same level, somewhere you may never
  have seen ("You are wrenched elsewhere.").
- **Charges**: some traps go off only once or twice; then they are spent and gone.
- **The same spells** come on [scrolls](scrolls.md): a scroll of [fireball](scrolls.md#scroll-of-fireball), [lightning](scrolls.md#scroll-of-lightning), or [frost](scrolls.md#scroll-of-frost)
  casts what the trap does, where you aim it.

The **[Trap sense](talents.md#talent-trap-sense)** talent halves the [damage](combat.md#damage) a trap does to you (rounded down, at least 1; see
[Talents](talents.md)), and magic that wards off traps takes its percent off as well (see
[Magic items](magic-items.md)), a creature's too. Together they never take more than **75%**
off: a trap always hurts.

A trap's blow wounds like any other: a big one spills [blood](combat.md#blood) and can leave you
[bleeding](status-effects.md#bleeding). Traps can kill, you as well as creatures. Creatures
killed by a trap drop what they carried, as from any death, but a trap's kill earns you no
[experience](experience.md) and counts for no [quest](quests.md), even if you lured the creature
there; if a trap kills you, the death screen names it ("Killed by a fireball trap"). Someone who
makes way for you onto a trap you stood on sets it off too.

No trap lies on water that can [freeze](temperature.md#water-freezes), and a [teleport](traps.md#trap-teleport) never sets you down on [ice](temperature.md#walking-on-ice).

## Disarming traps

If you are trained in **[Disable device](skills.md#skill-disable-device)** ([thieves](classes.md#thief) are), you can try to take apart a trap you
know of, on the floor or on a door, from the cell next to it: **[Disarm a trap](abilities.md#ability-disarm)** in the
[abilities menu](abilities.md) (`a`), then the direction. A trap you have not found cannot be
disarmed: trying where none is known only tells you "You know of no trap there.", whether one is
hidden there or not, and takes no time.

A try is a [check](diseases.md#checks): two twelve-sided dice, plus your [Dexterity](stats.md#stat-dexterity)
modifier, plus what your [Disable device](skills.md#skill-disable-device) gives (+8 at 100, +12 at 200; see
[Skills](skills.md)), against the trap's **difficulty**, the same number finding it takes (deeper
in a dungeon, both are harder). Reach it and the trap is gone for good ("You disarm the arrow
trap."). Fall short and it stays ("You fail to disarm the arrow trap."); fall short by **5 or
more** and it goes off at you ("You set off the arrow trap!"), as if you had set it off where you
stand: the [arrow](traps.md#trap-arrow) shoots at you from its slit, the [spears](traps.md#trap-spear) strike at you, the [fireball](traps.md#trap-fireball) bursts
around the trap, the [lightning](traps.md#trap-lightning-bolt) runs from it through you, you tumble into the [pit](traps.md#trap-pit) and lose your
next action climbing back out, the [alarm](traps.md#trap-alarm) rings, or you are wrenched elsewhere. A trap with
charges spends one. A natural 2 always fails and a natural 24 always succeeds.

Every try takes **12 seconds** at a normal pace, whatever comes of it, and may train [Disable
device](skills.md#skill-disable-device). You can try again as often as you like: nothing remembers a failed try, and time (and the
risk) is the only price. Disarming earns no [experience](experience.md).

{{#include generated/trap-disarm.md}}

## Trapped doors

A trap may lie on a door instead of the floor. It belongs to the door: walking through the
doorway never sets it off, and stepping into it never asks. It goes off when someone **opens**
the door (`o`, or walking into it; unlocking it with a [key](doors.md#keys) too, or [picking its lock](doors.md#picking-locks)), you or a
creature that opens doors, and at every **[kick](doors.md#kicking)** at the door, whether the door breaks or holds. It goes off at
whoever opened or kicked it, from the door: an [arrow](traps.md#trap-arrow) shoots from the wall beside the door at them,
a blade strikes at them, lightning runs from the door through them, and a [fireball](traps.md#trap-fireball) bursts out on
both sides of the door, even while it is still shut. [Pits](traps.md#trap-pit) are never on doors.

The trap stays with the door: shut the door again and it is set again, ready to go off at the
next one who opens it (unless it has run out of charges). A door you kick in takes its trap with
it. [Searching](searching.md#searching-1) finds a door's trap like any hidden trap ("You notice that the door is trapped: an
arrow trap."), and once you know it, it shows as `^` in place of the door. A [secret door](doors.md#secret-doors) is never
trapped. A door's trap can be disarmed like any other (see [Disarming traps](#disarming-traps)).

## Trapped chests

A shut [chest](chests.md) may be trapped too, with any trap a door may have (never a [pit](traps.md#trap-pit)): there are no traps
made only for chests. It goes off when you open the chest, kick it open, or fail badly to disarm
it, and a **[fireball](traps.md#trap-fireball) or [frost](traps.md#trap-frost)** trap goes off inside the box and **blows the chest apart**, taking
much of what it held with it. See [Chests](chests.md#trapped-chests).

## Where traps are

- **[Generated dungeons](dungeons.md)** hide a few traps on every level, never on or next to the [stairs](terrain.md#stairs), and
  never where a creature walks its rounds. Some [rooms](special-rooms.md) hide their own: an offering before an [altar](special-rooms.md#room-altar)
  may not be unguarded.
- **Cēaptūn** and the [overworld](overworld.md) have no traps.
