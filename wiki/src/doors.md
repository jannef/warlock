# Doors

A closed door (`+`) blocks your way, your sight, and light, and muffles sound. Walk into it, or
press `o` and a direction, to open it; `c` and a direction closes an open door (`'`) again. See
[Terrain](terrain.md#doors) for how doors block and are remembered.

## Locked doors

Some doors are **locked**. A locked door looks exactly like any other closed door, and you only
find out when you try to open it:

```
The door is locked.
```

Trying takes no time. To get through, you need its key, a thief's touch (picking the lock,
below), or a strong leg (kicking it in, further below).

## Keys

Keys are tools (`(`) you carry. Each key is made for one kind of lock, and a **skeleton key**
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

If you are trained in the **Open lock** skill (thieves are), the abilities menu (`a`) has **Pick
lock**: choose it, then a direction, and you try the lock of the closed door there, without a key.

- **A try** is a check: two twelve-sided dice, plus your Dexterity modifier (half of what your
  Dexterity is over 10, rounded down: 14 gives +2, 18 gives +4, 8 gives −1), plus what Open lock
  gives (+8 at 100, see [Skills](skills.md)), against the lock's **difficulty**. Two ones always
  fail and two twelves always work.
- **Locks** are of difficulty 14 unless the place says otherwise; a vault's or a level's lock may
  be harder or easier. In the Depths, every level down makes a lock one harder: 15 on the second
  level, 21 on the eighth.
- **A try takes 12 seconds** at a normal pace, whatever comes of it. If it fails, `You fail to pick
  the lock.` and the door stays locked. You may try again as often as you like: only time is
  lost, and nothing about the lock changes.
- **When it works**, the lock is gone and you open the door at once, as a key would:
  `You pick the lock and open the door.` That takes another 6 seconds, as opening a door does.
  The door stays unlocked: close it again and it opens like any door.
- **A trapped door** sets its trap off at you as you open it, picked or not (see
  [Traps](traps.md#trapped-doors)). A try that fails sets nothing off.
- Every try may earn Open lock a training mark. Trying a door that is not locked, or anything
  else, takes no time: `That door is not locked.`, `There is no lock there.` Without the skill,
  `You do not know how to pick locks.`

How often a try works, by the lock's difficulty and what you add (your Dexterity modifier plus
Open lock's bonus):

{{#include generated/lock-picking.md}}

## Kicking

Press `K` and a direction to kick. A kick takes a normal action (6 seconds).

- **A closed door**, locked or not, may break. The chance is your Strength in the door's
  sturdiness: against a sturdiness of 30, Strength 9 breaks it three kicks in ten, Strength 15
  one in two, and Strength 30 every time. Your **Athletics** skill adds to your Strength for this
  (see [Skills](skills.md)), and kicking doors trains it. Kicking a door takes its 6 seconds
  whatever your stance. A door that holds says `WHAMM!`; one that breaks says
  `The door crashes open!` and stays a **broken door** (`,`, splinters in the doorway) for good:
  you walk and see through it, and it can never be closed or locked again.
- **Every kick at a door is loud.** It bangs on the door itself, so it is heard on both sides,
  and creatures that hear it come to look (see [Hearing](hearing.md)).
- **A creature** you kick takes an unarmed blow, whatever you wield: your bare foot hits like a
  fist, with your Strength (see [Combat](combat.md)), and draws on and trains your **Unarmed**
  skill (see [Weapon skills](weapon-skills.md)). "You kick the giant rat." Kicking a creature
  that is not hostile attacks it all the same. Like every attack, it takes longer in a cowardly
  stance and less time with magic that quickens your attacks.
- **Anything else** takes no time: `You kick at empty space.` where you could walk (an open or
  broken door too), and kicking a wall would only hurt your foot.

{{#include generated/doors.md}}

Your Strength never shows as a bonus anywhere in the game: these chances follow your Strength
score itself (see [Stats](stats.md)).

## Secret doors

Some doors are **secret**: they look like the wall around them, and are that wall, until you find
them, searching or in passing (see [Searching](searching.md)). A secret door is never locked.
Once found, it is a closed door like any other, and you remember it.

## Trapped doors

Some doors are **trapped**: opening one, or kicking it, sets off the trap on it, while walking
through the open doorway does not. See [Traps](traps.md#trapped-doors).

## Creatures and doors

Some creatures open closed doors (kobolds do; rats do not). A locked door stops them unless they
carry a key that fits it: then they unlock it as they open it, as you do. A kobold guard carrying
the key to the room it guards will walk right through. No creature kicks doors or picks locks
(yet).

## Where you find them

- In the hub, the pantry off the door corridor has a locked west door; its iron key lies in the
  storeroom across the corridor. (Its south door is not locked.)
- In the Depths, a few rooms with a single door are locked. The level's rusty key always lies
  somewhere you can reach without opening a locked door, and you never need one to find the
  stairs.
- A guard post in the Depths keeps its store room locked; the kobold on guard carries the brass
  key. A skeleton key lies in the store room.
