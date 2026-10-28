# Chests

A **chest** stands on its own spot on the floor, like a door in a doorway: shut (`⊟`) it is in
your way, open (`⊔`) or smashed (`⊠`) you can step onto it, and what it held lies there. Chests
are never picked up and carried.

## What a chest is called

Every chest has a level, the level of the place it stands in (the dungeon floor's depth), and its
level gives its name, its colour, and how sturdy it is. Look at one (`l`) to see what it is: a
**coffer** on the first floors, a **hutch** or a **banded chest** deeper, a **strongbox** deeper
still. The old names are for old things:

{{#include generated/chest-descriptions.md}}

You never see what a shut chest holds, nor whether it is locked, until you try it.

## Opening a chest

Walk into a shut chest, or press `o` and its direction, to lift the lid:

```
You open the coffer.
Inside you find a dagger and 12 gold coins.
```

That takes 6 seconds at a normal pace. Whatever it held now lies in the open chest, as a pile you
pick up as any (step in and press `g`). Nothing in it is identified for you. An open chest stays
open.

## Locked chests

Many chests are **locked**:

```
The coffer is locked.
```

Trying takes no time. Only a **skeleton key** opens a chest's lock, unless the place gave that
chest a lock of its own (then its key opens it too), and unlocking and opening take 12 seconds
together:

```
You unlock the coffer with your skeleton key and open it.
```

Without a key:

- **Pick lock** works on a chest's lock exactly as on a door's (see
  [Doors](doors.md#picking-locks)): `You pick the lock and open the coffer.`
- **Knock**, the wizard's word of unbinding (from level 4, 5 mana, the time of a spell; see
  [Classes](classes.md)): choose it in the abilities menu (`a`) and a direction, and the lock
  there, of a chest or a door, springs open: `The lock springs open with a click.` Nothing opens
  by itself: open it afterwards as an unlocked one. Where nothing locked stands, Knock is refused
  and costs nothing: `There is no lock there to spring.`
- **Kick it open** (below).

## Kicking a chest open

Press `k` and a chest's direction to kick it, as you kick a door (6 seconds, and just as loud). A
chest breaks with a chance of your Strength in its **sturdiness**; your Athletics skill and the
Lockbreaker talent add to your Strength for this, as for a door. A chest that holds says
`You kick the coffer. It holds.`, and nothing else happens: kick as often as you like.

A chest that breaks is **smashed**: `The coffer bursts apart!` Its lock is gone, and **each stack
inside breaks with an even chance**, one after another; the rest lies in the splinters. **Coins
and loose gems never break** under a boot. The **Deft breaker** talent (Strength 12) halves the
chance.

```
The coffer bursts apart!
The potion of healing and the scroll of fireball are destroyed.
Inside you find a dagger and 12 gold coins.
```

{{#include generated/chest-bands.md}}

## Blasts

A blast (a fireball trap's, a scroll of fireball's) that reaches a shut chest and is stronger than
its **hardness** (the last column above) **blows it apart**: `The coffer is blown apart!` Each
stack inside breaks with the same even chance, and **nothing is spared**: the coins and the gems
too. A fireball's blast (1000) blows apart anything up to a strongbox; the great chests outlast
it.

{{#include generated/chest-breaking.md}}

You cannot dig into a chest.

## Telling locked from unlocked

With the **Chest-wise** talent (Intelligence 11 or Search 15), you tell at a glance whether a shut
chest you see is locked: look mode calls it "a locked coffer" or "an unlocked coffer".
