# Traps

Some floors are trapped, and some doors. A trap lies on a cell, **hidden**: it looks like the
floor around it, or the door it is on, and nothing tells you it is there until it goes off.
Whoever steps onto a trap sets it off, you and the creatures alike; a door's trap goes off when
someone opens or kicks the door (see [Trapped doors](#trapped-doors)).

## Finding traps

You learn where a trap is when you find it, searching or in passing (see
[Searching](searching.md)), or when it goes off:

- when **you** set it off;
- when you **see a creature** set it off. Creatures know no more about traps than you do, so they
  walk onto them; a trap that goes off where you cannot see it stays hidden. You may still hear
  it, or see the blast of a fireball where you see part of it.

From then on the trap is **known**: it shows as `^` in its own color, over anything lying on it,
and you remember it like the terrain, even out of sight. Look mode names it: "A fireball trap, on
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

The chance depends on your **dodge**: your Dexterity modifier as your armour and burden leave it
(heavy armour and a heavy load cap it, as in [combat](combat.md)), plus every **Dodge** bonus you
have (the Tumble skill, talents, magic, your stance, and your state). It is 55% at a dodge of 0,
10% more for each point above that and 10% less for each point below, never less than **15%**
and never more than **90%**: it is always a gamble.

{{#include generated/trap-avoid.md}}

A trap you have not found gets no such chance: you do not know to step lightly. Nor does a
step you did not choose, such as being turned back at a shop's door, and creatures never try.
Avoiding a trap counts as dodging for training Tumble and Dexterity. A door's trap is not stepped
on; it goes off when the door opens (see [Trapped doors](#trapped-doors)). The **Light step**
talent's chance comes on top: if you fail to avoid a trap, it may still save you, and it works
on hidden traps too (see [Talents](talents.md)).

## What traps do

{{#include generated/traps.md}}

- **Attacks**: an arrow or spears attack like a weapon, rolled against the Defence of whoever
  stepped on the trap (see [Combat](combat.md)): they can miss ("You dodge the arrow.", "The spear
  glances off your armour."), hit, or hit hard. An arrow flies like a shot (see
  [Ranged combat](ranged.md)): it attacks whoever stands in its way, and lands where it falls, to
  be picked up.
- **Blasts**: a fireball burns everyone around the trap and a lightning bolt strikes everyone
  along its line, you included if you are there, with no roll to hit. Walls stop both, but a
  fireball blows the softer walls it reaches apart (see [Digging and walls](digging.md)) and melts
  the ice it reaches, and whoever stands on that ice falls in. A frost trap bites everyone around
  it with cold and freezes the water it reaches at once, into ice that holds a laden person (see
  [Temperature and ice](temperature.md)). The lightning runs one way, along the longest clear
  line from the trap unless the place says otherwise.
- **A pit** costs you your next action as you climb out, as well as the fall.
- **An alarm** rings a bell that every creature within earshot hears; the ones who have enemies
  come to look (see [Hearing](hearing.md)).
- **A teleport trap** moves you to another free place on the same level, somewhere you may never
  have seen ("You are wrenched elsewhere.").
- **Charges**: some traps go off only once or twice; then they are spent and gone.
- **The same spells** come on [scrolls](scrolls.md): a scroll of fireball, lightning, or frost
  casts what the trap does, where you aim it.

The **Trap sense** talent halves the damage a trap does to you (rounded down, at least 1; see
[Talents](talents.md)), and magic that wards off traps takes its percent off as well (see
[Magic items](magic-items.md)), a creature's too. Together they never take more than **75%**
off: a trap always hurts.

A trap's blow wounds like any other: a big one spills blood and can leave you
[bleeding](status-effects.md#bleeding). Traps can kill, you as well as creatures. Creatures
killed by a trap drop what they carried, as from any death, but a trap's kill earns you no
[experience](experience.md) and counts for no [quest](quests.md), even if you lured the creature
there; if a trap kills you, the death screen names it ("Killed by a fireball trap"). Someone who
makes way for you onto a trap you stood on sets it off too.

No trap lies on water that can freeze, and a teleport never sets you down on ice.

## Disarming traps

If you are trained in **Disable device** (thieves are), you can try to take apart a trap you
know of, on the floor or on a door, from the cell next to it: **Disarm a trap** in the
[abilities menu](abilities.md) (`a`), then the direction. A trap you have not found cannot be
disarmed: trying where none is known only tells you "You know of no trap there.", whether one is
hidden there or not, and takes no time.

A try is a [check](diseases.md#checks): two twelve-sided dice, plus your Dexterity
modifier, plus what your Disable device gives (+8 at 100, +12 at 200; see
[Skills](skills.md)), against the trap's **difficulty**, the same number finding it takes (deeper
in a dungeon, both are harder). Reach it and the trap is gone for good ("You disarm the arrow
trap."). Fall short and it stays ("You fail to disarm the arrow trap."); fall short by **5 or
more** and it goes off at you ("You set off the arrow trap!"), as if you had set it off where you
stand: the arrow shoots at you from its slit, the spears strike at you, the fireball bursts
around the trap, the lightning runs from it through you, you tumble into the pit and lose your
next action climbing back out, the alarm rings, or you are wrenched elsewhere. A trap with
charges spends one. A natural 2 always fails and a natural 24 always succeeds.

Every try takes **12 seconds** at a normal pace, whatever comes of it, and may train Disable
device. You can try again as often as you like: nothing remembers a failed try, and time (and the
risk) is the only price. Disarming earns no [experience](experience.md).

{{#include generated/trap-disarm.md}}

## Trapped doors

A trap may lie on a door instead of the floor. It belongs to the door: walking through the
doorway never sets it off, and stepping into it never asks. It goes off when someone **opens**
the door (`o`, or walking into it; unlocking it with a key too, or picking its lock), you or a
creature that opens doors, and at every **kick** at the door, whether the door breaks or holds. It goes off at
whoever opened or kicked it, from the door: an arrow shoots from the wall beside the door at them,
a blade strikes at them, lightning runs from the door through them, and a fireball bursts out on
both sides of the door, even while it is still shut. Pits are never on doors.

The trap stays with the door: shut the door again and it is set again, ready to go off at the
next one who opens it (unless it has run out of charges). A door you kick in takes its trap with
it. Searching finds a door's trap like any hidden trap ("You notice that the door is trapped: an
arrow trap."), and once you know it, it shows as `^` in place of the door. A secret door is never
trapped. A door's trap can be disarmed like any other (see [Disarming traps](#disarming-traps)).

## Trapped chests

A shut chest may be trapped too, with any trap a door may have (never a pit): there are no traps
made only for chests. It goes off when you open the chest, kick it open, or fail badly to disarm
it, and a **fireball or frost** trap goes off inside the box and **blows the chest apart**, taking
much of what it held with it. See [Chests](chests.md#trapped-chests).

## Where traps are

- **Generated dungeons** hide a few traps on every level, never on or next to the stairs, and
  never where a creature walks its rounds. Some rooms hide their own: an offering before an altar
  may not be unguarded.
- **Cēaptūn** and the overworld have no traps.
