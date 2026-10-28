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

You never see what a shut chest holds, nor whether it is locked or trapped, until you try it
(or search it).

## Where chests are found

About one dungeon floor in five has a chest standing somewhere out in the open: in the middle of
a room or a cavern, never in a doorway or a corridor, never by the stairs, in a shop, or in a
vault. Each dungeon says how often:

{{#include generated/chest-places.md}}

A [treasure room](special-rooms.md) holds one or two more, behind its secret doors; a greater
treasure room, walled up with no way in but through a wall, two to four.

A chest found on a floor is of that floor's level. Many come locked, and the deeper the more
often; some are trapped, with one of the traps that dungeon hides on its floors (a dungeon
without traps has no trapped chests):

{{#include generated/chest-odds.md}}

## What a chest holds

A chest holds a few things rolled by dice, a purse of gold coins, and now and then loose gems.
Its items are found as loot is anywhere (some of them may turn out to be gems), at the floor's
level or a few levels above it for the greater chests, and a share of them is sure to be magic
or rare. Talents that make your finds richer (Treasure hunter, Hoard sense) count for what is in
a chest too.

{{#include generated/chest-loot.md}}

A coffer is about a floor's worth of loot and a handful of coins; a banded chest is two floors'
worth with a quarter of it magic. The coins and gems are what make a kick worth it: a boot
spares them, a fireball does not (below).

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

## Trapped chests

Some chests are **trapped**, with the same traps doors have (an arrow, spears, a fireball, frost,
lightning, an alarm bell, a teleport; never a pit). The trap is hidden, and goes off at you:

- when you **open** the chest, with a key, without one, or after picking its lock (Knock only
  springs the lock: the trap waits for the lid);
- when your **kick breaks it** open, as the kick lands (a kick that holds sets nothing off);
- when you **fail badly to disarm** it.

It comes out of the chest at whoever is beside it: an arrow shoots out of the box itself, spears
strike at you, lightning runs from the chest through you, a fireball or frost bursts around the
chest. Walking past never sets it off.

A **fireball or frost trap goes off inside the box** and **blows the chest apart**, however it was
set off: `The coffer is blown apart!` Each stack inside breaks with an even chance and **nothing is
spared**, coins and gems included, as in a blast. The other traps leave the chest as they found
it: open if you opened it, smashed if you kicked it.

```
You open the coffer.
A fireball explodes!
The fireball burns you.
The coffer is blown apart!
The 12 gold coins are destroyed.
Inside you find a dagger.
```

A chest blown apart by a blast from somewhere else (a scroll of fireball, another trap) springs
its own trap at no one: the trap is simply gone with the lid.

Searching finds a chest's trap like any hidden trap (`You notice that the coffer is trapped: an
arrow trap.`), and once you know it look mode says so: "A coffer, trapped: an arrow trap." The
chest keeps its own look on the map. A known trap on a chest can be **disarmed** like any (see
[Traps](traps.md#disarming-traps)): disarmed, the chest is an ordinary shut chest. **Trap sense**
halves a chest trap's blow, as any trap's.

## The Chest-wise talent

With the **Chest-wise** talent (Intelligence 11 or Search 15), you tell at a glance whether a shut
chest you see is locked: look mode calls it "a locked coffer" or "an unlocked coffer". And a
trapped chest gives itself away as soon as you see it, with no search and no chance of missing it:
`You notice that the coffer is trapped: a fireball trap.`
