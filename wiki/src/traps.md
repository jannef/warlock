# Traps

Some floors are trapped, and some doors. A trap starts out **hidden**: it looks like the floor
around it, or the door it is on, and nothing tells you it is there until it goes off. Whoever steps
onto a trap sets it off, you and the [creatures](creatures.md) alike. A door's trap goes off when
someone opens or kicks the [door](doors.md) (see [Trapped doors](#trapped-doors)).

## Finding traps

You learn where a trap is in one of three ways:

- you find it by [searching](searching.md#how-to-search) or [in passing](searching.md#noticing-in-passing),
  which your [Detect traps](skills.md#skill-detect-traps) skill helps (see [Searching](searching.md)
  and [Skills](skills.md));
- **you** set it off;
- you **see a creature** set it off. Creatures know no more about traps than you do, so they walk
  onto them. A trap that goes off where you cannot see it stays hidden. You may still hear it, or
  see the blast of a fireball where you can see part of it.

From then on the trap is **known**. It shows as `^` in its own colour, over anything lying on it,
and you remember it like the terrain, even out of sight. [Look mode](controls-prompts.md#look-mode) names
it: "A fireball trap, on stone floor.", or "A kobold (hostile, unhurt), on a pit on stone floor."
Some places show you their traps from the start.

Stepping onto a trap you know of asks first, in a small window: `Really step onto the fireball
trap?` Choose **Step onto it** (`y`) to step on, or **Stay back** (`n` or `Esc`, chosen at first)
to stay where you are. Holding a movement key stops you short of the trap instead of asking.

## Stepping past a known trap

When you step onto a known trap on purpose, you have a good chance to **avoid** it. You step
lightly over the pit, or deftly past the pressure plate before the arrow flies ("You deftly avoid
the arrow trap.", "You step lightly over the pit."). The trap stays where it is, still armed, and
the step takes its usual time. If you fail, it goes off as usual.

The chance depends on your **[dodge](combat.md#defence)**. That is your
[Dexterity](stats.md#stat-dexterity) modifier as your armour and [burden](burden.md#burden-levels) leave
it (heavy armour and a heavy load cap it, as in [combat](combat.md)), plus every Dodge bonus you
have: from the [Tumble](skills.md#skill-tumble) skill, [talents](talents.md), [magic](magic-items.md),
your [stance](stances.md#stances), and your state. The chance is 55% at a dodge of 0. Each point
above that adds 10% and each point below takes 10% away. It is never less than **15%** and never
more than **90%**, so it is always a gamble.

{{#include generated/trap-avoid.md}}

Only a trap you know of gives this chance; you do not know to step lightly round one you have not
found. Neither does a step you did not choose, such as being turned back at a [shop's](shops.md)
door, and creatures never try. Avoiding a trap counts as [dodging](combat.md#defence) for training
[Tumble](skills.md#skill-tumble) and [Dexterity](stats.md#stat-dexterity). A door's trap is not
stepped on: it goes off when the door opens (see [Trapped doors](#trapped-doors)).

The [Light step](talents-body.md#talent-light-step) talent's chance comes on top. If you fail to avoid a
trap, it may still save you, and it works on hidden traps too (see [Talents](talents.md)).

## What traps do

{{#include generated/traps.md}}

### Damage by danger

The damage above is a trap's in the shallowest places. Deeper down traps hit harder. What a trap
does depends on the [danger level](dungeons.md) of the place it lies in, rising in steps, the same
for a trap on the floor, a [door](#trapped-doors), or a [chest](chests.md). A place with no danger
at all counts as 0. At danger 0 to 2 no trap kills a new character at full
[hit points](hit-points.md#hit-points) in one go, even with a [critical hit](combat.md#criticals).
The column is the danger the step starts at.

{{#include generated/trap-damage.md}}

- **Attacks.** An [arrow](traps.md#trap-arrow) or [spears](traps.md#trap-spear) attack like a
  weapon, rolled against the [Defence](combat.md#defence) of whoever stepped on the trap (see
  [Combat](combat.md)). They can miss ("You dodge the arrow.", "The spear glances off your
  armour."), hit, or hit hard. An arrow flies like a shot (see [Ranged combat](ranged.md)): it
  attacks whoever stands in its way, and lands where it falls, ready to be picked up.
- **Blasts.** A [fireball](traps.md#trap-fireball) burns everyone around the trap. A
  [lightning bolt](traps.md#trap-lightning-bolt) strikes everyone along its line. Both hit you too
  if you are there, with no roll [to hit](combat.md#hitting). [Walls](terrain.md#walls) stop both.
  The lightning runs one way: along the longest clear line from the trap, unless the place says
  otherwise.
  - A fireball blows apart the softer [walls](terrain.md#walls) it reaches (see
    [Digging and walls](digging.md)). It melts the [ice](temperature.md#walking-on-ice) it reaches,
    and whoever stands on that ice [falls in](temperature.md#falling-in).
  - A [frost trap](traps.md#trap-frost) bites everyone around it with
    [cold](damage-types.md#damage-cold). It [freezes](temperature.md#water-freezes) the water it
    reaches at once, into ice that holds a laden person (see [Temperature and ice](temperature.md)).
- **[A pit](traps.md#trap-pit)** hurts you in the fall, and you lose your next action climbing out.
  Trained in [Escape artist](skills.md#skill-escape-artist), the climb takes half that from 50 and
  nothing from 100. Each climb trains Escape artist, and half the time
  [Athletics](skills.md#skill-athletics) too, though Athletics does not shorten it.
- **[An alarm](traps.md#trap-alarm)** rings a bell that every creature within
  [earshot](hearing.md#what-makes-noise) hears. Those who have [enemies](creatures.md#factions) come
  to look (see [Hearing](hearing.md)).
- **[A teleport trap](traps.md#trap-teleport)** moves you to another free place on the same level,
  perhaps somewhere you have never seen ("You are wrenched elsewhere.").
- **Charges.** Some traps go off only once or twice. Then they are spent and gone.
- **The same spells** come on [scrolls](scrolls.md). A scroll of
  [fireball](scrolls.md#scroll-of-fireball), [lightning](scrolls.md#scroll-of-lightning), or
  [frost](scrolls.md#scroll-of-frost) casts what the trap does, where you aim it.

The [Trap sense](talents-exploring.md#talent-trap-sense) talent halves the [damage](combat.md#damage) a trap
does to you, rounded down, at least 1 (see [Talents](talents.md)). Magic that wards off traps takes
its percent off as well (see [Magic items](magic-items.md)), on a creature too. Together they never
take more than **75%** off: a trap always hurts.

A trap's blow wounds like any other. A big one spills [blood](hit-points.md#blood) and can leave you
[bleeding](status-effects.md#bleeding). Traps can kill you as well as creatures. A creature killed
by a trap drops what it carried, as from any death. But a trap's kill earns you no
[experience](experience.md) and counts for no [quest](quests.md), even if you lured the creature
there. A trap you [rigged](#rigging-traps) yourself is the exception. If a trap kills you, the death screen names it ("Killed by a fireball trap"). Someone who
makes way for you onto a trap you stood on sets it off too.

No trap lies on water that can [freeze](temperature.md#water-freezes), and a
[teleport](traps.md#trap-teleport) never sets you down on [ice](temperature.md#walking-on-ice).

## Disarming traps

If you are trained in [Disable device](skills.md#skill-disable-device) ([thieves](class-thief.md)
are), you can try to take apart a trap you know of, on the floor or on a door. Stand next to it and
choose [Disarm a trap](abilities.md#ability-disarm) in the [abilities menu](abilities.md) (`a`),
then the direction. A trap you have not found cannot be disarmed. Trying where none is known only
says "You know of no trap there.", whether one is hidden there or not, and takes no time.

A try is a [check](diseases.md#checks): two twelve-sided dice, plus your
[Dexterity](stats.md#stat-dexterity) modifier, plus what your
[Disable device](skills.md#skill-disable-device) gives (+8 at 100, +12 at 200; see
[Skills](skills.md)). It is rolled against the trap's **difficulty**, the same number that finding
it takes. Deeper in a dungeon, both are harder.

- **Reach it** and the trap is gone for good ("You disarm the arrow trap.").
- **Fall short** and it stays ("You fail to disarm the arrow trap.").
- **Fall short by 5 or more** and it goes off at you ("You set off the arrow trap!"), as if you had
  set it off where you stand. The [arrow](traps.md#trap-arrow) shoots at you from its slit, the
  [spears](traps.md#trap-spear) strike at you, the [fireball](traps.md#trap-fireball) bursts around
  the trap, the [lightning](traps.md#trap-lightning-bolt) runs from it through you, you tumble into
  the [pit](traps.md#trap-pit) and lose your next action climbing back out, the
  [alarm](traps.md#trap-alarm) rings, or you are wrenched elsewhere. A trap with charges spends one.

A natural 2 always fails and a natural 24 always succeeds.

Every try takes **12 seconds** at a normal pace, whatever comes of it, and may train
[Disable device](skills.md#skill-disable-device). You can try as often as you like. Nothing
remembers a failed try; time, and the risk, are the only price. Disarming earns no
[experience](experience.md).

{{#include generated/trap-disarm.md}}

## Rigging traps

A [craftsperson](class-craftsperson.md) of level 25 can rig traps of their own, with
[Trapwright](class-craftsperson.md#talent-trapwright). Choose it in the [abilities menu](abilities.md), pick a
trap from the list, then the way: the trap goes on the cell beside you in that direction. The list
greys out a trap you lack something for, and says what.

{{#include generated/rigs.md}}

- **Where.** On a level, never the [overworld](overworld.md). The cell must be open floor with
  nobody on it, and no trap, [chest](chests.md), door, [stairs](terrain.md#stairs), shop floor,
  water, or [ice](temperature.md#walking-on-ice). A tripwire also needs a wall near, for its arrow
  to shoot from.
- **The work** takes the time above. A creature coming into view, or anything that interrupts
  you, stops it; then nothing is rigged and nothing used up, but the time is gone. If a hidden trap
  was there all along, you find it instead.
- **The check** comes at the end. [Woodcraft](skills.md#skill-woodcraft) helps, but you rig without
  it too. Made, the trap is in place, and the arrows or the billet are used up (a digging tool and
  a knife are kept). Failed, nothing is rigged and you keep what you had. On the worst roll the
  trap goes off at you as you set it, and stays with a charge less.
- **Your own trap** is [known](#finding-traps) to you: stepping onto it asks first, and you may
  [step past it](#stepping-past-a-known-trap). Creatures know nothing of it, and neither do your
  [allies](allies.md): keep your hound away from it. You can [disarm](#disarming-traps) it like any
  other.
- **Its kills are yours.** A creature your trap kills earns you its
  [experience](experience.md) and counts for your [quests](quests.md), as if your own arrow had
  killed it. Its damage never grows with [danger](#damage-by-danger).

## Trapped doors

A trap may lie on a door instead of the floor. It belongs to the door: walking through the doorway
never sets it off, and stepping into it never asks.

It goes off when someone **opens** the door, you or a creature that opens doors. That means `o`,
walking into it, unlocking it with a [key](doors.md#keys), or [picking its lock](doors.md#picking-locks).
It also goes off at every **[kick](doors.md#kicking-a-door-open)** at the door, whether the door breaks or
holds. It strikes whoever opened or kicked it, from the door. An [arrow](traps.md#trap-arrow)
shoots at them from the wall beside the door, a blade strikes at them, lightning runs from the door
through them. A [fireball](traps.md#trap-fireball) bursts out on both sides of the door, even while
it is still shut. [Pits](traps.md#trap-pit) are never on doors.

The trap stays with the door. Shut the door again and the trap is set again, ready for the next one
who opens it, unless it has run out of charges. A door you kick in takes its trap with it.
[Searching](searching.md#how-to-search) finds a door's trap like any hidden trap ("You notice that the
door is trapped: an arrow trap."). Once you know it, it shows as `^` in place of the door. A
[secret door](doors.md#secret-doors) is never trapped. A door's trap can be disarmed like any other
(see [Disarming traps](#disarming-traps)).

## Trapped chests

A shut [chest](chests.md) may be trapped too, with any trap a door may have (never a
[pit](traps.md#trap-pit)). No traps are made only for chests. The trap goes off when you open the
chest, kick it open, or fail badly to disarm it. A **[fireball](traps.md#trap-fireball) or
[frost](traps.md#trap-frost)** trap goes off inside the box and **blows the chest apart**, taking
much of what it held with it. See [Chests](chests.md#trapped-chests).

## Where traps are

- **[Generated dungeons](dungeons.md)** hide a few traps on every level. They are never on or next
  to the [stairs](terrain.md#stairs), and never where a creature walks its rounds. Some
  [rooms](special-rooms.md) hide their own: an offering before an [altar](special-rooms.md#room-altar)
  may not be unguarded.
- **Cēaptūn** and the [overworld](overworld.md) have no traps.

See also: [Searching](searching.md), [Doors](doors.md), and [Chests](chests.md).
