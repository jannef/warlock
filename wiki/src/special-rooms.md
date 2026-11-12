# Special rooms

Now and then a room of the dungeon is not like the others. A **special room** changes the rules
while you are inside it: the [light](light.md#what-light-is), the
[noise](hearing.md#what-makes-noise) you make, how fast you [heal](healing.md#how-fast), who dares
to follow you in, or the floor under your feet.

This page lists every kind. Some have pages of their own: [Teeming dens](teeming-dens.md), full of
sleeping creatures; [Altars, forges, and springs](altars-forges-springs.md), the rooms that
hold something to use; and [Cursed, haunted, and hallowed rooms](cursed-and-hallowed.md), where
the dead rise or are kept out.

## Finding one

The first time you step into a special room, a message tells you what it is:

```
The light seems to drain away here.
```

From then on you know the room. [Look mode](controls.md#look-mode) names it on any of its floor
you see or remember ("A stone floor in an echoing hall.", "You remember a stone floor here, in a
sanctuary."). Before you have been inside, it is just a room, though you may notice something odd
about it from the doorway: an unnatural darkness, a faint warm glow, [water](terrain.md#water) on
the floor. Some rooms give themselves away from outside: the first time you see any part of one, a
line hints at what lies there.

Special rooms are not terribly common, but not rare either. About three levels of a generated
[dungeon](dungeons.md) in five have one, and a level never has more than one, though some
[dungeons](dungeons.md) allow more. In the rooms-and-corridors levels it is a whole room, walls
and doors included, but never the room you arrive in. In the caverns it is a patch of the cave,
well away from where you arrive. Cēaptūn's Mynster is a sanctuary.

## By dungeon

Each [dungeon](dungeons.md) may have odds of its own, and some of its levels other odds again. A
dungeon may be full of flooded rooms, or never have a [shop](shops.md).

- "A room is special" is the chance for each room (each of eight tries in a cavern), until the
  level has as many as "At most a level".
- The kind of room is then drawn. The rooms a dungeon favours keep their fixed share of the draws,
  and a room it bans never comes. Every other room comes by its weight in the table of rooms
  below, so a dungeon gets every kind of room it does not ban.
- "A shop" is the chance that a rooms-and-corridors level has one, if one of its rooms suits (see
  [Shops](shops.md)).

{{#include generated/branch-odds.md}}

## Whole levels

Very rarely, a room's effect does not stay in a room. When a level draws a room that can spread,
there is a small chance (its "Whole level" chance below) that the effect covers the **whole
level** instead, and the room itself is plain. You are told each time you arrive on such a level,
and the effect holds everywhere on it. Where a room's effect and a whole level's meet, both apply:
a [shadowed room](special-rooms.md#room-shadowed) on a gloomy level is darker still. Two rooms can
do this: the [draining room](special-rooms.md#room-draining) and the
[cursed room](special-rooms.md#room-cursed).

## The rooms

"Weight" is how often a room is drawn against the others. "Share" is the part of all draws it gets
where a dungeon favours or bans nothing. "Only" says where it can come at all.

{{#include generated/special-rooms.md}}

- **Shadowed room.** Magical darkness. Every [light](light.md#what-light-is) that reaches into the
  room keeps only part of its strength there: your [lantern](lantern.md), wall
  [torches](light.md#light-sources), fires, and the [sun](sun.md#sky-light) or
  [moon](sun.md#moonlight) through an opening alike. With normal eyes you see little inside unless
  a light is close. Creatures there are hidden just as you are, and they see you no better.
  [Darkvision](vision.md#the-four-types) works as anywhere else.
- **Echoing hall.** Everything you do in it is [loud](hearing.md#how-loud): your
  [footsteps](hearing.md#what-makes-noise), opening a [door](doors.md), fighting. Louder sounds
  carry farther, so creatures much farther away hear you (see [Hearing](hearing.md)). The same
  goes for creatures making noise in it.
- **Hushed chamber.** The opposite: [noise](hearing.md#what-makes-noise) made inside barely
  carries. A good place to sneak through, or to fight without drawing a crowd.
- **Sanctuary.** A place of peace with a faint warm glow, enough to see by. You
  [heal](healing.md#how-fast) much faster while you stand in it (see
  [Hit points and death](hit-points.md#getting-better)). Creatures [hostile](creatures.md#factions) to you will
  not step in; they stay outside, waiting. They can still strike at you across the threshold if
  you stand in the doorway, and shoot at you from outside. Creatures that are not hostile to you
  come and go as they please.
- **Flooded room.** Ankle-deep water over the floor. Wading through it is slower than walking, and
  every step splashes loudly (see [Terrain](terrain.md#water)).
- **Black chamber.** A dark that swallows every light, your [lantern](lantern.md) included: nothing
  inside is lit at all. Only [darkvision](vision.md#the-four-types) sees in it, so creatures inside
  are hidden from normal eyes, and you from theirs.
- **Draining room.** Something in it drinks at magic. Everyone inside who has
  [mana](mana.md#how-much), you and creatures alike, is
  [Ebbing](status-effects.md#status-ebbing) (see [Status effects](status-effects.md)) for as long
  as they stay and half a minute after they leave. [Ebbing](status-effects.md#status-ebbing) drains
  a point of [mana](mana.md#how-much) every 20 seconds, and none comes back meanwhile, so a fight
  between [casters](magic.md) in a draining room soon becomes a brawl. It glows a faint cold
  violet. One floor in twenty that draws a draining room is drained throughout instead.
- **Sweltering chamber.** Hot air: 25 °C warmer than the level around it, with a faint red glow.
  The heat does nothing to you, but [ice](temperature.md#walking-on-ice) melts in it (see
  [Temperature and ice](temperature.md)). The [status bar](the-screen.md#the-screen) shows the heat
  as you step in.
- **Frigid hall.** Bitter cold: 25 °C colder than the level around it, with a faint blue glow. Its
  [water](terrain.md#water) freezes even on a mild level, so a pool in it is usually solid
  [ice](temperature.md#walking-on-ice). The cold does nothing to you.
- **Silent vault.** A dead silence. Every [noise](hearing.md#what-makes-noise) made inside is a
  hundredth as loud, so a fight there is barely heard even next door (see [Hearing](hearing.md)).
  Everyone inside, you and creatures alike, is [Silenced](status-effects.md#status-silenced) (see
  [Status effects](status-effects.md)) for as long as they stay and ten seconds after they leave.
  No [spell](magic.md#the-spells) can be cast, no [scroll](scrolls.md) read, and no
  [voice](hearing.md#throwing-your-voice) thrown. A [wizard](classes.md#wizard) and a
  [kobold shaman](creatures.md#creature-kobold-shaman) fight with sticks in it; a
  [thief](classes.md#thief) walks through unheard.
- **Heavy hall.** The stones pull at whatever you carry. Inside, your
  [carrying limit](burden.md#how-much-you-can-carry) is half what it is anywhere else, so a load
  that was light makes you [burdened](burden.md#burden-1), and armour that only burdened you
  [strains](burden.md#burden-1) you (see [Burden](burden.md)). Creatures carry less inside too. If
  what you carry would overload you inside, you cannot go in ("You carry too much to go in
  there."). If you overload yourself inside by picking something up, you can still step back out.
- **Teeming den.** A room full of sleeping [creatures](creatures.md) and the things they have
  gathered. You may smell it before you see it, and you hear it when you step in. See
  [Teeming dens](teeming-dens.md).
- **Hall of mirrors.** Glass on every side. Everyone in it, you too, is shown with two reflections
  that look exactly like them, standing elsewhere in the room and shifting every turn. Nothing on
  the screen tells a reflection from the real thing; [look mode](controls.md#look-mode) only knows
  your own ("Your reflection"). Swing at or walk into a reflection and your blow meets only glass,
  which costs you the blow's time. An [arrow](ranged.md#shooting-and-throwing) or a thrown dagger
  aimed at one flies on through it. An archer outside the hall shooting at you while you are inside
  aims at you or at one of your reflections, each as likely. Creatures inside the hall know it and
  are never fooled. [Spells](magic.md#the-spells) that fly to the first creature in their way pass
  the reflections by. Only rooms of 80 floor cells or fewer can be one.
- **Unstable room.** A small room, 60 floor cells at most, whose floor shimmers. Every step you take
  inside has a one in ten chance to fling you somewhere else on the level, as a
  [teleport trap](traps.md#trap-teleport) does ("You are wrenched elsewhere."). Creatures stepping
  in are flung just the same, so fleeing through one is a gamble, and so is chasing someone into
  it.
- **Collapsed floor.** A room whose floor is cracked and sagging (shown as `:`, "cracked floor"),
  found only on a level with another below it. Only the middle of the room is cracked. The floor
  next to its walls and doors is always sound, so you can always walk round the cracks from one
  door to another.

  Every step you take onto the cracked stone has a 15 % chance to give way, 30 % while you are
  [burdened](burden.md#burden-1) or worse ("The floor gives way beneath you!"). You fall to the
  level below and land somewhere open on it, never behind a
  [locked door](doors.md#locked-doors) or in a [shop](shops.md). You take 2d6
  [damage](combat.md#damage), which can kill you, and are
  [Staggered](status-effects.md#status-staggered). It is a quick way down, if you can take the
  bruises. Creatures never fall through.
- **Treasure room.** Wealth nobody has touched in a long time. Inside lie one or two chests of the
  level's depth (see [Chests](chests.md)), two to five heaps of [gold](shops.md#money) coins (three
  six-sided dice times two coins: 6 to 36 coins a heap), and one to three loose [gems](gems.md).

  Every door of it is a [secret door](doors.md#secret-doors), so you must
  [search](searching.md#searching-1) the walls to find the way in, or [dig](digging.md#digging) in.
  In a cavern it is a box cut into the rock with one hidden door. Nothing guards it and nothing
  seals it: its only dangers are its chests' [traps](traps.md#finding-traps) and the time it takes
  to find. Each time you arrive on its level until you have been inside, *something tells you
  there is wealth hidden on this level*.
- **Greater treasure room.** A fortune walled up in the rock. Inside lie two to four
  [chests](chests.md) of the level's depth, four to ten heaps of [gold](shops.md#money) coins (four
  six-sided dice times five coins: 20 to 120 coins a heap), and two to six loose [gems](gems.md).

  It has **no door and no corridor at all**: nothing leads there, and the only way in is through a
  wall. One of its walls, beside a passage you can reach, is always thin. Search beside it (as for
  a [secret door](doors.md#secret-doors), at the same difficulty) and *the wall here sounds
  hollow*. Look at it and it is a hollow stone wall. Dig through it with a pick-axe, or
  [blast](digging.md#explosions) it open (a [scroll of fireball](scrolls.md#scroll-of-fireball)
  does it). Any other wall of it can be [dug](digging.md#digging) too, if you guess well. Without a
  digging tool or a blast you cannot get in, so come back with one.

  Nothing the level needs is ever inside, no [trap](traps.md#finding-traps) is hidden in it, and
  no creature wanders in. Each time you arrive on its level until you have been inside,
  *something tells you a fortune lies walled up on this level*.
- **Tempting hoard.** Only on a level with a danger. Three items lie on its floor, better than the
  level's own (three item levels above it), and no one guards them.

  The moment you pick anything up inside, the room seals. Every door of it slams shut and locks
  ("The doors slam shut!"); a door with something in the doorway stays open. Two to four creatures
  [hostile](creatures.md#factions) to you, a step more dangerous than the level's own, appear at
  the room's edges, where you cannot see them if there is such a place. The
  [locks](doors.md#locked-doors) are of the level's depth, and no key but a
  [skeleton key](doors.md#keys) fits. [Pick](doors.md#picking-locks) them,
  [kick](doors.md#kicking) the doors in, or fight. A cave chamber has no doors, so there only the
  guardians come. A room seals once.
- **Cursed room.** The dead do not rest in it: see [Cursed rooms](cursed-and-hallowed.md#cursed-rooms) below. Only on a
  level with a danger. One floor in twenty that draws a cursed room is cursed throughout instead.
- **Haunted room.** A fine camp spot that is not. [Rest](resting.md#how-resting-works) in it and
  the dead come, once every hour (see [Haunted rooms](cursed-and-hallowed.md#haunted-rooms) below). Only on a level with
  a danger.
- **Altar room.** An old altar stands in the middle of the room, its god long forgotten, and a
  faint glow rises from it; you notice it from the doorway. Stand beside it and you may
  [pray](abilities.md#ability-pray) (see [Forgotten altars](altars-forges-springs.md#forgotten-altars)).
  Only rooms of 9 floor cells or more can be one.
- **Forge.** A cold forge with an old anvil in the middle of the room; you smell soot and cold
  iron from the doorway. Stand beside the anvil and you may [temper](altars-forges-springs.md#forges)
  metal arms or armour you carry (see [Forges](altars-forges-springs.md#forges)). Only rooms of 9
  floor cells or more can be one.
- **Shrine.** A squat stone shrine stands in the middle of the room. Like a forgotten altar it has
  a hidden side, but only **[good or evil](alignment.md)**, either as likely, and its stone's light
  tells which: gold for good, red for evil. It cares only whether you are on its side (see
  [Alignment](alignment.md)).
  - Of its side, you [heal](healing.md#how-fast) twice as fast inside ("You feel welcome here.").
  - Of the other side, you are [Unwelcome](status-effects.md#status-unwelcome) (see
    [Status effects](status-effects.md)) for as long as you stay and half a minute after you
    leave. Wounds [heal](healing.md#how-fast) at half the pace and your blows go a little astray
    ("Something here wants you gone.").
  - A neutral character is ignored ("An old shrine. It ignores you.").

  Creatures are treated the same way by their own side, so a good shrine is a poor place for an
  orc to fight you. There is nothing to pray to. Only rooms of 9 floor cells or more can be one.
- **Scriptorium.** Shelves of rotting scrolls stand in the middle of the room (shown as `≡`), and
  you may smell the dust and old vellum before you see it. Two to four [scrolls](scrolls.md) lie on
  its floor, unread, of the kinds the level would give anyway. Inside, the chance of a
  [training mark](skills.md#getting-better) in
  [Decipher script](skills.md#skill-decipher-script) is doubled:
  [making out a scroll](scrolls.md#making-out-a-scroll) or reading a
  [grimoire](magic.md#learning-spells-from-grimoires) through earns two marks here instead of one,
  and reading a scroll earns one every time instead of half the time.
  Only rooms of 9 floor cells or more can be one.

The [Old delver](talents.md#talent-old-delver) talent (Delver; [Willpower](stats.md#stat-willpower)
11 or [Dexterity](stats.md#stat-dexterity) 12) makes you at home in strange rooms. A
[collapsed floor](special-rooms.md#room-collapsed) never gives way under you, the
[Ebbing](status-effects.md#status-ebbing) of a [draining room](special-rooms.md#room-draining) never
takes you, and nothing you kill ever rises in a [cursed room](special-rooms.md#room-cursed). With
[Old hand](talents.md#talent-old-hand) after it (level 10), a room that quickens healing or mana
quickens it half again more for you: a [sanctuary](special-rooms.md#room-sanctuary)'s three times
as fast becomes four times. You can also lift one thing from a
[tempting hoard](special-rooms.md#room-hoard) before it seals; the second seals it.

Only what happens inside counts. A sound made in an [echoing hall](special-rooms.md#room-echoing)
is [loud](hearing.md#how-loud) wherever it is heard, while one made outside and heard inside is not
changed. Light outside a [shadowed room](special-rooms.md#room-shadowed) is as bright as ever, even
right at its door.
