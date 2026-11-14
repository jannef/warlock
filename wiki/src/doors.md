# Doors

A closed door (`+`) blocks your way, your [sight](sight.md#line-of-sight), and
[light](light.md#what-light-is), and muffles [sound](hearing.md). To open it, walk into it or press
`o` and a direction. To close an open door (`'`), press `c` and a direction. See
[Terrain](terrain.md#doors) for how doors block and are remembered.

## Locked doors

Some doors are **locked**. A locked door looks exactly like any other closed door, and you only
find out when you try to open it:

```
The door is locked.
```

Trying takes no time. To get through, you need its [key](#keys), a thief's touch
([picking the lock](#picking-locks)), or a strong leg ([kicking it in](#kicking-a-door-open)).

## Keys

Keys are [tools](objects.md#class-tools) (`(`) you carry. Each key is made for one kind of lock.
A **skeleton key** opens every lock. Some locks have no key of their own: only a skeleton key opens
them.

When you open a locked door (`o`, or walking into it) while you carry a key that fits, you unlock
it and open it in one go:

```
You unlock the door with your iron key and open it.
```

That takes as long as unlocking and opening together: 12 seconds at a normal pace (see
[Turns and movement](turns.md)). You keep the key, and the door stays unlocked, so after you close
it, it opens like any door. If you carry both the key made for a lock and a skeleton key, you use
the one made for it.

{{#include generated/keys.md}}

## Picking locks

If you are trained in the [Open lock](skills.md#skill-open-lock) skill
([thieves](class-thief.md) are), the [abilities menu](abilities.md#the-abilities-menu) (`a`) has
**Pick lock**. Choose it, then a direction, and you try the lock of the closed door there without a
key.

- **A try** is a check: two twelve-sided dice, plus your [Dexterity](stats.md#stat-dexterity)
  [modifier](stats.md#modifiers), plus what [Open lock](skills.md#skill-open-lock) gives (+8 at 100,
  see [Skills](skills.md)), against the lock's **difficulty**. The modifier is half of what your
  Dexterity is over 10, rounded down: 14 gives +2, 18 gives +4, 8 gives −1. Two ones always fail
  and two twelves always work.
- **Locks** are of difficulty 14 unless the place says otherwise; a vault's or a level's lock may
  be harder or easier. In a [generated dungeon](dungeons.md), each level down makes a lock one
  harder: 15 on the second level, 21 on the eighth.
- **A try takes 12 seconds** at a [normal pace](turns.md#what-actions-cost), whatever comes of it.
  If it fails, `You fail to pick the lock.` and the door stays locked. You may try again as often
  as you like. Only time is lost, and nothing about the lock changes.
- **When it works**, the lock is gone and you open the door at once, as a key would:
  `You pick the lock and open the door.` That takes another 6 seconds, as opening a door does. The
  door stays unlocked.
- **A trapped door** sets its trap off at you as you open it, picked or not (see
  [Traps](traps.md#trapped-doors)). A failed try sets nothing off.
- Every try may earn [Open lock](skills.md#skill-open-lock) a
  [training mark](skills.md#improving-a-skill). Trying a door that is not locked, or anything else,
  takes no time: `That door is not locked.`, `There is no lock there.` Without the skill:
  `You do not know how to pick locks.`

How often a try works, by the lock's difficulty and what you add (your
[Dexterity](stats.md#stat-dexterity) [modifier](stats.md#modifiers) plus
[Open lock's](skills.md#skill-open-lock) bonus):

{{#include generated/lock-picking.md}}

A force spell's blast breaks every shut door it reaches, locked or trapped, however sturdy (see
[spell types](spell-types.md)).

## Kicking a door open

Press `k` and a direction to kick. A kick takes a normal action (6 seconds).

- **A closed door**, locked or not, may break. The chance is your
  [Strength](stats.md#stat-strength) against the door's sturdiness. Against a sturdiness of 30,
  Strength 9 breaks it three kicks in ten, Strength 15 one in two, and Strength 30 every time.
  Your [Athletics](skills.md#skill-athletics) skill adds to your Strength for this (see
  [Skills](skills.md)), and kicking doors trains it. The
  [Lockbreaker](talents.md#talent-lockbreaker) talent adds 6 (see [Talents](talents.md)), and magic
  boots that kick harder add their number (see [Magic items](magic-items.md)), for doors and
  [chests](chests.md) alike. Kicking a door takes its 6 seconds whatever your
  [stance](stances.md#stances).

  A door that holds says `WHAMM!`. One that breaks says `The door crashes open!` and stays a
  **broken door** (`,`, splinters in the doorway) for good. You can walk and see through it, and
  it can never be closed or locked again.
- **Every kick at a door is [loud](hearing.md#how-loud).** It bangs on the door itself, so it is
  heard on both sides, and creatures that hear it come to look (see [Hearing](hearing.md)).
- **A creature** you kick takes an unarmed blow, whatever you wield ("You kick the giant rat.").
  Your bare foot hits like a fist, with your [Strength](stats.md#stat-strength) (see
  [Combat](combat.md)), and uses and trains your
  [Unarmed](weapon-skills.md#weapon-skill-unarmed) skill (see [Weapon skills](weapon-skills.md)).
  Kicking a creature that is not [hostile](creatures.md#factions) attacks it all the same. Like
  every attack, a kick takes longer in a cowardly [stance](stances.md#stances) and less time with
  magic that quickens your attacks.
- **A statue** of something turned to stone can be kicked too, but it is far sturdier than a door
  (the table's last row). Most kicks only thud against the stone, though each trains you as a
  door's does, [Strength](stats.md#stat-strength) with it. Broken, it falls to rubble and leaves
  what it held.
- **Anything else** takes no time. Where you could walk, an open or broken door included, it says
  `You kick at empty space.` Kicking a wall would only hurt your foot.

{{#include generated/doors.md}}

These chances follow your [Strength](stats.md#stat-strength) score itself, which never shows as a
bonus anywhere in the game (see [Stats](stats.md)).

Chests are kicked open the same way, and their locks are picked and opened as a door's (see
[Chests](chests.md)).

## Wedging a door

Anyone trained in [Woodcraft](skills.md#skill-woodcraft) can **wedge** a closed door shut, which
is handy for a rest or a retreat. Choose [Wedge a door](abilities.md#ability-wedge) in the
abilities menu and a direction. It takes a turn (a [normal action](turns.md#what-actions-cost)),
and the door stays wedged until it is opened or broken.

- **A creature that opens doors cannot open a wedged one.** It kicks it instead, one kick a turn.
  Its chance to break the door is its [Strength](stats.md#stat-strength) against the door's
  sturdiness plus the wedge's points, as when you kick a locked door in. Woodcraft makes the wedge
  hold better: +8 at Woodcraft 100, +12 at 200, nothing at 0. Your own kicks are rolled against the
  same.
- **You open it as usual**: `You pull the wedge free and open the door.` The wedge is not an item.
- A wedge that is refused takes no time and says why: you lack Woodcraft, there is no closed door
  that way, or the door is already wedged.
- Every wedge earns [Woodcraft](skills.md#skill-woodcraft) a [training mark](skills.md#improving-a-skill).

## Secret doors

Some doors are **secret**. They look like the wall around them, and are that wall to you, until
you find them by [searching](searching.md#how-to-search) or
[in passing](searching.md#noticing-in-passing) (see [Searching](searching.md)). A secret door is
never [locked](#locked-doors). Once found, it is a closed door like any other, and you
[remember](sight.md#memory) it.

## Trapped doors

Some doors are **trapped**. Opening or kicking one sets off the [trap](traps.md) on it; walking
through the open doorway does not. See [Traps](traps.md#trapped-doors).

## Creatures and doors

Some creatures open closed doors: [kobolds](creatures.md#creature-kobold) do,
[rats](creatures.md#creature-rat) do not. A locked door stops them unless they carry a
[key](#keys) that fits it; then they unlock it as they open it, as you do. A
[kobold](creatures.md#creature-kobold) guard carrying the key to the room it guards will walk right
through. No creature picks locks, and none kicks a door unless it is wedged (see
[Wedging a door](#wedging-a-door)).

## Where you find them

- In a [generated dungeon](dungeons.md), a few rooms with a single door are locked. The level's
  [rusty key](#keys) always lies somewhere you can reach without opening a locked door, and you
  never need a key to find the [stairs](terrain.md#stairs).
- A guard post in a dungeon, with its guard and company, keeps its store room locked. The
  [kobold](creatures.md#creature-kobold) on guard carries the brass key, and a
  [skeleton key](#keys) lies in the store room.
