# Chests

A **chest** stands on its own spot on the floor, like a door in a doorway: shut (`⊟`) it is in
your way, open (`⊔`) or smashed (`⊠`) you can step onto it, and what it held lies there. Chests
are never picked up and carried.

## What a chest is called

Every chest has a level, the level of the place it stands in (the [dungeon floor](dungeons.md)'s depth), and its
level gives its name, its colour, and how sturdy it is. [Look](controls.md#look-mode) at one (`l`) to see what it is: a
**coffer** on the first floors, a **hutch** or a **banded chest** deeper, a **strongbox** deeper
still. The old names are for old things:

{{#include generated/chest-descriptions.md}}

You never see what a shut chest holds, nor whether it is [locked](#locked-chests) or [trapped](#trapped-chests), until you try it
(or [search it](searching.md#searching-1)).

## Where chests are found

About one [dungeon floor](dungeons.md) in five has a chest standing somewhere out in the open: in the middle of
a room or a cavern, never in a [doorway](doors.md) or a corridor, never by the [stairs](terrain.md#stairs), in a [shop](shops.md), or in a
vault. Each dungeon says how often:

{{#include generated/chest-places.md}}

A [treasure room](special-rooms.md) holds one or two more, behind its [secret doors](doors.md#secret-doors); a greater
[treasure room](special-rooms.md#room-treasury), walled up with no way in but through a wall, two to four. A few of the dungeons'
own halls and hoards keep chests of their own besides, [locked](#locked-chests) or [trapped](#trapped-chests) as their owners left
them; those are of the level of the floor they stand on too, and are never [mimics](mimics.md).

A chest found on a floor is of that floor's level. Many come [locked](#locked-chests), and the deeper the more
often; some are [trapped](#trapped-chests), with one of the [traps](traps.md) that dungeon hides on its floors (a dungeon
without traps has no trapped chests); and a few are no chest at all but a
[mimic](mimics.md), never locked or trapped:

{{#include generated/chest-odds.md}}

## What a chest holds

A chest holds a few things rolled by dice, a purse of [gold coins](shops.md#money), and now and then loose [gems](gems.md).
Its items are found as [loot](magic-items.md#finding-them) is anywhere (some of them may turn out to be gems), at the floor's
level or a few levels above it for the greater chests, and a share of them is sure to be [magic](magic-items.md)
or [rare](magic-items.md#rarity). Talents that make your finds richer ([Treasure hunter](talents.md#talent-treasure-hunter), [Hoard sense](talents.md#talent-hoard-sense)) count for what is in
a chest too.

{{#include generated/chest-loot.md}}

A coffer is about a floor's worth of loot and a handful of coins; a banded chest is two floors'
worth with a quarter of it [magic](magic-items.md). The coins and gems are what make a kick worth it: a boot
spares them, a fireball does not (below).

## Opening a chest

Walk into a shut chest, or press `o` and its direction, to lift the lid:

```
You open the coffer.
Inside you find a dagger and 12 gold coins.
```

That takes 6 seconds at a normal pace. Whatever it held now lies in the open chest, as a [pile](objects.md#piles) you
[pick up](objects.md#picking-up) as any (step in and press `g`). Nothing in it is [identified](magic-items.md#unidentified-items) for you. An open chest stays
open.

## Locked chests

Many chests are **locked**:

```
The coffer is locked.
```

Trying takes no time. Only a **[skeleton key](doors.md#keys)** opens a chest's lock, unless the place gave that
chest a lock of its own (then its key opens it too), and unlocking and opening take 12 seconds
together:

```
You unlock the coffer with your skeleton key and open it.
```

Without a key:

- **[Pick lock](abilities.md#ability-pick-lock)** works on a chest's lock exactly as on a door's (see
  [Doors](doors.md#picking-locks)): `You pick the lock and open the coffer.`
- **[Knock](abilities.md#ability-knock)**, the [wizard](classes.md#wizard)'s word of unbinding (from level 4, 5 [mana](mana.md#how-much), the time of a spell; see
  [Classes](classes.md)): choose it in the [abilities menu](abilities.md#the-abilities-menu) (`a`) and a direction, and the lock
  there, of a chest or a door, springs open: `The lock springs open with a click.` Nothing opens
  by itself: open it afterwards as an unlocked one. Where nothing locked stands, Knock is refused
  and costs nothing: `There is no lock there to spring.`
- **Kick it open** (below).

## Kicking a chest open

Press `k` and a chest's direction to kick it, as you [kick a door](doors.md#kicking) (6 seconds, and just as [loud](hearing.md#how-loud)). A
chest breaks with a chance of your [Strength](stats.md#stat-strength) in its **sturdiness**; your [Athletics](skills.md#skill-athletics) skill and the
[Lockbreaker](talents.md#talent-lockbreaker) talent add to your Strength for this, as for a door. A chest that holds says
`You kick the coffer. It holds.`, and nothing else happens: kick as often as you like.

A chest that breaks is **smashed**: `The coffer bursts apart!` Its lock is gone, and **each [stack](objects.md#piles)
inside breaks with an even chance**, one after another; the rest lies in the splinters. **Coins
and loose [gems](gems.md) never break** under a boot. The **[Deft breaker](talents.md#talent-deft-breaker)** talent (Strength 12) halves the
chance.

```
The coffer bursts apart!
The potion of healing and the scroll of fireball are destroyed.
Inside you find a dagger and 12 gold coins.
```

{{#include generated/chest-bands.md}}

## Blasts

A [blast](digging.md#explosions) (a [fireball trap's](traps.md#trap-fireball), a [scroll of fireball](scrolls.md#scroll-of-fireball)'s) that reaches a shut chest and is stronger than
its **hardness** (the last column above) **blows it apart**: `The coffer is blown apart!` Each
[stack](objects.md#piles) inside breaks with the same even chance, and **nothing is spared**: the coins and the [gems](gems.md)
too. A fireball's blast (1000) blows apart anything up to a strongbox; the great chests outlast
it.

{{#include generated/chest-breaking.md}}

You cannot [dig into](digging.md#digging) a chest.

## Trapped chests

Some chests are **trapped**, with the same [traps](traps.md) [doors](doors.md#trapped-doors) have (an [arrow](traps.md#trap-arrow), [spears](traps.md#trap-spear), a [fireball](traps.md#trap-fireball), [frost](traps.md#trap-frost),
[lightning](traps.md#trap-lightning-bolt), an [alarm bell](traps.md#trap-alarm), a [teleport](traps.md#trap-teleport); never a [pit](traps.md#trap-pit)). The trap is hidden, and goes off at you:

- when you **open** the chest, with a key, without one, or after picking its lock ([Knock](abilities.md#ability-knock) only
  springs the lock: the trap waits for the lid);
- when your **kick breaks it** open, as the kick lands (a kick that holds sets nothing off);
- when you **fail badly to [disarm](traps.md#disarming-traps)** it.

It comes out of the chest at whoever is beside it: an [arrow](traps.md#trap-arrow) shoots out of the box itself, [spears](traps.md#trap-spear)
strike at you, [lightning](traps.md#trap-lightning-bolt) runs from the chest through you, a [fireball](traps.md#trap-fireball) or [frost](traps.md#trap-frost) bursts around the
chest. Walking past never sets it off.

A **[fireball](traps.md#trap-fireball) or [frost trap](traps.md#trap-frost) goes off inside the box** and **blows the chest apart**, however it was
set off: `The coffer is blown apart!` Each [stack](objects.md#piles) inside breaks with an even chance and **nothing is
spared**, coins and [gems](gems.md) included, as in a blast. The other traps leave the chest as they found
it: open if you opened it, smashed if you kicked it.

```
You open the coffer.
A fireball explodes!
The fireball burns you.
The coffer is blown apart!
The 12 gold coins are destroyed.
Inside you find a dagger.
```

A chest blown apart by a blast from somewhere else (a [scroll of fireball](scrolls.md#scroll-of-fireball), another [trap](traps.md)) springs
its own trap at no one: the trap is simply gone with the lid.

[Searching](searching.md#searching-1) (and your **[Detect traps](skills.md#skill-detect-traps)** skill) finds a chest's trap like any [hidden trap](traps.md#finding-traps) (`You notice that the coffer is trapped: an
arrow trap.`), and once you know it [look mode](controls.md#look-mode) says so: "A coffer, trapped: an arrow trap." The
chest keeps its own look on the map. A known trap on a chest can be **disarmed** like any (see
[Traps](traps.md#disarming-traps)): disarmed, the chest is an ordinary shut chest. **[Trap sense](talents.md#talent-trap-sense)**
halves a chest trap's blow, as any trap's.

## The Chest-wise talent

With the **[Chest-wise](talents.md#talent-chest-wise)** talent ([Intelligence](stats.md#stat-intelligence) 11 or [Detect traps](skills.md#skill-detect-traps) 15), you tell at a glance whether a shut
chest you see is locked: [look mode](controls.md#look-mode) calls it "a locked coffer" or "an unlocked coffer". And a
trapped chest gives itself away as soon as you see it, with no search and no chance of missing it:
`You notice that the coffer is trapped: a fireball trap.`
