# Doors

A closed door (`+`) blocks your way, your [sight](sight.md#line-of-sight), and [light](light.md#what-light-is), and muffles [sound](hearing.md). Walk into it, or
press `o` and a direction, to open it; `c` and a direction closes an open door (`'`) again. See
[Terrain](terrain.md#doors) for how doors block and are remembered.

## Locked doors

Some doors are **locked**. A locked door looks exactly like any other closed door, and you only
find out when you try to open it:

```
The door is locked.
```

Trying takes no time. To get through, you need its [key](#keys), a thief's touch ([picking the lock](#picking-locks),
below), or a strong leg ([kicking it in](#kicking), further below).

## Keys

Keys are [tools](objects.md#class-tools) (`(`) you carry. Each key is made for one kind of lock, and a **skeleton key**
opens every lock. Some locks take no key at all: only a skeleton key opens them.

When you open a locked door (`o`, or walking into it) while you carry a key that fits, you unlock
it and open it in one go:

```
You unlock the door with your iron key and open it.
```

That takes as long as unlocking and opening together, 12 seconds at a normal pace (see
[Turns and movement](turns.md)). You keep the key, and the door stays unlocked: close it again
and it opens like any door. If you carry both the key made for a lock and a skeleton key, you use
the one made for it.

{{#include generated/keys.md}}

## Picking locks

If you are trained in the **[Open lock](skills.md#skill-open-lock)** skill ([thieves](classes.md#thief) are), the [abilities menu](abilities.md#the-abilities-menu) (`a`) has **Pick
lock**: choose it, then a direction, and you try the lock of the closed door there, without a key.

- **A try** is a check: two twelve-sided dice, plus your [Dexterity](stats.md#stat-dexterity) [modifier](stats.md#modifiers) (half of what your
  Dexterity is over 10, rounded down: 14 gives +2, 18 gives +4, 8 gives −1), plus what [Open lock](skills.md#skill-open-lock)
  gives (+8 at 100, see [Skills](skills.md)), against the lock's **difficulty**. Two ones always
  fail and two twelves always work.
- **Locks** are of difficulty 14 unless the place says otherwise; a vault's or a level's lock may
  be harder or easier. In a [generated dungeon](dungeons.md), every level down makes a lock one harder: 15 on the second
  level, 21 on the eighth.
- **A try takes 12 seconds** at a [normal pace](turns.md#what-actions-cost), whatever comes of it. If it fails, `You fail to pick
  the lock.` and the door stays locked. You may try again as often as you like: only time is
  lost, and nothing about the lock changes.
- **When it works**, the lock is gone and you open the door at once, as a key would:
  `You pick the lock and open the door.` That takes another 6 seconds, as opening a door does.
  The door stays unlocked: close it again and it opens like any door.
- **A trapped door** sets its trap off at you as you open it, picked or not (see
  [Traps](traps.md#trapped-doors)). A try that fails sets nothing off.
- Every try may earn [Open lock](skills.md#skill-open-lock) a [training mark](skills.md#getting-better). Trying a door that is not locked, or anything
  else, takes no time: `That door is not locked.`, `There is no lock there.` Without the skill,
  `You do not know how to pick locks.`

How often a try works, by the lock's difficulty and what you add (your [Dexterity](stats.md#stat-dexterity) [modifier](stats.md#modifiers) plus
[Open lock's](skills.md#skill-open-lock) bonus):

{{#include generated/lock-picking.md}}

## Kicking

Press `k` and a direction to kick. A kick takes a normal action (6 seconds).

- **A closed door**, locked or not, may break. The chance is your [Strength](stats.md#stat-strength) in the door's
  sturdiness: against a sturdiness of 30, Strength 9 breaks it three kicks in ten, Strength 15
  one in two, and Strength 30 every time. Your **[Athletics](skills.md#skill-athletics)** skill adds to your Strength for this
  (see [Skills](skills.md)), and kicking doors trains it. So do the **[Lockbreaker](talents.md#talent-lockbreaker)** talent (+6,
  see [Talents](talents.md)) and magic boots that kick harder (their number, see
  [Magic items](magic-items.md)), for doors and [chests](chests.md) alike. Kicking a door takes its 6 seconds
  whatever your [stance](combat.md#stances). A door that holds says `WHAMM!`; one that breaks says
  `The door crashes open!` and stays a **broken door** (`,`, splinters in the doorway) for good:
  you walk and see through it, and it can never be closed or locked again.
- **Every kick at a door is [loud](hearing.md#how-loud).** It bangs on the door itself, so it is heard on both sides,
  and creatures that hear it come to look (see [Hearing](hearing.md)).
- **A creature** you kick takes an unarmed blow, whatever you wield: your bare foot hits like a
  fist, with your [Strength](stats.md#stat-strength) (see [Combat](combat.md)), and draws on and trains your **[Unarmed](weapon-skills.md#weapon-skill-unarmed)**
  skill (see [Weapon skills](weapon-skills.md)). "You kick the giant rat." Kicking a creature
  that is not [hostile](creatures.md#factions) attacks it all the same. Like every attack, it takes longer in a cowardly
  [stance](combat.md#stances) and less time with magic that quickens your attacks.
- **Anything else** takes no time: `You kick at empty space.` where you could walk (an open or
  broken door too), and kicking a wall would only hurt your foot.

{{#include generated/doors.md}}

Your [Strength](stats.md#stat-strength) never shows as a bonus anywhere in the game: these chances follow your Strength
score itself (see [Stats](stats.md)).

Chests are kicked open the same way, and their locks are picked and opened as a door's: see
[Chests](chests.md).

## Secret doors

Some doors are **secret**: they look like the wall around them, and are that wall, until you find
them, [searching](searching.md#searching-1) or [in passing](searching.md#noticing-in-passing) (see [Searching](searching.md)). A secret door is never [locked](#locked-doors).
Once found, it is a closed door like any other, and you [remember](sight.md#memory) it.

## Trapped doors

Some doors are **trapped**: opening one, or kicking it, sets off the [trap](traps.md) on it, while walking
through the open doorway does not. See [Traps](traps.md#trapped-doors).

## Creatures and doors

Some creatures open closed doors ([kobolds](creatures.md#creature-kobold) do; [rats](creatures.md#creature-rat) do not). A locked door stops them unless they
carry a [key](#keys) that fits it: then they unlock it as they open it, as you do. A [kobold](creatures.md#creature-kobold) guard carrying
the key to the room it guards will walk right through. No creature kicks doors or picks locks
(yet).

## Where you find them

- In a [generated dungeon](dungeons.md), a few rooms with a single door are locked. The level's [rusty key](#keys)
  always lies somewhere you can reach without opening a locked door, and you never need one to
  find the [stairs](terrain.md#stairs).
- A guard post in a dungeon keeps its store room locked; the [kobold](creatures.md#creature-kobold) on guard carries the brass
  key. A [skeleton key](#keys) lies in the store room.
