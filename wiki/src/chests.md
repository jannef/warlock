# Chests

A **chest** stands on its own spot on the floor, as a door stands in a doorway. Shut (`⊟`), it is
in your way. Open (`⊔`) or smashed (`⊠`), you can step onto it, and what it held lies there.
Chests are never picked up and carried.

## What a chest is called

Every chest has a level: the level of the place it stands in (the
[dungeon floor](dungeons.md)'s depth). Its level gives its name, its colour, and how sturdy it is.
[Look](controls.md#look-mode) at one (`l`) to see what it is: a **coffer** on the first floors, a
**hutch** or a **banded chest** deeper, a **strongbox** deeper still. The old names are for old
things:

{{#include generated/chest-descriptions.md}}

You never see what a shut chest holds, or whether it is [locked](#locked-chests) or
[trapped](#trapped-chests), until you try it (or [search it](searching.md#searching-1)).

## Where chests are found

About one [dungeon floor](dungeons.md) in five has a chest standing out in the open, in the middle
of a room or a cavern. A chest is never in a [doorway](doors.md) or a corridor, by the
[stairs](terrain.md#stairs), in a [shop](shops.md), or in a vault. Each dungeon says how often:

{{#include generated/chest-places.md}}

A [treasure room](special-rooms.md) holds one or two more, behind its
[secret doors](doors.md#secret-doors). A greater [treasure room](special-rooms.md#room-treasury),
walled up with no way in but through a wall, holds two to four. A few of the dungeons' own halls
and hoards keep chests of their own besides, [locked](#locked-chests) or [trapped](#trapped-chests)
as their owners left them. Those are of the level of the floor they stand on too, and are never
[mimics](mimics.md).

A chest found on a floor is of that floor's level. Many come [locked](#locked-chests), more often
the deeper you go. Some are [trapped](#trapped-chests) with one of the [traps](traps.md) that
dungeon hides on its floors; a dungeon without traps has no trapped chests. A few are no chest at
all but a [mimic](mimics.md), which is never locked or trapped:

{{#include generated/chest-odds.md}}

## What a chest holds

A chest holds a few things rolled by dice, a purse of [gold coins](shops.md#money), and now and
then loose [gems](gems.md). Its items are found as [loot](magic-items.md#finding-them) is anywhere,
so some may turn out to be gems. They are of the floor's level, or a few levels above it for the
greater chests, and a share of them is sure to be [magic](magic-items.md) or
[rare](magic-items.md#rarity). Talents that make your finds richer
([Treasure hunter](talents.md#talent-treasure-hunter), [Hoard sense](horoscope.md#talent-hoard-sense))
count for what is in a chest too.

{{#include generated/chest-loot.md}}

A coffer holds about a floor's worth of loot and a handful of coins. A banded chest holds two
floors' worth, a quarter of it [magic](magic-items.md). The coins and gems are what make kicking a
chest worth it: a boot spares them, a fireball does not (below).

## Opening a chest

Walk into a shut chest, or press `o` and its direction, to lift the lid:

```
You open the coffer.
Inside you find a dagger and 12 gold coins.
```

That takes 6 seconds at a normal pace. Whatever it held now lies in the open chest as a
[pile](objects.md#piles). [Pick it up](objects.md#picking-up) as any other: step in and press `g`.
Nothing in it is [identified](magic-items.md#unidentified-items) for you. An open chest stays open.

## Locked chests

Many chests are **locked**:

```
The coffer is locked.
```

Trying takes no time. Only a [skeleton key](doors.md#keys) opens a chest's lock, unless the place
gave that chest a lock of its own; then its own key opens it too. Unlocking and opening take 12
seconds together:

```
You unlock the coffer with your skeleton key and open it.
```

Without a key:

- **[Pick lock](abilities.md#ability-pick-lock)** works on a chest's lock exactly as on a door's
  (see [Doors](doors.md#picking-locks)): `You pick the lock and open the coffer.`
- **[Knock](abilities.md#ability-knock)** is the [wizard](classes.md#wizard)'s word of unbinding,
  from level 4, for 5 [mana](mana.md#how-much) and the time of a spell (see [Classes](classes.md)).
  Choose it in the [abilities menu](abilities.md#the-abilities-menu) (`a`) and a direction, and the
  lock there, of a chest or a door, springs open: `The lock springs open with a click.` Nothing
  opens by itself; open it afterwards as an unlocked one. Where nothing locked stands, Knock is
  refused and costs nothing: `There is no lock there to spring.`
- **Kick it open** (below).

## Kicking a chest open

Press `k` and a chest's direction to kick it, as you [kick a door](doors.md#kicking): 6 seconds,
and just as [loud](hearing.md#how-loud). Your chance to break it is your
[Strength](stats.md#stat-strength) against its **sturdiness**. Your
[Athletics](skills.md#skill-athletics) skill and the [Lockbreaker](talents.md#talent-lockbreaker)
talent add to your Strength for this, as for a door. A chest that holds says
`You kick the coffer. It holds.`, and nothing else happens, so kick as often as you like.

A chest that breaks is **smashed**: `The coffer bursts apart!` Its lock is gone, and **each
[stack](objects.md#piles) inside has an even chance to break**, one after another. The rest lies in
the splinters. **Coins and loose [gems](gems.md) never break** under a boot. The
[Lockbreaker](talents.md#talent-lockbreaker) talent (Strength 12) halves the chance.

```
The coffer bursts apart!
The potion of healing and the scroll of fireball are destroyed.
Inside you find a dagger and 12 gold coins.
```

{{#include generated/chest-bands.md}}

## Blasts

A [blast](digging.md#explosions), such as a [fireball trap's](traps.md#trap-fireball) or a
[scroll of fireball](scrolls.md#scroll-of-fireball)'s, **blows apart** a shut chest it reaches if
it is stronger than the chest's **hardness** (the last column above): `The coffer is blown apart!`
Each [stack](objects.md#piles) inside breaks with the same even chance, and **nothing is spared**,
the coins and the [gems](gems.md) included. A fireball's blast (1000) blows apart anything up to a
strongbox; the great chests outlast it.

{{#include generated/chest-breaking.md}}

You cannot [dig into](digging.md#digging) a chest.

## Trapped chests

Some chests are **trapped**, with the same [traps](traps.md) that [doors](doors.md#trapped-doors)
have: an [arrow](traps.md#trap-arrow), [spears](traps.md#trap-spear), a
[fireball](traps.md#trap-fireball), [frost](traps.md#trap-frost),
[lightning](traps.md#trap-lightning-bolt), an [alarm bell](traps.md#trap-alarm), or a
[teleport](traps.md#trap-teleport), but never a [pit](traps.md#trap-pit). The trap is hidden, and
goes off at you:

- when you **open** the chest, with a key, without one, or after picking its lock.
  [Knock](abilities.md#ability-knock) only springs the lock; the trap waits for the lid;
- when your **kick breaks it** open, as the kick lands. A kick that holds sets nothing off;
- when you **fail badly to [disarm](traps.md#disarming-traps)** it.

The trap comes out of the chest at whoever is beside it. An [arrow](traps.md#trap-arrow) shoots out
of the box itself, [spears](traps.md#trap-spear) strike at you,
[lightning](traps.md#trap-lightning-bolt) runs from the chest through you, and a
[fireball](traps.md#trap-fireball) or [frost](traps.md#trap-frost) bursts around the chest. Walking
past never sets it off.

A **[fireball](traps.md#trap-fireball) or [frost trap](traps.md#trap-frost) goes off inside the
box** and **blows the chest apart**, however it was set off: `The coffer is blown apart!` Each
[stack](objects.md#piles) inside breaks with an even chance, and **nothing is spared**, coins and
[gems](gems.md) included, as in a blast. The other traps leave the chest as they found it: open if
you opened it, smashed if you kicked it.

```
You open the coffer.
A fireball explodes!
The fireball burns you.
The coffer is blown apart!
The 12 gold coins are destroyed.
Inside you find a dagger.
```

A chest blown apart by a blast from somewhere else (a
[scroll of fireball](scrolls.md#scroll-of-fireball), another [trap](traps.md)) sets off its own
trap at no one: the trap is simply gone with the lid.

[Searching](searching.md#searching-1), helped by your [Detect traps](skills.md#skill-detect-traps)
skill, finds a chest's trap like any [hidden trap](traps.md#finding-traps)
(`You notice that the coffer is trapped: an arrow trap.`). Once you know it,
[look mode](controls.md#look-mode) says so: "A coffer, trapped: an arrow trap." The chest keeps its
own look on the map. A known trap on a chest can be **disarmed** like any other (see
[Traps](traps.md#disarming-traps)), and the chest is then an ordinary shut chest.
[Trap sense](talents.md#talent-trap-sense) halves a chest trap's blow, as any trap's.

## The Chest-wise talent

With the [Chest-wise](talents.md#talent-chest-wise) talent ([Intelligence](stats.md#stat-intelligence)
11 or [Detect traps](skills.md#skill-detect-traps) 15), you can tell at a glance whether a shut
chest you see is locked. [Look mode](controls.md#look-mode) calls it "a locked coffer" or "an
unlocked coffer". A trapped chest gives itself away as soon as you see it, with no search and no
chance of missing it: `You notice that the coffer is trapped: a fireball trap.`
